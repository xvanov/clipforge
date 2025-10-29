// Recording-related TypeScript types

export type RecordingType = 'screen' | 'webcam' | 'screen_webcam';

export type RecordingStatus = 'preparing' | 'recording' | 'paused' | 'stopped' | 'failed';

export interface RecordingSession {
  id: string;
  type: RecordingType;
  status: RecordingStatus;
  output_path: string;
  started_at: string | null;
  stopped_at: string | null;
  duration: number | null;
  screen_source: string | null;
  camera_device: string | null;
  audio_sources: string[];
  resolution: string;
  fps: number;
  error_message: string | null;
  created_media_clip_id: string | null;
}

export interface RecordingConfig {
  type: RecordingType;
  screen_source_id: string | null;
  camera_device_id: string | null;
  audio_sources: string[];
  microphone_device_id: string | null; // Add specific microphone selection
  settings: RecordingSettings;
}

export interface RecordingSettings {
  resolution: string;
  fps: number;
}

export interface RecordingSource {
  id: string;
  name: string;
}

export interface ScreenSource {
  id: string;
  name: string;
  resolution: string;
}

export interface WindowSource {
  id: string;
  name: string;
  app: string;
}

export interface RecordingSources {
  screens: ScreenSource[];
  windows: WindowSource[];
  cameras: RecordingSource[];
  microphones: RecordingSource[]; // Add microphone list
}

export type Permission = 'screen' | 'camera' | 'microphone';

export interface PermissionStatus {
  screen: boolean;
  camera: boolean;
  microphone: boolean;
}

export interface PermissionResult {
  granted: PermissionStatus;
}

// Events

export interface RecordingStartedEvent {
  session_id: string;
}

export interface RecordingProgressEvent {
  session_id: string;
  duration: number;
}

export interface RecordingStoppedEvent {
  session_id: string;
  media_clip_id: string;
}
