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

// --- 構造体の定義を上に持ってくる ---
#[derive(Serialize, Deserialize, Debug)]
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

        while let Some(msg) = ws_reader.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let incoming: SignalMessage = serde_json::from_str(&text)?;

                match incoming.msg_type.as_str() {
                    "joined" => {
                        if incoming.device_type.as_deref() == Some("mobile") {
                            println!("📱 Mobile joined. Starting Handshake...");
                            self.start_handshake(&mut ws_writer).await?;
                        }
                    },
                    "signal" => {
                        if let Some(sig_val) = incoming.signal {
                            let sdp: RTCSessionDescription = serde_json::from_value(sig_val)?;
                            let pc_lock = self.pc.lock().await;
                            if let Some(pc) = &*pc_lock {
                                pc.set_remote_description(sdp).await?;
                                println!("✅ Remote Description set");
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
    where S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin 
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

        // DataChannel
        let dc = pc.create_data_channel("data", None).await?;
        let dc_storage = Arc::clone(&self.data_channel);
        let dc_clone = Arc::clone(&dc);

        dc.on_open(Box::new(move || {
            println!("✅ DataChannel opened!");
            let dcs = Arc::clone(&dc_storage);
            let dcc = Arc::clone(&dc_clone);
            Box::pin(async move {
                let mut storage = dcs.lock().await;
                *storage = Some(dcc);
            })
        }));

        // Video Track
        let tx = self.frame_tx.clone();
        pc.on_track(Box::new(move |track, _, _| {
            let tx_inner = tx.clone();
            Box::pin(async move {
                while let Ok((rtp, _)) = track.read_rtp().await {
                    let _ = tx_inner.send(rtp.payload.to_vec()).await;
                }
            })
        }));

        // SDP Offer
        let offer = pc.create_offer(None).await?;
        pc.set_local_description(offer).await?;

        let mut rx = pc.gathering_complete_promise().await;
        let _ = rx.recv().await;

        if let Some(final_offer) = pc.local_description().await {
            let signal_msg = SignalMessage {
                msg_type: "signal".to_string(),
                target: Some("mobile".to_string()),
                signal: Some(serde_json::to_value(final_offer)?),
                device_type: None,
            };
            let json = serde_json::to_string(&signal_msg)?;
            ws_writer.send(Message::Text(json.into())).await?;
        }

        let mut pc_storage = self.pc.lock().await;
        *pc_storage = Some(pc);

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