use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordingType {
    Screen,
    Webcam,
    #[serde(rename = "screen_webcam")]
    ScreenWebcam,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    Preparing,
    Recording,
    Paused,
    Stopped,
    Failed,
}

/// Represents an active or completed screen/webcam recording session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    /// Unique session identifier
    pub id: String,

    /// Recording type (screen, webcam, or both)
    #[serde(rename = "type")]
    pub recording_type: RecordingType,

    /// Current session status
    pub status: RecordingStatus,

    /// Path where recording will be saved
    pub output_path: String,

    /// Recording start timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    /// Recording stop timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<DateTime<Utc>>,

    /// Total recorded duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// Screen/window identifier being recorded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_source: Option<String>,

    /// Camera device identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_device: Option<String>,

    /// Audio input identifiers
    pub audio_sources: Vec<String>,

    /// Recording resolution (e.g., "1920x1080")
    pub resolution: String,

    /// Recording frame rate (default: 30)
    pub fps: u32,

    /// Error description if status = Failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// MediaClip ID created from recording
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_media_clip_id: Option<String>,
}

impl RecordingSession {
    /// Create a new recording session in Preparing state
    pub fn new(
        recording_type: RecordingType,
        output_path: String,
        resolution: String,
        fps: u32,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            recording_type,
            status: RecordingStatus::Preparing,
            output_path,
            started_at: None,
            stopped_at: None,
            duration: None,
            screen_source: None,
            camera_device: None,
            audio_sources: Vec::new(),
            resolution,
            fps,
            error_message: None,
            created_media_clip_id: None,
        }
    }

    /// Start recording (transition from Preparing to Recording)
    pub fn start(&mut self) {
        self.status = RecordingStatus::Recording;
        self.started_at = Some(Utc::now());
    }

    /// Stop recording (transition from Recording to Stopped)
    pub fn stop(&mut self) {
        self.status = RecordingStatus::Stopped;
        self.stopped_at = Some(Utc::now());

        // Calculate final duration
        if let (Some(start), Some(stop)) = (self.started_at, self.stopped_at) {
            let duration_ms = (stop - start).num_milliseconds();
            self.duration = Some(duration_ms as f64 / 1000.0);
        }
    }

    /// Mark recording as failed with error message
    #[allow(dead_code)]
    pub fn fail(&mut self, error: String) {
        self.status = RecordingStatus::Failed;
        self.error_message = Some(error);
        self.stopped_at = Some(Utc::now());
    }

    /// Update current recording duration
    pub fn update_duration(&mut self, duration_seconds: f64) {
        self.duration = Some(duration_seconds);
    }

    /// Validate recording configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate resolution format
        if !self.resolution.contains('x') {
            return Err(format!("Invalid resolution format: {}", self.resolution));
        }

        // Validate FPS
        if ![15, 24, 30, 60].contains(&self.fps) {
            return Err(format!(
                "Invalid FPS: {}. Must be 15, 24, 30, or 60",
                self.fps
            ));
        }

        // Validate screen_webcam type has both sources
        if self.recording_type == RecordingType::ScreenWebcam {
            if self.screen_source.is_none() {
                return Err("Screen source required for screen_webcam recording".to_string());
            }
            if self.camera_device.is_none() {
                return Err("Camera device required for screen_webcam recording".to_string());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    #[serde(rename = "type")]
    pub recording_type: RecordingType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_source_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_device_id: Option<String>,

    pub audio_sources: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub microphone_device_id: Option<String>, // Add specific microphone selection

    pub settings: RecordingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub resolution: String,
    pub fps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSource {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSource {
    pub id: String,
    pub name: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSource {
    pub id: String,
    pub name: String,
    pub app: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSources {
    pub screens: Vec<ScreenSource>,
    pub windows: Vec<WindowSource>,
    pub cameras: Vec<RecordingSource>,
    pub microphones: Vec<RecordingSource>, // Add microphone list
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Screen,
    Camera,
    Microphone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResult {
    pub granted: PermissionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub screen: bool,
    pub camera: bool,
    pub microphone: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_session_lifecycle() {
        let mut session = RecordingSession::new(
            RecordingType::Screen,
            "/tmp/test.mp4".to_string(),
            "1920x1080".to_string(),
            30,
        );

        assert_eq!(session.status, RecordingStatus::Preparing);
        assert!(session.started_at.is_none());

        session.start();
        assert_eq!(session.status, RecordingStatus::Recording);
        assert!(session.started_at.is_some());

        session.stop();
        assert_eq!(session.status, RecordingStatus::Stopped);
        assert!(session.stopped_at.is_some());
        assert!(session.duration.is_some());
    }

    #[test]
    fn test_recording_session_validation() {
        let mut session = RecordingSession::new(
            RecordingType::Screen,
            "/tmp/test.mp4".to_string(),
            "1920x1080".to_string(),
            30,
        );

        // Valid configuration
        assert!(session.validate().is_ok());

        // Invalid FPS
        session.fps = 99;
        assert!(session.validate().is_err());

        // Invalid resolution
        session.fps = 30;
        session.resolution = "invalid".to_string();
        assert!(session.validate().is_err());
    }

    #[test]
    fn test_screen_webcam_validation() {
        let mut session = RecordingSession::new(
            RecordingType::ScreenWebcam,
            "/tmp/test.mp4".to_string(),
            "1920x1080".to_string(),
            30,
        );

        // Missing both sources
        assert!(session.validate().is_err());

        // Only screen source
        session.screen_source = Some("screen-1".to_string());
        assert!(session.validate().is_err());

        // Both sources
        session.camera_device = Some("camera-1".to_string());
        assert!(session.validate().is_ok());
    }
}
