use tokio;
use std::sync::Arc;

mod web_rtc; 
mod slam;
mod signal_server;
mod video_decoder;

// モジュールの中から構造体（大文字）をインポート
use crate::web_rtc::WebRtc;
use crate::slam::Slam;
use crate::signal_server::SignalServer;
use crate::video_decoder::VideoFrameReconstructor;

use std::io::Write;

struct LookAI {
    rtc: Arc<WebRtc>,
    slam: Slam, // 型名は構造体名の「Slam」（大文字）
    reconstructor: VideoFrameReconstructor,
}

impl LookAI {
    async fn new() -> Self {
        Self {
            // WebRtc::new() が async でないなら .await を外す（先ほどのエラーより）
            rtc: Arc::new(WebRtc::new()),
            // Slam構造体のnewを呼ぶ
            slam: Slam::new(),
            reconstructor: VideoFrameReconstructor::new(),
        }
    }

    async fn start(&mut self) {
        let signaling_url = "ws://127.0.0.1:3001/ws";
        let rtc_clone = Arc::clone(&self.rtc);
        
        tokio::spawn(async move {
            if let Err(e) = rtc_clone.run(signaling_url).await {
                eprintln!("❌ WebRTC Error: {:?}", e);
            }
        });

        println!("🚀 LookAI Loop Started. Waiting for video packets...");
        
        let mut saved = false;

        let mut saved_count = 0; // 何パケット分保存したか

        loop {
            if let Some(packet) = self.rtc.receive_frame().await {
                // 重要: reconstructor.push_rtp が Some(frame) を返すまで待つ
                if let Some(complete_frame) = self.reconstructor.push_rtp(&packet) {
                    if !saved {
                        let mut file = std::fs::File::create("debug_frame.h264").unwrap();
                        // 結合済みの完全な NAL ユニットにスタートコードを付与
                        file.write_all(&[0, 0, 0, 1]).unwrap(); 
                        file.write_all(&complete_frame).unwrap(); 
                        file.flush().unwrap();
                        println!("✅ COMPLETE FRAME SAVED! Try playing 'debug_frame.h264'");
                        saved = true;
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Starting LookAI...");
    let mut look_ai_core = Arc::new(WebRtc::new());
    let server = Arc::new(SignalServer::new(Arc::clone(&look_ai_core)));
    tokio::spawn(server.start(3001));

    let mut look_ai = LookAI {
        rtc: Arc::clone(&look_ai_core),
        slam: Slam::new(),
        reconstructor: VideoFrameReconstructor::new(),
    };
    look_ai.start().await;
}