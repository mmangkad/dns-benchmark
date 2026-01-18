//! JSON output formatter.

use super::OutputFormatter;
use crate::benchmark::{BenchmarkResult, SerializableResult};
use crate::config::Config;
use crate::error::OutputError;
use serde::Serialize;
use std::io::Write;
use std::net::IpAddr;

/// JSON output formatter
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn write(
        &self,
        result: &BenchmarkResult,
        _config: &Config,
        _system_ips: &[IpAddr],
        writer: &mut dyn Write,
    ) -> Result<(), OutputError> {
        let output = JsonOutput::from(result);
        let json = serde_json::to_string_pretty(&output)?;
        writeln!(writer, "{}", json)?;
        Ok(())
    }
}

/// JSON output structure
#[derive(Debug, Serialize)]
struct JsonOutput {
    /// Benchmark metadata
    meta: JsonMeta,
    /// Results for each server
    results: Vec<SerializableResult>,
}

#[derive(Debug, Serialize)]
struct JsonMeta {
    domain: String,
    requests_per_server: u32,
    total_servers: usize,
    duration_ms: f64,
}

impl From<&BenchmarkResult> for JsonOutput {
    fn from(result: &BenchmarkResult) -> Self {
        Self {
            meta: JsonMeta {
                domain: result.domain.clone(),
                requests_per_server: result.requests_per_server,
                total_servers: result.servers.len(),
                duration_ms: result.duration.as_secs_f64() * 1000.0,
            },
            results: result.servers.iter().map(SerializableResult::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::ServerResult;
    use crate::dns::ServerSource;
    use std::time::Duration;

    fn make_test_result() -> BenchmarkResult {
        BenchmarkResult {
            servers: vec![ServerResult {
                name: "Test".to_string(),
                ip: "8.8.8.8".parse().unwrap(),
                source: ServerSource::Builtin,
                resolved_ip: Some("1.2.3.4".parse().unwrap()),
                total_requests: 10,
                successful_requests: 9,
                min_time: Some(Duration::from_millis(5)),
                max_time: Some(Duration::from_millis(50)),
                avg_time: Some(Duration::from_millis(20)),
                last_error: None,
            }],
            duration: Duration::from_secs(1),
            domain: "google.com".to_string(),
            requests_per_server: 10,
        }
    }

    #[test]
    fn test_json_output() {
        let result = make_test_result();
        let config = Config::default();
        let mut output = Vec::new();

        JsonFormatter.write(&result, &config, &[], &mut output).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        assert!(json_str.contains("\"domain\": \"google.com\""));
        assert!(json_str.contains("\"name\": \"Test\""));
    }
}
