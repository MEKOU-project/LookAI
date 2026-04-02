use nalgebra::Isometry3;
use serde::Serialize;

#[derive(Debug, Serialize)] // これで全体のデバッグ表示とJSON化が可能
pub struct SlamResult {
    pub pose: Isometry3<f32>,
    pub detected_objects: Vec<DetectedItem>,
}

#[derive(Debug, Serialize)] // 子構造体にも忘れずに！
pub struct DetectedItem {
    pub label: String,
    pub confidence: f32,
    pub position_3d: [f32; 3],
}

pub struct Slam;

impl Slam {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_all(&self, _frame: Vec<u8>) -> SlamResult {
        // ここに3050を唸らせる推論ロジックを詰め込む
        SlamResult {
            pose: Isometry3::identity(),
            detected_objects: vec![],
        }
    }
}