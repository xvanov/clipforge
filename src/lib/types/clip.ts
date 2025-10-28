// TypeScript types matching Rust models

export interface MediaClip {
  id: string;
  name: string;
  source_path: string;
  proxy_path: string | null;
  thumbnail_path: string | null;
  duration: number;
  resolution: string;
  width: number;
  height: number;
  fps: number;
  codec: string;
  audio_codec: string | null;
  file_size: number;
  bitrate: number | null;
  has_audio: boolean;
  imported_at: string; // ISO 8601 datetime
}

