use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaClip {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub proxy_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub duration: f64,
    pub resolution: String,
    pub width: i32,
    pub height: i32,
    pub fps: f64,
    pub codec: String,
    pub audio_codec: Option<String>,
    pub file_size: i64,
    pub bitrate: Option<i32>,
    pub has_audio: bool,
    pub imported_at: DateTime<Utc>,
    pub captions: Vec<String>, // Caption IDs - actual Caption model will be added later
}

impl MediaClip {
    pub fn new(
        source_path: String,
        duration: f64,
        width: i32,
        height: i32,
        fps: f64,
        codec: String,
        file_size: i64,
    ) -> Self {
        let name = std::path::Path::new(&source_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        MediaClip {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            source_path,
            proxy_path: None,
            thumbnail_path: None,
            duration,
            resolution: format!("{}x{}", width, height),
            width,
            height,
            fps,
            codec,
            audio_codec: None,
            file_size,
            bitrate: None,
            has_audio: false,
            imported_at: Utc::now(),
            captions: vec![],
        }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn is_4k(&self) -> bool {
        self.width >= 3840
    }

    pub fn is_hd(&self) -> bool {
        self.width >= 1920
    }
}

