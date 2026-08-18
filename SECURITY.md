# Security Policy

## Supported versions

Security fixes are applied to the latest release on `main`. Older tags may not receive backports.

| Version | Supported |
| ------- | --------- |
| 0.2.0 | yes |
| 0.1.0 and older | no |

## Reporting a vulnerability

**Do not** open public GitHub issues for security problems.

Report privately to **rtmongold@gmail.com** with:

- linkprobe version (`linkprobe --version` or git commit)
- OS and Rust toolchain if relevant
- exact command line or config
- steps to reproduce
- impact (what an attacker could do)

You can also use [GitHub Security Advisories](https://github.com/rtmongold/linkprobe/security/advisories/new) for private disclosure.

We aim to acknowledge reports within a few days and will coordinate on a fix and disclosure timeline.

## In scope

Issues we treat as security vulnerabilities include:

- remote code execution or command injection in linkprobe itself (e.g. via crafted server responses, CLI arguments, or file paths)
- memory-safety bugs in Rust code that are exploitable
- authentication or authorization flaws in the `--listen` HTTP server beyond its intended read-only `/metrics` scrape
- unintended credential disclosure (e.g. MQTT password logged or written to metrics output)

## Out of scope

These are expected behavior, misconfiguration, or third-party issues — please use regular [GitHub issues](https://github.com/rtmongold/linkprobe/issues) instead:

- measuring against servers you do not trust (LibreSpeed hosts, custom `--servers-url`, iperf3 endpoints)
- bugs or vulnerabilities in the external **`iperf3`** binary
- denial of service from running bandwidth tests (that is what the tool does)
- binding `--listen` to a public interface without a firewall (bind `127.0.0.1` unless you intend to expose metrics)
- missing TLS on MQTT when you point at a plain `mqtt://` broker
- feature requests and non-security bugs

## Safe defaults

- Prefer `--listen 127.0.0.1:9090` unless Prometheus runs on the same host or network segment.
- Treat `--mqtt-password` like any secret; avoid shell history and shared logs.
- Only probe servers and broker URLs you control or explicitly trust.