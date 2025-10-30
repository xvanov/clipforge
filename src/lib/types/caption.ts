// Caption types for AI Speech-to-Text

export interface Caption {
  id: string;
  media_clip_id: string;
  text: string;
  start_time: number;
  end_time: number;
  confidence?: number;
  language: string;
  styling?: CaptionStyle;
}

export interface CaptionStyle {
  font: string;
  size: number;
  color: string;
  background_color?: string;
  position: CaptionPosition;
  alignment: CaptionAlignment;
}

export type CaptionPosition = 'top' | 'center' | 'bottom';
export type CaptionAlignment = 'left' | 'center' | 'right';

export interface CaptionGenerationRequest {
  clip_id: string;
  language: string;
}

export interface CaptionGenerationProgress {
  job_id: string;
  progress: number;
  status: 'extracting_audio' | 'transcribing' | 'complete' | 'error';
  message?: string;
}

export interface CaptionGenerationResult {
  job_id: string;
  captions: Caption[];
}
