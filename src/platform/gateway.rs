//! Gateway/router detection for various platforms.

use crate::error::PlatformError;
use std::net::IpAddr;
use std::str::FromStr;

/// Detect the default gateway IP address
pub fn detect_gateway() -> Result<IpAddr, PlatformError> {
    #[cfg(target_os = "linux")]
    return linux::detect();

    #[cfg(target_os = "macos")]
    return macos::detect();

    #[cfg(target_os = "windows")]
    return windows::detect();

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(PlatformError::UnsupportedPlatform)
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs;
    use std::net::Ipv4Addr;
    use std::process::Command;

    const PROC_NET_ROUTE: &str = "/proc/net/route";

    pub fn detect() -> Result<IpAddr, PlatformError> {
        // Try /proc/net/route first (most reliable)
        if let Ok(content) = fs::read_to_string(PROC_NET_ROUTE) {
            if let Ok(ip) = parse_proc_net_route(&content) {
                return Ok(ip);
            }
        }

        // Fallback to `ip route`
        let output = Command::new("ip")
            .args(["route", "show", "default"])
            .output()
            .map_err(|e| PlatformError::CommandFailed {
                command: "ip route show default".into(),
                message: e.to_string(),
            })?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            parse_ip_route(&text)
        } else {
            Err(PlatformError::GatewayDetection("No default gateway found".into()))
        }
    }

    pub fn parse_proc_net_route(content: &str) -> Result<IpAddr, PlatformError> {
        for (i, line) in content.lines().enumerate() {
            if i == 0 {
                continue; // Skip header
            }

            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 3 {
                continue;
            }

            let destination = cols[1];
            let gateway_hex = cols[2];

            // Default route has destination 00000000
            if destination == "00000000" && gateway_hex.len() == 8 {
                // Gateway is in little-endian hex format
                let mut bytes = [0u8; 4];
                for (i, idx) in (0..8).step_by(2).enumerate() {
                    bytes[i] = u8::from_str_radix(&gateway_hex[idx..idx + 2], 16)
                        .map_err(|_| PlatformError::ParseError("Invalid hex in route".into()))?;
                }

                let ip = Ipv4Addr::from(u32::from_le_bytes(bytes));
                return Ok(IpAddr::V4(ip));
            }
        }

        Err(PlatformError::GatewayDetection("No default route in /proc/net/route".into()))
    }

    pub fn parse_ip_route(text: &str) -> Result<IpAddr, PlatformError> {
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0] == "default" {
                // Find "via" keyword and get the next token
                for (i, part) in parts.iter().enumerate() {
                    if *part == "via" && i + 1 < parts.len() {
                        return IpAddr::from_str(parts[i + 1])
                            .map_err(|_| PlatformError::ParseError("Invalid gateway IP".into()));
                    }
                }
            }
        }

        Err(PlatformError::GatewayDetection("No default gateway in ip route output".into()))
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::process::Command;

    pub fn detect() -> Result<IpAddr, PlatformError> {
        // Try `route -n get default` first
        if let Ok(output) = Command::new("route").args(["-n", "get", "default"]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Ok(ip) = parse_route_get_default(&text) {
                    return Ok(ip);
                }
            }
        }

        // Fallback to `netstat -rn`
        let output = Command::new("netstat")
            .arg("-rn")
            .output()
            .map_err(|e| PlatformError::CommandFailed {
                command: "netstat -rn".into(),
                message: e.to_string(),
            })?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            parse_netstat_rn(&text)
        } else {
            Err(PlatformError::GatewayDetection("netstat failed".into()))
        }
    }

    pub fn parse_route_get_default(text: &str) -> Result<IpAddr, PlatformError> {
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("gateway:") {
                if let Some(ip_str) = line.split_whitespace().nth(1) {
                    return IpAddr::from_str(ip_str)
                        .map_err(|_| PlatformError::ParseError("Invalid gateway IP".into()));
                }
            }
        }

        Err(PlatformError::GatewayDetection("No gateway in route output".into()))
    }

    pub fn parse_netstat_rn(text: &str) -> Result<IpAddr, PlatformError> {
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("default") {
                let cols: Vec<&str> = trimmed.split_whitespace().collect();
                if cols.len() >= 2 {
                    if let Ok(ip) = IpAddr::from_str(cols[1]) {
                        return Ok(ip);
                    }
                }
            }
        }

        Err(PlatformError::GatewayDetection("No default route in netstat output".into()))
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::process::Command;

    pub fn detect() -> Result<IpAddr, PlatformError> {
        let output = Command::new("route")
            .arg("PRINT")
            .output()
            .map_err(|e| PlatformError::CommandFailed {
                command: "route PRINT".into(),
                message: e.to_string(),
            })?;

        let text = String::from_utf8_lossy(&output.stdout);
        parse_route_print(&text)
    }

    pub fn parse_route_print(text: &str) -> Result<IpAddr, PlatformError> {
        let mut in_ipv4_section = false;

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let lower = line.to_lowercase();
            if lower.contains("ipv4") {
                in_ipv4_section = true;
                continue;
            }

            if !in_ipv4_section {
                continue;
            }

            // Look for default route: 0.0.0.0 0.0.0.0 <gateway> ...
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 3 && cols[0] == "0.0.0.0" && cols[1] == "0.0.0.0" {
                if let Ok(ip) = IpAddr::from_str(cols[2]) {
                    return Ok(ip);
                }
            }
        }

        Err(PlatformError::GatewayDetection("No default gateway in route output".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_proc_net_route() {
        let content = crate::load_test_fixture!("/gateway/linux_proc_net_route.txt");
        let ip = linux::parse_proc_net_route(content).unwrap();
        assert_eq!(ip.to_string(), "192.168.0.1");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_ip_route() {
        let content = crate::load_test_fixture!("/gateway/linux_ip_route_default.txt");
        let ip = linux::parse_ip_route(content).unwrap();
        assert_eq!(ip.to_string(), "192.168.0.1");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_parse_route_get_default() {
        let content = crate::load_test_fixture!("/gateway/mac_route_get_default.txt");
        let ip = macos::parse_route_get_default(content).unwrap();
        assert_eq!(ip.to_string(), "192.168.0.1");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_parse_netstat_rn() {
        let content = crate::load_test_fixture!("/gateway/mac_netstat_rn.txt");
        let ip = macos::parse_netstat_rn(content).unwrap();
        assert_eq!(ip.to_string(), "192.168.0.1");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parse_route_print() {
        let content = crate::load_test_fixture!("/gateway/windows_route_print.txt");
        let ip = windows::parse_route_print(content).unwrap();
        assert_eq!(ip.to_string(), "192.168.0.1");
    }
}
