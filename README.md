# linkprobe

Protocol-agnostic network link measurement in Rust (LibreSpeed and iperf3;
JSON, MQTT, and Prometheus text exporters).

Not affiliated with Ookla or speedtest.net.

## Status

LibreSpeed and iperf3 backend work via the `linkprobe` CLI. 

- LibreSpeed: measure a URL, pick from the public server list, or auto-select
  by ping.
- iperf3: requires the `iperf3` binary on `PATH`.
- After a run: human or `--json` stdout, optional MQTT publish, optional
  OpenMetrics file/stdout.

## Usage

```bash
# LibreSpeed - explicit server
cargo run -p linkprobe -- --server https://example-librespeed/
cargo run -p linkprobe -- --server https://example-librespeed/ --json

# LibreSpeed - discovery
cargo run -p linkprobe -- --list
cargo run -p linkprobe -- --server-id 52
cargo run -p linkprobe --                 # auto-pick lowest latency

# iperf3
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1 --port 5201 --duration 5 --json

# Prometheus OpenMetrics (stdout, or a file for node_exporter textfile collector)
cargo run -p linkprobe -- --server-id 52 --prometheus-text -
cargo run -p linkprobe -- --server-id 52 --prometheus-text /var/lib/node_exporter/textfile_collector/linkprobe.prom

# MQTT (requires a broker)
cargo run -p linkprobe -- --server-id 52 --mqtt-url mqtt://127.0.0.1:1883
cargo run -p linkprobe -- --server-id 52 --mqtt://127.0.0.1:1883 --mqtt-topic home/linkprobe
```

Optional path overrides (defaults match LibreSpeed):
- `--dl-path` (default: `backend/garbage.php`)
- `--ul-path` (default: `backend/empty.php`)
- `--ping-path` (default: `backend/empty.php`)

Optional: `--servers-url` to point at a custom LibreSpeed servers JSON.

MQTT extras: `--mqtt-username`, `mqtt-password`

## Crates

- `linkprobe-core` - measurement types, LibreSpeed/iperf3 backends, discovery, OpenMetrics
- `linkprobe` - CLI, MQTT publish

## License

MIT OR Apache-2.0

## History

Inspired by [speedtest-rs](https://github.com/nelsonjchen/speedtest-rs); see `NOTICE`.
