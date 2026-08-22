export type Provider = "aws";
export type Severity = "critical" | "high" | "medium" | "low" | "info";
export type LicenseState = "unlicensed" | "active" | "grace" | "invalid";
export type ReportFormat = "pdf" | "html" | "json" | "sarif" | "csv";

export interface Finding {
  fingerprint: string;
  ruleId: string;
  title: string;
  severity: Severity;
  resourceId: string;
  region: string | null;
  evidence: string;
  remediation: string;
}

export interface Scan {
  id: string;
  provider: Provider;
  accountId: string;
  scope: string[];
  scannedAt: string;
  productVersion: string;
  rulesetVersion: string;
  resourceCount: number;
  findings: Finding[];
  limitations: string[];
}

export interface ScanSummary {
  id: string;
  provider: Provider;
  accountId: string;
  scannedAt: string;
  resourceCount: number;
  findingCount: number;
}

export interface InventorySummary {
  provider: Provider;
  accountId: string;
  scope: string[];
  resourceCount: number;
  countsByKind: Record<string, number>;
  countsByRegion: Record<string, number>;
}

export interface ScanDiff {
  baseScanId: string;
  targetScanId: string;
  added: Finding[];
  resolved: Finding[];
  unchangedCount: number;
}

export interface LicenseStatus {
  state: LicenseState;
  lastVerifiedAt: string | null;
  graceExpiresAt: string | null;
}

export interface ExportResult { path: string; bytesWritten: number }
export interface CommandError { code: string; message: string }
