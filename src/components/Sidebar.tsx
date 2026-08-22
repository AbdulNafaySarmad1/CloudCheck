import { Icon } from "./Icon";
import type { LicenseStatus } from "../types";

export type Screen = "overview" | "inventory" | "findings" | "remediation" | "history" | "reports" | "license" | "settings";

const primary: { id: Screen; label: string }[] = [
  { id: "overview", label: "Overview" },
  { id: "inventory", label: "Inventory" },
  { id: "findings", label: "Findings" },
  { id: "remediation", label: "Remediation" },
  { id: "history", label: "Scan history" },
  { id: "reports", label: "Reports" },
];

export function Sidebar({ active, onNavigate, license }: { active: Screen; onNavigate: (screen: Screen) => void; license: LicenseStatus | null }) {
  const navButton = ({ id, label }: { id: Screen; label: string }) => (
    <button key={id} className={active === id ? "nav-item active" : "nav-item"} onClick={() => onNavigate(id)} aria-current={active === id ? "page" : undefined}>
      <Icon name={id} /><span>{label}</span>
    </button>
  );
  const state = license?.state ?? "unlicensed";
  return <aside className="sidebar">
    <div className="brand"><div className="brand-mark"><span /></div><div><strong>NOCTURNE</strong><small>CLOUDCHECK</small></div></div>
    <nav aria-label="Main navigation">{primary.map(navButton)}</nav>
    <div className="sidebar-spacer" />
    <div className="sidebar-meta">
      <button className={active === "license" ? "nav-item active" : "nav-item"} onClick={() => onNavigate("license")} aria-current={active === "license" ? "page" : undefined}>
        <Icon name="license" /><span>License</span><i className={`status-dot ${state}`} aria-label={`License ${state}`} />
      </button>
      <button className={active === "settings" ? "nav-item active" : "nav-item"} onClick={() => onNavigate("settings")} aria-current={active === "settings" ? "page" : undefined}>
        <Icon name="settings" /><span>Settings</span>
      </button>
    </div>
  </aside>;
}
