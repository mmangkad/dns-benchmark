//! Configuration management.

use crate::dns::{IpVersion, Protocol};
use crate::error::{ConfigError, Error};
use crate::output::OutputFormat;
use crate::{DEFAULT_DOMAIN, DEFAULT_REQUESTS, DEFAULT_TIMEOUT_SECS, DEFAULT_WORKERS};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration directory name
const CONFIG_DIR: &str = ".dns-benchmark";

/// Configuration file name
const CONFIG_FILE: &str = "config.toml";

/// Application configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Domain to test resolution
    pub domain: String,

    /// Number of concurrent workers
    pub workers: u16,

    /// Number of requests per server
    pub requests: u16,

    /// Timeout in seconds
    pub timeout: u64,

    /// DNS protocol (UDP or TCP)
    pub protocol: Protocol,

    /// IP version for name servers
    pub name_server_ip: IpVersion,

    /// IP version for lookups
    pub lookup_ip: IpVersion,

    /// Output format
    pub format: OutputFormat,

    /// Table style (for human-readable output)
    pub style: TableStyle,

    /// Path to custom servers file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_servers: Option<PathBuf>,

    /// Skip system DNS detection
    #[serde(default)]
    pub skip_system: bool,

    /// Skip gateway detection
    #[serde(default)]
    pub skip_gateway: bool,

    /// Disable adaptive timeout
    #[serde(default)]
    pub disable_adaptive_timeout: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            domain: DEFAULT_DOMAIN.to_string(),
            workers: DEFAULT_WORKERS,
            requests: DEFAULT_REQUESTS,
            timeout: DEFAULT_TIMEOUT_SECS,
            protocol: Protocol::default(),
            name_server_ip: IpVersion::default(),
            lookup_ip: IpVersion::default(),
            format: OutputFormat::default(),
            style: TableStyle::default(),
            custom_servers: None,
            skip_system: false,
            skip_gateway: false,
            disable_adaptive_timeout: false,
        }
    }
}

impl Config {
    /// Create a new config builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Get the path to the config file
    pub fn path() -> Result<PathBuf, ConfigError> {
        let user_dirs = UserDirs::new().ok_or(ConfigError::NoHomeDirectory)?;
        Ok(user_dirs.home_dir().join(CONFIG_DIR).join(CONFIG_FILE))
    }

    /// Check if config file exists
    pub fn exists() -> Result<bool, ConfigError> {
        Ok(Self::path()?.exists())
    }

    /// Load config from file
    pub fn load() -> Result<Self, Error> {
        let path = Self::path()?;
        if !path.exists() {
            return Err(Error::Config(ConfigError::NotFound(path)));
        }
        Self::load_from(&path)
    }

    /// Load config from a specific path
    pub fn load_from(path: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(path).map_err(|e| {
            ConfigError::ReadError {
                path: path.to_path_buf(),
                source: e,
            }
        })?;
        let config: Self = toml::from_str(&content).map_err(ConfigError::ParseError)?;
        Ok(config)
    }

    /// Load config or return default
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    /// Save config to default path
    pub fn save(&self) -> Result<(), Error> {
        let path = Self::path()?;
        self.save_to(&path)
    }

    /// Save config to a specific path
    pub fn save_to(&self, path: &Path) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ConfigError::WriteError {
                    path: parent.to_path_buf(),
                    source: e,
                }
            })?;
        }

        let content = toml::to_string_pretty(self).map_err(ConfigError::SerializeError)?;
        fs::write(path, content).map_err(|e| {
            ConfigError::WriteError {
                path: path.to_path_buf(),
                source: e,
            }
        })?;
        Ok(())
    }

    /// Delete config file
    pub fn delete() -> Result<(), Error> {
        let path = Self::path()?;
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                ConfigError::WriteError { path, source: e }
            })?;
        }
        Ok(())
    }

    /// Merge with another config (other takes precedence where set)
    pub fn merge(&mut self, other: &ConfigOverrides) {
        if let Some(domain) = &other.domain {
            self.domain.clone_from(domain);
        }
        if let Some(workers) = other.workers {
            self.workers = workers;
        }
        if let Some(requests) = other.requests {
            self.requests = requests;
        }
        if let Some(timeout) = other.timeout {
            self.timeout = timeout;
        }
        if let Some(protocol) = other.protocol {
            self.protocol = protocol;
        }
        if let Some(ip) = other.name_server_ip {
            self.name_server_ip = ip;
        }
        if let Some(ip) = other.lookup_ip {
            self.lookup_ip = ip;
        }
        if let Some(format) = other.format {
            self.format = format;
        }
        if let Some(style) = other.style {
            self.style = style;
        }
        if let Some(ref path) = other.custom_servers {
            self.custom_servers = Some(path.clone());
        }
        if other.skip_system {
            self.skip_system = true;
        }
        if other.skip_gateway {
            self.skip_gateway = true;
        }
        if other.disable_adaptive_timeout {
            self.disable_adaptive_timeout = true;
        }
    }

    /// Get timeout in milliseconds
    #[inline]
    pub const fn timeout_ms(&self) -> u64 {
        self.timeout * 1000
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "domain: {}", self.domain)?;
        writeln!(f, "workers: {}", self.workers)?;
        writeln!(f, "requests: {}", self.requests)?;
        writeln!(f, "timeout: {}s", self.timeout)?;
        writeln!(f, "protocol: {}", self.protocol)?;
        writeln!(f, "name_server_ip: {}", self.name_server_ip)?;
        writeln!(f, "lookup_ip: {}", self.lookup_ip)?;
        writeln!(f, "format: {}", self.format)?;
        writeln!(f, "style: {}", self.style)?;
        if let Some(ref path) = self.custom_servers {
            writeln!(f, "custom_servers: {}", path.display())?;
        }
        writeln!(f, "skip_system: {}", self.skip_system)?;
        writeln!(f, "skip_gateway: {}", self.skip_gateway)?;
        write!(f, "disable_adaptive_timeout: {}", self.disable_adaptive_timeout)
    }
}

/// Configuration overrides (all optional)
#[derive(Debug, Default, Clone)]
pub struct ConfigOverrides {
    pub domain: Option<String>,
    pub workers: Option<u16>,
    pub requests: Option<u16>,
    pub timeout: Option<u64>,
    pub protocol: Option<Protocol>,
    pub name_server_ip: Option<IpVersion>,
    pub lookup_ip: Option<IpVersion>,
    pub format: Option<OutputFormat>,
    pub style: Option<TableStyle>,
    pub custom_servers: Option<PathBuf>,
    pub skip_system: bool,
    pub skip_gateway: bool,
    pub disable_adaptive_timeout: bool,
}

/// Builder for creating Config
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.config.domain = domain.into();
        self
    }

    pub fn workers(mut self, workers: u16) -> Self {
        self.config.workers = workers;
        self
    }

    pub fn requests(mut self, requests: u16) -> Self {
        self.config.requests = requests;
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.config.protocol = protocol;
        self
    }

    pub fn name_server_ip(mut self, ip: IpVersion) -> Self {
        self.config.name_server_ip = ip;
        self
    }

    pub fn lookup_ip(mut self, ip: IpVersion) -> Self {
        self.config.lookup_ip = ip;
        self
    }

    pub fn format(mut self, format: OutputFormat) -> Self {
        self.config.format = format;
        self
    }

    pub fn style(mut self, style: TableStyle) -> Self {
        self.config.style = style;
        self
    }

    pub fn custom_servers(mut self, path: PathBuf) -> Self {
        self.config.custom_servers = Some(path);
        self
    }

    pub fn skip_system(mut self, skip: bool) -> Self {
        self.config.skip_system = skip;
        self
    }

    pub fn skip_gateway(mut self, skip: bool) -> Self {
        self.config.skip_gateway = skip;
        self
    }

    pub fn disable_adaptive_timeout(mut self, disable: bool) -> Self {
        self.config.disable_adaptive_timeout = disable;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}

/// Table output styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TableStyle {
    Empty,
    Blank,
    Ascii,
    Psql,
    Markdown,
    Modern,
    Sharp,
    #[default]
    Rounded,
    ModernRounded,
    Extended,
    Dots,
    ReStructuredText,
    AsciiRounded,
}

impl fmt::Display for TableStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Blank => write!(f, "blank"),
            Self::Ascii => write!(f, "ascii"),
            Self::Psql => write!(f, "psql"),
            Self::Markdown => write!(f, "markdown"),
            Self::Modern => write!(f, "modern"),
            Self::Sharp => write!(f, "sharp"),
            Self::Rounded => write!(f, "rounded"),
            Self::ModernRounded => write!(f, "modern-rounded"),
            Self::Extended => write!(f, "extended"),
            Self::Dots => write!(f, "dots"),
            Self::ReStructuredText => write!(f, "rst"),
            Self::AsciiRounded => write!(f, "ascii-rounded"),
        }
    }
}

impl std::str::FromStr for TableStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "empty" => Ok(Self::Empty),
            "blank" => Ok(Self::Blank),
            "ascii" => Ok(Self::Ascii),
            "psql" => Ok(Self::Psql),
            "markdown" | "md" => Ok(Self::Markdown),
            "modern" => Ok(Self::Modern),
            "sharp" => Ok(Self::Sharp),
            "rounded" => Ok(Self::Rounded),
            "modern-rounded" | "modernrounded" => Ok(Self::ModernRounded),
            "extended" => Ok(Self::Extended),
            "dots" => Ok(Self::Dots),
            "rst" | "restructuredtext" => Ok(Self::ReStructuredText),
            "ascii-rounded" | "asciirounded" => Ok(Self::AsciiRounded),
            _ => Err(Error::InvalidArgument(format!("Invalid table style: {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.domain, DEFAULT_DOMAIN);
        assert_eq!(config.workers, DEFAULT_WORKERS);
        assert_eq!(config.requests, DEFAULT_REQUESTS);
        assert_eq!(config.timeout, DEFAULT_TIMEOUT_SECS);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .domain("example.com")
            .workers(8)
            .requests(100)
            .timeout(5)
            .protocol(Protocol::Tcp)
            .build();

        assert_eq!(config.domain, "example.com");
        assert_eq!(config.workers, 8);
        assert_eq!(config.requests, 100);
        assert_eq!(config.timeout, 5);
        assert_eq!(config.protocol, Protocol::Tcp);
    }

    #[test]
    fn test_config_merge() {
        let mut config = Config::default();
        let overrides = ConfigOverrides {
            domain: Some("test.com".to_string()),
            workers: Some(4),
            ..Default::default()
        };

        config.merge(&overrides);

        assert_eq!(config.domain, "test.com");
        assert_eq!(config.workers, 4);
        assert_eq!(config.requests, DEFAULT_REQUESTS); // Unchanged
    }

    #[test]
    fn test_table_style_parsing() {
        assert_eq!(TableStyle::from_str("rounded").unwrap(), TableStyle::Rounded);
        assert_eq!(TableStyle::from_str("MARKDOWN").unwrap(), TableStyle::Markdown);
        assert!(TableStyle::from_str("invalid").is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config, parsed);
    }
}
