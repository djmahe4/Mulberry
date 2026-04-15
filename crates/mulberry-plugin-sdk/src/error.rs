use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Failed to load plugin library: {0}")]
    LoadError(String),

    #[error("Plugin symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Plugin initialization failed: {0}")]
    InitError(String),

    #[error("Plugin manifest invalid: {0}")]
    ManifestError(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
