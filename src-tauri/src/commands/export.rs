use crate::ffmpeg::export::{
    build_export_command, calculate_timeline_duration, generate_concat_file, parse_progress,
    ExportJob, ExportStatus,
};
use crate::models::export::ExportSettings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

/// Shared state for export jobs
#[derive(Clone)]
pub struct ExportState {
    jobs: Arc<Mutex<HashMap<String, ExportJobHandle>>>,
}

struct ExportJobHandle {
    job: ExportJob,
    process: Option<Child>,
}

impl ExportState {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Export timeline request
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub output_path: String,
    pub settings: ExportSettings,
}

/// Export job response
#[derive(Debug, Serialize)]
pub struct ExportJobResponse {
    pub job_id: String,
}

/// Export progress event payload
#[derive(Debug, Clone, Serialize)]
pub struct ExportProgressEvent {
    pub job_id: String,
    pub progress: f64,
    pub current_frame: u64,
    pub total_frames: u64,
    pub fps: f64,
    pub eta_seconds: u64,
}

/// Export complete event payload
#[derive(Debug, Clone, Serialize)]
pub struct ExportCompleteEvent {
    pub job_id: String,
    pub output_path: String,
}

/// Export error event payload
#[derive(Debug, Clone, Serialize)]
pub struct ExportErrorEvent {
    pub job_id: String,
    pub error: String,
}

/// Export cancelled event payload
#[derive(Debug, Clone, Serialize)]
pub struct ExportCancelledEvent {
    pub job_id: String,
}

/// Export timeline to video file
#[tauri::command]
pub async fn export_timeline(
    request: ExportRequest,
    export_state: State<'_, ExportState>,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<ExportJobResponse, String> {
    // Get project data directly from the live AppState (not from cached copy)
    let project = app_state
        .project
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No project loaded".to_string())?;

    eprintln!("[Export] Project has {} tracks", project.tracks.len());
    eprintln!(
        "[Export] Media library has {} clips",
        project.media_library.len()
    );

    // Validate output path
    let output_path = PathBuf::from(&request.output_path);
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            return Err(format!(
                "Output directory does not exist: {}",
                parent.display()
            ));
        }
    }

    // Create temporary directory for concat file
    let temp_dir = std::env::temp_dir().join(format!("clipforge_export_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Generate concat file
    let concat_file = generate_concat_file(&project.tracks, &project.media_library, &temp_dir)?;

    // Build FFmpeg command
    let cmd = build_export_command(&concat_file, &output_path, &request.settings)?;

    // Create export job
    let job_id = uuid::Uuid::new_v4().to_string();
    let job = ExportJob {
        id: job_id.clone(),
        output_path: request.output_path.clone(),
        status: ExportStatus::Preparing,
    };

    // Store job in state
    {
        let mut jobs = export_state.jobs.lock().unwrap();
        jobs.insert(
            job_id.clone(),
            ExportJobHandle {
                job: job.clone(),
                process: None,
            },
        );
    }

    // Calculate total duration for progress tracking
    let total_duration = calculate_timeline_duration(&project.tracks);

    // Spawn export task
    let job_id_clone = job_id.clone();
    let app_handle_clone = app_handle.clone();
    let export_state_arc = Arc::new(export_state.inner().clone());
    let export_state_for_complete = export_state_arc.clone();
    let export_state_for_error = export_state_arc.clone();
    let output_path_clone = request.output_path.clone();

    tokio::spawn(async move {
        match run_export(
            cmd,
            job_id_clone.clone(),
            total_duration,
            app_handle_clone.clone(),
            export_state_arc,
        )
        .await
        {
            Ok(_) => {
                // Emit completion event
                let _ = app_handle_clone.emit_all(
                    "export_complete",
                    ExportCompleteEvent {
                        job_id: job_id_clone.clone(),
                        output_path: output_path_clone,
                    },
                );

                // Update job status
                let mut jobs = export_state_for_complete.jobs.lock().unwrap();
                if let Some(handle) = jobs.get_mut(&job_id_clone) {
                    handle.job.status = ExportStatus::Complete;
                }
            }
            Err(e) => {
                // Emit error event
                let _ = app_handle_clone.emit_all(
                    "export_error",
                    ExportErrorEvent {
                        job_id: job_id_clone.clone(),
                        error: e.clone(),
                    },
                );

                // Update job status
                let mut jobs = export_state_for_error.jobs.lock().unwrap();
                if let Some(handle) = jobs.get_mut(&job_id_clone) {
                    handle.job.status = ExportStatus::Failed;
                }

                // Clean up partial file
                let _ = std::fs::remove_file(&output_path_clone);
            }
        }

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);
    });

    Ok(ExportJobResponse { job_id })
}

/// Run export process and emit progress events
async fn run_export(
    cmd: Command,
    job_id: String,
    total_duration: f64,
    app_handle: AppHandle,
    export_state: Arc<ExportState>,
) -> Result<(), String> {
    // Log the FFmpeg command for debugging
    eprintln!("[Export] FFmpeg command: {:?}", cmd);

    // Convert to tokio command for async execution
    let mut tokio_cmd = TokioCommand::from(cmd);

    let mut child = tokio_cmd
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn FFmpeg process: {}", e))?;

    // Update job status
    {
        let mut jobs = export_state.jobs.lock().unwrap();
        if let Some(handle) = jobs.get_mut(&job_id) {
            handle.job.status = ExportStatus::Rendering;
        }
    }

    // Collect all FFmpeg output for error reporting
    let mut all_output = String::new();

    // Read stderr for progress and errors
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            // Store all output for error reporting
            all_output.push_str(&line);
            all_output.push('\n');

            // Also log to console for debugging
            eprintln!("[FFmpeg] {}", line);

            // Parse progress
            if let Some(progress) = parse_progress(&line, total_duration) {
                // Emit progress event
                let _ = app_handle.emit_all(
                    "export_progress",
                    ExportProgressEvent {
                        job_id: job_id.clone(),
                        progress: progress.progress,
                        current_frame: progress.current_frame,
                        total_frames: progress.total_frames,
                        fps: progress.fps,
                        eta_seconds: progress.eta_seconds,
                    },
                );
            }
        }
    }

    // Wait for process to complete
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for FFmpeg process: {}", e))?;

    if !status.success() {
        // Return detailed error with FFmpeg output
        let error_msg = if all_output.is_empty() {
            format!("FFmpeg export failed with status: {}", status)
        } else {
            // Get last 10 lines of output for error message
            let lines: Vec<&str> = all_output.lines().rev().take(10).collect();
            let recent_output = lines.into_iter().rev().collect::<Vec<_>>().join("\n");
            format!(
                "FFmpeg export failed with status: {}\n\nRecent output:\n{}",
                status, recent_output
            )
        };
        return Err(error_msg);
    }

    Ok(())
}

/// Cancel ongoing export
#[tauri::command]
pub async fn cancel_export(
    job_id: String,
    export_state: State<'_, ExportState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut jobs = export_state.jobs.lock().unwrap();

    let handle = jobs
        .get_mut(&job_id)
        .ok_or_else(|| format!("Export job not found: {}", job_id))?;

    // Kill the FFmpeg process
    if let Some(mut process) = handle.process.take() {
        process
            .kill()
            .map_err(|e| format!("Failed to kill export process: {}", e))?;
    }

    // Update status
    handle.job.status = ExportStatus::Cancelled;

    // Clean up partial output file
    if std::fs::remove_file(&handle.job.output_path).is_ok() {
        // File deleted successfully
    }

    // Emit cancelled event
    let _ = app_handle.emit_all(
        "export_cancelled",
        ExportCancelledEvent {
            job_id: job_id.clone(),
        },
    );

    Ok(())
}
