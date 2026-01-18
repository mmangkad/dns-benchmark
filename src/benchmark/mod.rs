//! High-performance async DNS benchmarking engine.

mod engine;
mod result;
mod resolver;

pub use engine::BenchmarkEngine;
pub use result::{BenchmarkResult, ServerResult, TimingResult, SerializableResult};
pub(crate) use resolver::create_resolver;

use crate::config::Config;
use crate::dns::{get_builtin_servers, load_custom_servers, DnsServer};
use crate::error::Error;
use crate::platform::{get_gateway_dns_server, get_system_dns_servers};
use std::collections::HashSet;

/// Collect all DNS servers to benchmark based on configuration
pub fn collect_servers(config: &Config) -> Result<Vec<DnsServer>, Error> {
    let mut servers = Vec::new();
    let mut seen_ips = HashSet::new();

    // 1. Load custom servers or builtin list
    let base_servers = if let Some(ref path) = config.custom_servers {
        load_custom_servers(path, config.name_server_ip)?
    } else {
        get_builtin_servers(config.name_server_ip)
    };

    for server in base_servers {
        if seen_ips.insert(server.ip()) {
            servers.push(server);
        }
    }

    // 2. Add system DNS servers if enabled
    if !config.skip_system {
        match get_system_dns_servers(config.name_server_ip) {
            Ok(system_servers) => {
                for server in system_servers {
                    if seen_ips.insert(server.ip()) {
                        servers.push(server);
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to detect system DNS: {e}");
            }
        }
    }

    // 3. Add gateway DNS if enabled
    if !config.skip_gateway {
        match get_gateway_dns_server(config.name_server_ip) {
            Ok(Some(server)) => {
                if seen_ips.insert(server.ip()) {
                    servers.push(server);
                }
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Warning: Failed to detect gateway: {e}");
            }
        }
    }

    Ok(servers)
}

/// Check if a server is responsive (quick test)
pub async fn is_server_responsive(
    server: &DnsServer,
    config: &Config,
    timeout_ms: u64,
) -> bool {
    let resolver = create_resolver(
        server.addr,
        config.protocol.into(),
        timeout_ms,
        config.lookup_ip.into(),
    );

    match resolver.lookup_ip("google.com").await {
        Ok(_) => true,
        Err(_) => false,
    }
}
