import { useState, type FormEvent } from "react";
import { Badge, InlineError, PageHeader, Spinner } from "../../components/ui";
import { formatDate } from "../../lib/format";
import type { LicenseStatus } from "../../types";

export function License({ status, busy, error, onActivate, onDeactivate }: { status: LicenseStatus | null; busy: boolean; error: string | null; onActivate: (key: string) => Promise<void>; onDeactivate: () => Promise<void> }) {
  const [key, setKey] = useState("");
  const state = status?.state ?? "unlicensed";
  const active = state === "active" || state === "grace";
  const submit = (event: FormEvent) => { event.preventDefault(); const value = key; setKey(""); void onActivate(value); };
  return <><PageHeader eyebrow="Device authorization" title="License" description="Activate this installation with your Lemon Squeezy license key." action={<Badge tone={active ? "green" : state === "invalid" ? "red" : "neutral"}>{state.toUpperCase()}</Badge>} />
    {error && <InlineError message={error} />}
    <div className="license-layout"><section className="license-status-card"><div className={`license-orbit ${active ? "active" : ""}`}><span /></div><span className="eyebrow">CURRENT STATE</span><h2>{state === "active" ? "This device is licensed" : state === "grace" ? "Offline grace period" : state === "invalid" ? "License needs attention" : "Activation required"}</h2><p>{active ? "Scanning and report exports are enabled on this device." : "Inventory inspection and history remain available. Activate to run scans and exports."}</p><dl><dt>Last verified</dt><dd>{formatDate(status?.lastVerifiedAt ?? null)}</dd>{status?.graceExpiresAt && <><dt>Grace expires</dt><dd>{formatDate(status.graceExpiresAt)}</dd></>}</dl></section>
      <section className="panel license-action">{active ? <><span className="eyebrow">MANAGE INSTALLATION</span><h2>Deactivate this device</h2><p>The license key is held by the operating-system credential store, never browser storage. Deactivation requires network access.</p><button className="button danger" disabled={busy} onClick={() => void onDeactivate()}>{busy ? <Spinner label="Deactivating" /> : "Deactivate license"}</button></> : <form onSubmit={submit}><span className="eyebrow">ACTIVATE</span><h2>Enter your license key</h2><p>The key is sent directly by the Rust core to Lemon Squeezy and stored in the OS credential store.</p><label htmlFor="license-key">License key<input id="license-key" type="password" value={key} onChange={(event) => setKey(event.target.value)} minLength={8} maxLength={256} autoComplete="off" required /></label><button className="button primary" disabled={busy || key.length < 8}>{busy ? <Spinner label="Activating" /> : "Activate license"}</button></form>}</section></div>
  </>;
}
