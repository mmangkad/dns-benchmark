//! Built-in DNS server lists.

use std::net::{Ipv4Addr, Ipv6Addr};

/// Built-in IPv4 DNS servers: (name, ip)
pub static BUILTIN_SERVERS_V4: &[(&str, Ipv4Addr)] = &[
    // Google
    ("Google", Ipv4Addr::new(8, 8, 8, 8)),
    ("Google", Ipv4Addr::new(8, 8, 4, 4)),
    // Cloudflare
    ("Cloudflare", Ipv4Addr::new(1, 1, 1, 1)),
    ("Cloudflare", Ipv4Addr::new(1, 0, 0, 1)),
    // Quad9
    ("Quad9", Ipv4Addr::new(9, 9, 9, 9)),
    ("Quad9", Ipv4Addr::new(149, 112, 112, 112)),
    // OpenDNS
    ("OpenDNS", Ipv4Addr::new(208, 67, 222, 222)),
    ("OpenDNS", Ipv4Addr::new(208, 67, 220, 220)),
    // AdGuard
    ("AdGuard", Ipv4Addr::new(94, 140, 14, 14)),
    ("AdGuard", Ipv4Addr::new(94, 140, 15, 15)),
];

/// Built-in IPv6 DNS servers: (name, ip)
pub static BUILTIN_SERVERS_V6: &[(&str, Ipv6Addr)] = &[
    // Google
    ("Google", Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)),
    ("Google", Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8844)),
    // Cloudflare
    ("Cloudflare", Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)),
    ("Cloudflare", Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001)),
    // Quad9
    ("Quad9", Ipv6Addr::new(0x2620, 0x00fe, 0, 0, 0, 0, 0, 0x00fe)),
    ("Quad9", Ipv6Addr::new(0x2620, 0x00fe, 0, 0, 0, 0, 0, 0x0009)),
    // OpenDNS
    ("OpenDNS", Ipv6Addr::new(0x2620, 0x0119, 0x0035, 0, 0, 0, 0, 0x0035)),
    ("OpenDNS", Ipv6Addr::new(0x2620, 0x0119, 0x0053, 0, 0, 0, 0, 0x0053)),
    // AdGuard
    ("AdGuard", Ipv6Addr::new(0x2a10, 0x50c0, 0, 0, 0, 0, 0x0ad1, 0x00ff)),
    ("AdGuard", Ipv6Addr::new(0x2a10, 0x50c0, 0, 0, 0, 0, 0x0ad2, 0x00ff)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v4_servers_not_empty() {
        assert!(!BUILTIN_SERVERS_V4.is_empty());
        assert_eq!(BUILTIN_SERVERS_V4.len(), 10);
    }

    #[test]
    fn test_v6_servers_not_empty() {
        assert!(!BUILTIN_SERVERS_V6.is_empty());
        assert_eq!(BUILTIN_SERVERS_V6.len(), 10);
    }

    #[test]
    fn test_v4_servers_valid() {
        for (name, ip) in BUILTIN_SERVERS_V4 {
            assert!(!name.is_empty());
            assert!(!ip.is_unspecified());
        }
    }

    #[test]
    fn test_v6_servers_valid() {
        for (name, ip) in BUILTIN_SERVERS_V6 {
            assert!(!name.is_empty());
            assert!(!ip.is_unspecified());
        }
    }
}
