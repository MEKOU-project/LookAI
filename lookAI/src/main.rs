use tokio;
use std::sync::Arc;

mod web_rtc; 
mod slam;

// モジュールの中から構造体（大文字）をインポート
use crate::web_rtc::WebRtc;
use crate::slam::Slam;

struct LookAI {
    rtc: Arc<WebRtc>,
    slam: Slam, // 型名は構造体名の「Slam」（大文字）
}

impl LookAI {
    async fn new() -> Self {
        Self {
            // WebRtc::new() が async でないなら .await を外す（先ほどのエラーより）
            rtc: Arc::new(WebRtc::new()),
            // Slam構造体のnewを呼ぶ
            slam: Slam::new(),
        }
    }

    async fn start(&mut self) {
        let signaling_url = "ws://127.0.0.1:3001";

        let rtc_clone = Arc::clone(&self.rtc);
        tokio::spawn(async move {
            if let Err(e) = rtc_clone.run(signaling_url).await {
                eprintln!("❌ WebRTC Error: {:?}", e);
            }
        });

        println!("WebRTC connecting to {}...", signaling_url);
        loop {
            if let Some(frame) = self.rtc.receive_frame().await {
                // Slam.rs側で定義したメソッド名（calculate_all など）に合わせて呼び出し
                let data = self.slam.calculate_all(frame); 
                
                // dataがSlamResult構造体なら、文字列化して送る
                self.rtc.send_slam_data(format!("{:?}", data)).await;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Starting LookAI...");
    let mut look_ai = LookAI::new().await;
    look_ai.start().await;
}