use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub track_type: TrackType,
    pub order: u32,
    pub clips: Vec<TimelineClip>,
    pub visible: bool,
    pub locked: bool,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackType {
    Main,
    Overlay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineClip {
    pub id: String,
    pub media_clip_id: String,
    pub track_id: String,
    pub start_time: f64,
    pub in_point: f64,
    pub out_point: f64,
    pub layer_order: u32,
    pub transform: Option<Transform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
}

impl Track {
    pub fn new(name: String, track_type: TrackType) -> Self {
        Track {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            track_type,
            order: 0,
            clips: Vec::new(),
            visible: true,
            locked: false,
            volume: 1.0,
        }
    }

    pub fn duration(&self) -> f64 {
        self.clips
            .iter()
            .map(|c| c.end_time())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }
}

impl TimelineClip {
    pub fn new(
        media_clip_id: String,
        track_id: String,
        start_time: f64,
        in_point: f64,
        out_point: f64,
    ) -> Self {
        TimelineClip {
            id: uuid::Uuid::new_v4().to_string(),
            media_clip_id,
            track_id,
            start_time,
            in_point,
            out_point,
            layer_order: 0,
            transform: None,
        }
    }

    pub fn duration(&self) -> f64 {
        self.out_point - self.in_point
    }

    pub fn end_time(&self) -> f64 {
        self.start_time + self.duration()
    }
}

