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

Monitors filings, parses insider transaction details, and stores results in Redis.

## Features

- Monitors current feeds continuously
- Parses key transaction fields (issuer, insider, shares, price, transaction code, dates)
- Writes each record to Redis
- Uses authority-scoped Redis keys
- Colored logs via shared helpers

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
