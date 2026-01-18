//! # DNS Benchmark
//!
//! A high-performance, async DNS benchmarking library and CLI tool.
//!
//! This crate provides functionality to benchmark DNS servers and find the fastest
//! ones for your network location. It supports multiple output formats, custom server
//! lists, and automatic detection of system DNS and gateway servers.
//!
//! ## Features
//!
//! - **Async-first design** - Leverages Tokio for efficient concurrent benchmarking
//! - **Multiple output formats** - Table, JSON, XML, CSV
//! - **Cross-platform** - Works on Linux, macOS, and Windows
//! - **Configurable** - Extensive CLI options with persistent configuration
//! - **Smart detection** - Auto-detects system DNS and gateway servers
//!
//! ## Author
//!
//! Mohammad Miadh Angkad <MAngkad.BSDSBA2027@aim.edu>

pub mod benchmark;
pub mod cli;
pub mod config;
pub mod dns;
pub mod error;
pub mod output;
pub mod platform;

// Re-exports for convenience
pub use benchmark::{BenchmarkEngine, BenchmarkResult, ServerResult};
pub use config::Config;
pub use dns::{DnsServer, IpVersion, Protocol};
pub use error::{Error, Result};
pub use output::{OutputFormat, OutputFormatter};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default domain to test DNS resolution
pub const DEFAULT_DOMAIN: &str = "google.com";

/// Default number of concurrent workers
pub const DEFAULT_WORKERS: u16 = 16;

/// Default number of requests per server
pub const DEFAULT_REQUESTS: u16 = 50;

/// Default timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 2;

#[cfg(test)]
mod tests {
    /// Load test fixture files
    #[macro_export]
    macro_rules! load_test_fixture {
        ($path:expr) => {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests", $path))
        };
    }
}
