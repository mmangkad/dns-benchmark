//! Async benchmark execution engine.

use super::resolver::create_resolver;
use super::result::{BenchmarkResult, ServerResult, TimingResult};
use crate::config::Config;
use crate::dns::DnsServer;
use crate::output::OutputFormat;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// Adaptive timeout configuration
const REDUCE_TIMEOUT_AFTER_FAILURES: u32 = 8;
const REDUCED_TIMEOUT_MS: u64 = 500;
const MINIMIZE_TIMEOUT_AFTER_FAILURES: u32 = 16;
const MINIMAL_TIMEOUT_MS: u64 = 100;

/// Progress bar tick interval
const PROGRESS_TICK_MS: u64 = 80;

/// Async benchmark engine
pub struct BenchmarkEngine {
    config: Config,
    servers: Vec<DnsServer>,
}

impl BenchmarkEngine {
    /// Create a new benchmark engine
    pub fn new(config: Config, servers: Vec<DnsServer>) -> Self {
        Self { config, servers }
    }

    /// Run the benchmark
    pub async fn run(self) -> BenchmarkResult {
        let start_time = Instant::now();
        let server_count = self.servers.len();

        // Print config summary for human-readable output
        if self.config.format == OutputFormat::Table {
            self.print_config_summary();
        }

        // Create multi-progress for per-server progress bars
        let multi_progress = Arc::new(MultiProgress::new());
        let results: Arc<Mutex<Vec<ServerResult>>> = Arc::new(Mutex::new(Vec::with_capacity(server_count)));

        // Semaphore to limit concurrent benchmarks
        let semaphore = Arc::new(Semaphore::new(self.config.workers as usize));

        // Spawn benchmark tasks
        let mut tasks = JoinSet::new();

        for server in self.servers {
            let config = self.config.clone();
            let results = Arc::clone(&results);
            let semaphore = Arc::clone(&semaphore);
            let mp = Arc::clone(&multi_progress);

            tasks.spawn(async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await.unwrap();

                // Create per-server progress bar
                let pb = if config.format == OutputFormat::Table {
                    let pb = mp.add(ProgressBar::new(config.requests as u64));
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template("{spinner:.cyan} {msg:<40} [{bar:25.cyan/blue}] {pos}/{len}")
                            .unwrap()
                            .progress_chars("━━╸"),
                    );
                    pb.set_message(format!("{} ({})", server.name, server.ip()));
                    pb.enable_steady_tick(Duration::from_millis(PROGRESS_TICK_MS));
                    Some(pb)
                } else {
                    None
                };

                // Run benchmark for this server
                let server_result = benchmark_server(&server, &config, pb.as_ref()).await;

                // Store result
                results.lock().push(server_result);

                // Finish and remove progress bar
                if let Some(pb) = pb {
                    pb.finish_and_clear();
                }
            });
        }

        // Wait for all tasks to complete
        while tasks.join_next().await.is_some() {}

        // Sort results by average time
        let mut servers = Arc::try_unwrap(results)
            .expect("All tasks completed")
            .into_inner();
        servers.sort_by_key(|r| r.sort_key());

        let duration = start_time.elapsed();

        BenchmarkResult {
            servers,
            duration,
            domain: self.config.domain.clone(),
            requests_per_server: self.config.requests as u32,
        }
    }

    /// Print configuration summary
    fn print_config_summary(&self) {
        println!(
            "\n{} DNS benchmark\n",
            style("Starting").cyan().bold()
        );
        println!(
            "  {} {}",
            style("Domain:").dim(),
            style(&self.config.domain).green()
        );
        println!(
            "  {} {} servers × {} requests = {} total",
            style("Scope:").dim(),
            style(self.servers.len()).yellow(),
            style(self.config.requests).yellow(),
            style(self.servers.len() * self.config.requests as usize).yellow().bold()
        );
        println!(
            "  {} {} workers, {}s timeout, {}",
            style("Config:").dim(),
            self.config.workers,
            self.config.timeout,
            self.config.protocol
        );
        println!();
    }
}

/// Benchmark a single DNS server
async fn benchmark_server(
    server: &DnsServer,
    config: &Config,
    progress: Option<&ProgressBar>,
) -> ServerResult {
    let mut measurements = Vec::with_capacity(config.requests as usize);

    // Adaptive timeout state
    let base_timeout_ms = config.timeout_ms();
    let mut current_timeout_ms = base_timeout_ms;
    let mut consecutive_failures: u32 = 0;

    for _ in 0..config.requests {
        let resolver = create_resolver(
            server.addr,
            config.protocol.into(),
            current_timeout_ms,
            config.lookup_ip.into(),
        );

        let start = Instant::now();
        let result = resolver.lookup_ip(config.domain.as_str()).await;
        let duration = start.elapsed();

        let timing = match result {
            Ok(lookup) => {
                consecutive_failures = 0;
                if !config.disable_adaptive_timeout {
                    current_timeout_ms = base_timeout_ms; // Reset timeout on success
                }

                let ip = lookup.iter().next().expect("At least one IP in response");
                TimingResult::Success { duration, ip }
            }
            Err(e) => {
                let error = e.to_string();
                let timing = TimingResult::Failure { error };

                // Adaptive timeout logic
                if !config.disable_adaptive_timeout && timing.is_timeout() {
                    consecutive_failures += 1;

                    if consecutive_failures >= MINIMIZE_TIMEOUT_AFTER_FAILURES {
                        current_timeout_ms = MINIMAL_TIMEOUT_MS;
                    } else if consecutive_failures >= REDUCE_TIMEOUT_AFTER_FAILURES {
                        current_timeout_ms = current_timeout_ms.min(REDUCED_TIMEOUT_MS);
                    }
                }

                timing
            }
        };

        measurements.push(timing);

        if let Some(pb) = progress {
            pb.inc(1);
        }
    }

    ServerResult::from_measurements(server, measurements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::ServerSource;
    use std::net::IpAddr;

    fn make_test_config() -> Config {
        Config::builder()
            .workers(2)
            .requests(3)
            .timeout(1)
            .build()
    }

    fn make_test_server(ip: &str) -> DnsServer {
        DnsServer::from_ip(
            "Test",
            ip.parse::<IpAddr>().unwrap(),
            ServerSource::Builtin,
        )
    }

    #[tokio::test]
    async fn test_benchmark_engine_creation() {
        let config = make_test_config();
        let servers = vec![make_test_server("8.8.8.8")];
        let engine = BenchmarkEngine::new(config, servers);
        assert_eq!(engine.servers.len(), 1);
    }
}
