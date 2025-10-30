use crate::ai::whisper::{parse_srt_file, transcribe_audio, WhisperConfig};
use crate::commands::media::AppState;
use crate::ffmpeg::{extract_audio_to_wav, get_temp_audio_path};
use crate::models::caption::Caption;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CaptionGenerationRequest {
    pub clip_id: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionGenerationProgress {
    pub job_id: String,
    pub progress: f64,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionGenerationResult {
    pub job_id: String,
    pub captions: Vec<Caption>,
}

/// Generate captions for a media clip using AI speech-to-text
#[tauri::command]
pub async fn generate_captions(
    clip_id: String,
    language: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    println!(
        "[CAPTIONS] generate_captions called for clip: {}, language: {}",
        clip_id, language
    );

    let job_id = uuid::Uuid::new_v4().to_string();

    // Find the media clip
    let media_library = state.media_library.lock().unwrap();
    let clip = media_library
        .iter()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| format!("Media clip not found: {}", clip_id))?;

    let source_path = clip.source_path.clone();
    let clip_id_copy = clip_id.clone();
    let has_audio = clip.has_audio;
    drop(media_library);

    println!(
        "[CAPTIONS] Clip found: {}, has_audio: {}",
        source_path, has_audio
    );

    // Validate clip has audio
    if !has_audio {
        return Err("Media clip does not contain audio track".to_string());
    }

    // Spawn background task for caption generation
    let job_id_clone = job_id.clone();
    let language_clone = language.clone();
    let state_clone = Arc::new(state.inner().clone());

    tokio::spawn(async move {
        println!(
            "[CAPTIONS] Background task started for job: {}",
            job_id_clone
        );

        let result = generate_captions_task(
            &job_id_clone,
            &clip_id_copy,
            &source_path,
            &language_clone,
            app_handle.clone(),
            state_clone.clone(),
        )
        .await;

        match result {
            Ok(captions) => {
                println!(
                    "[CAPTIONS] Caption generation successful! Generated {} captions",
                    captions.len()
                );

                // Emit success event
                let _ = app_handle.emit_all(
                    "caption_generation_complete",
                    CaptionGenerationResult {
                        job_id: job_id_clone.clone(),
                        captions: captions.clone(),
                    },
                );

                // Update media clip with captions
                let mut media_library = state_clone.media_library.lock().unwrap();
                if let Some(clip) = media_library.iter_mut().find(|c| c.id == clip_id_copy) {
                    clip.captions = captions;
                    println!("[CAPTIONS] Updated media clip with captions");
                }
            }
            Err(e) => {
                println!("[CAPTIONS] Caption generation FAILED: {}", e);

                // Emit error event
                let _ = app_handle.emit_all(
                    "caption_generation_error",
                    CaptionGenerationProgress {
                        job_id: job_id_clone,
                        progress: 0.0,
                        status: "error".to_string(),
                        message: Some(e),
                    },
                );
            }
        }
    });

    println!("[CAPTIONS] Returning job_id: {}", job_id);
    Ok(job_id)
}

/// Background task to generate captions
async fn generate_captions_task(
    job_id: &str,
    clip_id: &str,
    source_path: &str,
    language: &str,
    app_handle: tauri::AppHandle,
    _state: Arc<AppState>,
) -> Result<Vec<Caption>, String> {
    println!("[CAPTIONS TASK] Starting for clip: {}", clip_id);

    // Step 1: Extract audio
    emit_progress(
        &app_handle,
        job_id,
        0.1,
        "extracting_audio",
        Some("Extracting audio from video..."),
    );
    println!("[CAPTIONS TASK] Step 1: Extracting audio...");

    let audio_path = get_temp_audio_path(clip_id);
    println!("[CAPTIONS TASK] Audio path: {:?}", audio_path);

    match extract_audio_to_wav(source_path, audio_path.to_str().unwrap()).await {
        Ok(_) => println!("[CAPTIONS TASK] Audio extracted successfully"),
        Err(e) => {
            println!("[CAPTIONS TASK] Audio extraction FAILED: {}", e);
            return Err(e);
        }
    }

    // Step 2: Transcribe audio with Whisper
    emit_progress(
        &app_handle,
        job_id,
        0.3,
        "transcribing",
        Some("Transcribing audio with AI..."),
    );
    println!("[CAPTIONS TASK] Step 2: Transcribing with Whisper...");

    let whisper_config = WhisperConfig {
        language: language.to_string(),
        ..Default::default()
    };

    println!(
        "[CAPTIONS TASK] Whisper config: executable={}, model={}, lang={}",
        whisper_config.executable_path, whisper_config.model_path, whisper_config.language
    );

    let srt_path = match transcribe_audio(&audio_path, &whisper_config).await {
        Ok(path) => {
            println!(
                "[CAPTIONS TASK] Transcription successful! SRT file: {:?}",
                path
            );
            path
        }
        Err(e) => {
            println!("[CAPTIONS TASK] Transcription FAILED: {}", e);
            let _ = tokio::fs::remove_file(audio_path).await;
            return Err(e);
        }
    };

    // Step 3: Parse SRT file
    emit_progress(
        &app_handle,
        job_id,
        0.9,
        "parsing",
        Some("Parsing captions..."),
    );
    println!("[CAPTIONS TASK] Step 3: Parsing SRT file...");

    let captions = match parse_srt_file(&srt_path, clip_id.to_string(), language.to_string()).await
    {
        Ok(caps) => {
            println!("[CAPTIONS TASK] Parsed {} captions", caps.len());
            caps
        }
        Err(e) => {
            println!("[CAPTIONS TASK] Parsing FAILED: {}", e);
            let _ = tokio::fs::remove_file(audio_path).await;
            let _ = tokio::fs::remove_file(srt_path).await;
            return Err(e);
        }
    };

    // Step 4: Cleanup
    emit_progress(
        &app_handle,
        job_id,
        1.0,
        "complete",
        Some("Caption generation complete!"),
    );
    println!("[CAPTIONS TASK] Step 4: Cleanup...");

    // Clean up temporary files
    let _ = tokio::fs::remove_file(audio_path).await;
    let _ = tokio::fs::remove_file(srt_path).await;

    println!("[CAPTIONS TASK] Task completed successfully!");

    Ok(captions)
}

/// Emit progress event
fn emit_progress(
    app_handle: &tauri::AppHandle,
    job_id: &str,
    progress: f64,
    status: &str,
    message: Option<&str>,
) {
    let _ = app_handle.emit_all(
        "caption_generation_progress",
        CaptionGenerationProgress {
            job_id: job_id.to_string(),
            progress,
            status: status.to_string(),
            message: message.map(|s| s.to_string()),
        },
    );
}

/// Update caption text and timing
#[tauri::command]
pub async fn update_caption(
    clip_id: String,
    caption_id: String,
    text: Option<String>,
    start_time: Option<f64>,
    end_time: Option<f64>,
    state: State<'_, AppState>,
) -> Result<Caption, String> {
    let mut media_library = state.media_library.lock().unwrap();

    let clip = media_library
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| format!("Media clip not found: {}", clip_id))?;

    let caption = clip
        .captions
        .iter_mut()
        .find(|c| c.id == caption_id)
        .ok_or_else(|| format!("Caption not found: {}", caption_id))?;

    // Update fields if provided
    if let Some(new_text) = text {
        caption.text = new_text;
    }
    if let Some(new_start) = start_time {
        caption.start_time = new_start;
    }
    if let Some(new_end) = end_time {
        caption.end_time = new_end;
    }

    // Validate updated caption
    caption.validate()?;

    Ok(caption.clone())
}

/// Delete a caption
#[tauri::command]
pub async fn delete_caption(
    clip_id: String,
    caption_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut media_library = state.media_library.lock().unwrap();

    let clip = media_library
        .iter_mut()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| format!("Media clip not found: {}", clip_id))?;

    let initial_len = clip.captions.len();
    clip.captions.retain(|c| c.id != caption_id);

    if clip.captions.len() == initial_len {
        return Err(format!("Caption not found: {}", caption_id));
    }

    Ok(())
}
