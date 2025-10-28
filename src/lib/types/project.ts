import type { MediaClip } from './clip';
import type { Track } from './timeline';

export interface Project {
  id: string;
  name: string;
  created_at: string; // ISO 8601 datetime
  modified_at: string;
  file_path: string | null;
  version: string;
  tracks: Track[];
  media_library: MediaClip[];
  export_settings: ExportSettings;
  auto_save_enabled: boolean;
  last_auto_save: string | null;
}

export interface ExportSettings {
  resolution: Resolution;
  codec: Codec;
  quality: Quality;
  fps: number | null;
  audio_codec: AudioCodec;
  audio_bitrate: number;
  hardware_acceleration: boolean;
}

export type Resolution = 'source' | '2160p' | '1440p' | '1080p' | '720p' | '480p';
export type Codec = 'h264' | 'hevc' | 'vp9';
export type Quality = 'high' | 'medium' | 'low';
export type AudioCodec = 'aac' | 'mp3' | 'opus';

