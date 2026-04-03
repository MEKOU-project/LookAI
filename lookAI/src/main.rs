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
        
        loop {
            // 1. WebRTCからRTPパケット（断片）を受け取る
            if let Some(packet) = self.rtc.receive_frame().await {
                
                // 2. 再構成器に投入して、1フレーム（NALユニット等）に結合されるのを待つ
                if let Some(complete_frame) = self.reconstructor.push_rtp(&packet) {
                    
                    // 3. 1フレーム分が完成した時だけ、SLAM計算を実行
                    let data = self.slam.calculate_all(complete_frame); 
                    
                    // 4. 結果をスマホへ送り返す
                    self.rtc.send_slam_data(format!("{:?}", data)).await;
                }
            }
            
            // 待機時間はパケット受信に合わせて調整（10msは適正ですが、receive_frameがブロッキングなら不要な場合も）
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