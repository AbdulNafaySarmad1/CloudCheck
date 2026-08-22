# Nocturne CloudCheck Frontend Implementation

## Scope

The placeholder static page was replaced with a React 19, strict TypeScript, and Vite frontend embedded by Tauri 2. The interface is responsive from the desktop shell's 900 px minimum width down to narrow webview sizes. It has no application telemetry, external font request, router, state library, icon package, or browser persistence dependency.

The frontend presents only capabilities exposed by the Rust core. Version 0.1 accepts normalized AWS JSON snapshots. Live AWS collection and update discovery/installation are identified as unavailable rather than simulated in JavaScript.

## Screen Map

| Screen | Purpose | Primary states |
| --- | --- | --- |
| Overview | Onboarding, provider adapter status, absolute snapshot path inspection, scan launch, current-session recent paths | Initial, inspecting, inventory ready, scanning, unlicensed, error |
| Inventory | Resource count, type composition, region distribution, and scope summary | Empty, populated |
| Findings | Severity summary, search/filter, prioritized finding list, evidence detail drawer | Empty, no matches, populated |
| Remediation | Finding list focused on recommended actions and detailed remediation | Empty, filtered, detail |
| Scan history | Persisted scan summaries, scan selection, baseline/target diff | Loading, empty, populated, diff, error |
| Reports | PDF, HTML, JSON, SARIF, and CSV export to an existing absolute directory | Empty, exporting, success, error |
| License | Activation, active/grace/invalid/unlicensed status, and deactivation | Loading, active, grace, invalid, unlicensed, error |
| Settings & About | Privacy behavior, update integration status, path retention, and version | Informational |

## Component And State Architecture

- `src/App.tsx` owns the small application state: active screen, inspected inventory, selected scan, history, diff, license status, export result, current-session paths, one active operation, and a redacted UI error.
- `src/features/` separates workspace, inventory, findings/remediation, history, reports, licensing, and settings screens.
- `src/components/` contains shared navigation, icons, page headings, badges, loading, empty, and error states.
- `src/lib/ipc.ts` is the only Tauri invocation boundary. Each exported function maps to one Rust command and has explicit argument and result types.
- `src/types.ts` mirrors the serializable Rust domain contract. Security and business decisions remain in Rust.
- A single operation lock in `App` prevents overlapping snapshot, scan, history, diff, report, and license actions from the UI. Rust independently serializes license operations and validates all privileged inputs.
- Recent snapshot paths are kept in React memory only. They are not written to `localStorage`, `sessionStorage`, cookies, or IndexedDB.

No state library or client-side router is needed for the current single-window desktop flow. Scan history and non-secret license metadata remain owned by the Rust SQLite repository. The license key is handed directly to the activation IPC call, cleared from the controlled input before awaiting the result, never returned to JavaScript, and persisted only by the Rust OS-keyring adapter.

## IPC Contract Consumed

| Command | Frontend use |
| --- | --- |
| `inspect_snapshot(path)` | Validate and summarize the selected AWS snapshot before scanning |
| `run_snapshot_scan(path)` | Run the licensed deterministic scan and select its persisted result |
| `list_scans(limit)` | Load up to 50 local scan summaries at startup and after a scan |
| `get_scan(scanId)` | Open an immutable historical scan in findings/remediation/reports |
| `diff_scans(baseScanId, targetScanId)` | Calculate added, resolved, and unchanged findings |
| `export_scan(scanId, directory, filename, format)` | Render and safely write PDF/HTML/JSON/SARIF/CSV through Rust |
| `activate_license(licenseKey)` | Activate through the Rust Lemon Squeezy client and OS keyring |
| `license_status()` | Obtain redacted active/grace/invalid/unlicensed status |
| `deactivate_license()` | Deactivate the current installation through Rust |

Unknown invocation failures are converted to a generic `IPC_UNAVAILABLE` UI error. Known backend errors consume only the stable `code` and redacted `message`; raw Rust errors are not displayed.

## Accessibility

- Navigation and actions use native buttons, forms, labels, radios, selects, and headings.
- The active navigation item has `aria-current="page"`; license state and the finding dialog have accessible names.
- Loading and export completion are announced through status/live regions. Errors use `role="alert"`.
- Keyboard focus uses a high-contrast two-pixel indicator and is not removed.
- Color is supplemented with text labels, counts, state names, and structure.
- The finding detail is a labelled modal dialog, can be closed with its named button, and does not rely on hover.
- Motion is restrained and disabled under `prefers-reduced-motion`.
- Desktop, compact sidebar, and horizontally scrollable narrow-screen navigation layouts retain access to every screen.

Focus trapping and Escape-to-close for the finding drawer are not yet implemented. The basic PDF renderer is not a tagged accessible PDF; HTML and JSON are preferable for accessible downstream processing.

## Security Decisions

- React text rendering is used for all snapshot, finding, evidence, resource, path, and backend error strings. There is no `dangerouslySetInnerHTML` or imported-content execution.
- No secrets or paths are persisted in browser storage. No frontend logging is present.
- There is no shell, process, SQL, arbitrary HTTP, filesystem plugin, or updater invocation from JavaScript.
- Snapshot and export paths are plain inputs to narrow Rust commands; Rust remains responsible for canonicalization, extension checks, size limits, safe destination names, and no-overwrite writes.
- Scan and export controls respect the redacted license state, but Rust performs the authoritative license gate.
- Buttons are disabled while privileged operations are pending, and handlers also reject duplicate calls.
- The production UI has no remote assets. It operates under the existing restrictive Tauri CSP.
- Report filenames are generated from sanitized account IDs, fixed date text, and a fixed format extension. Rust still validates every supplied filename.
- The UI makes no claim that local licensing is unbreakable DRM.

## Verification

Verification performed on 2026-08-22:

| Command | Result |
| --- | --- |
| `npm run typecheck` | PASS, strict TypeScript project build |
| `npm run lint` | PASS, zero ESLint warnings |
| `npm run test` | PASS, 3 files and 5 tests |
| `npm run build` | PASS, Vite production assets generated in `frontend-dist/` |
| `npm audit --audit-level=high` | PASS, zero reported vulnerabilities |
| `npm run tauri -- build --no-bundle` on host | BLOCKED, host lacks `glib-2.0`/GTK/WebKit development metadata |
| Tauri container `npm run tauri -- build --no-bundle` | PASS, optimized Linux binary at `target/release/nocturne-cloudcheck` |

Tests cover cross-platform path display, backend-safe default report filenames, malformed display dates, long IDs, safe rendering of injection-shaped finding/evidence strings, initial IPC state loading, and semantic navigation controls.

The container uses Node 18.20.4 and emitted Vite's Node-version warning because Vite 7 requires Node 20.19+ or 22.12+. The frontend still built and the Rust/Tauri release binary linked successfully in that container. The direct host frontend build ran on the host's supported Node runtime without that warning. The CI image should be upgraded before relying on it for release builds.

## Limitations

- Version 0.1 has no Rust IPC command for a native file/directory picker, live AWS collection, settings persistence, update discovery, or update installation. The UI accepts validated absolute paths and labels unavailable features honestly.
- Recent snapshot paths are session-only. Persisting them requires a dedicated non-secret Rust workspace contract and user-controlled retention behavior.
- Findings expose deterministic evidence and remediation but do not mutate cloud resources or track workflow status; remediation remains an operator action.
- The current test suite is component-level and does not automate a running Tauri webview or OS keyring/Lemon Squeezy interaction.
- Installer generation, Windows/macOS behavior, code signing, notarization, package signing, and signed update installation were not tested in this frontend pass.
- Visual rendering was exercised through the production compilation and DOM tests, not screenshot regression or assistive-technology automation.
