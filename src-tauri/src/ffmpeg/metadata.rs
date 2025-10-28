// FFmpeg metadata extraction using ffprobe
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub duration: f64,
    pub resolution: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
    pub audio_codec: Option<String>,
    pub bitrate: Option<u64>,
    pub has_audio: bool,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

/// Extract metadata from video file using ffprobe
pub async fn extract_metadata(file_path: &str) -> Result<VideoMetadata, String> {
    // Run ffprobe to get JSON output
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            file_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_output = String::from_utf8_lossy(&output.stdout);
    let ffprobe_data: FfprobeOutput = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    // Find video and audio streams
    let video_stream = ffprobe_data
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video"))
        .ok_or("No video stream found")?;

    let audio_stream = ffprobe_data
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"));

    // Extract video properties
    let width = video_stream.width.ok_or("Width not found")?;
    let height = video_stream.height.ok_or("Height not found")?;
    let codec = video_stream.codec_name.clone().ok_or("Codec not found")?;

    // Parse frame rate (e.g., "30/1" -> 30.0)
    let fps = if let Some(fps_str) = &video_stream.r_frame_rate {
        parse_frame_rate(fps_str)?
    } else {
        30.0 // Default fallback
    };

    // Parse duration
    let duration = ffprobe_data
        .format
        .duration
        .as_ref()
        .and_then(|d| d.parse::<f64>().ok())
        .ok_or("Duration not found")?;

    // Parse bitrate
    let bitrate = video_stream
        .bit_rate
        .as_ref()
        .or(ffprobe_data.format.bit_rate.as_ref())
        .and_then(|b| b.parse::<u64>().ok());

    Ok(VideoMetadata {
        duration,
        resolution: format!("{}x{}", width, height),
        width,
        height,
        fps,
        codec,
        audio_codec: audio_stream.and_then(|s| s.codec_name.clone()),
        bitrate,
        has_audio: audio_stream.is_some(),
    })
}

/// Parse frame rate string like "30/1" or "30000/1001"
fn parse_frame_rate(fps_str: &str) -> Result<f64, String> {
    let parts: Vec<&str> = fps_str.split('/').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid frame rate format: {}", fps_str));
    }

    let numerator: f64 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid numerator: {}", parts[0]))?;
    let denominator: f64 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid denominator: {}", parts[1]))?;

    if denominator == 0.0 {
        return Err("Frame rate denominator cannot be zero".to_string());
    }

    Ok(numerator / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frame_rate() {
        assert_eq!(parse_frame_rate("30/1").unwrap(), 30.0);
        assert_eq!(parse_frame_rate("60/1").unwrap(), 60.0);
        // NTSC frame rate
        assert!((parse_frame_rate("30000/1001").unwrap() - 29.97).abs() < 0.01);
    }
}
