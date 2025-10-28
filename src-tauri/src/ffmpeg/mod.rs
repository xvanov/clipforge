// FFmpeg integration module
// Provides video processing capabilities: metadata extraction, thumbnails, export

pub mod metadata;
pub mod thumbnails;

pub use metadata::{extract_metadata, VideoMetadata};
pub use thumbnails::{generate_thumbnail, ThumbnailQueue};
