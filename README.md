# linkprobe

Protocol-agnostic network link measurement in Rust (libreSpeed, iperf3, and
JSON/MQTT-style exporters).

Not affiliated iwht Ookla or speedtest.net.

## Status

Early scaffold: core types and CLI stub. Measurement backends coming next.

## Crates

- `linkprobe-core` - measurement types and engine trait
- `linkprobe` - command-line interface

## License

MIT OR Apache-2.0

## History

Inspired by [speedtest-rs](https://github.com/nelsonjchen/speedtest-rs); see `NOTICE`
