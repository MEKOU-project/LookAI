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
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTPCodecType;
use webrtc::rtp_transceiver::rtp_transceiver_direction::RTCRtpTransceiverDirection;
use webrtc::rtp_transceiver::RTCRtpTransceiverInit;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::rtp::packetizer::Depacketizer;

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
        let (ws_stream, _) = connect_async(signaling_url).await?;
        let (mut ws_writer, mut ws_reader) = ws_stream.split();
        
        println!("📡 Connected to Signaling Server: {}", signaling_url);

        let reg = SignalMessage {
            msg_type: "register".to_string(),
            device_type: Some("pc".to_string()),
            ..Default::default()
        };
        ws_writer.send(Message::Text(serde_json::to_string(&reg)?.into())).await?;

        let (ice_tx, mut ice_rx) = mpsc::channel::<SignalMessage>(32);

        println!("⚠️ Initial Handshake Attempt...");
        self.start_handshake(&mut ws_writer, ice_tx.clone()).await?;

        loop {
            tokio::select! {
                Some(msg) = ws_reader.next() => {
                    let msg = msg?;
                    if let Message::Text(text) = msg {
                        let incoming: SignalMessage = serde_json::from_str(&text)?;
                        match incoming.msg_type.as_str() {
                            "joined" | "register" | "join" | "request_offer" => {
                                if incoming.device_type.as_deref() == Some("mobile") || incoming.msg_type == "request_offer" {
                                    println!("📱 Mobile detected! Re-starting Handshake...");
                                    self.start_handshake(&mut ws_writer, ice_tx.clone()).await?;
                                }
                            },
                            "signal" => {
                                if let Some(sig_val) = incoming.signal {
                                    let pc_lock = self.pc.lock().await;
                                    if let Some(pc) = &*pc_lock {
                                        let sdp_type = sig_val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                        if sdp_type == "answer" {
                                            let sdp = serde_json::from_value(sig_val)?;
                                            pc.set_remote_description(sdp).await?;
                                            println!("✅ WebRTC Answer set.");
                                        } else if let Some(cand_obj) = sig_val.get("candidate") {
                                            let cand_str = if let Some(obj) = cand_obj.as_object() {
                                                obj.get("candidate").and_then(|v| v.as_str()).unwrap_or("")
                                            } else {
                                                cand_obj.as_str().unwrap_or("")
                                            };
                                            if !cand_str.is_empty() {
                                                let _ = pc.add_ice_candidate(RTCIceCandidateInit {
                                                    candidate: cand_str.to_string(),
                                                    ..Default::default()
                                                }).await;
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                },
                Some(ice_msg) = ice_rx.recv() => {
                    let json = serde_json::to_string(&ice_msg)?;
                    let _ = ws_writer.send(Message::Text(json.into())).await;
                }
            }
        }
    }

    async fn start_handshake<S>(&self, ws_writer: &mut S, ice_tx: mpsc::Sender<SignalMessage>) -> Result<()> 
    where S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send 
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

        pc.on_ice_candidate(Box::new(move |candidate| {
            let tx = ice_tx.clone();
            Box::pin(async move {
                if let Some(cand) = candidate {
                    if let Ok(json_cand) = cand.to_json() {
                        let msg = SignalMessage {
                            msg_type: "signal".to_string(),
                            target: Some("mobile".to_string()),
                            signal: Some(serde_json::to_value(json_cand).unwrap()),
                            ..Default::default()
                        };
                        let _ = tx.send(msg).await;
                    }
                }
            })
        }));

        // RTCRtpTransceiverInit は Default がないので全フィールドを明示
        pc.add_transceiver_from_kind(RTPCodecType::Video, Some(RTCRtpTransceiverInit {
            direction: RTCRtpTransceiverDirection::Recvonly,
            send_encodings: vec![],
        })).await?;

        let dc = pc.create_data_channel("data", None).await?;
        let dc_storage = Arc::clone(&self.data_channel);
        let dc_clone = Arc::clone(&dc);
        dc.on_open(Box::new(move || {
            let dcs = Arc::clone(&dc_storage);
            let dcc = Arc::clone(&dc_clone);
            Box::pin(async move {
                let mut storage = dcs.lock().await;
                *storage = Some(dcc);
                println!("📊 DataChannel opened!");
            })
        }));

        let tx = self.frame_tx.clone();
        pc.on_track(Box::new(move |track, _, _| {
            let tx_inner = tx.clone();
            Box::pin(async move {
                // depacketizer は削除
                while let Ok((rtp, _)) = track.read_rtp().await {
                    // payload をそのまま送信（RTPヘッダーは抜いた状態）
                    if !rtp.payload.is_empty() {
                        let _ = tx_inner.send(rtp.payload.to_vec()).await;
                    }
                }
            })
        }));

        let offer = pc.create_offer(None).await?;
        pc.set_local_description(offer.clone()).await?;

        let offer_msg = SignalMessage {
            msg_type: "signal".to_string(),
            target: Some("mobile".to_string()),
            signal: Some(serde_json::to_value(offer)?),
            ..Default::default()
        };
        ws_writer.send(Message::Text(serde_json::to_string(&offer_msg)?.into())).await?;

        let mut pc_lock = self.pc.lock().await;
        *pc_lock = Some(pc);

        Ok(())
    }

    pub async fn receive_frame(&self) -> Option<Vec<u8>> {
        let mut rx = self.frame_rx.lock().await;
        rx.recv().await
    }

    pub async fn send_slam_data(&self, data: String) -> Result<()> {
        let dc_lock = self.data_channel.lock().await;
        if let Some(dc) = &*dc_lock {
            let _ = dc.send_text(data).await;
        }
        Ok(())
    }
}