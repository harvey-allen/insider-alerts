# syntax=docker/dockerfile:1

FROM rust:1.83-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/insider_alert_monitor /usr/local/bin/insider_alert_monitor

ENV MONITORS=SEC
ENV REDIS_URL=redis://redis:6379/

CMD ["insider_alert_monitor"]
