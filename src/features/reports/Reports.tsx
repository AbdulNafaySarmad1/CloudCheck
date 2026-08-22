import { useState, type FormEvent } from "react";
import { EmptyState, InlineError, PageHeader, Spinner } from "../../components/ui";
import { defaultExportName } from "../../lib/format";
import type { ExportResult, ReportFormat, Scan } from "../../types";

const formats: { id: ReportFormat; label: string; detail: string }[] = [
  { id: "pdf", label: "PDF", detail: "Portable review copy" },
  { id: "html", label: "HTML", detail: "Standalone report" },
  { id: "json", label: "JSON", detail: "Complete scan record" },
  { id: "sarif", label: "SARIF", detail: "Security tool exchange" },
  { id: "csv", label: "CSV", detail: "Spreadsheet analysis" },
];

export function Reports({ scan, busy, error, result, onExport, onStart }: { scan: Scan | null; busy: boolean; error: string | null; result: ExportResult | null; onExport: (directory: string, filename: string, format: ReportFormat) => Promise<void>; onStart: () => void }) {
  const [format, setFormat] = useState<ReportFormat>("pdf");
  const [directory, setDirectory] = useState("");
  const [filename, setFilename] = useState("");
  if (!scan) return <><PageHeader eyebrow="Controlled output" title="Reports" description="Export results from the native renderer in audit-friendly formats." /><EmptyState icon="reports" title="Nothing to export" action={<button className="button primary" onClick={onStart}>Start a scan</button>}>Select or run a scan before creating a report.</EmptyState></>;
  const actualFilename = filename || defaultExportName(scan, format);
  const submit = (event: FormEvent) => { event.preventDefault(); void onExport(directory.trim(), actualFilename, format); };
  return <><PageHeader eyebrow={`SCAN / ${scan.id.slice(0, 8)}`} title="Export report" description="Rendering and file writes happen in Rust with safe-path and no-overwrite enforcement." />
    {error && <InlineError message={error} />}{result && <div className="success-banner" role="status"><strong>Report exported</strong><code>{result.path}</code><span>{result.bytesWritten.toLocaleString()} bytes</span></div>}
    <form className="report-layout" onSubmit={submit}><section className="panel"><div className="panel-title"><h2>1. Choose format</h2><span>5 AVAILABLE</span></div><div className="format-grid">{formats.map((item) => <label key={item.id} className={format === item.id ? "format-card selected" : "format-card"}><input type="radio" name="format" value={item.id} checked={format === item.id} onChange={() => { setFormat(item.id); setFilename(""); }} /><strong>{item.label}</strong><span>{item.detail}</span><i>{item.id}</i></label>)}</div></section><section className="panel export-destination"><div className="panel-title"><h2>2. Destination</h2><span>LOCAL FILE</span></div><label>Existing absolute directory<input value={directory} onChange={(event) => setDirectory(event.target.value)} placeholder="/home/you/reports" spellCheck="false" /></label><label>Filename<input value={filename} onChange={(event) => setFilename(event.target.value)} placeholder={defaultExportName(scan, format)} spellCheck="false" /></label><div className="export-preview"><span>OUTPUT</span><code>{directory ? `${directory.replace(/[\\/]$/, "")}/${actualFilename}` : actualFilename}</code></div><button className="button primary" disabled={busy || !directory.trim()}>{busy ? <Spinner label="Exporting" /> : `Export ${format.toUpperCase()}`}</button><small>Existing files are never overwritten. Exported reports can contain sensitive cloud metadata.</small></section></form>
  </>;
}
