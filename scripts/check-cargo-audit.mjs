import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";

const policy = JSON.parse(await readFile("security/advisory-policy.json", "utf8"));
const reviewDeadline = new Date(`${policy.reviewBy}T23:59:59Z`);
if (Number.isNaN(reviewDeadline.valueOf()) || new Date() > reviewDeadline) {
  console.error(`Rust advisory review expired on ${policy.reviewBy}; security review is required.`);
  process.exit(1);
}
const result = spawnSync("cargo", ["audit", "--json"], { encoding: "utf8", maxBuffer: 20 * 1024 * 1024 });
let report;
try {
  report = JSON.parse(result.stdout);
} catch {
  process.stderr.write(result.stderr || result.stdout || "cargo audit did not return JSON.\n");
  process.exit(result.status ?? 1);
}

const vulnerabilities = report.vulnerabilities?.list ?? [];
if (vulnerabilities.length > 0) {
  console.error(`RustSec found ${vulnerabilities.length} vulnerability advisory(s).`);
  for (const item of vulnerabilities) console.error(`${item.advisory.id}: ${item.package.name} ${item.package.version}`);
  process.exit(1);
}

const warnings = Object.values(report.warnings ?? {}).flat();
const reviewed = new Set(policy.reviewedInformationalAdvisories.map((entry) => entry.id));
const unexpected = warnings.filter((warning) => !reviewed.has(warning.advisory.id));
if (unexpected.length > 0) {
  console.error("RustSec returned unreviewed informational advisories:");
  for (const warning of unexpected) console.error(`${warning.advisory.id}: ${warning.package.name} ${warning.package.version}`);
  process.exit(1);
}

console.log(`RustSec: 0 vulnerabilities; ${warnings.length} known informational warning(s) reviewed in security/advisory-policy.json.`);
