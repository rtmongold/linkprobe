# linkprobe

Protocol-agnostic network link measurement in Rust (LibreSpeed and iperf3;
JSON, MQTT, and Prometheus text exporters).

Not affiliated with Ookla or speedtest.net.

## What linkprobe is

- **Link measurement** over LibreSpeed-compatible HTTPS or the system `iperf3` binary.
- **Two surfaces:** the [`linkprobe`](crates/linkprobe) CLI for ops and homelab use, and the
  [`linkprobe-core`](crates/linkprobe-core) library to embed the same engines in your own apps.
- **Outputs:** human-readable text, JSON, OpenMetrics (file, stdout, or HTTP scrape), and
  optional MQTT publish.

## What linkprobe is not

- **Not Ookla or speedtest.net** — no affiliation and no proprietary speedtest SDK.
- **Not a bundled speedtest engine** — measurements run against LibreSpeed-compatible servers
  you choose or public lists, or against an `iperf3` endpoint you control.
- **Not [probe-rs](https://crates.io/crates/probe-rs)** — linkprobe measures network links;
  probe-rs is an embedded debugging toolkit.

## Features

- LibreSpeed: measure a URL, pick from the public list, or auto-select by lowest ping.
  `--server-id` and auto-pick try up to two more hosts if the first still fails after HTTP retries.
- iperf3: requires `iperf3` on `PATH`; optional `--list` / `--server-id` from the public server JSON.
- After a run: human or `--json` stdout, optional MQTT publish, optional
  OpenMetrics file/stdout or HTTP scrape via `--listen`.

## CLI modes

| Mode | Flags | Behavior |
|------|-------|----------|
| One-shot | default (no `--listen`) | Run one probe; print human text or `--json` to stdout |
| Prometheus text | `--prometheus-text [PATH]` | Write OpenMetrics text; `-` prints metrics to stdout instead of human output |
| Scrape daemon | `--listen ADDR` + `--interval SECS` | Background HTTP server on GET `/metrics`; reprobe on an interval |
| MQTT push | `--mqtt-url URL` | After each probe, publish the JSON `RunResult` (default topic: `linkprobe/result`) |

Modes can be combined where it makes sense (for example one-shot probe plus `--prometheus-text`
and `--mqtt-url`).

## Server selection

| Input | Behavior |
|-------|----------|
| `--server URL` or `--server HOST` | Single explicit LibreSpeed base URL or iperf3 host only (no list failover) |
| `--server-id N` | Pick entry `N` from `--list`; LibreSpeed auto-pick / `--server-id` may try up to two more list servers after retries |
| `--server` omitted (LibreSpeed default backend) | Fetch the public list, rank by ping, probe the fastest |
| `--list` | Print server ids and names, then exit |

LibreSpeed probes retry each HTTP phase up to three times. When using auto-pick or `--server-id`,
linkprobe may rotate through up to two additional list servers (by ping order) if the preferred
host still fails. On rotation you will see `linkprobe: <name> failed, trying next server` on stderr.

## Requirements

Runs on Linux, macOS, and Windows. CI builds and tests all three.

- Rust stable (see `rust-toolchain.toml`)
- LibreSpeed: outbound HTTPS
- iperf3 backend: `iperf3` on `PATH` (`iperf3.exe` on Windows)
- MQTT: a reachable broker
- Prometheus scrape: `--listen` bind address (prefer `127.0.0.1`)

iperf3 install:

- Linux: distro package (`iperf3`)
- macOS: `brew install iperf3`
- Windows: [iperf3 Windows builds](https://github.com/esnet/iperf) or `winget` / `choco`; put `iperf3` on `PATH`

Pre-built binaries are attached to [GitHub Releases](https://github.com/rtmongold/linkprobe/releases)
(Linux x86_64, macOS Apple Silicon, Windows x86_64). The iperf3 backend still needs
`iperf3` on `PATH`; LibreSpeed does not.

## Usage

```bash
# LibreSpeed - explicit server
cargo run -p linkprobe -- --server https://example-librespeed/
cargo run -p linkprobe -- --server https://example-librespeed/ --json

# LibreSpeed - discovery
cargo run -p linkprobe -- --list
cargo run -p linkprobe -- --server-id 52
cargo run -p linkprobe --                 # auto-pick; failover if first host flakes

# iperf3
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1 --port 5201 --duration 5 --json

# iperf3 UDP (jitter + packet loss; -b default 10M)
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1 --udp
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1 --udp --bandwidth 50M --duration 5

# iperf3 - discovery
cargo run -p linkprobe -- --backend iperf3 --list
cargo run -p linkprobe -- --backend iperf3 --server-id 1 --duration 5

# Prometheus OpenMetrics (stdout, or a file for node_exporter textfile collector)
cargo run -p linkprobe -- --server-id 52 --prometheus-text -
cargo run -p linkprobe -- --server-id 52 --prometheus-text /var/lib/node_exporter/textfile_collector/linkprobe.prom

# Prometheus scrape (daemon; GET /metrics)
cargo run -p linkprobe -- --server-id 52 --listen 127.0.0.1:9090 --interval 300
curl -s http://127.0.0.1:9090/metrics | head

# MQTT (requires a broker)
cargo run -p linkprobe -- --server-id 52 --mqtt-url mqtt://127.0.0.1:1883
cargo run -p linkprobe -- --server-id 52 --mqtt://127.0.0.1:1883 --mqtt-topic home/linkprobe
```

Optional path overrides (defaults match LibreSpeed):
- `--dl-path` (default: `backend/garbage.php`)
- `--ul-path` (default: `backend/empty.php`)
- `--ping-path` (default: `backend/empty.php`)

Optional: `--servers-url` for a custom server list (LibreSpeed or iperf3 JSON, depending on `--backend`).

MQTT extras: `--mqtt-username`, `--mqtt-password`

## Crates

- `linkprobe-core` — measurement types, LibreSpeed/iperf3 backends, discovery, OpenMetrics formatting
- `linkprobe` — CLI, MQTT client, Prometheus scrape HTTP server

Library API docs: `cargo doc -p linkprobe-core --open` (or docs.rs after publish).

## License

MIT OR Apache-2.0

## History

Inspired by [speedtest-rs](https://github.com/nelsonjchen/speedtest-rs); see [NOTICE](NOTICE).
