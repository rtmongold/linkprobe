# linkprobe

Protocol-agnostic network link measurement in Rust (LibreSpeed and iperf3;
JSON, MQTT, and Prometheus text exporters).

Not affiliated with Ookla or speedtest.net.

## Status

LibreSpeed and iperf3 backend work via the `linkprobe` CLI. 

- LibreSpeed: measure a URL, pick from the public list, or auto-select by ping.
  `--server-id` and auto-pick try up to two more hosts if the first still fails after HTTP retries.
- iperf3: requires `iperf3` on `PATH`; optional `--list` / `--server-id` from the public server JSON.
- After a run: human or `--json` stdout, optional MQTT publish, optional
  OpenMetrics file/stdout or HTTP scrape via `--listen`.

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

MQTT extras: `--mqtt-username`, `mqtt-password`

Public LibreSpeed hosts can drop connections: linkprobe retries each phase up to three times,
then auto-pick and `--server-id` try up to two more list servers (by ping). Explicit `--server`
URLs are single-host only. On rotation you will see `linkprobe: <name> failed, trying next server`
on stderr.

## Crates

- `linkprobe-core` - measurement types, LibreSpeed/iperf3 backends, discovery, OpenMetrics
- `linkprobe` - CLI, MQTT, scrape HTTP

## License

MIT OR Apache-2.0

## History

Inspired by [speedtest-rs](https://github.com/nelsonjchen/speedtest-rs); see `NOTICE`.
