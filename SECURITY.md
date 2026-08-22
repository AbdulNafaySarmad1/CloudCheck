# Security Policy

## Supported Versions

Security fixes are provided for the current Nocturne CloudCheck `0.1.x` release. Pre-release builds and older release lines are not supported unless explicitly stated in release notes.

## Report A Vulnerability

Report suspected vulnerabilities privately to `security@nocturnesystems.com`. Do not open a public issue or publish proof of concept before coordination.

Include:

- Affected version, OS, package type, and component.
- Impact and realistic attack scenario.
- Reproduction steps or a minimal proof of concept.
- Relevant logs or screenshots after removing license keys, AWS data, credentials, personal data, and other secrets.
- Whether the issue is already public and how you would like to be credited.

We will use the contact details in your report to investigate, request clarification, coordinate a fix and release where appropriate, and discuss disclosure timing. Please allow a reasonable opportunity to remediate before disclosure and coordinate public details with us. This policy promises no response or remediation SLA and offers no bug bounty or payment.

## Architecture And Boundaries

Nocturne CloudCheck is a local Tauri desktop application. A React UI calls a narrow typed Rust IPC layer. Rust validates untrusted paths and data, runs a deterministic rule engine, stores local history in parameterized SQLite, renders reports, and contacts only fixed Lemon Squeezy HTTPS licensing endpoints in version 0.1. License keys are stored in the OS keyring and are not returned to the UI or stored in SQLite.

Snapshot reads are bounded and reject symlink imports. Exports validate their destination and do not overwrite existing files. HTML output escapes imported values, CSV output neutralizes spreadsheet formula triggers, and internal failures are reduced to stable, non-sensitive errors. Live AWS collection and update download/installation are not shipped in 0.1.

Local license enforcement is pragmatic deterrence, not unbreakable DRM. A determined attacker with control of the device can patch a client binary. Processes running as the same OS user, a compromised runtime, and modified binaries are outside the application's protection boundary.

## Protect Sensitive Data

Treat AWS snapshots, scan history, findings, and exported reports as sensitive. They can reveal account identifiers, resource configuration, regions, and security weaknesses.

- Provide only normalized configuration data needed for analysis; never add AWS credentials, session tokens, private keys, license keys, or unrelated secrets.
- Keep snapshots and reports in access-controlled directories and protect copies and backups.
- Remember that SQLite history and exported reports are not encrypted at rest by the application and rely on OS and filesystem protections.
- Prefer HTML or JSON over PDF when full Unicode fidelity matters.
- Sanitize all diagnostic material before sending it to support or a security contact.

## Research Safety

Test only systems and data you own or are authorized to assess. Avoid privacy violations, service disruption, persistence, social engineering, and access to unrelated data. If testing encounters customer or third-party data, stop, preserve minimal evidence, and report privately. Coordinated disclosure is requested, but this document does not grant legal safe harbor or make legal guarantees.
