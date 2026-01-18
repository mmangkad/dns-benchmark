//! Output formatting for benchmark results.

mod csv;
mod json;
mod table;
mod xml;

pub use self::csv::CsvFormatter;
pub use self::json::JsonFormatter;
pub use self::table::TableFormatter;
pub use self::xml::XmlFormatter;

use crate::benchmark::BenchmarkResult;
use crate::config::Config;
use crate::error::OutputError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::net::IpAddr;
use std::str::FromStr;

/// Output format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable table
    #[default]
    Table,
    /// JSON format
    Json,
    /// XML format
    Xml,
    /// CSV format
    Csv,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::Json => write!(f, "json"),
            Self::Xml => write!(f, "xml"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" | "human" | "human-readable" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "xml" => Ok(Self::Xml),
            "csv" => Ok(Self::Csv),
            _ => Err(crate::Error::InvalidArgument(format!("Invalid output format: {s}"))),
        }
    }
}

/// Trait for output formatters
pub trait OutputFormatter {
    /// Write benchmark results to the given writer
    fn write(
        &self,
        result: &BenchmarkResult,
        config: &Config,
        system_ips: &[IpAddr],
        writer: &mut dyn Write,
    ) -> Result<(), OutputError>;
}

/// Get the appropriate formatter for a format
pub fn get_formatter(format: OutputFormat) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Table => Box::new(TableFormatter),
        OutputFormat::Json => Box::new(JsonFormatter),
        OutputFormat::Xml => Box::new(XmlFormatter),
        OutputFormat::Csv => Box::new(CsvFormatter),
    }
}

/// Format a duration in milliseconds with appropriate precision
pub fn format_duration_ms(ms: f64) -> String {
    if ms < 1.0 {
        format!("{:.3}ms", ms)
    } else if ms < 10.0 {
        format!("{:.2}ms", ms)
    } else if ms < 100.0 {
        format!("{:.1}ms", ms)
    } else {
        format!("{:.0}ms", ms)
    }
}

/// Get color code based on response time
pub fn get_time_color(ms: f64) -> console::Color {
    if ms <= 30.0 {
        console::Color::Green
    } else if ms <= 80.0 {
        console::Color::Yellow
    } else {
        console::Color::Red
    }
}

/// Get color code based on success rate
pub fn get_success_color(rate: f64) -> console::Color {
    if rate >= 100.0 {
        console::Color::Green
    } else if rate >= 50.0 {
        console::Color::Yellow
    } else if rate >= 20.0 {
        console::Color::Red
    } else {
        console::Color::Magenta
    }
}
