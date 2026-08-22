import { useEffect, useState } from "react";
import { Sidebar, type Screen } from "./components/Sidebar";
import { Overview } from "./features/workspace/Overview";
import { Inventory } from "./features/inventory/Inventory";
import { FindingsView } from "./features/findings/Findings";
import { History } from "./features/history/History";
import { Reports } from "./features/reports/Reports";
import { License } from "./features/license/License";
import { Settings } from "./features/settings/Settings";
import { errorMessage, ipc } from "./lib/ipc";
import type { ExportResult, InventorySummary, LicenseStatus, ReportFormat, Scan, ScanDiff, ScanSummary } from "./types";

type Operation = "inspect" | "scan" | "history" | "diff" | "export" | "license" | null;

export default function App() {
  const [screen, setScreen] = useState<Screen>("overview");
  const [path, setPath] = useState("");
  const [recents, setRecents] = useState<string[]>([]);
  const [inventory, setInventory] = useState<InventorySummary | null>(null);
  const [scan, setScan] = useState<Scan | null>(null);
  const [scans, setScans] = useState<ScanSummary[]>([]);
  const [diff, setDiff] = useState<ScanDiff | null>(null);
  const [license, setLicense] = useState<LicenseStatus | null>(null);
  const [operation, setOperation] = useState<Operation>("history");
  const [error, setError] = useState<string | null>(null);
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);

  useEffect(() => {
    let current = true;
    Promise.allSettled([ipc.listScans(), ipc.licenseStatus()]).then(([historyResult, licenseResult]) => {
      if (!current) return;
      if (historyResult.status === "fulfilled") setScans(historyResult.value);
      if (licenseResult.status === "fulfilled") setLicense(licenseResult.value);
      if (historyResult.status === "rejected" && licenseResult.status === "rejected") setError(errorMessage(historyResult.reason));
      setOperation(null);
    });
    return () => { current = false; };
  }, []);

  async function inspect(selectedPath: string) {
    if (operation || !selectedPath) return;
    setOperation("inspect"); setError(null); setInventory(null); setScan(null);
    try {
      const result = await ipc.inspectSnapshot(selectedPath);
      setPath(selectedPath); setInventory(result);
      setRecents((existing) => [selectedPath, ...existing.filter((item) => item !== selectedPath)].slice(0, 5));
    } catch (caught) { setError(errorMessage(caught)); }
    finally { setOperation(null); }
  }

  async function runScan() {
    if (operation || !path) return;
    setOperation("scan"); setError(null);
    try {
      const result = await ipc.runSnapshotScan(path);
      setScan(result);
      setScans(await ipc.listScans());
      setScreen("findings");
    } catch (caught) { setError(errorMessage(caught)); }
    finally { setOperation(null); }
  }

  async function openScan(scanId: string) {
    if (operation) return;
    setOperation("history"); setError(null);
    try { setScan(await ipc.getScan(scanId)); setScreen("findings"); }
    catch (caught) { setError(errorMessage(caught)); }
    finally { setOperation(null); }
  }

  async function compareScans(base: string, target: string) {
    if (operation) return;
    setOperation("diff"); setError(null);
    try { setDiff(await ipc.diffScans(base, target)); }
    catch (caught) { setError(errorMessage(caught)); }
    finally { setOperation(null); }
  }

  async function exportScan(directory: string, filename: string, format: ReportFormat) {
    if (operation || !scan) return;
    setOperation("export"); setError(null); setExportResult(null);
    try { setExportResult(await ipc.exportScan(scan.id, directory, filename, format)); }
    catch (caught) { setError(errorMessage(caught)); }
    finally { setOperation(null); }
  }

  async function activateLicense(key: string) {
    if (operation) return;
    setOperation("license"); setError(null);
    try { setLicense(await ipc.activateLicense(key)); }
    catch (caught) { setError(errorMessage(caught)); }
    finally { setOperation(null); }
  }

  async function deactivateLicense() {
    if (operation) return;
    setOperation("license"); setError(null);
    try { await ipc.deactivateLicense(); setLicense(await ipc.licenseStatus()); }
    catch (caught) { setError(errorMessage(caught)); }
    finally { setOperation(null); }
  }

  function navigate(next: Screen) { setError(null); setScreen(next); }
  const start = () => navigate("overview");
  const content = {
    overview: <Overview path={path} inventory={inventory} latestScan={scan} history={scans} recents={recents} license={license} busy={operation === "inspect" || operation === "scan" ? operation : null} error={error} onInspect={inspect} onScan={runScan} onOpenHistory={(id) => void openScan(id)} />,
    inventory: <Inventory inventory={inventory} onStart={start} />,
    findings: <FindingsView scan={scan} onStart={start} />,
    remediation: <FindingsView scan={scan} remediationOnly onStart={start} />,
    history: <History scans={scans} loading={operation === "history"} diff={diff} error={error} onOpen={(id) => void openScan(id)} onDiff={compareScans} />,
    reports: <Reports scan={scan} busy={operation === "export"} error={error} result={exportResult} onExport={exportScan} onStart={start} />,
    license: <License status={license} busy={operation === "license"} error={error} onActivate={activateLicense} onDeactivate={deactivateLicense} />,
    settings: <Settings version={scan?.productVersion ?? __APP_VERSION__} />,
  }[screen];

  return <div className="app-shell">
    <Sidebar active={screen} onNavigate={navigate} license={license} />
    <div className="main-column"><div className="topbar"><span>LOCAL WORKSPACE</span><div><i className="local-dot" /> No cloud upload</div></div><main id="main-content" tabIndex={-1}>{content}</main></div>
    <div className="sr-only" aria-live="polite">{operation ? `${operation} operation in progress` : exportResult ? "Export complete" : ""}</div>
  </div>;
}
