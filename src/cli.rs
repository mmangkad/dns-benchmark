//! Command-line interface definitions.

use crate::config::{ConfigOverrides, TableStyle};
use crate::dns::{IpVersion, Protocol};
use crate::output::OutputFormat;

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

const ABOUT: &str = r#"
🌐 DNS Benchmark - Find the fastest DNS servers for your location

A high-performance DNS benchmarking tool that tests response times
across multiple DNS providers to help optimize your internet experience.
"#;

const AFTER_HELP: &str = r#"
EXAMPLES:
    dns-benchmark                           # Run with default settings
    dns-benchmark --requests 100            # Run 100 requests per server
    dns-benchmark --format json             # Output as JSON
    dns-benchmark --custom-servers dns.txt  # Use custom server list
    dns-benchmark config init               # Create config file
    dns-benchmark config set --workers 8    # Update config
"#;

/// DNS Benchmark CLI
#[derive(Debug, Parser)]
#[command(
    name = "dns-benchmark",
    about = ABOUT,
    after_help = AFTER_HELP,
    version,
    author,
    propagate_version = true,
)]
pub struct Cli {
    #[command(flatten)]
    pub options: BenchOptions,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Benchmark options
#[derive(Debug, Args)]
pub struct BenchOptions {
    /// Domain to test DNS resolution
    #[arg(short, long, value_name = "DOMAIN")]
    pub domain: Option<String>,

    /// Number of concurrent workers
    #[arg(short, long, value_name = "NUM", value_parser = clap::value_parser!(u16).range(1..=256))]
    pub workers: Option<u16>,

    /// Number of requests per DNS server
    #[arg(short, long, value_name = "NUM", value_parser = clap::value_parser!(u16).range(1..=1000))]
    pub requests: Option<u16>,

    /// Timeout in seconds for each request
    #[arg(short, long, value_name = "SECS", value_parser = clap::value_parser!(u64).range(1..=60))]
    pub timeout: Option<u64>,

    /// DNS protocol to use
    #[arg(short, long, value_enum)]
    pub protocol: Option<CliProtocol>,

    /// IP version for name servers
    #[arg(long = "ns-ip", value_enum)]
    pub name_server_ip: Option<CliIpVersion>,

    /// IP version for lookups
    #[arg(long = "lookup-ip", value_enum)]
    pub lookup_ip: Option<CliIpVersion>,

    /// Output format
    #[arg(short, long, value_enum)]
    pub format: Option<CliFormat>,

    /// Table style (for table output)
    #[arg(short, long, value_enum)]
    pub style: Option<CliStyle>,

    /// Path to custom DNS server list file
    #[arg(long, value_name = "FILE")]
    pub custom_servers: Option<PathBuf>,

    /// Skip system DNS detection
    #[arg(long)]
    pub skip_system: bool,

    /// Skip gateway DNS detection
    #[arg(long)]
    pub skip_gateway: bool,

    /// Disable adaptive timeout optimization
    #[arg(long)]
    pub no_adaptive_timeout: bool,

    /// Save current options to config file
    #[arg(long)]
    pub save_config: bool,
}

impl BenchOptions {
    /// Convert to ConfigOverrides
    pub fn to_overrides(&self) -> ConfigOverrides {
        ConfigOverrides {
            domain: self.domain.clone(),
            workers: self.workers,
            requests: self.requests,
            timeout: self.timeout,
            protocol: self.protocol.map(Into::into),
            name_server_ip: self.name_server_ip.map(Into::into),
            lookup_ip: self.lookup_ip.map(Into::into),
            format: self.format.map(Into::into),
            style: self.style.map(Into::into),
            custom_servers: self.custom_servers.clone(),
            skip_system: self.skip_system,
            skip_gateway: self.skip_gateway,
            disable_adaptive_timeout: self.no_adaptive_timeout,
        }
    }
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommand),
}

/// Config subcommands
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Initialize config file with default values
    Init,

    /// Display current configuration
    Show,

    /// Update configuration values
    Set(ConfigSetArgs),

    /// Reset configuration to defaults
    Reset,

    /// Delete configuration file
    Delete,

    /// Show config file path
    Path,
}

/// Arguments for config set command
#[derive(Debug, Args)]
pub struct ConfigSetArgs {
    #[command(flatten)]
    pub options: BenchOptions,
}

// CLI enum types that map to internal types

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliProtocol {
    Udp,
    Tcp,
}

impl From<CliProtocol> for Protocol {
    fn from(p: CliProtocol) -> Self {
        match p {
            CliProtocol::Udp => Protocol::Udp,
            CliProtocol::Tcp => Protocol::Tcp,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliIpVersion {
    V4,
    V6,
}

impl From<CliIpVersion> for IpVersion {
    fn from(v: CliIpVersion) -> Self {
        match v {
            CliIpVersion::V4 => IpVersion::V4,
            CliIpVersion::V6 => IpVersion::V6,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliFormat {
    Table,
    Json,
    Xml,
    Csv,
}

impl From<CliFormat> for OutputFormat {
    fn from(f: CliFormat) -> Self {
        match f {
            CliFormat::Table => OutputFormat::Table,
            CliFormat::Json => OutputFormat::Json,
            CliFormat::Xml => OutputFormat::Xml,
            CliFormat::Csv => OutputFormat::Csv,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliStyle {
    Empty,
    Blank,
    Ascii,
    Psql,
    Markdown,
    Modern,
    Sharp,
    Rounded,
    ModernRounded,
    Extended,
    Dots,
    Rst,
    AsciiRounded,
}

impl From<CliStyle> for TableStyle {
    fn from(s: CliStyle) -> Self {
        match s {
            CliStyle::Empty => TableStyle::Empty,
            CliStyle::Blank => TableStyle::Blank,
            CliStyle::Ascii => TableStyle::Ascii,
            CliStyle::Psql => TableStyle::Psql,
            CliStyle::Markdown => TableStyle::Markdown,
            CliStyle::Modern => TableStyle::Modern,
            CliStyle::Sharp => TableStyle::Sharp,
            CliStyle::Rounded => TableStyle::Rounded,
            CliStyle::ModernRounded => TableStyle::ModernRounded,
            CliStyle::Extended => TableStyle::Extended,
            CliStyle::Dots => TableStyle::Dots,
            CliStyle::Rst => TableStyle::ReStructuredText,
            CliStyle::AsciiRounded => TableStyle::AsciiRounded,
        }
    }
}
