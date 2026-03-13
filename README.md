```text
██╗███╗   ██╗███████╗██╗██████╗ ███████╗██████╗ 
██║████╗  ██║██╔════╝██║██╔══██╗██╔════╝██╔══██╗
██║██╔██╗ ██║███████╗██║██║  ██║█████╗  ██████╔╝
██║██║╚██╗██║╚════██║██║██║  ██║██╔══╝  ██╔══██╗
██║██║ ╚████║███████║██║██████╔╝███████╗██║  ██║
╚═╝╚═╝  ╚═══╝╚══════╝╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝

 █████╗ ██╗     ███████╗██████╗ ████████╗
██╔══██╗██║     ██╔════╝██╔══██╗╚══██╔══╝
███████║██║     █████╗  ██████╔╝   ██║   
██╔══██║██║     ██╔══╝  ██╔══██╗   ██║   
██║  ██║███████╗███████╗██║  ██║   ██║   
╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝   ╚═╝
```

## About

Insider Alerts is a lightweight monitoring service for regulatory disclosure feeds. It continuously ingests filings, extracts insider transaction data, and stores normalized records in Redis for downstream alerting, analytics, and automation.

Built with Rust for reliability and speed, the project is designed to support multiple authorities through a modular monitor architecture.

## Features

- Monitors current feeds continuously
- Parses key transaction fields (issuer, insider, shares, price, transaction code, dates)
- Writes each record to Redis
- Uses authority-scoped Redis keys
- Prevents duplicate Redis entries per authority during a run

## Current Monitor Support

- `SEC` monitor implemented in `src/authorities/sec.rs`
- Monitor selection via `MONITORS` (currently expects `SEC`)

## Project Structure

- `src/main.rs` - App entrypoint, monitor selection, Redis writes
- `src/authorities/sec.rs` - SEC monitor and Form 4 XML parsing
- `src/helpers.rs` - Shared colored logging helpers

## Requirements

- Rust (stable)
- Redis server running locally (or set `REDIS_URL`)

## Configuration

Environment variables:

- `MONITORS` - Which monitors to run (currently `SEC`)
- `REDIS_URL` - Redis connection string (default: `redis://127.0.0.1/`)

## Redis Keys

- `monitor:all:transactions` - all unique transaction JSON payloads
- `monitor:SEC:transactions` - authority-scoped unique transaction payloads
- `monitor:SEC:transaction_ids` - dedupe set used before list insertion
