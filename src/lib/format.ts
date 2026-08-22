import type { Finding, Severity } from "../types";

export const severityRank: Record<Severity, number> = {
  critical: 5, high: 4, medium: 3, low: 2, info: 1,
};

export function formatDate(value: string | null): string {
  if (!value) return "Never";
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? "Unknown" : new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium", timeStyle: "short",
  }).format(date);
}

export function shortId(value: string): string {
  return value.length > 18 ? `${value.slice(0, 8)}...${value.slice(-6)}` : value;
}

export function countSeverity(findings: Finding[], severity: Severity): number {
  return findings.filter((finding) => finding.severity === severity).length;
}

export function fileNameFromPath(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

export function defaultExportName(scan: { accountId: string; scannedAt: string }, extension: string): string {
  const day = scan.scannedAt.slice(0, 10);
  const safeAccount = scan.accountId.replace(/[^a-zA-Z0-9_-]/g, "-");
  return `cloudcheck-${safeAccount}-${day}.${extension}`;
}
