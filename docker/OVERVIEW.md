# DNS Benchmark (Docker)

A high-performance DNS benchmarking tool to find the fastest DNS servers for your location.

## Quick Start

```sh
# Build from source
git clone https://github.com/mmangkad/dns-benchmark.git
cd dns-benchmark
docker build -t dns-benchmark -f docker/Dockerfile .

# Run benchmark
docker run --rm -it dns-benchmark
```

## Usage Examples

```sh
# Basic benchmark
docker run --rm -it dns-benchmark

# Custom options
docker run --rm -it dns-benchmark --requests 100 --workers 8

# JSON output
docker run --rm -it dns-benchmark --format json

# Use custom server list
docker run --rm -it \
  -v /path/to/servers.txt:/servers.txt:ro \
  dns-benchmark --custom-servers /servers.txt

# Host networking for accurate latency (Linux)
docker run --rm -it --network host dns-benchmark
```

## Output Formats

```sh
# Table (default)
docker run --rm -it dns-benchmark

# JSON
docker run --rm -it dns-benchmark --format json

# CSV
docker run --rm dns-benchmark --format csv > results.csv

# XML
docker run --rm -it dns-benchmark --format xml
```

## License

MIT OR Apache-2.0

## Author

Mohammad Miadh Angkad — [MAngkad.BSDSBA2027@aim.edu](mailto:MAngkad.BSDSBA2027@aim.edu)
