# r-burp

A lightweight, secure, privacy-oriented desktop HTTP proxy toolkit.

Inspired by [Burp Suite](https://portswigger.net/burp) and [Caido](https://caido.io/), built with modern technologies for stringent performance and security standards.

## Features

- **HTTP/HTTPS Proxy** — Intercept and inspect HTTP/HTTPS traffic with full request/response details
- **HTTPS Interception** — Full MITM via dynamic per-domain certificate generation signed by the local CA
- **Request Interceptor** — Pause, modify, and forward requests in real-time through a clean UI
- **Intercept Rules** — Create automated rules to modify headers, body content, and query parameters
- **HAR Export** — Export captured traffic as HAR files for analysis and sharing
- **Certificate Management** — Generate and manage CA certificates for HTTPS interception
- **Secure by Design** — Rust backend with Tauri's capability-based security model
- **Warm Minimalism UI** — Clean, professional interface with Cursor-inspired design
- **Cross-Platform** — macOS, Windows, and Linux support
- **Open Source** — MIT licensed, transparent, community-driven

## Performance

- **O(1) Transaction Lookup** — Ring buffer (`VecDeque`) + `HashMap` index replaces linear scan
- **Domain Cert Cache** — LRU cache (256 entries) for generated TLS certificates avoids re-signing on repeated connections
- **Push-based UI Updates** — Tauri events replace polling; the frontend updates only when new data arrives
- **Zero-copy Body Text** — Body text is decoded on demand client-side; no eager UTF-8 allocation per request
- **Pre-compiled Regex Rules** — Intercept rule regexes are compiled once on add, not per request

## Screenshots

> _Coming soon — the application is currently in early development._

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust |
| Framework | Tauri v2 |
| Frontend | Next.js 16, TypeScript, Tailwind CSS v4 |
| Design | Warm minimalism with OKLCH color system |

## Getting Started

### Prerequisites

- Rust 1.77.2 or later
- Node.js 18 or later
- Tauri CLI (`cargo install tauri-cli --version "^2"`)

### Development

```bash
# Install frontend dependencies
cd src && npm install

# Start the development server (runs Next.js + Tauri together)
cargo tauri dev
```

### Building

```bash
# Build for your current platform
cargo tauri build

# Built artifacts will be in src-tauri/target/release/bundle/
```

### Useful Commands

```bash
npm run lint          # Run ESLint
npm run test          # Run Rust unit tests
npm run clippy        # Run Rust linter
npm run audit         # Run security audits (npm + cargo)
```

## Project Structure

```
r-burp/
├── src-tauri/              # Rust backend + Tauri configuration
│   ├── src/
│   │   ├── lib.rs          # Tauri app entry point
│   │   ├── commands.rs     # Tauri commands (backend logic)
│   │   ├── main.rs         # Native app entry point
│   │   ├── proxy.rs        # Proxy engine (traffic capture, VecDeque ring buffer)
│   │   ├── server.rs       # HTTP/HTTPS proxy server
│   │   ├── intercept.rs    # Request interception engine
│   │   ├── certs.rs        # Certificate management (CA + domain cert LRU cache)
│   │   └── error.rs        # Typed error types (CertError, ProxyError)
│   ├── capabilities/       # Tauri permission capabilities
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── src/                    # Next.js frontend
│   ├── app/                # App router pages
│   │   ├── proxy/          # Proxy control panel
│   │   ├── interceptor/    # Request interceptor UI
│   │   ├── requests/       # Intercept rules management
│   │   └── settings/       # Certificate & app settings
│   ├── components/         # Shared UI components
│   ├── lib/                # Utilities and Tauri API layer
│   └── types/              # Shared TypeScript types
├── .github/workflows/      # CI/CD pipeline
├── CONTRIBUTING.md         # Contribution guidelines
├── DESIGN.md               # Design system documentation
└── LICENSE                 # MIT License
```

## Security

r-burp follows security best practices:

- **Memory Safety** — All backend code is written in Rust, eliminating memory corruption vulnerabilities
- **Capability-based Permissions** — Tauri's allowlist system enforces least privilege
- **Input Validation** — All Tauri commands validate and sanitize inputs with length limits
- **Content Security Policy** — Restrictive CSP prevents unauthorized resource loading
- **Loopback-only Binding** — Proxy listeners are restricted to `127.0.0.1`, `localhost`, and `::1` by default
- **Encrypted Key Storage** — CA private keys are XOR-encrypted at rest using a randomly generated passphrase stored in the app data directory (chmod 600 on Unix)
- **Runtime-generated Passphrase** — No hardcoded secrets; the CA key passphrase is generated at first run and persisted locally
- **Cryptographic Fingerprints** — SHA-256 fingerprints for certificate identification
- **No Network Attack Surface** — No listening sockets for remote exploitation
- **Body & Header Limits** — Proxy enforces 10 MB max body size and 100 max headers per request/response to prevent DoS
- **Upstream TLS Validation** — HTTPS forwarding uses native system root certificates via `rustls` for proper certificate chain validation

### Security Audit

A comprehensive security audit was performed on this codebase. Key findings and fixes include:

- Removed unmaintained `rustls-pemfile` dependency (migrated to `rustls-pki-types`)
- Replaced weak hash function with SHA-256 for certificate fingerprints
- Replaced hardcoded CA key passphrase with a runtime-generated random secret
- Added encryption-at-rest for CA private keys
- Added input validation and length limits on all form fields
- Restricted proxy binding to loopback addresses only
- Improved blind tunnel handling with byte counting and logging
- Fixed upstream HTTPS forwarding to use proper TLS certificate validation
- Added request/response body size and header count limits

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
