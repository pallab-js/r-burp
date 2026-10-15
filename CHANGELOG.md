# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Content-Length recomputation** — the `Content-Length` header is now recalculated
  whenever a request or response body is modified (via an interactive intercept or a
  `ReplaceBody` rule). Previously a stale length could cause clients and upstreams to
  hang or truncate.
- **Hop-by-hop header stripping** — `connection`, `keep-alive`, `proxy-authenticate`,
  `proxy-authorization`, `proxy-connection`, `te`, `trailer`, `transfer-encoding`, and
  `upgrade` are now stripped from forwarded requests and relayed responses, per
  RFC 7230 §6.1. Previously only `connection` was removed, risking request smuggling and
  framing conflicts with buffered bodies.
- **Certificate Subject Alternative Name** — generated per-domain certificates now set
  an explicit SAN (`DnsName`). Modern browsers validate hostnames against the SAN rather
  than the Common Name, so interception certificates are now accepted correctly.

### Changed

- **CA key signing** — domain-certificate signing reuses the cached CA `KeyPair` instead
  of re-parsing the CA key PEM on every cache miss.
- **Documentation** — the README was rewritten to be minimal and accurate; the key-storage
  description now correctly reflects AES-256-GCM encryption at rest (previously described
  as XOR).
- **Package metadata** — corrected the crate description, authors, license, and repository
  fields.
