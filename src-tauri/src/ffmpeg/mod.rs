// FFmpeg integration module
// Provides video processing capabilities: metadata extraction, thumbnails, proxy generation, export

pub mod audio;
pub mod export;
pub mod metadata;
pub mod proxy;
pub mod thumbnails;

pub use audio::{extract_audio_to_wav, get_temp_audio_path};
pub use metadata::extract_metadata;
pub use proxy::{generate_proxy, needs_proxy};
pub use thumbnails::generate_thumbnail;
