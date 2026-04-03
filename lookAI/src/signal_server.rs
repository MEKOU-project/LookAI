use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{extract::ws, routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{StreamExt, SinkExt}; // SinkExt が必要
use dashmap::DashMap;
use tokio::sync::mpsc; // 送信用チャネル

use crate::web_rtc::{WebRtc, SignalMessage};

pub struct SignalServer {
    webrtc: Arc<WebRtc>,
    // デバイスIDと、そのソケットへメッセージを送るための送信機を紐付け
    clients: DashMap<String, mpsc::UnboundedSender<Message>>,
}

impl SignalServer {
    pub fn new(webrtc: Arc<WebRtc>) -> Self {
        Self { 
            webrtc,
            clients: DashMap::new(),
        }
    }

    pub async fn start(self: Arc<Self>, port: u16) {
        let server_clone = Arc::clone(&self);

        let app = Router::new().route("/ws", get(move |ws: WebSocketUpgrade| {
            let s = Arc::clone(&server_clone);
            async move {
                ws.on_upgrade(move |socket| async move {
                    s.handle_socket(socket).await;
                })
            }
        }));

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        println!("🚀 Signal Server running on ws://{}", addr);
        axum::serve(listener, app.into_make_service()).await.unwrap();
    }

    async fn handle_socket(&self, socket: WebSocket) {
        let (mut sender, mut receiver) = socket.split();
        // このソケット専用のチャネルを作成
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // 送信タスクを分離（rx から受け取ったものを実際のソケットへ流す）
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if sender.send(msg).await.is_err() { break; }
            }
        });

        let mut current_device_id: Option<String> = None;

        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(incoming) = serde_json::from_str::<SignalMessage>(&text) {
                    match incoming.msg_type.as_str() {
                        "register" | "join" => {
                            if let Some(id) = incoming.device_type.clone() {
                                println!("✅ Registered: {}", id);
                                self.clients.insert(id.clone(), tx.clone());
                                current_device_id = Some(id.clone());

                                // mobileが来たらpcに通知
                                if id == "mobile" {
                                    if let Some(pc_tx) = self.clients.get("pc") {
                                        let notify = serde_json::to_string(&SignalMessage {
                                            msg_type: "request_offer".to_string(),
                                            target: Some("pc".to_string()),
                                            ..Default::default()
                                        }).unwrap();
                                        let _ = pc_tx.send(Message::Text(notify));
                                    }
                                }
                            }
                        }
                        "signal" => {
                            if let Some(target) = incoming.target {
                                if let Some(target_tx) = self.clients.get(&target) {
                                    let _ = target_tx.send(Message::Text(text));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // 切断時にマップから削除
        if let Some(id) = current_device_id {
            self.clients.remove(&id);
        }
    }
}