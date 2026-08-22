import { PageHeader } from "../../components/ui";

export function Settings({ version }: { version: string }) {
  return <><PageHeader eyebrow="Application" title="Settings & About" description="Local behavior, release status, and product information." />
    <div className="settings-list"><section><div><span className="eyebrow">PRIVACY</span><h2>Local-first operation</h2><p>Snapshots, findings, and scan history stay on this device. No product telemetry or cloud upload is configured.</p></div><span className="setting-state enabled">ENABLED</span></section><section><div><span className="eyebrow">UPDATES</span><h2>Signed updates</h2><p>The native core contains update verification primitives, but update discovery and installation are not configured in this release.</p></div><span className="setting-state">NOT CONFIGURED</span></section><section><div><span className="eyebrow">WORKSPACE HISTORY</span><h2>Recent snapshot paths</h2><p>Paths used in this session are kept only in memory and are cleared when the application closes.</p></div><span className="setting-state enabled">SESSION ONLY</span></section></div>
    <footer className="about"><div className="brand-mark"><span /></div><div><strong>Nocturne CloudCheck</strong><span>Version {version}</span><p>Deterministic cloud configuration security scanning.</p></div><div><span>RULESET</span><code>Shown per completed scan</code></div></footer>
  </>;
}
