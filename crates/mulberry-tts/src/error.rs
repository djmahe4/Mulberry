use thiserror::Error;

#[derive(Debug, Error)]
pub enum TtsError {
    #[error("TTS engine not available: {0}")]
    EngineNotAvailable(String),

    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),

    #[error("Invalid input text: {0}")]
    InvalidInput(String),

    #[error("External process error: {0}")]
    ProcessError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
