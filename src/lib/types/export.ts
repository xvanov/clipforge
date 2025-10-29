// Export types matching Rust models

export type ExportResolution = 'source' | '2160p' | '1440p' | '1080p' | '720p' | '480p';

export type VideoCodec = 'h264' | 'hevc' | 'vp9';

export type ExportQuality = 'high' | 'medium' | 'low';

export type AudioCodec = 'aac' | 'mp3' | 'opus';

export interface ExportSettings {
  resolution: ExportResolution;
  codec: VideoCodec;
  quality: ExportQuality;
  fps?: number;
  audio_codec: AudioCodec;
  audio_bitrate: number;
  hardware_acceleration: boolean;
}

export interface ExportRequest {
  output_path: string;
  settings: ExportSettings;
}

export interface ExportJobResponse {
  job_id: string;
}

export interface ExportProgressEvent {
  job_id: string;
  progress: number; // 0.0 - 1.0
  current_frame: number;
  total_frames: number;
  fps: number;
  eta_seconds: number;
}

export interface ExportCompleteEvent {
  job_id: string;
  output_path: string;
}

export interface ExportErrorEvent {
  job_id: string;
  error: string;
}

export interface ExportCancelledEvent {
  job_id: string;
}

export const DEFAULT_EXPORT_SETTINGS: ExportSettings = {
  resolution: '1080p',
  codec: 'h264',
  quality: 'high',
  audio_codec: 'aac',
  audio_bitrate: 192,
  hardware_acceleration: true,
};

