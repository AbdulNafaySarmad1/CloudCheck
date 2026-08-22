import { invoke } from "@tauri-apps/api/core";
import type {
  CommandError,
  ExportResult,
  InventorySummary,
  LicenseStatus,
  ReportFormat,
  Scan,
  ScanDiff,
  ScanSummary,
} from "../types";

async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error: unknown) {
    if (isCommandError(error)) throw error;
    throw { code: "IPC_UNAVAILABLE", message: "The secure desktop service is unavailable." } satisfies CommandError;
  }
}

function isCommandError(error: unknown): error is CommandError {
  if (typeof error !== "object" || error === null) return false;
  const value = error as Record<string, unknown>;
  return typeof value.code === "string" && typeof value.message === "string";
}

export const ipc = {
  inspectSnapshot: (path: string) => command<InventorySummary>("inspect_snapshot", { path }),
  runSnapshotScan: (path: string) => command<Scan>("run_snapshot_scan", { path }),
  listScans: (limit = 50) => command<ScanSummary[]>("list_scans", { limit }),
  getScan: (scanId: string) => command<Scan>("get_scan", { scanId }),
  diffScans: (baseScanId: string, targetScanId: string) =>
    command<ScanDiff>("diff_scans", { baseScanId, targetScanId }),
  exportScan: (scanId: string, directory: string, filename: string, format: ReportFormat) =>
    command<ExportResult>("export_scan", { scanId, directory, filename, format }),
  activateLicense: (licenseKey: string) => command<LicenseStatus>("activate_license", { licenseKey }),
  licenseStatus: () => command<LicenseStatus>("license_status"),
  deactivateLicense: () => command<void>("deactivate_license"),
};

export function errorMessage(error: unknown): string {
  return isCommandError(error) ? error.message : "The operation could not be completed.";
}
