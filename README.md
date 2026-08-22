# Nocturne CloudCheck

Nocturne CloudCheck 0.1 is a paid, local-first desktop application for reviewing normalized AWS configuration snapshots. It inventories supported resources, applies deterministic security checks, stores scan history locally, compares scans, and exports reports without uploading snapshot or scan data.

## What 0.1 Supports

- Normalized AWS JSON snapshots containing S3 buckets, IAM policies, security groups, CloudTrail trails, and account settings.
- Checks for public S3 access, disabled S3 default encryption, wildcard IAM action and resource, public SSH/RDP ingress, missing root MFA, and non-multi-region CloudTrail.
- Inventory summaries by resource type and region.
- Findings with severity, evidence, and operator remediation guidance.
- Local scan history and added, resolved, and unchanged finding diffs.
- PDF, HTML, JSON, SARIF 2.1.0, and CSV exports.
- Licensed use with a seven-day offline grace period after a successful online activation or validation.

Nocturne CloudCheck reports only explicit facts in the supplied snapshot. It does not treat omitted properties as proof of a problem.

## Not Included In 0.1

- Live AWS account collection or AWS credential handling.
- Changes to AWS resources or automated remediation.
- Update discovery, automatic downloads, or automatic installation.
- Persisted recent-file shortcuts beyond the current session.
- A native file or directory picker; paths are entered as absolute paths.

## Input

Version 0.1 accepts the normalized AWS JSON format demonstrated in [`examples/aws-snapshot.json`](examples/aws-snapshot.json). Use an absolute path to a regular, non-symlink `.json` file. Inputs are strictly validated and limited to 20 MiB and 10,000 uniquely identified resources.

Start with the example:

```text
/absolute/path/to/nocturne-cloudcheck/examples/aws-snapshot.json
```

The normalized file is an interchange format, not an AWS API response. Preserve the documented snapshot and resource fields and remove unknown fields before import.

## Install

Installation is currently verified only for Linux `.deb` and `.rpm` packages.

Debian or Ubuntu:

```bash
sudo apt install ./Nocturne\ CloudCheck_0.1.0_amd64.deb
```

Fedora or RHEL-compatible systems:

```bash
sudo rpm -i ./Nocturne\ CloudCheck-0.1.0-1.x86_64.rpm
```

Verify the package against the checksum supplied with the release before installing it. Windows and macOS packaging, signing, installation, and runtime behavior have not yet been verified; they are not supported release targets for 0.1 at this time.

## Activate And Use

1. Open **License**, enter the license key delivered after purchase, and select **Activate**. Activation requires access to Lemon Squeezy over HTTPS and a working OS credential store.
2. On **Overview**, enter the absolute path to a normalized snapshot and inspect it.
3. Run the scan. An active license or valid offline grace state is required.
4. Review inventory, findings, evidence, and remediation guidance.
5. Open **Scan history**, select a baseline and target scan, and run a diff to see added, resolved, and unchanged findings.
6. Open **Reports**, choose a stored scan and format, then provide an existing absolute destination directory. Exports never overwrite an existing report.

Inspection and existing history remain available without an active license; running scans and exporting reports do not.

## Offline Grace And Deactivation

After successful online activation or validation, temporary network or licensing-service unavailability permits up to seven days of offline licensed use. An explicitly invalid license does not receive grace, and licensed operations fail closed after grace expires.

Deactivation requires network access. It releases the remote activation first and then removes local license material. Deactivate before moving a single-installation license to another machine.

## Local Data And Privacy

Snapshots are read locally and are not uploaded. Scan history and non-secret license metadata are stored in `cloudcheck.db` under the application's OS data directory. The license key is stored in the OS keyring, not SQLite. Reports are written only to the directory you choose. Recent snapshot paths exist only for the current app session.

The SQLite database and exported reports are not encrypted at rest; they rely on your OS account and filesystem protections and may contain sensitive cloud metadata. See [PRIVACY.md](PRIVACY.md) and [SECURITY.md](SECURITY.md).

## Troubleshooting

- **Snapshot rejected:** Confirm the path is absolute, the file ends in `.json`, is not a symlink, follows `examples/aws-snapshot.json`, contains no unknown fields or duplicate resource IDs, and is within the input limits.
- **Activation fails:** Confirm internet access, the license key, and availability of Keychain, Windows Credential Manager, or Secret Service. There is no plaintext credential fallback.
- **Scan or export is blocked:** Open **License** and check whether the state is active, grace, invalid, or unlicensed.
- **Export fails:** Use an existing absolute directory and a new filename containing only ASCII letters, numbers, `.`, `_`, or `-`, with the extension matching the selected format.
- **PDF text is incomplete:** The basic PDF renderer has limited Unicode support. Use HTML or JSON when full Unicode fidelity is required.

## Version And Support

- Product version: `0.1.0`
- Ruleset version: shown in each scan and exported report
- Customer support: `sales@nocturnesystems.com`
- License and purchase support: `sales@nocturnesystems.com`

When requesting support, include the product version, OS and package type, the displayed error code, and non-sensitive reproduction steps. Do not send snapshots, reports, license keys, credentials, or other secrets unless a secure support channel is explicitly arranged.
