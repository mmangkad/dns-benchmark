//! DNS server definitions and types.

mod servers;

pub use servers::BUILTIN_SERVERS_V4;
pub use servers::BUILTIN_SERVERS_V6;

use crate::error::{DnsError, Error};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::str::FromStr;

/// DNS server representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DnsServer {
    /// Human-readable name
    pub name: String,
    /// Socket address (IP + port)
    pub addr: SocketAddr,
    /// Source of this server entry
    pub source: ServerSource,
}

impl DnsServer {
    /// Create a new DNS server entry
    #[inline]
    pub const fn new(name: String, addr: SocketAddr, source: ServerSource) -> Self {
        Self { name, addr, source }
    }

    /// Create from IP address with default DNS port (53)
    pub fn from_ip(name: impl Into<String>, ip: IpAddr, source: ServerSource) -> Self {
        Self::new(name.into(), SocketAddr::new(ip, 53), source)
    }

    /// Get the IP address
    #[inline]
    pub const fn ip(&self) -> IpAddr {
        self.addr.ip()
    }

    /// Check if this is an IPv4 server
    #[inline]
    pub const fn is_ipv4(&self) -> bool {
        self.addr.ip().is_ipv4()
    }

    /// Check if this is an IPv6 server
    #[inline]
    pub const fn is_ipv6(&self) -> bool {
        self.addr.ip().is_ipv6()
    }

    /// Check if this server matches the given IP version
    #[inline]
    pub const fn matches_ip_version(&self, version: IpVersion) -> bool {
        match version {
            IpVersion::V4 => self.is_ipv4(),
            IpVersion::V6 => self.is_ipv6(),
        }
    }
}

impl fmt::Display for DnsServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.addr.ip())
    }
}

/// Source of a DNS server entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ServerSource {
    /// Built-in server list
    #[default]
    Builtin,
    /// Custom server file
    Custom,
    /// System DNS configuration
    System,
    /// Network gateway/router
    Gateway,
}

impl fmt::Display for ServerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin => write!(f, "builtin"),
            Self::Custom => write!(f, "custom"),
            Self::System => write!(f, "system"),
            Self::Gateway => write!(f, "gateway"),
        }
    }
}

/// DNS protocol to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// UDP (faster, less reliable)
    #[default]
    Udp,
    /// TCP (more reliable, slightly slower)
    Tcp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Udp => write!(f, "udp"),
            Self::Tcp => write!(f, "tcp"),
        }
    }
}

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "udp" => Ok(Self::Udp),
            "tcp" => Ok(Self::Tcp),
            _ => Err(Error::InvalidArgument(format!("Invalid protocol: {s}"))),
        }
    }
}

impl From<Protocol> for hickory_resolver::proto::xfer::Protocol {
    fn from(p: Protocol) -> Self {
        match p {
            Protocol::Udp => Self::Udp,
            Protocol::Tcp => Self::Tcp,
        }
    }
}

/// IP version preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IpVersion {
    /// IPv4 only
    #[default]
    V4,
    /// IPv6 only
    V6,
}

impl fmt::Display for IpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V4 => write!(f, "v4"),
            Self::V6 => write!(f, "v6"),
        }
    }
}

impl FromStr for IpVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "v4" | "ipv4" | "4" => Ok(Self::V4),
            "v6" | "ipv6" | "6" => Ok(Self::V6),
            _ => Err(Error::InvalidArgument(format!("Invalid IP version: {s}"))),
        }
    }
}

impl From<IpVersion> for hickory_resolver::config::LookupIpStrategy {
    fn from(v: IpVersion) -> Self {
        match v {
            IpVersion::V4 => Self::Ipv4Only,
            IpVersion::V6 => Self::Ipv6Only,
        }
    }
}

/// Load custom DNS servers from a file
///
/// Expected format: `name;ip:port` per line
pub fn load_custom_servers(path: &Path, ip_version: IpVersion) -> Result<Vec<DnsServer>, Error> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        Error::Dns(DnsError::CustomFileError {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    })?;

    parse_custom_servers(&content, ip_version, path)
}

/// Parse custom servers from string content
pub fn parse_custom_servers(
    content: &str,
    ip_version: IpVersion,
    path: &Path,
) -> Result<Vec<DnsServer>, Error> {
    let mut servers = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != 2 {
            return Err(Error::Dns(DnsError::InvalidLineFormat { line: line_num + 1 }));
        }

        let name = parts[0].trim().to_string();
        let addr_str = parts[1].trim();

        let addr: SocketAddr = addr_str.parse().map_err(|_| {
            Error::Dns(DnsError::CustomFileError {
                path: path.to_path_buf(),
                message: format!("Invalid address at line {}: {}", line_num + 1, addr_str),
            })
        })?;

        let server = DnsServer::new(name, addr, ServerSource::Custom);

        // Filter by IP version
        if server.matches_ip_version(ip_version) {
            servers.push(server);
        }
    }

    Ok(servers)
}

/// Get the builtin DNS server list for the given IP version
pub fn get_builtin_servers(ip_version: IpVersion) -> Vec<DnsServer> {
    match ip_version {
        IpVersion::V4 => BUILTIN_SERVERS_V4
            .iter()
            .map(|(name, ip)| {
                DnsServer::from_ip(*name, IpAddr::V4(*ip), ServerSource::Builtin)
            })
            .collect(),
        IpVersion::V6 => BUILTIN_SERVERS_V6
            .iter()
            .map(|(name, ip)| {
                DnsServer::from_ip(*name, IpAddr::V6(*ip), ServerSource::Builtin)
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_dns_server_creation() {
        let server = DnsServer::from_ip("Test", Ipv4Addr::new(8, 8, 8, 8).into(), ServerSource::Builtin);
        assert_eq!(server.name, "Test");
        assert!(server.is_ipv4());
        assert!(!server.is_ipv6());
        assert!(server.matches_ip_version(IpVersion::V4));
        assert!(!server.matches_ip_version(IpVersion::V6));
    }

    #[test]
    fn test_protocol_parsing() {
        assert_eq!(Protocol::from_str("udp").unwrap(), Protocol::Udp);
        assert_eq!(Protocol::from_str("TCP").unwrap(), Protocol::Tcp);
        assert!(Protocol::from_str("invalid").is_err());
    }

    #[test]
    fn test_ip_version_parsing() {
        assert_eq!(IpVersion::from_str("v4").unwrap(), IpVersion::V4);
        assert_eq!(IpVersion::from_str("ipv6").unwrap(), IpVersion::V6);
        assert_eq!(IpVersion::from_str("4").unwrap(), IpVersion::V4);
        assert!(IpVersion::from_str("invalid").is_err());
    }

    #[test]
    fn test_parse_custom_servers() {
        let content = r#"
# Comment line
Google;8.8.8.8:53
Cloudflare;1.1.1.1:53
"#;
        let path = Path::new("test.txt");
        let servers = parse_custom_servers(content, IpVersion::V4, path).unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "Google");
        assert_eq!(servers[1].name, "Cloudflare");
    }

    #[test]
    fn test_builtin_servers() {
        let v4_servers = get_builtin_servers(IpVersion::V4);
        let v6_servers = get_builtin_servers(IpVersion::V6);

        assert!(!v4_servers.is_empty());
        assert!(!v6_servers.is_empty());

        for server in &v4_servers {
            assert!(server.is_ipv4());
        }
        for server in &v6_servers {
            assert!(server.is_ipv6());
        }
    }
}
