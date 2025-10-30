// AI integration module
// Provides AI-powered features: speech-to-text captions

pub mod whisper;

#[allow(unused_imports)]
pub use whisper::{parse_srt_file, transcribe_audio, WhisperConfig};
