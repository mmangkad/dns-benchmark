//! Benchmark result types and statistics.

use crate::dns::{DnsServer, ServerSource};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Result of benchmarking a single DNS server
#[derive(Debug, Clone)]
pub struct ServerResult {
    /// Server name
    pub name: String,
    /// Server IP address
    pub ip: IpAddr,
    /// Server source
    pub source: ServerSource,
    /// Last successfully resolved IP
    pub resolved_ip: Option<IpAddr>,
    /// Total number of requests made
    pub total_requests: u32,
    /// Number of successful requests
    pub successful_requests: u32,
    /// Minimum response time
    pub min_time: Option<Duration>,
    /// Maximum response time
    pub max_time: Option<Duration>,
    /// Average response time
    pub avg_time: Option<Duration>,
    /// Last error message if any
    pub last_error: Option<String>,
}

impl ServerResult {
    /// Create a new server result from measurements
    pub fn from_measurements(server: &DnsServer, measurements: Vec<TimingResult>) -> Self {
        let total = measurements.len() as u32;
        let mut successful = 0u32;
        let mut total_time = Duration::ZERO;
        let mut min_time: Option<Duration> = None;
        let mut max_time: Option<Duration> = None;
        let mut resolved_ip: Option<IpAddr> = None;
        let mut last_error: Option<String> = None;

        for m in &measurements {
            match m {
                TimingResult::Success { duration, ip } => {
                    successful += 1;
                    total_time += *duration;
                    resolved_ip = Some(*ip);

                    min_time = Some(min_time.map_or(*duration, |min| min.min(*duration)));
                    max_time = Some(max_time.map_or(*duration, |max| max.max(*duration)));
                }
                TimingResult::Failure { error } => {
                    last_error = Some(error.clone());
                }
            }
        }

        let avg_time = if successful > 0 {
            Some(total_time / successful)
        } else {
            None
        };

        Self {
            name: server.name.clone(),
            ip: server.ip(),
            source: server.source,
            resolved_ip,
            total_requests: total,
            successful_requests: successful,
            min_time,
            max_time,
            avg_time,
            last_error,
        }
    }

    /// Get success rate as a percentage
    #[inline]
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Check if this server is from system DNS
    #[inline]
    pub fn is_system(&self) -> bool {
        matches!(self.source, ServerSource::System)
    }

    /// Check if this server is a gateway
    #[inline]
    pub fn is_gateway(&self) -> bool {
        matches!(self.source, ServerSource::Gateway)
    }

    /// Check if all requests failed
    #[inline]
    pub fn all_failed(&self) -> bool {
        self.successful_requests == 0
    }

    /// Get the sort key (avg time or max duration for failures)
    pub fn sort_key(&self) -> Duration {
        self.avg_time.unwrap_or(Duration::MAX)
    }
}

/// Result of a single timing measurement
#[derive(Debug, Clone)]
pub enum TimingResult {
    /// Successful resolution
    Success {
        duration: Duration,
        ip: IpAddr,
    },
    /// Failed resolution
    Failure {
        error: String,
    },
}

impl TimingResult {
    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        match self {
            Self::Success { .. } => false,
            Self::Failure { error } => {
                let lower = error.to_lowercase();
                lower.contains("timeout") || lower.contains("timed out")
            }
        }
    }
}

/// Complete benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Results for each server, sorted by average time
    pub servers: Vec<ServerResult>,
    /// Total benchmark duration
    pub duration: Duration,
    /// Domain that was tested
    pub domain: String,
    /// Number of requests per server
    pub requests_per_server: u32,
}

impl BenchmarkResult {
    /// Get the fastest server (lowest average time)
    pub fn fastest(&self) -> Option<&ServerResult> {
        self.servers.first()
    }

    /// Get servers that had 100% success rate
    pub fn fully_successful(&self) -> impl Iterator<Item = &ServerResult> {
        self.servers.iter().filter(|s| s.success_rate() >= 100.0)
    }

    /// Get servers that completely failed
    pub fn completely_failed(&self) -> impl Iterator<Item = &ServerResult> {
        self.servers.iter().filter(|s| s.all_failed())
    }
}

/// Serializable result entry for output formatters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableResult {
    pub name: String,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_ip: Option<String>,
    pub total_requests: u32,
    pub successful_requests: u32,
    pub success_rate: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<&ServerResult> for SerializableResult {
    fn from(r: &ServerResult) -> Self {
        Self {
            name: r.name.clone(),
            ip: r.ip.to_string(),
            resolved_ip: r.resolved_ip.map(|ip| ip.to_string()),
            total_requests: r.total_requests,
            successful_requests: r.successful_requests,
            success_rate: r.success_rate(),
            min_ms: r.min_time.map(|d| d.as_secs_f64() * 1000.0),
            max_ms: r.max_time.map(|d| d.as_secs_f64() * 1000.0),
            avg_ms: r.avg_time.map(|d| d.as_secs_f64() * 1000.0),
            error: if r.all_failed() { r.last_error.clone() } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn make_server() -> DnsServer {
        DnsServer::from_ip("Test", IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), ServerSource::Builtin)
    }

    #[test]
    fn test_server_result_all_success() {
        let server = make_server();
        let measurements = vec![
            TimingResult::Success {
                duration: Duration::from_millis(10),
                ip: "1.2.3.4".parse().unwrap(),
            },
            TimingResult::Success {
                duration: Duration::from_millis(20),
                ip: "1.2.3.4".parse().unwrap(),
            },
        ];

        let result = ServerResult::from_measurements(&server, measurements);

        assert_eq!(result.total_requests, 2);
        assert_eq!(result.successful_requests, 2);
        assert_eq!(result.success_rate(), 100.0);
        assert_eq!(result.min_time, Some(Duration::from_millis(10)));
        assert_eq!(result.max_time, Some(Duration::from_millis(20)));
        assert_eq!(result.avg_time, Some(Duration::from_millis(15)));
        assert!(result.resolved_ip.is_some());
        assert!(!result.all_failed());
    }

    #[test]
    fn test_server_result_all_failed() {
        let server = make_server();
        let measurements = vec![
            TimingResult::Failure { error: "timeout".to_string() },
            TimingResult::Failure { error: "timeout".to_string() },
        ];

        let result = ServerResult::from_measurements(&server, measurements);

        assert_eq!(result.total_requests, 2);
        assert_eq!(result.successful_requests, 0);
        assert_eq!(result.success_rate(), 0.0);
        assert!(result.min_time.is_none());
        assert!(result.avg_time.is_none());
        assert!(result.all_failed());
    }

    #[test]
    fn test_timing_result_is_timeout() {
        let timeout = TimingResult::Failure { error: "request timed out".to_string() };
        let other = TimingResult::Failure { error: "network error".to_string() };
        let success = TimingResult::Success {
            duration: Duration::from_millis(10),
            ip: "1.2.3.4".parse().unwrap(),
        };

        assert!(timeout.is_timeout());
        assert!(!other.is_timeout());
        assert!(!success.is_timeout());
    }
}
