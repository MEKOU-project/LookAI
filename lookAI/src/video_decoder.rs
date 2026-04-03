use std::fs::OpenOptions;
use std::io::Write;
use webrtc::util::Unmarshal;
use webrtc::rtp::packet::Packet;
use webrtc::rtp::codecs::h264::H264Packet;
use webrtc::rtp::packetizer::Depacketizer;

pub struct VideoFrameReconstructor {
    depayloader: H264Packet,
    file_path: String,
}

impl VideoFrameReconstructor {
    pub fn new() -> Self {
        let path = "output.h264".to_string();
        let _ = std::fs::File::create(&path); // 新規作成
        Self {
            depayloader: H264Packet::default(),
            file_path: path,
        }
    }

    pub fn push_rtp(&mut self, rtp_payload: &[u8]) -> Option<Vec<u8>> {
        println!("📥 Received packet: {} bytes", rtp_payload.len()); // 通過確認1

        let mut reader = &rtp_payload[..];
        if let Ok(packet) = Packet::unmarshal(&mut reader) {
            println!("✅ RTP Unmarshal success. Payload size: {}", packet.payload.len()); // 通過確認2
            
            if let Ok(h264_payload) = self.depayloader.depacketize(&packet.payload) {
                if !h264_payload.is_empty() {
                    println!("🎬 H.264 Depacketize success! size: {}", h264_payload.len());
                    
                    // 毎回開いて閉じる（または flush する）ことで強制的にディスクに書き込む
                    if let Ok(mut file) = OpenOptions::new().append(true).open(&self.file_path) {
                        let _ = file.write_all(&[0, 0, 0, 1]); 
                        let _ = file.write_all(&h264_payload);
                        let _ = file.sync_all(); // ★ OSに「今すぐディスクに書け」と命令する
                    }
                }
            } else {
                println!("⚠️ Depacketize failed (possibly not H.264 or fragment)");
            }
        } else {
            println!("❌ RTP Unmarshal failed");
        }
        None
    }
}