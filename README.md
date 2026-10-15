# r-burp

A lightweight, privacy-oriented desktop HTTP/HTTPS proxy for inspecting and modifying web traffic.

Built with a Rust backend and a Tauri + Next.js frontend. Inspired by [Burp Suite](https://portswigger.net/burp) and [Caido](https://caido.io/).

> **Status:** early development. Interfaces and features may change.

## Features

- **HTTP/HTTPS proxy** — capture and inspect full request/response details.
- **HTTPS interception** — MITM via per-domain certificates signed by a local CA.
- **Request interceptor** — pause, modify, and forward requests in real time.
- **Intercept rules** — automatically rewrite headers, body, and query parameters.
- **HAR export** — save captured traffic for analysis and sharing.
- **Certificate management** — generate and export the local CA certificate.

## Tech Stack

| Layer     | Technology                                   |
| --------- | -------------------------------------------- |
| Backend   | Rust, Tokio, Hyper, Rustls                   |
| Framework | Tauri v2                                     |
| Frontend  | Next.js, TypeScript, Tailwind CSS            |

## Getting Started

### Prerequisites

- Rust 1.77.2+
- Node.js 18+
- Tauri CLI — `cargo install tauri-cli --version "^2"`

### Development

```bash
cd src && npm install   # install frontend dependencies
cargo tauri dev         # run Next.js + Tauri together
```

### Build

```bash
cargo tauri build       # artifacts in src-tauri/target/release/bundle/
```

### Scripts

Run from the repository root:

```bash
npm run lint     # ESLint (frontend)
npm run test     # Rust unit tests
npm run clippy   # Rust linter (clippy -D warnings)
npm run audit    # Dependency CVE scan (npm + cargo)
```

## Project Structure

```
r-burp/
├── src-tauri/            # Rust backend + Tauri config
│   └── src/
│       ├── lib.rs        # App entry point and shared runtime
│       ├── commands.rs   # Tauri commands exposed to the frontend
│       ├── proxy.rs      # Traffic capture engine (ring buffer + index)
│       ├── server.rs     # HTTP/HTTPS proxy server
│       ├── intercept.rs  # Interception engine and rules
│       ├── certs.rs      # CA and per-domain certificate management
│       └── error.rs      # Typed errors
├── src/                  # Next.js frontend
│   ├── app/              # Pages: proxy, interceptor, requests, settings
│   ├── components/       # Shared UI components
│   ├── lib/              # Tauri API bindings and utilities
│   └── types/            # Shared TypeScript types
├── .github/workflows/    # CI pipeline
├── CHANGELOG.md
├── CONTRIBUTING.md
├── DESIGN.md
└── LICENSE
```

## Design Notes

- **O(1) transaction lookup** — a `VecDeque` ring buffer with a `HashMap` index; oldest entries are evicted at capacity.
- **Domain certificate cache** — an LRU cache (256 entries) reuses generated TLS certificates across connections.
- **Push-based UI** — the backend emits Tauri events on new traffic instead of the frontend polling.
- **Pre-compiled rules** — intercept-rule regexes are compiled once when the rule is added.
- **Request/response limits** — 10 MB max body size and 100 max headers per message, enforced while reading.

## Security

- **Memory safety** — the backend is written in Rust.
- **Loopback-only binding** — listeners are restricted to `127.0.0.1`, `localhost`, and `::1`.
- **Encrypted key storage** — the CA private key is encrypted at rest with AES-256-GCM using a runtime-generated passphrase stored in the app data directory (`chmod 600` on Unix).
- **No hardcoded secrets** — the passphrase is generated on first run and persisted locally.
- **Valid certificates** — per-domain certificates include a Subject Alternative Name and are signed by the local CA; SHA-256 is used for certificate fingerprints.
- **RFC-compliant forwarding** — hop-by-hop headers are stripped and `Content-Length` is recomputed after any body modification.
- **Upstream TLS validation** — HTTPS forwarding validates certificates against native system root stores via `rustls`.
- **Restrictive CSP** — a strict Content Security Policy limits resource loading in the UI.

> HTTPS interception requires installing the generated CA certificate in your system or browser trust store. Only do this on machines you control, and remove the CA when you are done.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
