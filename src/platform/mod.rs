//! Platform-specific detection for system DNS and gateway.

mod gateway;
mod system;

pub use gateway::detect_gateway;
pub use system::detect_system_dns;

use crate::dns::{DnsServer, IpVersion, ServerSource};
use crate::error::PlatformError;
use std::net::IpAddr;

/// Detect system DNS servers and return them as DnsServer entries
pub fn get_system_dns_servers(ip_version: IpVersion) -> Result<Vec<DnsServer>, PlatformError> {
    let (primary, secondary) = detect_system_dns()?;

    let mut servers = Vec::with_capacity(2);

    // Add primary if it matches the IP version
    if matches_ip_version(&primary, ip_version) {
        servers.push(DnsServer::from_ip("System DNS (Primary)", primary, ServerSource::System));
    }

    // Add secondary if present and matches the IP version
    if let Some(sec) = secondary {
        if matches_ip_version(&sec, ip_version) && sec != primary {
            servers.push(DnsServer::from_ip("System DNS (Secondary)", sec, ServerSource::System));
        }
    }

    Ok(servers)
}

/// Detect gateway and return as DnsServer if it responds to DNS
pub fn get_gateway_dns_server(ip_version: IpVersion) -> Result<Option<DnsServer>, PlatformError> {
    match detect_gateway() {
        Ok(ip) => {
            if matches_ip_version(&ip, ip_version) {
                Ok(Some(DnsServer::from_ip("Gateway (Router)", ip, ServerSource::Gateway)))
            } else {
                Ok(None)
            }
        }
        Err(_) => Ok(None),
    }
}

/// Check if an IP address matches the requested version
#[inline]
fn matches_ip_version(ip: &IpAddr, version: IpVersion) -> bool {
    match version {
        IpVersion::V4 => ip.is_ipv4(),
        IpVersion::V6 => ip.is_ipv6(),
    }
}
