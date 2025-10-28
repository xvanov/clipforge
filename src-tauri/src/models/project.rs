use super::clip::MediaClip;
use super::timeline::{Track, TrackType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub file_path: Option<String>,
    pub version: String,
    pub tracks: Vec<Track>,
    pub media_library: Vec<MediaClip>,
    pub export_settings: ExportSettings,
    pub auto_save_enabled: bool,
    pub last_auto_save: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub resolution: Resolution,
    pub codec: Codec,
    pub quality: Quality,
    pub fps: Option<u32>,
    pub audio_codec: AudioCodec,
    pub audio_bitrate: u32,
    pub hardware_acceleration: bool,
}

#[allow(dead_code, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Resolution {
    Source,
    #[serde(rename = "2160p")]
    UHD4K,
    #[serde(rename = "1440p")]
    QHD,
    #[serde(rename = "1080p")]
    FullHD,
    #[serde(rename = "720p")]
    HD,
    #[serde(rename = "480p")]
    SD,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    H264,
    Hevc,
    Vp9,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    High,
    Medium,
    Low,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    Aac,
    Mp3,
    Opus,
}

impl Default for ExportSettings {
    fn default() -> Self {
        ExportSettings {
            resolution: Resolution::FullHD,
            codec: Codec::H264,
            quality: Quality::High,
            fps: None,
            audio_codec: AudioCodec::Aac,
            audio_bitrate: 192,
            hardware_acceleration: true,
        }
    }
}

#[allow(dead_code)]
impl Project {
    pub fn new(name: String) -> Self {
        let mut project = Project {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            file_path: None,
            version: "1.0.0".to_string(),
            tracks: Vec::new(),
            media_library: Vec::new(),
            export_settings: ExportSettings::default(),
            auto_save_enabled: true,
            last_auto_save: None,
        };

        // Create default main track
        project
            .tracks
            .push(Track::new("Main Track".to_string(), TrackType::Main));

        project
    }

    pub fn mark_modified(&mut self) {
        self.modified_at = Utc::now();
    }
}
