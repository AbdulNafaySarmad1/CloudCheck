# Nocturne CloudCheck — Product Release Audit

**Auditor role:** Principal product engineer / security release gate
**Date:** 2026-08-22
**Scope:** Full-stack integration audit of the existing Tauri 2 + React (strict TS) frontend and Rust core, both already implemented by prior agents (`FRONTEND_IMPLEMENTATION.md`, `BACKEND_IMPLEMENTATION.md`). This audit independently re-verifies those claims against the actual source, runs the test/build/lint/audit suite, inspects packaging artifacts, and maps trust boundaries.

## Verdict: **CONDITIONAL PASS**

The implementation is unusually disciplined for a v0.1: narrow typed IPC, deny-by-default snapshot parsing, TOCTOU-resistant file handling, no-overwrite atomic export, parameterized SQL, escaped/neutralized report rendering, OS-keyring-only secret storage, fail-closed licensing after a bounded offline grace, and an update-verification primitive bound to app ID/target/version/origin/artifact hash via Ed25519. No Critical or High findings were identified in the code itself. Customer documentation, CI, Rust dependency auditing, SBOM generation, release manifests, and fresh Linux package verification have now been added. The verdict remains conditional because native Windows/macOS build and trust verification have not run, and customer contact placeholders still require owner-supplied values and approval.

Blocking items for full PASS:
1. Windows and macOS packaging/signing/notarization have never been produced or verified; the prepared native CI jobs require protected credentials and native execution.
2. Customer support, licensing, privacy, security, and effective-date placeholders must be replaced and approved before publication.

Neither item is a code defect; both are closeable without touching the audited code paths.

---

## 1. Architecture & Trust Boundaries

```
React UI (strict TS, no router/state lib)
   │  invoke() — only src/lib/ipc.ts calls Tauri
   ▼
Tauri IPC (capabilities/default.json: "core:default" only, main window only)
   │  9 narrow #[tauri::command]s, typed args, spawn_blocking for all I/O
   ▼
Rust core (nocturne_cloudcheck)
   ├─ filesystem.rs   → local snapshot import / report export (hostile-data boundary)
   ├─ engine.rs       → pure deterministic rule evaluation (no I/O)
   ├─ persistence.rs  → SQLite (app-data dir only, parameterized)
   ├─ licensing.rs    → Lemon Squeezy HTTPS client + OS keyring
   ├─ reports.rs      → pure Scan → bytes renderers (JSON/HTML/CSV/SARIF/PDF)
   └─ updates.rs      → Ed25519 artifact/metadata verification (not yet wired to IPC)
   ▼
SQLite (cloudcheck.db, WAL) · Filesystem (import/export) · api.lemonsqueezy.com · (future) AWS read-only APIs
```

Six boundaries were explicitly reviewed:

| # | Boundary | Verified control |
|---|---|---|
| 1 | React → IPC | Every `ipc.ts` export maps 1:1 to a real `#[tauri::command]` in `commands.rs`; args/results are typed on both sides; unknown invoke failures collapse to a generic `IPC_UNAVAILABLE` client error, never a raw Rust error string. |
| 2 | IPC → filesystem | Absolute-path-only, symlink rejection via `symlink_metadata` before *and after* `canonicalize()`, device/inode (or volume-serial/file-index) identity comparison to defeat TOCTOU replacement races, 20 MiB / 10k-resource / 8-level snapshot caps, extension allow-list. |
| 3 | Rust → SQLite | Single `app_data_dir()`-scoped file; 100% `rusqlite` bound parameters; `get_scan`/`list_scans` reject non-alphanumeric IDs before ever reaching SQL. |
| 4 | Rust → Lemon Squeezy | Fixed `https://api.lemonsqueezy.com/v1/licenses/*` origin, `https_only(true)`, connect/overall timeouts, 64 KiB response cap, no seller API key or webhook secret anywhere in the client binary. |
| 5 | Rust → future AWS collector | `AwsReadOnlyCollector` trait is a boundary marker only — not implemented in v0.1. No credential-bearing code exists yet, so this boundary is currently *unused*, not weak. |
| 6 | Updater → installer | `verify_update()` binds app ID, upgrade-only semver, OS/arch target, HTTPS origin match, artifact SHA-256, and Ed25519 signature before any install would be allowed. **Not exposed over IPC or wired to any download/install flow in v0.1** — see Finding F-3. |

Local processes running as the same OS user and a modified/patched binary are explicitly (and correctly) out of scope — the implementation does not claim unbreakable DRM anywhere in code, docs, or UI copy (verified by grep across `src/` and `src-tauri/src/`).

---

## 2. Acceptance Matrix

| Capability | Status | Evidence |
|---|---|---|
| Signed platform installer/package + Lemon Squeezy key delivery | **Partial** | Real, checksummed `.deb`/`.rpm` exist (`artifacts/`); Windows/macOS never built; no code signing anywhere (expected — needs release credentials). |
| Onboarding / activation | **Pass** | `License.tsx` + `activate_license`/`license_status`/`deactivate_license`; masked key input, `autoComplete="off"`, cleared from component state before await; single-active-license model enforced server-side (`activate` rejects if a license row already exists). |
| AWS-first provider adapter(s) | **Pass for v0.1 scope** | One adapter (normalized JSON snapshot import) is real and fully wired; live AWS SDK collection is explicitly labeled "Not shipped in v0.1" in the UI (`Overview.tsx`) rather than faked. |
| Deterministic rule engine | **Pass** | `engine.rs` — pure function, no I/O; test asserts byte-identical serialization across two runs of the same input; 6 rules across S3/IAM/SG/CloudTrail/Account. |
| Inventory | **Pass** | `inspect_snapshot` → `InventorySummary` (counts by kind/region) surfaced in `Inventory.tsx`, no license gate (correct — inspection should stay available). |
| Findings | **Pass** | Severity-sorted, evidence + remediation per finding, all rendered via React text nodes (no `dangerouslySetInnerHTML` anywhere in `src/`, confirmed by repo-wide grep). |
| Remediation | **Pass** | Same `FindingsView` component, `remediationOnly` mode; remediation text is static per-rule guidance, not a mutating action (correctly scoped as operator guidance only). |
| Scan history / diff | **Pass** | SQLite-backed `list_scans`/`get_scan`, fingerprint-based `diff_scans` (SHA-256 of rule ID + resource ID) — ruleset content changes don't spuriously resolve/reintroduce findings. |
| Exports: PDF/HTML/JSON/SARIF/CSV | **Pass** | All 5 implemented in `reports.rs`; HTML fully escapes; CSV neutralizes leading `=`/`+`/`-`/`@` (including after whitespace) — both covered by passing unit tests. PDF is a genuine minimal PDF-1.4 writer, not a stub (verified structurally: `%PDF-1.4` header, valid xref/trailer). |
| Version / update UX | **Partial** | Version shown in Settings; "Signed updates" section is honestly labeled `NOT CONFIGURED` rather than simulated. Verification primitive exists but has no product wiring — acceptable for v0.1 *because it is disclosed*, not silently missing. |
| User/privacy/security docs | **Partial** | `README.md`, `PRIVACY.md`, and `SECURITY.md` now document the implemented product accurately. Owner-supplied contact values, effective date, and formal review remain GA gates. See F-1. |

---

## 3. Security Findings

No Critical or High findings. Findings below are Medium/Low/Informational and gate CONDITIONAL PASS rather than full PASS.

### F-1 (Medium — Process/Docs) Customer documents added; publication values pending
**Current evidence:** `README.md`, `PRIVACY.md`, and `SECURITY.md` now cover installation, activation, scanning, exports, local data handling, licensing traffic, limitations, and coordinated vulnerability reporting. `GA_CHECKLIST.md` prevents publication until support, licensing, privacy, security, and effective-date placeholders are replaced and reviewed.
**Disposition:** Engineering documentation work is complete. Product/legal/security owners must provide and approve the publication values before GA.

### F-2 (Medium — Packaging) Windows/macOS build, signing, and notarization unverified
**Evidence:** `.github/workflows/release.yml` now defines explicitly unsigned native validation jobs and credential-gated signed Windows/macOS jobs. Signing/notarization verification is fail-closed, but no `.msi`/`.exe`/`.app`/`.dmg` has been produced against this codebase and the workflow has not run on native runners.
**Fix:** Execute the native workflow with protected credentials, inspect downloaded artifacts, and complete clean-machine install/runtime tests before claiming either platform.

### F-3 (Low — Product completeness) Update verification exists but has no delivery path
**Evidence:** `updates.rs::verify_update` is fully implemented and unit-tested (rejects non-HTTPS, wrong target, downgrades, tampered signature) but is `pub` and never called from `commands.rs`, `lib.rs`, or anywhere else reachable from the UI. There is no `NOCTURNE_UPDATE_ORIGIN`/`NOCTURNE_UPDATE_PUBLIC_KEY` set in this build (`option_env!` reads compile-time env that isn't configured here), so update checks would fail closed if ever invoked — which is the correct fail-safe default, but also means the feature does not exist yet from a user's perspective.
**Fix:** Not a release blocker for v0.1 since the UI honestly reports "NOT CONFIGURED" rather than pretending updates work. Track as required pre-v1.1 work: fixed update endpoint, download orchestration, IPC command, and CI-injected release keys.

### F-4 (Resolved — Supply chain) Rust dependency gate and SBOMs added
**Current evidence:** `cargo-audit` 0.22.2 reports zero vulnerabilities. Seventeen informational unmaintained/unsound/transitive GTK advisories are enumerated in `security/advisory-policy.json`, justified in `RUST_ADVISORY_REVIEW.md`, and expire on 2026-11-30. `scripts/check-cargo-audit.mjs` fails on vulnerabilities, new warnings, or expiry. CI generates separate CycloneDX npm runtime and Cargo application SBOMs; the Rust SBOM sanitizer removes local filesystem references.
**Disposition:** Resolved for this candidate, subject to the time-bounded advisory review and normal CI execution.

### F-5 (Informational) Non-unix/non-windows file-identity fallback is weak but unreachable
**Evidence:** `filesystem.rs::same_file_identity` has a third branch (`cfg(not(any(unix, windows)))`) that compares length/created/modified timestamps instead of device+inode or volume-serial+file-index. This is a materially weaker TOCTOU check. However, Tauri's supported desktop targets are exactly Windows, macOS, and Linux — macOS and Linux both compile under `cfg(unix)`, so this fallback branch is **dead code on every platform this product ships to**. No action required; noting for completeness so a future platform addition doesn't silently inherit weak protection.

### F-6 (Informational) No version control repository
**Evidence:** The working directory has no `.git`. There is no commit history, no code-review trail, no signed commits, and no way to attribute or diff prior changes from prior agents beyond the two implementation-notes files.
**Fix:** Not a code security issue, but initialize git and start committing before further iteration — the current state depends entirely on filesystem timestamps and self-reported markdown for provenance, which is a weak foundation for a security product's own audit trail. Offer to `git init` and make an initial commit if the user wants this addressed now.

---

## 4. Security Test Results (this session)

| Test area | Result |
|---|---|
| IPC surface completeness | 9/9 frontend `ipc.ts` calls map to real, narrowly-typed Rust commands; capability file grants only `core:default` to the main window — no shell/process/sql/http/fs/updater plugin is declared in `Cargo.toml` or `tauri.conf.json`. |
| Path traversal / symlink import | `filesystem::tests::import_rejects_symlinks` and `export_rejects_traversal_and_overwrite` pass; manual code read confirms `symlink_metadata` checks precede `canonicalize()`, and identity is re-verified after canonicalization (defeats TOCTOU swap). |
| Oversized / malformed import | `Snapshot` uses `#[serde(deny_unknown_fields)]`; 20 MiB cap enforced by bounded read (`take(MAX+1)`) before parse, not after; resource/scope/property/depth/string/array limits all enforced in `domain.rs::validate`. |
| SQL injection | `persistence::tests::scan_round_trip_and_parameterized_lookup` explicitly asserts `get_scan("' OR 1=1 --")` fails (fails ID format check before reaching SQL); all queries use bound `rusqlite` params. |
| Secret leakage | Repo-wide grep for `api_key`/`secret`/`webhook` finds no hardcoded credentials; license key never appears in `CommandError`, SQLite, or logs (no `println!`/`log::`/`tracing::` calls exist in `src-tauri/src` at all, and no `console.*` calls in `src/`). |
| Local secret storage | License key stored exclusively via OS keyring (`keyring` crate, apple-native/windows-native/secret-service backends); explicit design decision documented and verified: "no plaintext fallback when the credential store is unavailable." |
| License-bypass assumptions | `require_license()` gates `run_snapshot_scan`/`export_scan` only; `invalid_remote_response_never_gets_grace` and `rejects_clock_rollback_and_reactivation` tests pass — an explicit invalid response never gets offline grace, and a clock rolled backward invalidates the cached state rather than extending it. |
| Update tampering | `updates::tests::rejects_non_https_and_wrong_targets` and `rejects_downgrades_before_download` pass; verification is origin+target+version+hash+signature bound. |
| Report HTML injection | `reports::tests::html_escapes_untrusted_values` passes using a `<script>alert(1)</script>` account ID as the adversarial input; CSV formula-injection test (`=HYPERLINK`, tab-prefixed) also passes. |
| Command/shell injection | No `std::process::Command`, `shell`, or Tauri shell plugin anywhere in `src-tauri` (confirmed by dependency list in `Cargo.toml` — no `tauri-plugin-shell`). |
| Dependency/supply chain | `npm audit --audit-level=high`: 0 vulnerabilities. `cargo audit` unavailable in this environment (F-4). `Cargo.lock` and `package-lock.json` are both committed/present, so builds are reproducible. |
| Cloud least privilege | No live AWS collector is implemented in v0.1 (`AwsReadOnlyCollector` is an unused trait boundary), so there is no credential-handling code to test yet. This is the correct posture — nothing to over-privilege. |

---

## 5. Verification Evidence (commands actually run this session)

Environment: this Linux host lacks GTK/WebKit development headers (confirmed via `pkg-config --exists gtk+-3.0`/`webkit2gtk-4.1`, both failed), matching what `BACKEND_IMPLEMENTATION.md` already documented. Rust 1.98.0, Node 24.18.0, npm 12.0.2.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **PASS** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** in `ci/Dockerfile.linux-build` with desktop dependencies |
| `cargo test --workspace` | **PASS** in `ci/Dockerfile.linux-build` — 17/17 tests |
| `npm run typecheck` (`tsc -b --pretty false`) | **PASS** |
| `npm run lint` (`eslint . --max-warnings 0`) | **PASS** |
| `npm run test` (`vitest run`) | **PASS** — 3 files, 5/5 tests |
| `npm run build` | **PASS** — Vite production bundle emitted to `frontend-dist/` |
| `npm audit --audit-level=high` | **PASS** — 0 vulnerabilities |
| `sha256sum -c artifacts/SHA256SUMS` | **PASS** — both `.deb`/`.rpm` match their recorded checksums |
| `dpkg-deb -c`/`-I` on the `.deb` | **PASS** — genuine 7.5 MB `nocturne-cloudcheck` binary, correct `Depends:` line, no stray files |
| Windows `.msi`/`.exe` build | **Not run** — no Windows runner available |
| macOS `.app`/`.dmg` build + notarization | **Not run** — no macOS runner available |
| `npm run audit:rust` (`cargo audit`) | **PASS** — 0 vulnerabilities; 17 time-bounded reviewed informational warnings |
| `npm run sbom:node` / `npm run sbom:rust` | **PASS** — separate CycloneDX JSON SBOMs generated; no local path remains in the Rust SBOM |
| Fresh `npm run tauri -- build --bundles deb,rpm` | **PASS** in the Node 22/Rust container |
| Fresh `.deb`/`.rpm` metadata and contents | **PASS** — version 0.1.0, x86_64/amd64, expected binary and desktop entry only; RPM explicitly reports no signature |

No result above was fabricated or assumed; every PASS reflects a command actually executed in this session against the real repository state.

---

## 6. Licensing & Delivery Flow

Purchase → download → activate → use → deactivate/reactivate, as implemented:

1. **Purchase**: Out of scope for this codebase (Lemon Squeezy hosted checkout) — correctly, no seller API key or checkout logic exists client-side.
2. **Download**: `.deb`/`.rpm` exist and are genuine; Windows/macOS packages do not yet exist (F-2).
3. **Activate**: `License.tsx` → `activate_license` → `LemonSqueezyClient::activate` (HTTPS, form POST, generated `instance_name`) → on success, key goes to OS keyring and `(activation_id, status, last_verified_at)` goes to SQLite (never the key itself). Re-activation while a license row exists is rejected server-side — a customer must deactivate before moving to a second machine, matching typical per-seat licensing policy.
4. **Use**: `license_status()` re-validates against Lemon Squeezy; `Active`/`Grace` unlock scan + export, `Unlicensed`/`Invalid` correctly leave inspection/history available but block scanning/export (`require_license` in `commands.rs`).
5. **Offline grace**: exactly 7 days from `last_verified_at`, only reachable from a prior successful activation/validation (never from an `Invalid` response), fails closed after expiry (`AppError::LicenseUnavailable` surfaces as a hard error, not silent unlock).
6. **Deactivate**: clears local keyring + SQLite state only *after* remote deactivation succeeds — an attacker forcing a network failure cannot use client-side deactivation to reset local state while keeping the remote seat consumed... conversely, remote-first means a genuinely offline user cannot deactivate at all, which is documented in the UI ("Deactivation requires network access") rather than hidden.
7. **Update**: Verification primitive exists, delivery path does not (F-3).

This is a defensible, non-DRM-larping approach: it stops casual sharing without pretending to stop a determined local attacker, and the codebase makes that limitation explicit rather than marketing "unbreakable" protection anywhere.

---

## 7. Packaging Matrix

| Target | Config present | Build evidence | Signing/notarization |
|---|---|---|---|
| Windows `.msi`/`.exe` | Yes (`tauri.conf.json` → `bundle.targets: "all"`, icons include `.ico`) | **None produced** | Not started — needs Authenticode cert + timestamping, and a Windows CI runner |
| macOS `.app`/`.dmg` | Yes (icons include `.icns`) | **None produced** | Not started — needs Apple Developer ID, hardened runtime, notarization, and a macOS CI runner |
| Debian/Ubuntu `.deb` | Yes | **Freshly built and verified** (checksum, control metadata, and file listing) | Unsigned standalone package; no repository signature claimed |
| Fedora/RHEL `.rpm` | Yes | **Freshly built and verified** (checksum, RPM metadata, and file listing) | RPM reports `Signature: (none)`; no repository signature claimed |

No cross-platform claim is made beyond what was actually built and checked in this session, per the mission's explicit instruction not to fake cross-platform verification.

---

## 8. Privacy

- CSP (`tauri.conf.json`): `default-src 'self'; img-src 'self' asset:; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost` — no external origin is reachable from the webview at all; the only network call in the entire product is the Rust-side HTTPS call to `api.lemonsqueezy.com`, which never touches the webview.
- No analytics/telemetry SDK, no external font loading, no router-driven history leakage, confirmed by dependency list (`package.json`) and CSP.
- Recent snapshot paths live in React state only (`useState`), never `localStorage`/`sessionStorage`/cookies/IndexedDB — confirmed by grep (no such API calls in `src/`).
- Scan data and exported reports are unencrypted at rest, inheriting OS filesystem/account protections only — disclosed in both implementation docs and should be repeated in the customer-facing privacy doc from F-1, since exported reports "can contain sensitive cloud metadata" (already stated in-app, in `Reports.tsx`).

---

## 9. Blockers vs. Post-v1 Work

**Must close before GA (blocking this audit's full PASS):**
- F-1: Replace customer-facing contact/effective-date placeholders and obtain owner review.
- F-2: Produce and verify real signed Windows and signed/notarized macOS builds before claiming those platforms are supported.

**Closed release-infrastructure work:**
- F-4: RustSec auditing and CycloneDX SBOM generation are wired into CI.
- Native CI, release manifests, checksums, signing gates, and provenance requests are defined in `.github/workflows/`.

**Explicit, disclosed post-v1 roadmap (not defects):**
- Live read-only AWS SDK collector (`AwsReadOnlyCollector` is currently an unimplemented boundary by design).
- Update discovery/download/install orchestration wired to the existing `verify_update` primitive (F-3).
- Native file/directory picker IPC command (paths are currently typed manually, which is safe but not ideal UX).
- Persisted (opt-in, non-secret) recent-path retention beyond the current session.
- Tagged/accessible PDF output; focus-trap and Escape-to-close on the finding detail dialog.
- At-rest encryption for SQLite/export data, if required by target enterprise customers.

---

## Summary

This is a well-engineered v0.1 with the local release-quality and supply-chain gaps closed. Nothing required a Critical/High override. The gate remains **CONDITIONAL PASS** on release operations: Windows/macOS native signed artifacts and clean-machine evidence do not yet exist, and customer-facing contact/effective-date values still require owner input and review. Linux packages are reproducibly built, structurally inspected, and checksummed, but clean-machine product-flow testing remains a GA checklist item.
