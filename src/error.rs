//! Error types for the DNS benchmarking library.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the library
#[derive(Debug, Error)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// DNS resolution error
    #[error("DNS error: {0}")]
    Dns(#[from] DnsError),

    /// Output formatting error
    #[error("Output error: {0}")]
    Output(#[from] OutputError),

    /// Platform detection error
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to find home directory
    #[error("Could not determine home directory")]
    NoHomeDirectory,

    /// Failed to read config file
    #[error("Failed to read config file at {path}: {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Failed to write config file
    #[error("Failed to write config file at {path}: {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Failed to parse config file
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Failed to serialize config
    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    /// Config file does not exist
    #[error("Config file does not exist at {0}")]
    NotFound(PathBuf),

    /// Invalid config value
    #[error("Invalid config value for '{key}': {message}")]
    InvalidValue { key: String, message: String },
}

/// DNS-related errors
#[derive(Debug, Error)]
pub enum DnsError {
    /// Resolution timeout
    #[error("DNS resolution timed out")]
    Timeout,

    /// No response received
    #[error("No response from DNS server")]
    NoResponse,

    /// Resolution failed
    #[error("DNS resolution failed: {0}")]
    ResolutionFailed(String),

    /// Invalid server address
    #[error("Invalid DNS server address: {0}")]
    InvalidAddress(String),

    /// Failed to read custom servers file
    #[error("Failed to read custom servers file at {path}: {message}")]
    CustomFileError { path: PathBuf, message: String },

    /// Invalid line in custom servers file
    #[error("Invalid line format at line {line}: expected 'name;address:port'")]
    InvalidLineFormat { line: usize },
}

/// Output formatting errors
#[derive(Debug, Error)]
pub enum OutputError {
    /// IO error during output
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// CSV writing error
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// XML writing error
    #[error("XML error: {0}")]
    Xml(String),

    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Platform detection errors
#[derive(Debug, Error)]
pub enum PlatformError {
    /// System DNS detection failed
    #[error("Failed to detect system DNS servers: {0}")]
    SystemDnsDetection(String),

    /// Gateway detection failed
    #[error("Failed to detect gateway: {0}")]
    GatewayDetection(String),

    /// Unsupported platform
    #[error("Unsupported platform")]
    UnsupportedPlatform,

    /// Command execution failed
    #[error("Failed to execute command '{command}': {message}")]
    CommandFailed { command: String, message: String },

    /// Parse error
    #[error("Failed to parse output: {0}")]
    ParseError(String),
}

/// Result type alias using our Error
pub type Result<T> = std::result::Result<T, Error>;

impl From<hickory_resolver::ResolveError> for DnsError {
    fn from(e: hickory_resolver::ResolveError) -> Self {
        let msg = e.to_string().to_lowercase();
        if msg.contains("timeout") || msg.contains("timed out") {
            DnsError::Timeout
        } else if msg.contains("no connections") || msg.contains("no response") {
            DnsError::NoResponse
        } else {
            DnsError::ResolutionFailed(e.to_string())
        }
    }
}

impl From<hickory_resolver::ResolveError> for Error {
    fn from(e: hickory_resolver::ResolveError) -> Self {
        Error::Dns(e.into())
    }
}
