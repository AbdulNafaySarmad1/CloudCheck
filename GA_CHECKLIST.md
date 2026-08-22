# Nocturne CloudCheck 0.1 GA Checklist

Reusable pre-tag gate. Every item must begin unchecked for each release candidate and be verified against the candidate artifacts.

## Mandatory GA Gate

- [ ] Frontend checks pass.
- [ ] Rust checks pass.
- [ ] Rust vulnerability scan passes or has been reviewed under the documented exception policy.
- [ ] SBOM is produced and retained.
- [ ] Windows package is built.
- [ ] Windows package signature and timestamp are verified.
- [ ] macOS package is built.
- [ ] macOS signing is verified.
- [ ] macOS notarization and stapling are verified.
- [ ] Linux `.deb` is built.
- [ ] Linux `.rpm` is built.
- [ ] Artifact SHA-256 checksums and release manifests are generated.
- [ ] Customer README is reviewed.
- [ ] Privacy document is reviewed.
- [ ] Security reporting document is reviewed.
- [ ] Lemon Squeezy product and licensed variant are configured.
- [ ] License activation is tested.
- [ ] License validation is tested.
- [ ] Offline grace is tested.
- [ ] License deactivation is tested.
- [ ] Clean-machine installation is tested on every advertised platform.
- [ ] Test purchase is completed.
- [ ] Every exported report format is tested.
- [ ] Release artifacts, manifests, checksums, SBOMs, and provenance are archived.
- [ ] Release version is tagged only after approval.

## Customer Documentation

- [ ] `README.md` is customer-facing and appropriate for paying 0.1 users.
- [ ] README explains what Nocturne CloudCheck is and its local-first model.
- [ ] README lists supported resources, deterministic checks, inventory, findings, remediation guidance, history, diffs, and all five export formats.
- [ ] README lists non-features: no live AWS collection, AWS mutation, automated remediation, update discovery/installation, persisted recent paths, or native path picker.
- [ ] README identifies normalized AWS JSON as the only 0.1 input and links `examples/aws-snapshot.json`.
- [ ] README documents absolute `.json` path, regular non-symlink file, strict schema, 20 MiB, and 10,000-resource limits.
- [ ] README installation claims are limited to verified Linux `.deb` and `.rpm` packages.
- [ ] README explicitly states Windows and macOS are not yet verified or supported for 0.1.
- [ ] README covers activation, inspection, licensed scanning, findings, remediation, history, diffing, and export.
- [ ] README explains active/grace license gates, seven-day conditional offline grace, fail-closed expiry, and no grace for an invalid response.
- [ ] README explains network-required deactivation and moving a license between machines.
- [ ] README explains local snapshot, SQLite, OS keyring, report, and session-only recent-path storage.
- [ ] README includes practical troubleshooting for import, licensing, scan/export, filename, and PDF limitations.
- [ ] README version, customer support, and licensing support placeholders are replaced with release values.
- [ ] README makes no live AWS, automatic remediation, cross-platform, or auto-update claim.

## Privacy Documentation

- [ ] `PRIVACY.md` accurately states that snapshot analysis is local-first.
- [ ] Privacy notice states snapshots, findings, reports, and history are not uploaded and no product telemetry is collected.
- [ ] Privacy notice identifies local reports, session-only recent paths, and SQLite scan history.
- [ ] Privacy notice describes Lemon Squeezy licensing contact and the licensing data flow.
- [ ] Privacy notice states the license key is stored only in the OS keyring and non-secret license metadata is stored in SQLite.
- [ ] Privacy notice discloses that SQLite data and reports are not encrypted at rest and rely on OS/filesystem protections.
- [ ] Privacy notice warns that snapshots, history, reports, and backups may contain sensitive cloud metadata.
- [ ] Privacy notice states live AWS access and AWS credential handling are not included in 0.1 and future AWS functionality requires updated disclosure.
- [ ] Privacy notice avoids legal, compliance, confidentiality, or security guarantees.
- [ ] Privacy, licensing, and effective-date placeholders are replaced and reviewed by the appropriate owner.

## Security Documentation

- [ ] `SECURITY.md` identifies the current `0.1.x` line as supported.
- [ ] Security contact placeholder is replaced with a monitored private reporting channel.
- [ ] Reporting guidance requests version, platform, component, impact, reproduction, sanitized evidence, public-status, and credit details.
- [ ] The investigation, clarification, remediation/release, and disclosure-coordination process is described accurately.
- [ ] Coordinated private disclosure and a reasonable remediation opportunity are requested.
- [ ] No response SLA, remediation SLA, safe harbor, bounty, payment, or legal guarantee is invented.
- [ ] Architecture summary covers React, narrow Tauri IPC, Rust validation/rules, SQLite, reports, filesystem, and Lemon Squeezy.
- [ ] Security policy states that live AWS collection and update download/installation are absent in 0.1.
- [ ] Security policy describes local DRM realistically and does not claim an unbreakable client-side license gate.
- [ ] Security policy gives safety guidance for sensitive snapshots, history, findings, reports, backups, diagnostics, and secrets.

## Release Evidence

- [ ] The release version and ruleset version are correct in the application, package metadata, reports, and documentation.
- [ ] Linux `.deb` and `.rpm` are built from the tagged candidate on the intended release toolchain.
- [ ] Linux packages install, launch, activate, inspect, scan, diff, export, deactivate, and uninstall on supported distributions.
- [ ] Release checksums are generated, retained, published, and independently verified.
- [ ] Any Linux repository distribution uses the intended package-signing process.
- [ ] Windows is not advertised unless native `.msi`/`.exe` build, installation, runtime, Authenticode signing, and timestamping are verified.
- [ ] macOS is not advertised unless native `.app`/`.dmg` build, runtime, Developer ID signing, hardened runtime, and notarization are verified.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with desktop features.
- [ ] `cargo test --workspace` passes.
- [ ] `npm run typecheck`, `npm run lint`, `npm run test`, and `npm run build` pass.
- [ ] `npm audit --audit-level=high` passes under the release vulnerability policy.
- [ ] `cargo audit` or `cargo deny check advisories` runs under the release vulnerability policy.
- [ ] A release SBOM is generated and retained.
- [ ] Release CI uses protected secrets, appropriately pinned actions or dependencies, and native platform runners where claimed.
- [ ] No unresolved Critical or High security finding remains; lower-severity accepted risks have an owner and disposition.

## Product And Data Verification

- [ ] The normalized example snapshot imports successfully and malformed, oversized, duplicate, deeply nested, unknown-field, relative-path, and symlink inputs fail safely.
- [ ] The six documented deterministic checks produce stable findings from explicit normalized facts only.
- [ ] Inventory, finding evidence, remediation guidance, local history, and added/resolved/unchanged diffs behave as documented.
- [ ] PDF, HTML, JSON, SARIF 2.1.0, and CSV exports are verified, never overwrite existing files, and preserve documented escaping/neutralization behavior.
- [ ] License keys do not appear in UI responses, SQLite, reports, logs, diagnostics, or support artifacts.
- [ ] Activation/validation uses only fixed Lemon Squeezy HTTPS endpoints and the OS keyring has no plaintext fallback.
- [ ] Active, grace, invalid, unlicensed, offline-expiry, clock-rollback, reactivation, and network-required deactivation paths are verified.
- [ ] Snapshots are not uploaded, telemetry is absent, recent paths remain session-only, and reports are written only to the selected local directory.
- [ ] SQLite and report at-rest encryption limitations and OS protection expectations match the shipped behavior and customer documentation.
- [ ] The UI does not expose or claim live AWS collection, update discovery, automatic download, or update installation.
- [ ] Support staff have a secure process for handling reports without requesting snapshots, reports, credentials, or license keys through an unsafe channel.

## Tag Approval

- [ ] All customer-facing placeholders are replaced and links resolve.
- [ ] Release notes state supported platforms, known limitations, security-relevant changes, and upgrade instructions without unsupported claims.
- [ ] Final artifacts match the reviewed checksums and the exact candidate approved by product, engineering, security, privacy, and release owners.
- [ ] The release tag is created only after every applicable checkbox above is verified.
