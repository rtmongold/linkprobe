# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

### Added
- GitHub Actions CI (fmt, clippy, test on ubuntu/stable).
- iperf3 backend (spanws system `iperf3 -J`) and CLI `--backend` / `--port` / `--duration`.

### Changed
- README: accurate status, usage examples; fix typo.

## [0.1.0] - 2026-08-15

### Added
- Initial workspace scaffold: `linkprobe-core` library and `linkprobe` CLI stub.
- LibreSpeed HTTP backend in `linkprobe-core` (ping/jitter, download, upload)
  with blocking reqwest and mockito unit tests.
- `linkprobe` CLI: `--server <URL>`, optional path overrides, human output and `--json`.
