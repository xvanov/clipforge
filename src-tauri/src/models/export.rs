use serde::{Deserialize, Serialize};

/// Export settings for rendering timeline to video file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Output resolution
    pub resolution: ExportResolution,
    /// Video codec
    pub codec: VideoCodec,
    /// Encoding quality
    pub quality: ExportQuality,
    /// Override frame rate (null = use source fps)
    pub fps: Option<u32>,
    /// Audio codec
    pub audio_codec: AudioCodec,
    /// Audio bitrate in kbps
    pub audio_bitrate: u32,
    /// Enable hardware encoding
    pub hardware_acceleration: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::upper_case_acronyms)]
pub enum ExportResolution {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "2160p")]
    UHD4K, // 3840x2160
    #[serde(rename = "1440p")]
    QHD, // 2560x1440
    #[serde(rename = "1080p")]
    FullHD, // 1920x1080
    #[serde(rename = "720p")]
    HD, // 1280x720
    #[serde(rename = "480p")]
    SD, // 854x480
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::upper_case_acronyms)]
pub enum VideoCodec {
    #[serde(rename = "h264")]
    H264,
    #[serde(rename = "hevc")]
    HEVC,
    #[serde(rename = "vp9")]
    VP9,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportQuality {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::upper_case_acronyms)]
pub enum AudioCodec {
    #[serde(rename = "aac")]
    AAC,
    #[serde(rename = "mp3")]
    MP3,
    #[serde(rename = "opus")]
    Opus,
}

impl ExportResolution {
    /// Get resolution dimensions (width, height)
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            ExportResolution::Source => None,
            ExportResolution::UHD4K => Some((3840, 2160)),
            ExportResolution::QHD => Some((2560, 1440)),
            ExportResolution::FullHD => Some((1920, 1080)),
            ExportResolution::HD => Some((1280, 720)),
            ExportResolution::SD => Some((854, 480)),
        }
    }
}

impl VideoCodec {
    /// Get FFmpeg codec name
    pub fn ffmpeg_codec(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "libx264",
            VideoCodec::HEVC => "libx265",
            VideoCodec::VP9 => "libvpx-vp9",
        }
    }

    /// Get output file extension
    #[allow(dead_code)]
    pub fn extension(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "mp4",
            VideoCodec::HEVC => "mp4",
            VideoCodec::VP9 => "webm",
        }
    }
}

impl ExportQuality {
    /// Get CRF value for quality (lower = better quality)
    pub fn crf_value(&self) -> u32 {
        match self {
            ExportQuality::High => 18,
            ExportQuality::Medium => 23,
            ExportQuality::Low => 28,
        }
    }
}

impl AudioCodec {
    /// Get FFmpeg audio codec name
    pub fn ffmpeg_codec(&self) -> &'static str {
        match self {
            AudioCodec::AAC => "aac",
            AudioCodec::MP3 => "libmp3lame",
            AudioCodec::Opus => "libopus",
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            resolution: ExportResolution::FullHD,
            codec: VideoCodec::H264,
            quality: ExportQuality::High,
            fps: None,
            audio_codec: AudioCodec::AAC,
            audio_bitrate: 192,
            hardware_acceleration: true,
        }
    }
}
