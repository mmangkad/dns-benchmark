FROM rust:1.93.0 AS build

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo install --locked --path .

FROM debian:trixie-slim

COPY --from=build /usr/local/cargo/bin/dns-benchmark /usr/bin/dns-benchmark

ENTRYPOINT ["dns-benchmark"]
CMD ["--skip-system", "--skip-gateway"]
