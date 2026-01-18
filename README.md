# DNS Benchmark

A high-performance DNS benchmarking tool to find the fastest DNS servers for your location.

## Features

- **Built-in DNS servers** — Includes popular providers like Google, Cloudflare, Quad9, OpenDNS, and more
- **Automatic detection** — Detects system DNS and default gateway (router) DNS
- **Async benchmarking** — High-performance concurrent testing with progress tracking
- **Multiple output formats** — Table, JSON, XML, or CSV
- **Cross-platform** — Works on Linux, Windows, and macOS
- **Configurable** — Customize requests, timeout, protocol, and more
- **Docker support** — Run in a containerized environment

## Preview

```
Starting DNS benchmark

  Domain: google.com
  Scope: 12 servers × 50 requests = 600 total
  Config: 16 workers, 2s timeout, udp

╭────────────────────────┬─────────────────┬─────────────────┬──────────────┬─────────┬────────┬────────╮
│         Server         │   IP Address    │   Resolved IP   │ Success Rate │   Min   │  Max   │ Avg ↑  │
├────────────────────────┼─────────────────┼─────────────────┼──────────────┼─────────┼────────┼────────┤
│ Google                 │ 8.8.4.4         │ 142.251.220.238 │ 50/50 (100%) │ 1.03ms  │ 1.46ms │ 1.31ms │
│ Cloudflare             │ 1.1.1.1         │ 142.250.196.238 │ 50/50 (100%) │ 25.3ms  │ 27.2ms │ 26.2ms │
│ Quad9                  │ 9.9.9.9         │ 142.250.4.139   │ 50/50 (100%) │ 26.3ms  │ 33.1ms │ 28.7ms │
│ ...                    │ ...             │ ...             │ ...          │ ...     │ ...    │ ...    │
╰────────────────────────┴─────────────────┴─────────────────┴──────────────┴─────────┴────────┴────────╯
```

## Installation

### From Source (Cargo)

```sh
# Clone the repository
git clone https://github.com/mmangkad/dns-benchmark.git
cd dns-benchmark

# Build and install
cargo install --path .
```

### Docker (Build from Source)

```sh
# Clone the repository
git clone https://github.com/mmangkad/dns-benchmark.git
cd dns-benchmark

# Build the Docker image
docker build -t dns-benchmark -f docker/Dockerfile .

# Run
docker run --rm -it dns-benchmark
```

## Usage

```sh
# Run with default settings
dns-benchmark

# Run with custom options
dns-benchmark --requests 100 --workers 8 --timeout 3

# Output as JSON
dns-benchmark --format json

# Use custom DNS server list
dns-benchmark --custom-servers servers.txt

# IPv6 mode
dns-benchmark --ns-ip v6 --lookup-ip v6
```

## Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--domain` | Domain to resolve | google.com |
| `--workers` | Number of concurrent workers | 16 |
| `--requests` | Requests per DNS server | 50 |
| `--timeout` | Timeout in seconds | 2 |
| `--protocol` | Protocol (udp/tcp) | udp |
| `--ns-ip` | Name server IP version (v4/v6) | v4 |
| `--lookup-ip` | Lookup IP version (v4/v6) | v4 |
| `--format` | Output format (table/json/xml/csv) | table |
| `--style` | Table style | rounded |
| `--custom-servers` | Path to custom server list | - |
| `--skip-system` | Skip system DNS detection | false |
| `--skip-gateway` | Skip gateway DNS detection | false |
| `--no-adaptive-timeout` | Disable adaptive timeout | false |
| `--save-config` | Save options to config file | - |

## Configuration

DNS Benchmark supports persistent configuration:

```sh
# Create config file with defaults
dns-benchmark config init

# View current configuration
dns-benchmark config show

# Update configuration
dns-benchmark config set --workers 8 --requests 100

# Reset to defaults
dns-benchmark config reset

# Delete config file
dns-benchmark config delete
```

## Custom DNS Server List

Create a text file with one server per line in format: `Name;IP:PORT` (port is required, usually 53).
For IPv6, wrap the address in brackets: `[IPv6]:PORT`.

IPv4 example:
```
MyDNS;192.168.1.1:53
Corporate DNS;10.0.0.53:53
```

IPv6 example:
```
MyDNSv6;[2606:4700:4700::1111]:53
Corporate DNS v6;[2001:4860:4860::8888]:53
```

Then use it:

```sh
# IPv4 servers
dns-benchmark --custom-servers my-servers-v4.txt --ns-ip v4

# IPv6 servers
dns-benchmark --custom-servers my-servers-v6.txt --ns-ip v6
```

## Built-in DNS Servers

Built-in providers: Google, Cloudflare, Quad9, OpenDNS, AdGuard.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Author

**Mohammad Miadh Angkad** — [MAngkad.BSDSBA2027@aim.edu](mailto:MAngkad.BSDSBA2027@aim.edu)

- GitHub: [github.com/mmangkad](https://github.com/mmangkad)
