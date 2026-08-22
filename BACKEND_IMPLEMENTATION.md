# Nocturne CloudCheck Backend Implementation

## Scope

This repository contains a Tauri 2 native host and a reusable Rust core for local AWS configuration snapshot analysis. The implementation does not upload snapshots, findings, or scan history. Version 0.1 accepts normalized AWS JSON snapshots; a live AWS SDK collector is represented by the `AwsReadOnlyCollector` boundary but is not shipped in this version.

## Modules

| Module | Responsibility |
| --- | --- |
| `domain.rs` | Versioned snapshot, inventory, scan, finding, severity, and diff types plus structural limits. |
| `aws.rs` | Bounded AWS snapshot parser and the read-only collector abstraction. |
| `engine.rs` | Deterministic AWS rules, stable fingerprints, prioritization, and scan diffing. |
| `persistence.rs` | SQLite migration, scan history, and non-secret license metadata. |
| `filesystem.rs` | Absolute-path validation, symlink-resistant imports, bounded reads, and no-overwrite exports. |
| `reports.rs` | Rendering of a completed scan to JSON, HTML, CSV, SARIF 2.1.0, and PDF. |
| `licensing.rs` | Lemon Squeezy client-safe activation, validation, deactivation, grace policy, and secret-store abstraction. |
| `updates.rs` | HTTPS and Ed25519 artifact-verification hook using a compile-time public key. |
| `commands.rs` | Narrow typed Tauri IPC orchestration and license gates. |

The rule engine has no filesystem, network, database, Tauri, or report-rendering dependency. A scan captures product and ruleset versions, scope, scan time, limitations, normalized evidence, and remediation. Stable fingerprints are SHA-256 hashes of rule ID and resource ID, so a ruleset release does not make an unchanged issue look resolved and reintroduced. Findings are sorted by severity, rule ID, and resource ID. Blocking filesystem, SQLite, report, and network work runs on Tauri's blocking pool rather than its async IPC executor.

## IPC Contract

Only the main Tauri window has `core:default`. No shell, process, arbitrary HTTP, SQL, updater, or unrestricted filesystem plugin is enabled.

| Command | Input | Result | License gate |
| --- | --- | --- | --- |
| `inspect_snapshot` | absolute JSON `path` | inventory summary | No |
| `run_snapshot_scan` | absolute JSON `path` | persisted scan and findings | Active/grace |
| `list_scans` | optional limit, 1-100 | scan summaries | No |
| `get_scan` | validated scan ID | full scan | No |
| `diff_scans` | two validated scan IDs | added/resolved/unchanged | No |
| `export_scan` | scan ID, existing absolute directory, safe filename, fixed format enum | path and byte count | Active/grace |
| `activate_license` | license key | redacted status only | No |
| `license_status` | none | active/grace/invalid/unlicensed | No |
| `deactivate_license` | none | empty success | No |

Raw internal errors are mapped to stable error codes and generic messages. License keys are accepted only by the activation command and are never returned or logged. Privileged license operations are serialized to prevent duplicate activation/deactivation races.

## Snapshot Contract

The schema is demonstrated by `examples/aws-snapshot.json`. It is strict at snapshot and resource boundaries (`deny_unknown_fields`), is capped at 20 MiB, 10,000 uniquely identified resources, 128 scope entries, 128 properties per resource, eight nested property levels, and bounded property strings/collections. Supported normalized resources are S3 buckets, IAM policies, security groups, CloudTrail trails, and account settings.

Input must be an absolute `.json` path to a regular non-symlink file. The implementation checks metadata before canonicalization and compares opened-file metadata to reduce replacement races. Export accepts an existing directory plus a basename containing only ASCII alphanumerics, `.`, `_`, or `-`; the extension must match the selected format. It writes a mode-0600 temporary file, syncs it, then hard-links it to a non-existing destination, so an existing report is never silently replaced.

## Rules

Ruleset `2026.08.1` includes public S3 access, disabled S3 default encryption, wildcard IAM action plus resource, public SSH/RDP ingress, missing root MFA, and non-multi-region CloudTrail. Checks intentionally report only explicit normalized facts; absent properties do not become findings. This prevents a partial snapshot from being treated as proof of a vulnerability.

## SQLite Schema

The database lives under Tauri's application data directory as `cloudcheck.db`, with WAL, foreign keys, and a five-second busy timeout.

```text
schema_migrations(version PK, applied_at)
scans(id PK, provider, account_id, scanned_at, resource_count,
      finding_count, scan_json)
license_state(singleton PK CHECK = 1, activation_id, status,
              last_verified_at)
```

All values use bound `rusqlite` parameters. The license key is never stored in SQLite. `scan_json` is the immutable report input; indexed summary columns support history without recomputing domain results.

## Licensing

`LemonSqueezyClient` uses only `https://api.lemonsqueezy.com/v1/licenses/{activate,validate,deactivate}` with form fields documented for client-side license operations. It has HTTPS-only networking, fixed endpoints, connect/overall timeouts, and no seller API key or webhook secret. The activated key is stored through the operating-system keyring (`Keychain` on macOS, Windows Credential Manager, Secret Service on Linux). There is deliberately no plaintext fallback when the credential store is unavailable.

After successful online activation or validation, an unavailable service/network permits a seven-day offline grace period measured from `last_verified_at`. An explicit invalid response never receives grace. After grace expires the operation fails closed. Deactivation clears local material only after remote success. If secure storage or local metadata persistence fails during activation, the client attempts remote rollback.

This is pragmatic license enforcement, not unbreakable DRM. A determined local attacker can patch a client binary. Authorization secrets and seller operations must remain server-side.

## Trust Boundaries

1. React to Tauri IPC: all strings and enums are untrusted; Rust validates lengths, IDs, paths, formats, and license state.
2. Tauri to filesystem/import: snapshots are hostile data; reads are bounded and strict, symlink imports are rejected, and exports do not overwrite.
3. Rust to SQLite: only the internal app-data path is opened and every dynamic value is parameterized.
4. Rust to Lemon Squeezy: only a fixed HTTPS origin is contacted; keys enter request bodies and the OS credential store, not diagnostics or SQLite.
5. Rust to future AWS collector: collectors must use least-privilege read-only AWS APIs and normalize results before domain evaluation. Credentials must remain inside the SDK provider chain and must not enter snapshots, findings, or logs.
6. Updater to installer: source validation requires a compile-time HTTPS origin before download. `verify_update` binds the application ID, upgrade-only version, OS/architecture target, URL, and SHA-256 artifact digest to an Ed25519 signature and compile-time public key. Installation must not proceed if configuration or verification fails.

Local processes running as the same OS user, a compromised webview/runtime, and a modified binary remain outside the protection boundary. Code signing and OS package verification reduce distribution tampering but do not make client code secret.

## Reports

Rendering is a pure operation over a stored `Scan`; rules never write presentation formats. JSON preserves the complete scan. HTML escapes every untrusted value and contains no script. CSV quotes all fields and prefixes spreadsheet formula triggers. SARIF includes stable fingerprints and normalized evidence/remediation properties. PDF uses a minimal deterministic PDF 1.4 writer and replaces unsupported non-ASCII glyphs with `?`; JSON/HTML should be preferred where full Unicode fidelity is required.

Given identical serialized scan input, a format renderer produces identical bytes. Scan IDs and timestamps are created before rendering and are expected to differ between separate scan executions.

## Updates And Packaging

Production update builds must set `NOCTURNE_UPDATE_ORIGIN` to the fixed HTTPS scheme/host/port and `NOCTURNE_UPDATE_PUBLIC_KEY` to the base64-encoded 32-byte Ed25519 public key at compile time. The corresponding private key belongs only in protected release CI. The verification hook is intentionally not exposed over IPC. A release integration must validate the envelope before fetching, then invoke full artifact verification before installation.

Release Cargo settings enable single codegen unit, LTO, size optimization, abort-on-panic, and symbol stripping. Tauri uses current-platform `all` bundles: `.deb`/`.rpm` (and supported Linux formats), `.app`/`.dmg` on macOS, and `.msi`/NSIS `.exe` on Windows. Each platform must build on its native CI runner. Production release requirements remain:

- Windows Authenticode certificate and timestamping.
- Apple Developer ID signing, hardened runtime, and notarization.
- Linux repository/package signing where distributed through a repository.
- Protected CI secrets, pinned release workflow actions, generated SBOM, and retained checksums.

No cross-platform package or signature is claimed from a Linux-only verification run.

## Verification

The test suite covers deterministic rules, finding diffs, strict snapshot parsing, duplicate resources, nesting bounds, SQLite round trips and injection-shaped IDs, offline licensing and clock-rollback paths, reactivation rejection, path traversal, symlink import, no-overwrite export, HTML escaping, CSV formula injection, PDF framing, and update source/downgrade rejection paths.

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run build
npm run tauri build
```

On a host without GTK/WebKit development packages, core-only checks remain available with `--no-default-features`; this is not a substitute for the native Tauri build.

Verification on 2026-08-22 used Rust 1.98.0 and a Debian Bookworm container defined by `ci/Dockerfile.linux-build` because the host lacked GTK/WebKit headers. Results:

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS with desktop features.
- `cargo test --workspace`: PASS, 17 tests.
- `npm run build`: PASS.
- `npm audit --audit-level=high`: PASS, zero reported vulnerabilities.
- `npm run tauri -- build --bundles deb,rpm`: PASS; optimized `.deb` and `.rpm` bundles produced.
- Windows and macOS package generation/signing: not run on this Linux runner.

## Focused Security Review

The implementation was reviewed for IPC overreach, shell execution, event-loop blocking, path traversal and replacement races, symlink imports, overwrite behavior, oversized/deep JSON and report amplification, SQL injection, report injection, credential persistence, licensing fail-open and clock rollback, SSRF/replay/cross-target updates, secret/error leakage, and activation races. Fixes included strict unknown-field and duplicate-resource rejection, lower resource and report limits, blocking-pool IPC work, file/directory identity checks, CSV formula neutralization after leading whitespace, atomic no-overwrite export, bounded license responses, generic IPC errors, serialized license operations, reactivation rejection, remote activation rollback, and signed target/version/origin-bound update metadata.

## Limitations

- Version 0.1 imports normalized snapshots and does not yet ship an AWS SDK live collector. The collector interface is present to isolate future credential-bearing code.
- The update verification primitive is implemented, but release-specific download UX, fixed update endpoint, and installation orchestration require product infrastructure.
- OS code signing, notarization, package signing, and the embedded production update public key require release credentials and CI.
- The hand-written PDF renderer uses Helvetica/ASCII and has basic layout rather than full typography or tagged-PDF accessibility.
- SQLite data and exported reports are not encrypted at rest. They inherit OS account and filesystem protections and can contain sensitive cloud metadata.
- Path operations compare stable file/directory identity around each operation and fail safely, but do not use platform directory-handle-relative APIs. A malicious process already running as the same OS user can still race filesystem names and is outside the stated local trust boundary.
