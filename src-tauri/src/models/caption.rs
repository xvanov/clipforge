use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a timestamped text caption/subtitle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Caption {
    /// Unique caption identifier
    pub id: String,
    /// Parent media clip reference
    pub media_clip_id: String,
    /// Caption text content
    pub text: String,
    /// Caption start time in seconds (relative to clip)
    pub start_time: f64,
    /// Caption end time in seconds
    pub end_time: f64,
    /// Speech recognition confidence (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Language code (ISO 639-1, e.g., "en", "es")
    pub language: String,
    /// Caption styling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styling: Option<CaptionStyle>,
}

/// Caption styling options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionStyle {
    /// Font family (default: "Arial")
    pub font: String,
    /// Font size in points (default: 24)
    pub size: u32,
    /// Text color (hex format, e.g., "#FFFFFF")
    pub color: String,
    /// Background color (hex or "transparent")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// Screen position
    pub position: CaptionPosition,
    /// Text alignment
    pub alignment: CaptionAlignment,
}

/// Caption position on screen
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptionPosition {
    Top,
    Center,
    Bottom,
}

/// Text alignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptionAlignment {
    Left,
    Center,
    Right,
}

impl Caption {
    /// Create a new caption
    pub fn new(
        media_clip_id: String,
        text: String,
        start_time: f64,
        end_time: f64,
        language: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            media_clip_id,
            text,
            start_time,
            end_time,
            confidence: None,
            language,
            styling: Some(CaptionStyle::default()),
        }
    }

    /// Get caption duration
    #[allow(dead_code)]
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Get word count
    #[allow(dead_code)]
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    /// Validate caption
    pub fn validate(&self) -> Result<(), String> {
        if self.text.is_empty() {
            return Err("Caption text cannot be empty".to_string());
        }
        if self.text.len() > 500 {
            return Err("Caption text exceeds maximum length (500 chars)".to_string());
        }
        if self.start_time < 0.0 {
            return Err("Start time cannot be negative".to_string());
        }
        if self.end_time <= self.start_time {
            return Err("End time must be greater than start time".to_string());
        }
        if let Some(conf) = self.confidence {
            if !(0.0..=1.0).contains(&conf) {
                return Err("Confidence must be between 0.0 and 1.0".to_string());
            }
        }
        Ok(())
    }
}

impl Default for CaptionStyle {
    fn default() -> Self {
        Self {
            font: "Arial".to_string(),
            size: 24,
            color: "#FFFFFF".to_string(),
            background_color: Some("transparent".to_string()),
            position: CaptionPosition::Bottom,
            alignment: CaptionAlignment::Center,
        }
    }
}

impl CaptionStyle {
    /// Validate caption style
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        if self.font.is_empty() {
            return Err("Font cannot be empty".to_string());
        }
        if !(12..=72).contains(&self.size) {
            return Err("Font size must be between 12 and 72".to_string());
        }
        // Validate hex color format
        let hex_pattern = regex::Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();
        if !hex_pattern.is_match(&self.color) {
            return Err("Color must be in hex format (#RRGGBB)".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caption_creation() {
        let caption = Caption::new(
            "clip-123".to_string(),
            "Hello world".to_string(),
            0.0,
            2.5,
            "en".to_string(),
        );
        assert_eq!(caption.media_clip_id, "clip-123");
        assert_eq!(caption.text, "Hello world");
        assert_eq!(caption.duration(), 2.5);
        assert_eq!(caption.word_count(), 2);
    }

    #[test]
    fn test_caption_validation() {
        let mut caption = Caption::new(
            "clip-123".to_string(),
            "Valid caption".to_string(),
            0.0,
            2.5,
            "en".to_string(),
        );
        assert!(caption.validate().is_ok());

        // Empty text
        caption.text = "".to_string();
        assert!(caption.validate().is_err());

        // Invalid time range
        caption.text = "Valid caption".to_string();
        caption.end_time = caption.start_time;
        assert!(caption.validate().is_err());
    }
}
