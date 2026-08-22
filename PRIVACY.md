# Privacy Notice

This notice describes Nocturne CloudCheck 0.1's implemented data handling. Replace the contact placeholders before release. It is an operational summary, not legal advice, a warranty, or a guarantee of compliance with any law.

## Local-First Processing

Nocturne CloudCheck reads normalized AWS JSON snapshots and performs inventory, scanning, diffing, and report rendering on your device. It does not upload snapshots, findings, reports, or scan history. Version 0.1 has no analytics or product telemetry and loads no remote fonts or application assets.

The application does contact Lemon Squeezy over HTTPS to activate, validate, and deactivate a license. Those requests include the license information needed for Lemon Squeezy's client licensing service. Questions about purchase or license-service data should be directed to `sales@nocturnesystems.com` and, where applicable, Lemon Squeezy's own privacy materials.

## Data Stored On Your Device

- Imported snapshots remain at their original local paths; recent snapshot paths are held in memory only for the current session.
- Scan results and history are stored as SQLite data in `cloudcheck.db` under the application's OS data directory.
- SQLite stores non-secret license metadata such as activation ID, status, and last verification time. It does not store the license key.
- The license key is stored in the OS credential store: Keychain on macOS, Windows Credential Manager on Windows, or Secret Service on Linux. There is no plaintext fallback.
- Exported PDF, HTML, JSON, SARIF, and CSV reports are stored in the local directory you select.

Nocturne CloudCheck does not provide application-level encryption at rest for SQLite data or exported reports. These files inherit your OS account, credential-store, access-control, backup, and filesystem protections. Snapshots, history, and reports can contain sensitive cloud metadata; restrict access, secure backups, and delete them according to your organization's retention policy.

## Network And Future Features

Apart from Lemon Squeezy licensing, version 0.1 does not send application data to a product service. Live AWS collection is not included in 0.1, and the application does not access AWS credentials or AWS APIs. If a future release adds AWS connectivity, its credential and data handling will require an updated notice before that feature is offered.

## Your Choices

You choose which snapshot to inspect, which scans to retain through your management of the local application data, and where reports are exported. Deactivation requires a network connection and removes local license material only after remote deactivation succeeds. Uninstalling the application may not remove its data directory or reports; review and delete those locations separately when needed.

## Contact

- Privacy questions: `sales@nocturnesystems.com`
- Licensing questions: `sales@nocturnesystems.com`
- Effective date: August 22, 2026

No security measure or local-first design can guarantee absolute confidentiality. Processes running as your OS user, compromised devices or runtimes, modified binaries, and access to unencrypted local files remain outside the application's protection boundary.
