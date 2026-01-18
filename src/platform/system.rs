//! System DNS detection for various platforms.

use crate::error::PlatformError;
use std::net::IpAddr;
use std::str::FromStr;

/// Detect the system's configured DNS servers
///
/// Returns (primary, optional_secondary)
pub fn detect_system_dns() -> Result<(IpAddr, Option<IpAddr>), PlatformError> {
    #[cfg(target_os = "linux")]
    return linux::detect();

    #[cfg(target_os = "macos")]
    return macos::detect();

    #[cfg(target_os = "windows")]
    return windows::detect();

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(PlatformError::UnsupportedPlatform)
}

/// Helper to select primary and secondary from a list
fn select_servers(servers: Vec<IpAddr>) -> Result<(IpAddr, Option<IpAddr>), PlatformError> {
    if servers.is_empty() {
        Err(PlatformError::SystemDnsDetection("No DNS servers found".into()))
    } else {
        let secondary = servers.get(1).copied();
        Ok((servers[0], secondary))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs;

    const RESOLV_CONF: &str = "/etc/resolv.conf";

    pub fn detect() -> Result<(IpAddr, Option<IpAddr>), PlatformError> {
        let content = fs::read_to_string(RESOLV_CONF).map_err(|e| {
            PlatformError::SystemDnsDetection(format!("Failed to read {RESOLV_CONF}: {e}"))
        })?;

        let servers = parse_resolv_conf(&content);
        select_servers(servers)
    }

    pub fn parse_resolv_conf(content: &str) -> Vec<IpAddr> {
        content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                line.strip_prefix("nameserver ")
                    .and_then(|ip| IpAddr::from_str(ip.trim()).ok())
            })
            .collect()
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::process::Command;

    pub fn detect() -> Result<(IpAddr, Option<IpAddr>), PlatformError> {
        let output = Command::new("scutil")
            .arg("--dns")
            .output()
            .map_err(|e| PlatformError::CommandFailed {
                command: "scutil --dns".into(),
                message: e.to_string(),
            })?;

        let text = String::from_utf8_lossy(&output.stdout);
        let servers = parse_scutil_dns(&text);
        select_servers(servers)
    }

    pub fn parse_scutil_dns(text: &str) -> Vec<IpAddr> {
        text.lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.starts_with("nameserver[") {
                    line.split_whitespace()
                        .nth(2)
                        .and_then(|ip| IpAddr::from_str(ip).ok())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::process::Command;

    pub fn detect() -> Result<(IpAddr, Option<IpAddr>), PlatformError> {
        let output = Command::new("ipconfig")
            .arg("/all")
            .output()
            .map_err(|e| PlatformError::CommandFailed {
                command: "ipconfig /all".into(),
                message: e.to_string(),
            })?;

        let text = String::from_utf8_lossy(&output.stdout);
        let servers = parse_ipconfig(&text);
        select_servers(servers)
    }

    pub fn parse_ipconfig(text: &str) -> Vec<IpAddr> {
        let mut servers = Vec::new();
        let mut in_dns_section = false;

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.contains("DNS") && trimmed.contains(':') {
                // Start of DNS section - extract IP from this line
                if let Some(ip_str) = trimmed.split(':').nth(1) {
                    if let Ok(ip) = IpAddr::from_str(ip_str.trim()) {
                        servers.push(ip);
                        in_dns_section = true;
                    }
                }
            } else if in_dns_section && !trimmed.is_empty() {
                // Continuation lines might have more DNS servers
                if let Ok(ip) = IpAddr::from_str(trimmed) {
                    servers.push(ip);
                } else {
                    in_dns_section = false;
                }
            } else if trimmed.is_empty() {
                in_dns_section = false;
            }
        }

        servers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_resolv_conf() {
        let content = crate::load_test_fixture!("/system/linux_resolv.conf");
        let servers = linux::parse_resolv_conf(content);
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].to_string(), "8.8.8.8");
        assert_eq!(servers[1].to_string(), "1.1.1.1");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_parse_scutil_dns() {
        let content = crate::load_test_fixture!("/system/mac_scutil_dns.txt");
        let servers = macos::parse_scutil_dns(content);
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[0].to_string(), "8.8.8.8");
        assert_eq!(servers[1].to_string(), "1.1.1.1");
        assert_eq!(servers[2].to_string(), "192.168.1.1");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parse_ipconfig() {
        let content = crate::load_test_fixture!("/system/windows_ipconfig_all.txt");
        let servers = windows::parse_ipconfig(content);
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].to_string(), "8.8.8.8");
        assert_eq!(servers[1].to_string(), "1.1.1.1");
    }

    #[test]
    fn test_select_servers() {
        let servers = vec![
            "8.8.8.8".parse().unwrap(),
            "1.1.1.1".parse().unwrap(),
        ];
        let (primary, secondary) = select_servers(servers).unwrap();
        assert_eq!(primary.to_string(), "8.8.8.8");
        assert_eq!(secondary.unwrap().to_string(), "1.1.1.1");
    }

    #[test]
    fn test_select_servers_single() {
        let servers = vec!["8.8.8.8".parse().unwrap()];
        let (primary, secondary) = select_servers(servers).unwrap();
        assert_eq!(primary.to_string(), "8.8.8.8");
        assert!(secondary.is_none());
    }

    #[test]
    fn test_select_servers_empty() {
        let servers = vec![];
        assert!(select_servers(servers).is_err());
    }
}
