use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum Error {
    #[error("chrome: {0}")]
    Chrome(String),
    #[error("mcp: {0}")]
    Mcp(String),
    #[error("launch: {0}")]
    Launch(#[from] std::io::Error),
    #[error("timeout: port={port} after {attempts} attempts")]
    Timeout { port: u16, attempts: u8 },
}

impl From<chromiumoxide::error::CdpError> for Error {
    fn from(e: chromiumoxide::error::CdpError) -> Self {
        Self::Chrome(e.to_string())
    }
}
