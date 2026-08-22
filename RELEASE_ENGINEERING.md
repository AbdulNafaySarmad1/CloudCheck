# Nocturne CloudCheck Release Engineering

## Release Architecture

Releases move through these gates:

1. A reviewed Git commit passes `.github/workflows/ci.yml`: version consistency, frontend checks, npm audit, Rust formatting/lint/tests, a native Linux link, RustSec audit, and CycloneDX SBOM generation.
2. `.github/workflows/release.yml` builds packages on native GitHub-hosted runners. Manual `unsigned-validation` builds are explicitly labelled and retained for platform QA only.
3. A protected `signed-release` dispatch or a version tag builds production candidates. Windows signing and macOS signing/notarization jobs fail immediately if any required protected value is missing.
4. The Tauri bundler creates native packages. Platform tools verify signatures/notarization before a signed state can enter the release manifest.
5. `scripts/release-manifest.mjs` computes SHA-256 hashes and records application, version, platform, architecture, filename, and observed signing state without absolute local paths.
6. GitHub Actions retains packages, manifests, checksums, and CycloneDX SBOMs. Signed Windows/macOS jobs also request GitHub build-provenance attestations.
7. A release operator verifies the downloaded workflow artifacts on clean machines, completes `GA_CHECKLIST.md`, then uploads only approved customer installers and their checksums to Lemon Squeezy.

Unsigned validation artifacts are never GA artifacts. A successful build does not imply a valid signature, notarization, installation, or runtime test.

## CI Jobs

### Continuous Integration

`ci.yml` runs on pull requests and `main`:

- **Frontend and Rust quality (Ubuntu 24.04):** Node 22, `npm ci`, version check, typecheck, lint, tests, production frontend build, npm audit, Rust formatting, full desktop clippy/tests, and a no-bundle Tauri release link.
- **RustSec advisory gate:** installs locked `cargo-audit` 0.22.2 and fails on vulnerabilities reported for `Cargo.lock`. There are no ignored advisories or committed exceptions.
- **CycloneDX SBOM:** creates separate npm runtime and Cargo application SBOMs and retains them for 90 days.

### Native Release Candidates

`release.yml` runs manually or for `v*` tags:

| Job | Runner | Output | Trust state |
| --- | --- | --- | --- |
| Linux | Ubuntu 24.04 | `.deb`, `.rpm` | Package built; no repository/package signature claimed |
| Unsigned Windows validation | Windows 2025 | `.msi`, NSIS `.exe` | Explicitly unsigned and non-GA |
| Unsigned macOS validation | macOS 15 arm64 | `.app`, `.dmg` | Explicitly unsigned and non-GA |
| Signed Windows | Windows 2025 | `.msi`, NSIS `.exe` | Authenticode and RFC 3161 timestamp required and verified |
| Signed macOS | macOS 15 arm64 | `.app`, `.dmg` | Developer ID, hardened runtime, notarization/stapling, Gatekeeper verification required |

The native Windows/macOS jobs have not run in this local Linux environment. Their configuration is release preparation, not evidence of platform success.

## Required Release Values

Store secrets in a protected GitHub Actions environment or organization/repository secret store. Store non-secret fixed configuration as GitHub Actions variables. Do not place values in repository files, workflow YAML, build logs, artifacts, or `.env` files.

### Windows

| Name | Storage | Purpose |
| --- | --- | --- |
| `WINDOWS_CERTIFICATE_PFX_BASE64` | CI secret | Base64-encoded Authenticode PFX certificate and private key |
| `WINDOWS_CERTIFICATE_PASSWORD` | CI secret | PFX import password |
| `WINDOWS_TIMESTAMP_URL` | CI variable | HTTPS RFC 3161 timestamp service URL |

`NOCTURNE_WINDOWS_SIGNING_ENABLED=true` is set only by the signed job. The Tauri Windows signing command imports the PFX into the ephemeral current-user store, signs and verifies each binary, removes the certificate, and deletes the temporary PFX. The unsigned validation job sets it to `false` and labels its output accordingly.

### macOS

| Name | Storage | Purpose |
| --- | --- | --- |
| `APPLE_CERTIFICATE` | CI secret | Base64-encoded Developer ID Application certificate/P12 expected by Tauri |
| `APPLE_CERTIFICATE_PASSWORD` | CI secret | Certificate archive password |
| `APPLE_SIGNING_IDENTITY` | CI variable | Exact `Developer ID Application: ...` identity |
| `APPLE_ID` | CI secret | Apple account used for notarization |
| `APPLE_PASSWORD` | CI secret | App-specific password for notarization |
| `APPLE_TEAM_ID` | CI secret | Apple Developer team identifier |

`tauri.macos.conf.json` enables hardened runtime. The signed job does not label output notarized until `codesign`, `stapler`, and `spctl` verification all succeed.

### Update Verification

The Rust implementation reads these compile-time environment variables:

| Name | Purpose |
| --- | --- |
| `NOCTURNE_UPDATE_ORIGIN` | Fixed HTTPS scheme/host/port permitted for update artifacts |
| `NOCTURNE_UPDATE_PUBLIC_KEY` | Base64-encoded Ed25519 public verification key |

The private update signing key must remain in a separate protected signing service/CI secret, but no private-key environment name is defined because v0.1 has no update publishing or installation workflow. Do not invent or embed one. These two public build values are not set by current release jobs because update delivery is not shipped in v0.1.

### Lemon Squeezy

No Lemon Squeezy seller API key or webhook secret belongs in build CI or the binary. Desktop licensing uses customer license keys against Lemon Squeezy's fixed public license endpoints.

## Local Verification

Use Node 22 and the repository Rust toolchain:

```bash
npm ci
npm run version:check
npm run typecheck
npm run lint
npm run test
npm run build
npm audit --audit-level=high
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
npm run sbom:node
npm run sbom:rust
```

Linux native development headers are required for full desktop checks. `ci/Dockerfile.linux-build` provides the packaging environment. Install `cargo-audit` 0.22.2 and `cargo-cyclonedx` 0.5.9 with `--locked` when absent.

## Signing And Notarization

### Windows Authenticode

The signed job uses `tauri.windows.conf.json` and `scripts/sign-file-windows.ps1`. It requires a valid code-signing certificate with private key, SHA-256 file digest, and HTTPS RFC 3161 timestamping. Tauri invokes the sign command for application/package binaries. After bundling, PowerShell requires `Get-AuthenticodeSignature` status `Valid` and a timestamp certificate for every `.msi` and `.exe`. Any failure stops the release.

### macOS Developer ID

Tauri imports the protected Developer ID certificate, signs under hardened runtime, and submits with the configured Apple credentials. After bundling, CI verifies the `.app` signature, staples the notarization ticket to the `.dmg`, validates the ticket, and asks Gatekeeper to assess the image. A signed but unnotarized artifact is a failed production job, not a releasable fallback.

### Linux Packaging

Linux uses Tauri's Debian and RPM bundlers on Ubuntu 24.04. The workflow creates `.deb` and `.rpm`, SHA-256 checksums, and a manifest. These standalone packages are not described as repository-signed. If future distribution uses apt or rpm repositories, repository metadata/package signing needs a separately documented protected-key process.

## SBOM And Advisory Policy

`npm run audit:rust` is a hard CI gate for RustSec vulnerabilities and any new informational warning. The current time-bounded informational review is documented in `RUST_ADVISORY_REVIEW.md` and enumerated in `security/advisory-policy.json`; no `cargo audit` ignore suppresses it. If a future advisory cannot be immediately removed, a time-bounded exception must name the dependency/advisory, severity, runtime reachability, owner, reason, compensating controls, expiry date, and removal condition; the independent auditor must review it before merge.

Release SBOMs use CycloneDX JSON:

- `nocturne-cloudcheck-node.cdx.json` covers packaged npm runtime dependencies from `package-lock.json` and omits development-only packages.
- `nocturne-cloudcheck-rust.cdx.json` covers the Cargo workspace resolved by `Cargo.lock`.

SBOMs contain dependency metadata, not credentials. They are retained with release evidence and should accompany archived GA artifacts.

## Version Bump Process

The product version is duplicated only where required by package tooling. Rust runtime/report/update metadata derives from Cargo; the pre-scan About view derives from `package.json`; Tauri uses its own package metadata. To release `X.Y.Z`:

1. Run `npm version X.Y.Z --no-git-tag-version` to update `package.json` and `package-lock.json`.
2. Set `version = "X.Y.Z"` in `src-tauri/Cargo.toml`.
3. Set `"version": "X.Y.Z"` in `src-tauri/tauri.conf.json`.
4. Run `cargo check --workspace --no-default-features` so `Cargo.lock` records the workspace package version.
5. Run `npm run version:check X.Y.Z` and all local verification commands.
6. Build candidates and confirm package filenames, About, scan/report metadata, checksums, and manifests all show `X.Y.Z`.
7. Complete `GA_CHECKLIST.md`, create the annotated `vX.Y.Z` tag, and let protected release CI rebuild from that exact tag.

Do not edit generated artifact names or report metadata manually.

## Lemon Squeezy Delivery

Before a test purchase, the operator must configure:

1. Create the Nocturne CloudCheck product and a v0.1 desktop-license variant.
2. Enable license keys for the variant and choose an activation limit/seat count matching the published sales terms. The current app permits one locally active key at a time and requires deactivation before replacing it.
3. Confirm Lemon Squeezy generates a license key for completed orders. Do not create seller credentials in the desktop app.
4. Upload only independently verified GA installers: signed/timestamped `.msi` and NSIS `.exe` if Windows is approved, signed/notarized `.dmg` if macOS is approved, and verified `.deb`/`.rpm` for approved Linux distributions.
5. Use customer-facing filenames containing product, version, platform, and architecture. Upload/publish matching `SHA256SUMS`; retain release manifests and SBOMs in the internal archive.
6. Describe the seven-day conditional offline grace, seat/activation limit, network-required activation/validation/deactivation, and supported platforms accurately in checkout/download copy.
7. Complete a test-mode purchase using the exact production product/variant settings. Verify key delivery, activation, repeated validation, simulated service unavailability within grace, invalid/expired behavior, deactivation, seat release, download permissions, and receipt/download filenames.
8. Replace all customer documentation contact placeholders before enabling live sales.

Do not upload or advertise a platform artifact until its native build, signature/notarization, clean-machine install, launch, license flow, and uninstall have passed.

## Pre-Tag Release Checklist

- Review and complete every item in `GA_CHECKLIST.md` against artifacts built from the candidate commit.
- Confirm CI and RustSec gates are green with no unreviewed exception.
- Confirm both CycloneDX SBOMs and all release manifests/checksums are retained.
- Independently verify Windows Authenticode/timestamp and macOS signing/notarization on downloaded artifacts.
- Install and exercise every advertised package on a clean supported OS.
- Exercise purchase, activation, validation, grace, deactivation, scan, diff, and every export format.
- Replace support/privacy/security placeholders and obtain the required product/security/privacy approvals.
- Archive the exact approved artifacts, checksums, manifests, SBOMs, and CI provenance.
- Tag only after approval; never relabel an unsigned validation artifact as a signed release.
