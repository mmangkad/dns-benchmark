//! Async DNS resolver creation.

use hickory_resolver::config::{
    LookupIpStrategy,
    NameServerConfig,
    ResolveHosts,
    ResolverConfig,
    ResolverOpts,
};
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::proto::xfer::Protocol;
use hickory_resolver::TokioResolver;
use std::net::SocketAddr;
use std::time::Duration;

/// Create an async DNS resolver for a specific server
pub fn create_resolver(
    addr: SocketAddr,
    protocol: Protocol,
    timeout_ms: u64,
    lookup_strategy: LookupIpStrategy,
) -> TokioResolver {
    let mut config = ResolverConfig::new();
    let mut name_server = NameServerConfig::new(addr, protocol);
    name_server.trust_negative_responses = false;
    config.add_name_server(name_server);

    let mut opts = ResolverOpts::default();
    opts.attempts = 1;
    opts.timeout = Duration::from_millis(timeout_ms);
    opts.ip_strategy = lookup_strategy;
    opts.cache_size = 0; // Disable caching for accurate benchmarking
    opts.use_hosts_file = ResolveHosts::Never;

    TokioResolver::builder_with_config(config, TokioConnectionProvider::default())
        .with_options(opts)
        .build()
}
