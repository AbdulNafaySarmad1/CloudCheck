import { useState, type FormEvent } from "react";
import { Icon } from "../../components/Icon";
import { Badge, InlineError, PageHeader, Spinner } from "../../components/ui";
import { fileNameFromPath } from "../../lib/format";
import type { InventorySummary, LicenseStatus, Scan, ScanSummary } from "../../types";

interface Props {
  path: string;
  inventory: InventorySummary | null;
  latestScan: Scan | null;
  history: ScanSummary[];
  recents: string[];
  license: LicenseStatus | null;
  busy: "inspect" | "scan" | null;
  error: string | null;
  onInspect: (path: string) => Promise<void>;
  onScan: () => Promise<void>;
  onOpenHistory: (scanId: string) => void;
}

export function Overview(props: Props) {
  const [draftPath, setDraftPath] = useState(props.path);
  const submit = (event: FormEvent) => { event.preventDefault(); void props.onInspect(draftPath.trim()); };
  const licensed = props.license?.state === "active" || props.license?.state === "grace";

  return <>
    <PageHeader eyebrow="Workspace / AWS" title="Cloud posture, without the noise." description="Inspect normalized configuration locally, then run deterministic checks against a versioned ruleset." action={<Badge tone="blue">AWS FIRST</Badge>} />
    {props.error && <InlineError message={props.error} />}
    <section className="workspace-hero">
      <div className="workspace-copy">
        <span className="step-label">01 / CONNECT SOURCE</span>
        <h2>Start with an AWS snapshot</h2>
        <p>No credentials enter the webview. Select a normalized JSON snapshot collected with least-privilege, read-only access.</p>
        <form onSubmit={submit} className="path-form">
          <label htmlFor="snapshot-path">Absolute snapshot path</label>
          <div className="input-action"><input id="snapshot-path" value={draftPath} onChange={(event) => setDraftPath(event.target.value)} placeholder="/home/you/cloudcheck/aws-snapshot.json" autoComplete="off" spellCheck="false" /><button className="button secondary" disabled={!draftPath.trim() || props.busy !== null}>{props.busy === "inspect" ? <Spinner label="Inspecting" /> : "Inspect"}</button></div>
          <small>JSON only. The Rust core validates and reads this path; content is never executed or uploaded.</small>
        </form>
        <div className="adapter-row">
          <div className="adapter active"><div className="provider-mark">AWS</div><div><strong>Snapshot adapter</strong><span>Available now</span></div><Icon name="check" /></div>
          <div className="adapter disabled" aria-disabled="true"><div className="provider-mark muted">API</div><div><strong>Live read-only API</strong><span>Not shipped in v0.1</span></div></div>
        </div>
      </div>
      <div className="signal-panel" aria-label="Scan pipeline">
        <div className="signal-grid" />
        <div className="orb"><div /><span /></div>
        <div className="signal-caption"><span>LOCAL ANALYSIS ENGINE</span><strong>Deterministic by design</strong><p>Same normalized input. Same ruleset. Same result.</p></div>
      </div>
    </section>

    {props.inventory && <section className="ready-strip">
      <div><span className="pulse-dot" /><div><strong>{fileNameFromPath(props.path)}</strong><small>AWS account <code>{props.inventory.accountId}</code> / {props.inventory.resourceCount.toLocaleString()} resources</small></div></div>
      <button className="button primary" onClick={() => void props.onScan()} disabled={props.busy !== null || !licensed}>{props.busy === "scan" ? <Spinner label="Scanning" /> : <>Run security scan <Icon name="arrow" /></>}</button>
      {!licensed && <small className="license-hint">Activate a license to scan</small>}
    </section>}

    <div className="section-heading"><div><span className="eyebrow">WORKSPACES</span><h2>Recent activity</h2></div><span className="quiet">Session-only paths</span></div>
    <div className="activity-grid">
      {props.history.slice(0, 3).map((scan) => <button className="activity-card" key={scan.id} onClick={() => props.onOpenHistory(scan.id)}><span className="provider-tag">AWS</span><strong>{scan.accountId}</strong><small>{scan.resourceCount} resources / {scan.findingCount} findings</small><Icon name="arrow" /></button>)}
      {props.history.length === 0 && <div className="activity-empty"><Icon name="history" /><span>No completed scans yet</span></div>}
      {props.recents.filter((item) => item !== props.path).slice(0, 2).map((item) => <button className="activity-card" key={item} onClick={() => { setDraftPath(item); void props.onInspect(item); }}><span className="provider-tag">JSON</span><strong>{fileNameFromPath(item)}</strong><small>Reinspect snapshot</small><Icon name="arrow" /></button>)}
    </div>
  </>;
}
