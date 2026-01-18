//! CSV output formatter.

use super::OutputFormatter;
use crate::benchmark::BenchmarkResult;
use crate::config::Config;
use crate::error::OutputError;
use serde::Serialize;
use std::io::Write;
use std::net::IpAddr;

/// CSV output formatter
pub struct CsvFormatter;

impl OutputFormatter for CsvFormatter {
    fn write(
        &self,
        result: &BenchmarkResult,
        _config: &Config,
        _system_ips: &[IpAddr],
        writer: &mut dyn Write,
    ) -> Result<(), OutputError> {
        let mut csv_writer = csv::Writer::from_writer(writer);

        for server in &result.servers {
            let row = CsvRow {
                name: server.name.clone(),
                ip: server.ip.to_string(),
                resolved_ip: server.resolved_ip.map(|ip| ip.to_string()),
                total_requests: server.total_requests,
                successful_requests: server.successful_requests,
                success_rate: server.success_rate(),
                min_ms: server.min_time.map(|d| d.as_secs_f64() * 1000.0),
                max_ms: server.max_time.map(|d| d.as_secs_f64() * 1000.0),
                avg_ms: server.avg_time.map(|d| d.as_secs_f64() * 1000.0),
                error: if server.all_failed() {
                    server.last_error.clone()
                } else {
                    None
                },
            };
            csv_writer.serialize(row)?;
        }

        csv_writer.flush()?;
        Ok(())
    }
}

/// CSV row structure
#[derive(Debug, Serialize)]
struct CsvRow {
    name: String,
    ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_ip: Option<String>,
    total_requests: u32,
    successful_requests: u32,
    success_rate: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
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
    fn test_csv_output() {
        let result = make_test_result();
        let config = Config::default();
        let mut output = Vec::new();

        CsvFormatter.write(&result, &config, &[], &mut output).unwrap();

        let csv_str = String::from_utf8(output).unwrap();
        assert!(csv_str.contains("name,ip"));
        assert!(csv_str.contains("Test,8.8.8.8"));
    }
}
