# linkprobe

Protocol-agnostic network link measurement in Rust (LibreSpeed now; iperf3 and
JSON/MQTT-style exporters planned).

Not affiliated with Ookla or speedtest.net.

## Status

LibreSpeed HTTP backend works: ping/jitter, download, and upload via the
`linkprobe` CLI. Point `--server` at a LibreSpeed (or compatible) base URL.

Still planned: iperf3, server discovery, MQTT/Prometheus exporters.

## Usage

```bash
cargo run -p linkprobe -- --server https://example-librespeed/
cargo run -p linkprobe -- --server https://example-librespeed/ --json
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
