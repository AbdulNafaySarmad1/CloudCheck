import { useDeferredValue, useState } from "react";
import { EmptyState, PageHeader } from "../../components/ui";
import { countSeverity, severityRank } from "../../lib/format";
import type { Finding, Scan, Severity } from "../../types";

const severities: Severity[] = ["critical", "high", "medium", "low", "info"];

export function FindingsView({ scan, remediationOnly = false, onStart }: { scan: Scan | null; remediationOnly?: boolean; onStart: () => void }) {
  const [filter, setFilter] = useState<Severity | "all">("all");
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query.toLowerCase());
  const [selected, setSelected] = useState<Finding | null>(null);
  if (!scan) return <><PageHeader eyebrow="Deterministic rules" title={remediationOnly ? "Remediation" : "Findings"} description="Prioritized, evidence-backed checks from the local rule engine." /><EmptyState icon={remediationOnly ? "remediation" : "findings"} title="No scan results" action={<button className="button primary" onClick={onStart}>Start a scan</button>}>Run a licensed scan to review findings and actionable remediation.</EmptyState></>;

  const findings = [...scan.findings].sort((a, b) => severityRank[b.severity] - severityRank[a.severity]).filter((finding) => (filter === "all" || finding.severity === filter) && (!deferredQuery || `${finding.title} ${finding.resourceId} ${finding.ruleId}`.toLowerCase().includes(deferredQuery)));
  const title = remediationOnly ? "Remediation queue" : "Findings";
  return <><PageHeader eyebrow={`RULESET ${scan.rulesetVersion}`} title={title} description={remediationOnly ? "Work through the highest-impact configuration changes first." : `${scan.findings.length} checks require review for AWS account ${scan.accountId}.`} />
    <div className="severity-summary">{severities.map((severity) => <button key={severity} className={filter === severity ? "selected" : ""} onClick={() => setFilter(filter === severity ? "all" : severity)}><i className={`severity ${severity}`} /><span>{severity}</span><strong>{countSeverity(scan.findings, severity)}</strong></button>)}</div>
    <div className="table-tools"><label><span className="sr-only">Search findings</span><input type="search" placeholder="Search rule, resource, or finding" value={query} onChange={(event) => setQuery(event.target.value)} /></label><span>{findings.length} OF {scan.findings.length}</span></div>
    {findings.length === 0 ? <EmptyState icon="findings" title="No matching findings">Adjust the severity filter or search query.</EmptyState> : <div className="finding-list">{findings.map((finding, index) => <button className="finding-row" key={finding.fingerprint} onClick={() => setSelected(finding)}><span className={`severity-rail ${finding.severity}`} /><span className="finding-index">{String(index + 1).padStart(2, "0")}</span><div><div className="finding-title"><span className={`severity-label ${finding.severity}`}>{finding.severity}</span><strong>{finding.title}</strong></div><code>{finding.resourceId}</code></div><span className="rule-id">{finding.ruleId}</span><span className="region">{finding.region ?? "global"}</span></button>)}</div>}
    {selected && <div className="drawer-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) setSelected(null); }}><aside className="detail-drawer" role="dialog" aria-modal="true" aria-labelledby="finding-title"><button className="icon-button close-button" aria-label="Close details" onClick={() => setSelected(null)}>x</button><span className={`severity-label ${selected.severity}`}>{selected.severity}</span><h2 id="finding-title">{selected.title}</h2><dl><dt>RULE</dt><dd><code>{selected.ruleId}</code></dd><dt>RESOURCE</dt><dd><code>{selected.resourceId}</code></dd><dt>REGION</dt><dd>{selected.region ?? "Global"}</dd></dl><section><h3>Evidence</h3><p>{selected.evidence}</p></section><section className="remediation-block"><h3>Recommended action</h3><p>{selected.remediation}</p></section><small>Fingerprint <code>{selected.fingerprint}</code></small></aside></div>}
  </>;
}
