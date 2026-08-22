import { useState } from "react";
import { Badge, EmptyState, InlineError, PageHeader, Spinner } from "../../components/ui";
import { formatDate, shortId } from "../../lib/format";
import type { ScanDiff, ScanSummary } from "../../types";

interface Props { scans: ScanSummary[]; loading: boolean; diff: ScanDiff | null; error: string | null; onOpen: (id: string) => void; onDiff: (base: string, target: string) => Promise<void> }

export function History(props: Props) {
  const [base, setBase] = useState("");
  const [target, setTarget] = useState("");
  if (props.loading) return <><PageHeader eyebrow="Local database" title="Scan history" description="Immutable results retained on this device." /><div className="loading-block"><Spinner label="Loading scan history" /></div></>;
  return <><PageHeader eyebrow="Local database" title="Scan history" description="Compare deterministic results over time without uploading account metadata." />
    {props.error && <InlineError message={props.error} />}
    {props.scans.length === 0 ? <EmptyState icon="history" title="History is empty">Completed scans will appear here and remain available between sessions.</EmptyState> : <>
      <section className="diff-control"><div><span className="eyebrow">CHANGE ANALYSIS</span><h2>Compare scans</h2></div><label>Baseline<select value={base} onChange={(event) => setBase(event.target.value)}><option value="">Select scan</option>{props.scans.map((scan) => <option value={scan.id} key={scan.id}>{formatDate(scan.scannedAt)} / {scan.accountId}</option>)}</select></label><label>Target<select value={target} onChange={(event) => setTarget(event.target.value)}><option value="">Select scan</option>{props.scans.map((scan) => <option value={scan.id} key={scan.id}>{formatDate(scan.scannedAt)} / {scan.accountId}</option>)}</select></label><button className="button secondary" disabled={!base || !target || base === target} onClick={() => void props.onDiff(base, target)}>Compare</button></section>
      {props.diff && <div className="diff-result"><div><span>ADDED</span><strong className="bad">+{props.diff.added.length}</strong></div><div><span>RESOLVED</span><strong className="good">-{props.diff.resolved.length}</strong></div><div><span>UNCHANGED</span><strong>{props.diff.unchangedCount}</strong></div></div>}
      <div className="history-table" role="table" aria-label="Scan history"><div className="history-head" role="row"><span>SCANNED</span><span>ACCOUNT</span><span>RESOURCES</span><span>FINDINGS</span><span>SCAN ID</span></div>{props.scans.map((scan) => <button role="row" key={scan.id} onClick={() => props.onOpen(scan.id)}><span>{formatDate(scan.scannedAt)}</span><code>{scan.accountId}</code><span>{scan.resourceCount.toLocaleString()}</span><span><Badge tone={scan.findingCount ? "red" : "green"}>{scan.findingCount}</Badge></span><code>{shortId(scan.id)}</code></button>)}</div>
    </>}
  </>;
}
