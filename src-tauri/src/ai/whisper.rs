use crate::models::caption::Caption;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tokio::fs;

/// Whisper.cpp configuration
pub struct WhisperConfig {
    /// Path to whisper.cpp executable
    pub executable_path: String,
    /// Path to the model file (e.g., ggml-base.en.bin)
    pub model_path: String,
    /// Language code (e.g., "en", "es", "auto" for auto-detect)
    pub language: String,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        // Try to load config from ~/.clipforge/config.json
        if let Some(config) = Self::load_from_config() {
            return config;
        }

        // Fallback to default values
        Self {
            executable_path: "whisper-cli".to_string(),
            model_path: "~/.clipforge/models/ggml-base.en.bin".to_string(),
            language: "en".to_string(),
        }
    }
}

impl WhisperConfig {
    /// Load whisper config from ~/.clipforge/config.json
    fn load_from_config() -> Option<Self> {
        use std::fs;
        use std::path::PathBuf;

        let config_path: PathBuf = dirs::home_dir()?.join(".clipforge").join("config.json");

        if !config_path.exists() {
            return None;
        }

        let content = fs::read_to_string(config_path).ok()?;
        let config_json: serde_json::Value = serde_json::from_str(&content).ok()?;

        let whisper_config = config_json.get("whisper")?;

        Some(Self {
            executable_path: whisper_config.get("executable_path")?.as_str()?.to_string(),
            model_path: whisper_config
                .get("model_path")?
                .as_str()?
                .replace("~", &dirs::home_dir()?.to_string_lossy()),
            language: whisper_config
                .get("default_language")
                .and_then(|v| v.as_str())
                .unwrap_or("en")
                .to_string(),
        })
    }
}

/// Transcribe audio file using whisper.cpp
/// Returns path to SRT subtitle file
pub async fn transcribe_audio(
    audio_path: &Path,
    config: &WhisperConfig,
) -> Result<PathBuf, String> {
    // Validate input file
    if !audio_path.exists() {
        return Err(format!("Audio file not found: {}", audio_path.display()));
    }

    // Check if whisper executable exists
    let whisper_check = Command::new(&config.executable_path).arg("--help").output();

    if whisper_check.is_err() {
        return Err(format!(
            "Whisper.cpp not found at '{}'. Please install whisper.cpp and ensure it's in PATH or specify the correct path.",
            config.executable_path
        ));
    }

    // Output SRT file path
    // Note: whisper-cli appends .srt to the full filename, so audio.wav becomes audio.wav.srt
    let mut output_srt = audio_path.to_path_buf();
    output_srt.set_extension("wav.srt");

    // Run whisper.cpp CLI
    // Key arguments:
    // -m: model file path
    // -f: input audio file
    // -osrt: output SRT subtitle file
    // -l: language (or "auto" for detection)
    // -ml: max line length for captions
    let args = vec![
        "-m",
        &config.model_path,
        "-f",
        audio_path.to_str().unwrap(),
        "-osrt", // Output SRT format
        "-l",
        &config.language,
        "-ml",
        "50", // Max 50 chars per line
        "-t",
        "4", // Use 4 threads
    ];

    println!(
        "[WHISPER] Running command: {} {}",
        config.executable_path,
        args.join(" ")
    );
    println!("[WHISPER] Model path: {}", config.model_path);
    println!("[WHISPER] Audio path: {}", audio_path.display());

    let output = Command::new(&config.executable_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute whisper.cpp: {}", e))?;

    println!("[WHISPER] Command exit status: {:?}", output.status);

    // Capture both stdout and stderr for debugging
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("[WHISPER] STDOUT:\n{}", stdout);
    }
    if !stderr.is_empty() {
        println!("[WHISPER] STDERR:\n{}", stderr);
    }

    if !output.status.success() {
        return Err(format!(
            "Whisper.cpp transcription failed with exit code {:?}\nSTDERR: {}\nSTDOUT: {}",
            output.status.code(),
            stderr,
            stdout
        ));
    }

    // Verify SRT file was created
    if !output_srt.exists() {
        return Err(format!(
            "Transcription failed: SRT file not created at {}\nSTDERR: {}\nSTDOUT: {}",
            output_srt.display(),
            stderr,
            stdout
        ));
    }

    println!(
        "[WHISPER] SRT file created successfully: {}",
        output_srt.display()
    );
    Ok(output_srt)
}

/// Parse SRT subtitle file into Caption structs
pub async fn parse_srt_file(
    srt_path: &Path,
    media_clip_id: String,
    language: String,
) -> Result<Vec<Caption>, String> {
    let content = fs::read_to_string(srt_path)
        .await
        .map_err(|e| format!("Failed to read SRT file: {}", e))?;

    parse_srt_content(&content, media_clip_id, language)
}

/// Parse SRT content string into Caption structs
fn parse_srt_content(
    content: &str,
    media_clip_id: String,
    language: String,
) -> Result<Vec<Caption>, String> {
    let mut captions = Vec::new();
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 3 {
            continue; // Invalid block
        }

        // Line 0: Sequence number (ignore)
        // Line 1: Timestamp (00:00:01,500 --> 00:00:04,200)
        // Line 2+: Caption text

        let timestamp_line = lines[1];
        let text = lines[2..].join(" ");

        // Parse timestamps
        if let Some((start, end)) = parse_srt_timestamp(timestamp_line) {
            let caption = Caption::new(
                media_clip_id.clone(),
                text.trim().to_string(),
                start,
                end,
                language.clone(),
            );
            captions.push(caption);
        }
    }

    if captions.is_empty() {
        return Err("No captions found in SRT file".to_string());
    }

    Ok(captions)
}

/// Parse SRT timestamp line: "00:00:01,500 --> 00:00:04,200"
/// Returns (start_seconds, end_seconds)
fn parse_srt_timestamp(line: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = line.split(" --> ").collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parse_srt_time(parts[0])?;
    let end = parse_srt_time(parts[1])?;

    Some((start, end))
}

/// Parse SRT time format: "00:00:01,500" to seconds
fn parse_srt_time(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.split(',').collect();
    if parts.len() != 2 {
        return None;
    }

    let time_parts: Vec<&str> = parts[0].split(':').collect();
    if time_parts.len() != 3 {
        return None;
    }

    let hours: f64 = time_parts[0].parse().ok()?;
    let minutes: f64 = time_parts[1].parse().ok()?;
    let seconds: f64 = time_parts[2].parse().ok()?;
    let milliseconds: f64 = parts[1].parse().ok()?;

    Some(hours * 3600.0 + minutes * 60.0 + seconds + milliseconds / 1000.0)
}

/// Clean up temporary SRT file
#[allow(dead_code)]
pub async fn cleanup_srt_file(srt_path: &Path) -> Result<(), String> {
    if srt_path.exists() {
        fs::remove_file(srt_path)
            .await
            .map_err(|e| format!("Failed to remove SRT file: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_srt_time() {
        assert_eq!(parse_srt_time("00:00:01,500"), Some(1.5));
        assert_eq!(parse_srt_time("00:01:30,250"), Some(90.25));
        assert_eq!(parse_srt_time("01:00:00,000"), Some(3600.0));
    }

    #[test]
    fn test_parse_srt_timestamp() {
        let line = "00:00:01,500 --> 00:00:04,200";
        let result = parse_srt_timestamp(line);
        assert_eq!(result, Some((1.5, 4.2)));
    }

    #[test]
    fn test_parse_srt_content() {
        let srt = r#"1
00:00:01,500 --> 00:00:04,200
Hello world

2
00:00:05,000 --> 00:00:08,500
This is a test caption"#;

        let captions = parse_srt_content(srt, "clip-123".to_string(), "en".to_string()).unwrap();
        assert_eq!(captions.len(), 2);
        assert_eq!(captions[0].text, "Hello world");
        assert_eq!(captions[0].start_time, 1.5);
        assert_eq!(captions[0].end_time, 4.2);
        assert_eq!(captions[1].text, "This is a test caption");
    }
}
