# Contributing to linkprobe

Thanks for your interest. linkprobe measures network links in Rust via LibreSpeed and iperf3, with JSON, MQTT, and Prometheus exporters.

## Getting started

Install the Rust stable toolchain. This repo pins it in [`rust-toolchain.toml`](rust-toolchain.toml) (includes `rustfmt` and `clippy`).

```bash
git clone https://github.com/rtmongold/linkprobe.git
cd linkprobe
cargo build --workspace
cargo test --workspace
```

- **LibreSpeed** backend: outbound HTTPS only.
- **iperf3** backend: `iperf3` on `PATH`.

See [README.md](README.md) for usage examples.

## Before you open a PR

Run the same checks as CI ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If formatting fails, run `cargo fmt --all` and commit the result.

## Pull requests

- Target **`main`**.
- Keep changes focused; match the style of the crate you edit (`linkprobe-core` vs `linkprobe` CLI).
- Add or update tests when behavior changes. CI uses mocks and fixtures — no live network required.
- Put user-visible changes under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md).

## License and attribution

By contributing, you agree your work is licensed under **MIT OR Apache-2.0**, same as the project.

linkprobe is a new codebase inspired by [speedtest-rs](https://github.com/nelsonjchen/speedtest-rs). See [NOTICE](NOTICE). Not affiliated with Ookla or speedtest.net.
