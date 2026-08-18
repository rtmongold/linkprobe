# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

### Added
- GitHub Actions CI (fmt, clippy, test on ubuntu/stable).
- iperf3 backend (spawns system `iperf3 -J`) and CLI `--backend` / `--port` / `--duration`.
- Root `.gitignore` (ignore `/target`) and `rust-toolchain.toml` at repo root.
- LibreSpeed server discovery: `--list`, `--server-id`, `--servers-url`, auto-pick when `--server` is omitted.
- `RunResult` in `linkprobe-core` plus OpenMetrics formatter.
- CLI `--prometheus-text [PATH]` (stdout if omitted or `-`).
- MQTT publish: `--mqtt-url`, `--mqtt-topic`, `--mqtt-username`, `--mqtt-password`.
- Phase-aware errors(`download failed: ...`) and up to 3 retries on flaky LibreSpeed HTTP bodies.
- CLI prints `error: ...` to stderr instead of a Debug dump.
- iperf3 UDP mode: CLI `--udp` / `--bandwidth` (default 10M); fills jitter and packet loss.
- iperf3 server discovery: `--list`, `--server-id`, and `--servers-url` with iperf3 default when `--backend iperf3`.
- Prometheus scrape daemon: `--listen ADDR` and `--interval SECS`; serves GET `/metrics` with OpenMetrics text.

### Changed
- README: document LibreSpeed discovery and iperf3 usage.
- README: document Prometheus text and MQTT exporters.

## [0.1.0] - 2026-08-15

### Added
- Initial workspace scaffold: `linkprobe-core` library and `linkprobe` CLI stub.
- LibreSpeed HTTP backend in `linkprobe-core` (ping/jitter, download, upload)
  with blocking reqwest and mockito unit tests.
- `linkprobe` CLI: `--server <URL>`, optional path overrides, human output and `--json`.
