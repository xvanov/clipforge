use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tokio::fs;

/// Extract audio from video file to WAV format for speech recognition
pub async fn extract_audio_to_wav(video_path: &str, output_path: &str) -> Result<PathBuf, String> {
    // Validate input file exists
    if !Path::new(video_path).exists() {
        return Err(format!("Video file not found: {}", video_path));
    }

    // Create output directory if needed
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // FFmpeg command to extract audio as 16-bit PCM WAV (required by whisper.cpp)
    // -vn: no video
    // -acodec pcm_s16le: 16-bit PCM little-endian
    // -ar 16000: 16kHz sample rate (optimal for speech recognition)
    // -ac 1: mono audio (reduces file size, sufficient for speech)
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_path,
            "-vn", // No video
            "-acodec",
            "pcm_s16le", // 16-bit PCM
            "-ar",
            "16000", // 16kHz sample rate
            "-ac",
            "1",  // Mono
            "-y", // Overwrite output file
            output_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg audio extraction failed: {}", stderr));
    }

    // Verify output file was created
    let output_path_buf = PathBuf::from(output_path);
    if !output_path_buf.exists() {
        return Err("Audio extraction failed: output file not created".to_string());
    }

    Ok(output_path_buf)
}

/// Get temporary audio file path for a clip
pub fn get_temp_audio_path(clip_id: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("clipforge_audio_{}.wav", clip_id));
    path
}

/// Clean up temporary audio file
#[allow(dead_code)]
pub async fn cleanup_temp_audio(audio_path: &Path) -> Result<(), String> {
    if audio_path.exists() {
        fs::remove_file(audio_path)
            .await
            .map_err(|e| format!("Failed to remove temp audio file: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_audio_path() {
        let path = get_temp_audio_path("test-clip-123");
        assert!(path
            .to_str()
            .unwrap()
            .contains("clipforge_audio_test-clip-123.wav"));
    }

    // Note: Actual extraction tests require FFmpeg and sample video files
    // These should be integration tests run in CI with proper fixtures
}
