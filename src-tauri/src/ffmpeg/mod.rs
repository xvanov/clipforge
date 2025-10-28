// FFmpeg integration module
// Provides video processing capabilities: metadata extraction, thumbnails, proxy generation, export

pub mod metadata;
pub mod proxy;
pub mod thumbnails;

pub use metadata::{extract_metadata, VideoMetadata};
pub use proxy::{generate_proxy, needs_proxy};
pub use thumbnails::{generate_thumbnail, ThumbnailQueue};
