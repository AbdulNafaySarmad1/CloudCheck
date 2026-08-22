# Rust Advisory Review

Review date: 2026-08-22
Mandatory re-review: 2026-11-30 or before the next release, Tauri upgrade, or Linux webview dependency change, whichever occurs first.

`cargo audit` 0.22.2 reports zero vulnerability advisories for the locked 497-crate graph. It reports 17 informational warnings. `scripts/check-cargo-audit.mjs` fails CI on any vulnerability or any informational advisory not explicitly listed in `security/advisory-policy.json`. Removing a warning through dependency updates does not fail CI.

## RUSTSEC-2024-0429: glib 0.18.5

- **Classification/severity:** RustSec `INFO unsound`; no CVSS score. The affected `VariantStrIter` iterator methods can cause undefined behavior and null-pointer crashes in optimized builds.
- **Reachability:** `glib` is a transitive Linux desktop dependency through Tauri/WebKit GTK bindings. CloudCheck does not depend on `glib` directly and repository source does not call `glib::VariantStrIter`, but transitive framework reachability cannot be ruled out, so this is not treated as unreachable.
- **Current disposition:** Temporary acceptance for the v0.1 release-candidate audit, not a silent suppression. The patched `glib >=0.20` line is not selectable independently while the current supported Tauri/WebKit GTK stack resolves the 0.18 family. Forcing a major GTK binding replacement would be an unreviewed desktop architecture change.
- **Required action:** Re-test the newest compatible Tauri/WebKit dependency line before GA and every release. Remove the acceptance as soon as the native stack resolves `glib >=0.20`. If the advisory becomes a vulnerability, gains a material severity, or affected calls are found reachable, the release gate fails and v0.1 must not ship until remediated or independently re-approved.
- **Expiry:** 2026-11-30. The automated gate fails after that date until the policy and this review are renewed by release/security owners.

## Unmaintained Transitive Dependencies

RustSec reports ten GTK3-family warnings (`RUSTSEC-2024-0411` through `RUSTSEC-2024-0420`, excluding `0429`), `proc-macro-error` (`RUSTSEC-2024-0370`), and five `rust-unic` warnings (`RUSTSEC-2025-0075`, `0080`, `0081`, `0098`, `0100`). These entries have no CVSS score and report maintenance status rather than a known exploitable defect.

- **Reachability:** GTK3 packages are runtime transitive dependencies of Tauri's Linux webview stack. `proc-macro-error` is used by transitive macro/build tooling. The `rust-unic` packages are transitive parser dependencies. None is a direct CloudCheck dependency.
- **Current disposition:** Time-bounded acceptance because replacing these crates independently is not available without changing the Tauri/WebKit dependency stack. This does not exempt any vulnerability affecting them; a new vulnerability ID fails immediately.
- **Required action and expiry:** Review available Tauri/WebKit upgrades before every release and no later than 2026-11-30. Remove IDs as the dependency graph stops reporting them. Escalate any newly reported vulnerability, exploit, or maintenance-related platform incompatibility as a release blocker.
