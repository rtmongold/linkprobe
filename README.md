# linkprobe

Protocol-agnostic network link measurement in Rust (LibreSpeed now; iperf3 and
JSON/MQTT-style exporters planned).

Not affiliated with Ookla or speedtest.net.

## Status

LibreSpeed and iperf3 backend work via the `linkprobe` CLI. 
iperf3 requires the `iperf3` binary on `PATH`.

Still planned: server discovery, MQTT/Prometheus exporters.

## Usage

```bash
cargo run -p linkprobe -- --server https://example-librespeed/
cargo run -p linkprobe -- --server https://example-librespeed/ --json
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1
cargo run -p linkprobe -- --backend iperf3 --server 192.0.2.1 --port 5201 --duration 5 --json
```

Optional path overrides (defaults match LibreSpeed):
- `--dl-path` (default: `backend/garbage.php`)
- `--ul-path` (default: `backend/empty.php`)
- `--ping-path` (default: `backend/empty.php`)

## Crates

- `linkprobe-core` - measurement types and engine trait, LibreSpeed backend
- `linkprobe` - command-line interface

## License

MIT OR Apache-2.0

## History

Inspired by [speedtest-rs](https://github.com/nelsonjchen/speedtest-rs); see `NOTICE`.
