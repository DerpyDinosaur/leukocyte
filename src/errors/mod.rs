pub mod reporter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LeukoError {
    /* Shim errors */
    #[error("Failed to get leuko's path")]
    ExePathUnavailable,

    #[error("PATH is not defined in the environment")]
    PathEnvMissing,

    #[error("'{0}' could not find package manager")]
    NotFound(String),

    #[error("'{0}' is not a supported package manager")]
    UnsupportedPackageManager(String),

    /* Network errors */
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    /* Parsing Error */
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    /* Not yet implemented errors */
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
