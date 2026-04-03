use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTPCodecType;
use webrtc::rtp_transceiver::rtp_transceiver_direction::RTCRtpTransceiverDirection;
use webrtc::rtp_transceiver::RTCRtpTransceiverInit;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;

// --- 構造体の定義を上に持ってくる ---
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignalMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<serde_json::Value>,
    #[serde(rename = "deviceType", skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,
}

pub struct WebRtc {
    // ID管理ができるよう、あえてArc<RTCPeerConnection>のOptionにしています
    pc: Arc<Mutex<Option<Arc<RTCPeerConnection>>>>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    frame_rx: Mutex<mpsc::Receiver<Vec<u8>>>,
    frame_tx: mpsc::Sender<Vec<u8>>,
}

impl WebRtc {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(32);
        Self {
            pc: Arc::new(Mutex::new(None)),
            data_channel: Arc::new(Mutex::new(None)),
            frame_rx: Mutex::new(rx),
            frame_tx: tx,
        }
    }

    pub async fn run(&self, signaling_url: &str) -> Result<()> {
        // connect_async の戻り値を明示的に型指定して推論を助ける
        let (ws_stream, _) = connect_async(signaling_url).await?;
        let (mut ws_writer, mut ws_reader) = ws_stream.split();
        
        println!("📡 Connected to Signaling Server: {}", signaling_url);

        let reg = SignalMessage {
            msg_type: "register".to_string(),
            target: None,
            signal: None,
            device_type: Some("pc".to_string()),
        };

        let reg_json = serde_json::to_string(&reg)?;
        ws_writer.send(Message::Text(reg_json.into())).await?;

        println!("⚠️ Forced Handshake: Sending Offer to 'mobile'...");
        self.start_handshake(&mut ws_writer).await?;

        while let Some(msg) = ws_reader.next().await {
            let msg = msg?;
            println!("📩 RAW WS MSG: {:?}", msg);
            if let Message::Text(text) = msg {
                let incoming: SignalMessage = serde_json::from_str(&text)?;

                match incoming.msg_type.as_str() {
                    "joined" | "register" | "join"=> {
                        if incoming.device_type.as_deref() == Some("mobile") {
                            println!("📱 Mobile joined. Starting Handshake...");
                            self.start_handshake(&mut ws_writer).await?;
                        }
                    },
                    "request_offer" => {
                        println!("📱 Mobile detected by server. Re-sending Offer...");
                        self.start_handshake(&mut ws_writer).await?;
                    },
                    "signal" => {
                        if let Some(sig_val) = incoming.signal {
                            let pc_lock = self.pc.lock().await;
                            if let Some(pc) = &*pc_lock {
                                let sdp_type = sig_val.get("type").and_then(|v| v.as_str()).unwrap_or("");

                                if sdp_type == "answer" {
                                    let sdp: RTCSessionDescription = serde_json::from_value(sig_val)?;
                                    pc.set_remote_description(sdp).await?;
                                    println!("✅ Remote Description (Answer) set successfully!");
                                } else if let Some(candidate_obj) = sig_val.get("candidate") {
                                    // --- ここを修正：スマホからの入れ子構造をバラす ---
                                    let candidate_str = if let Some(obj) = candidate_obj.as_object() {
                                        obj.get("candidate").and_then(|v| v.as_str()).unwrap_or("")
                                    } else {
                                        candidate_obj.as_str().unwrap_or("")
                                    };

                                    if !candidate_str.is_empty() {
                                        let cand_init = RTCIceCandidateInit {
                                            candidate: candidate_str.to_string(),
                                            ..Default::default()
                                        };
                                        if let Err(e) = pc.add_ice_candidate(cand_init).await {
                                            eprintln!("❌ Failed to add Remote ICE candidate: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn start_handshake<S>(&self, ws_writer: &mut S) -> Result<()> 
    where 
        S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send 
    {
        let mut m = MediaEngine::default();
        m.register_default_codecs()?;
        let api = APIBuilder::new().with_media_engine(m).build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let pc = Arc::new(api.new_peer_connection(config).await?);

        // --- 1. Candidate を一時的に受けるチャンネルを作成 ---
        let (ice_tx, mut ice_rx) = mpsc::channel::<SignalMessage>(20);

        let ice_tx_clone = ice_tx.clone();
        pc.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
            let tx = ice_tx_clone.clone();
            Box::pin(async move {
                if let Some(cand) = candidate {
                    if let Ok(json_cand) = cand.to_json() {
                        let msg = SignalMessage {
                            msg_type: "signal".to_string(),
                            target: Some("mobile".to_string()),
                            signal: Some(serde_json::to_value(json_cand).unwrap()),
                            device_type: None,
                        };
                        let _ = tx.send(msg).await;
                    }
                }
            })
        }));

        // --- 2. 各種設定 (DataChannel, Track等) ---
        pc.add_transceiver_from_kind(
            RTPCodecType::Video,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                send_encodings: vec![],
            }),
        ).await?;

        let dc = pc.create_data_channel("data", None).await?;
        let dc_storage = Arc::clone(&self.data_channel);
        let dc_clone = Arc::clone(&dc);
        dc.on_open(Box::new(move || {
            let dcs = Arc::clone(&dc_storage);
            let dcc = Arc::clone(&dc_clone);
            Box::pin(async move {
                let mut storage = dcs.lock().await;
                *storage = Some(dcc);
            })
        }));

        let tx = self.frame_tx.clone();
        pc.on_track(Box::new(move |track, _, _| {
            let tx_inner = tx.clone();
            println!("🎥 Track received: Kind={}, ID={}", track.kind(), track.id());
            Box::pin(async move {
                while let Ok((rtp, _)) = track.read_rtp().await {
                    // パケットが届いているか確認するためのデバッグ（頻繁に出るため注意）
                    // println!("📦 RTP Payload: {} bytes", rtp.payload.len());
                    let _ = tx_inner.send(rtp.payload.to_vec()).await;
                }
            })
        }));

        // --- 3. Offer の作成と「最初の送信」 ---
        let offer = pc.create_offer(None).await?;
        pc.set_local_description(offer.clone()).await?;

        let initial_offer_msg = SignalMessage {
            msg_type: "signal".to_string(),
            target: Some("mobile".to_string()),
            signal: Some(serde_json::to_value(offer)?),
            device_type: None,
        };
        ws_writer.send(Message::Text(serde_json::to_string(&initial_offer_msg)?.into())).await?;

        // PCを保存
        let mut pc_lock = self.pc.lock().await;
        *pc_lock = Some(pc);
        drop(pc_lock); // 重要：ロックを解放してからループに入る

        // --- 4. 見つかった Candidate を順次 ws_writer で送信するループ ---
        // タイムアウトを設定して、ある程度集まったら handshake を抜ける（または run ループに戻る）
        // ここでは簡易的に、Candidateが途切れるまで（または一定時間）送信を続けます
        println!("📡 Sending ICE Candidates...");
        
        // 実際には run メソッド側で select! を使って処理するのが理想ですが、
        // まずは handshake 内で完結させる形です
        while let Ok(Some(ice_msg)) = tokio::time::timeout(std::time::Duration::from_millis(1000), ice_rx.recv()).await {
            let json = serde_json::to_string(&ice_msg)?;
            ws_writer.send(Message::Text(json.into())).await?;
        }

        println!("✅ Handshake sequence completed.");
        Ok(())
    }

    pub async fn receive_frame(&self) -> Option<Vec<u8>> {
        let mut rx = self.frame_rx.lock().await;
        rx.recv().await
    }

    pub async fn send_slam_data(&self, data: String) -> Result<()> {
        let dc_lock = self.data_channel.lock().await;
        if let Some(dc) = &*dc_lock {
            dc.send_text(data).await?; 
        }
        Ok(())
    }
}