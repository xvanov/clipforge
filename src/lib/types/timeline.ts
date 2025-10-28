export interface Track {
  id: string;
  name: string;
  type: TrackType;
  order: number;
  clips: TimelineClip[];
  visible: boolean;
  locked: boolean;
  volume: number;
}

export type TrackType = 'main' | 'overlay';

export interface TimelineClip {
  id: string;
  media_clip_id: string;
  track_id: string;
  start_time: number;
  in_point: number;
  out_point: number;
  layer_order: number;
  transform: Transform | null;
}

export interface Transform {
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
}

