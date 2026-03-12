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

Monitors SEC Form 4 filings, parses insider transaction details, and stores results in Redis.

## Features

- Monitors SEC current Form 4 feed continuously
- Parses key transaction fields (issuer, insider, shares, price, transaction code, dates)
- Writes each record to Redis
- Uses authority-scoped Redis keys (currently `SEC`)
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

Examples:

```zsh
export MONITORS=SEC
export REDIS_URL=redis://127.0.0.1/
```

## Run

```zsh
cargo run
```

Or explicitly:

```zsh
MONITORS=SEC cargo run
```

## Build Check

```zsh
cargo check
```

## Redis Keys

The app clears and rewrites monitor keys at startup.

- `monitor:all:transactions`
- `monitor:SEC:transactions`

Inspect data:

```zsh
redis-cli KEYS 'monitor:*'
redis-cli LRANGE monitor:all:transactions 0 -1
redis-cli LRANGE monitor:SEC:transactions 0 -1
```

## Notes

- SEC monitor currently filters to XML files ending with `form4.xml`
- Parsed output is stored as JSON strings in Redis lists
- `debug_xml/` can be used for XML debugging if debug-save logic is enabled in monitor code

## Troubleshooting

If you get Redis connection errors:

```zsh
redis-cli ping
```

Expected:

```text
PONG
```

If not running, start Redis (Homebrew):

```zsh
brew services start redis
```
