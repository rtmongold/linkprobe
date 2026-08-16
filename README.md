# linkprobe

Protocol-agnostic network link measurement in Rust (LibreSpeed and iperf3;
JSON/MQTT-style exporters planned).

Not affiliated with Ookla or speedtest.net.

## Status

LibreSpeed and iperf3 backend work via the `linkprobe` CLI. 

- LibreSpeed: measure a URL, pick from the public server list, or auto-select
  by ping.
- iperf3: requires the `iperf3` binary on `PATH`.

Still planned: MQTT/Prometheus exporters.

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
```

Optional path overrides (defaults match LibreSpeed):
- `--dl-path` (default: `backend/garbage.php`)
- `--ul-path` (default: `backend/empty.php`)
- `--ping-path` (default: `backend/empty.php`)

Optional: `--servers-url` to point at a custom LibreSpeed servers JSON.

## Crates

- `linkprobe-core` - measurement types, LibreSpeed/iperf3 backends, discovery
- `linkprobe` - command-line interface

## License

MIT OR Apache-2.0

## History

Inspired by [speedtest-rs](https://github.com/nelsonjchen/speedtest-rs); see `NOTICE`.
