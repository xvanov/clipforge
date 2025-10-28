// FFmpeg thumbnail generation with async task queue
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc;
use tokio::task;

/// Request to generate a thumbnail
#[derive(Debug, Clone)]
pub struct ThumbnailRequest {
    pub clip_id: String,
    pub source_path: String,
    pub output_path: String,
    pub timestamp: f64,
}

/// Result of thumbnail generation
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    pub clip_id: String,
    pub thumbnail_path: String,
}

/// Async queue for thumbnail generation
pub struct ThumbnailQueue {
    tx: mpsc::UnboundedSender<ThumbnailRequest>,
}

impl ThumbnailQueue {
    /// Create a new thumbnail queue and spawn worker task
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Result<ThumbnailResult, String>>) {
        let (req_tx, mut req_rx) = mpsc::unbounded_channel::<ThumbnailRequest>();
        let (result_tx, result_rx) = mpsc::unbounded_channel::<Result<ThumbnailResult, String>>();

        // Spawn worker task
        task::spawn(async move {
            while let Some(request) = req_rx.recv().await {
                let result = generate_thumbnail_internal(
                    &request.source_path,
                    &request.output_path,
                    request.timestamp,
                )
                .await
                .map(|path| ThumbnailResult {
                    clip_id: request.clip_id.clone(),
                    thumbnail_path: path,
                });

                let _ = result_tx.send(result);
            }
        });

        (Self { tx: req_tx }, result_rx)
    }

    /// Submit a thumbnail generation request
    pub fn submit(&self, request: ThumbnailRequest) -> Result<(), String> {
        self.tx
            .send(request)
            .map_err(|e| format!("Failed to submit thumbnail request: {}", e))
    }
}

/// Generate thumbnail image from video at specified timestamp
pub async fn generate_thumbnail(
    source_path: &str,
    output_path: &str,
    timestamp: f64,
) -> Result<String, String> {
    generate_thumbnail_internal(source_path, output_path, timestamp).await
}

async fn generate_thumbnail_internal(
    source_path: &str,
    output_path: &str,
    timestamp: f64,
) -> Result<String, String> {
    // Validate input file exists
    if !Path::new(source_path).exists() {
        return Err(format!("Source file not found: {}", source_path));
    }

    // Create output directory if it doesn't exist
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Run ffmpeg to extract frame as JPEG
    // -ss: seek to timestamp
    // -i: input file
    // -vframes 1: extract one frame
    // -q:v 2: JPEG quality (2 is high quality)
    // -f image2: force image format
    let output = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output file
            "-ss",
            &timestamp.to_string(),
            "-i",
            source_path,
            "-vframes",
            "1",
            "-q:v",
            "2",
            "-f",
            "image2",
            output_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Verify output file was created
    if !Path::new(output_path).exists() {
        return Err("Thumbnail file was not created".to_string());
    }

    Ok(output_path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thumbnail_queue() {
        let (queue, mut results) = ThumbnailQueue::new();

        // This test would need a real video file to work
        // For now, just verify the queue accepts requests
        assert!(queue
            .submit(ThumbnailRequest {
                clip_id: "test-1".to_string(),
                source_path: "/nonexistent.mp4".to_string(),
                output_path: "/tmp/thumb.jpg".to_string(),
                timestamp: 0.0,
            })
            .is_ok());

        // Drop queue to close channel
        drop(queue);

        // Verify we get a result (will be error since file doesn't exist)
        if let Some(result) = results.recv().await {
            assert!(result.is_err());
        }
    }
}

