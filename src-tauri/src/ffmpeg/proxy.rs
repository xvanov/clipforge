// FFmpeg proxy video generation for web-compatible playback
// Converts non-web-compatible formats (MOV, ProRes, etc.) to H.264/MP4
use std::path::Path;
use std::process::Command;

/// Check if a video format needs a proxy for web playback
/// Returns true for codecs that aren't natively supported in browsers
pub fn needs_proxy(codec: &str) -> bool {
    let codec_lower = codec.to_lowercase();

    // Web-compatible codecs (no proxy needed)
    let web_compatible = ["h264", "vp8", "vp9", "av1"];

    // If codec is not in the web-compatible list, we need a proxy
    !web_compatible.iter().any(|c| codec_lower.contains(c))
}

/// Generate a web-compatible proxy video (H.264/MP4)
/// This allows MOV, ProRes, HEVC, and other formats to play in the browser
pub async fn generate_proxy(source_path: &str, output_path: &str) -> Result<String, String> {
    // Validate input file exists
    if !Path::new(source_path).exists() {
        return Err(format!("Source file not found: {}", source_path));
    }

    // Create output directory if it doesn't exist
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Generate H.264/AAC proxy at 1080p max resolution
    // - Fast encoding preset for reasonable generation time
    // - Scale down to 1080p max (maintains aspect ratio)
    // - Constant Rate Factor (CRF) 23 for good quality/size balance
    let output = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output file
            "-i",
            source_path, // Input file
            "-c:v",
            "libx264", // H.264 video codec
            "-preset",
            "fast", // Fast encoding (good speed/quality)
            "-crf",
            "23", // Quality level (lower = better)
            "-vf",
            "scale='min(1920,iw)':'min(1080,ih)':force_original_aspect_ratio=decrease", // Scale to max 1080p
            "-c:a",
            "aac", // AAC audio codec
            "-b:a",
            "128k", // Audio bitrate
            "-movflags",
            "+faststart", // Enable progressive download
            "-pix_fmt",
            "yuv420p", // Ensure compatibility
            output_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg proxy generation failed: {}", stderr));
    }

    // Verify output file was created
    if !Path::new(output_path).exists() {
        return Err("Proxy file was not created".to_string());
    }

    Ok(output_path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_proxy() {
        // Web-compatible codecs (no proxy needed)
        assert!(!needs_proxy("h264"));
        assert!(!needs_proxy("H264"));
        assert!(!needs_proxy("vp8"));
        assert!(!needs_proxy("vp9"));
        assert!(!needs_proxy("av1"));

        // Non-web-compatible codecs (proxy needed)
        assert!(needs_proxy("hevc"));
        assert!(needs_proxy("prores"));
        assert!(needs_proxy("mpeg4"));
        assert!(needs_proxy("mjpeg"));
        assert!(needs_proxy("dnxhd"));
    }

    #[test]
    fn test_proxy_path_validation() {
        let result =
            tokio_test::block_on(generate_proxy("/nonexistent/file.mov", "/tmp/proxy.mp4"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
