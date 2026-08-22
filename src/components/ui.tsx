import type { ReactNode } from "react";
import { Icon } from "./Icon";

export function PageHeader({ eyebrow, title, description, action }: { eyebrow: string; title: string; description: string; action?: ReactNode }) {
  return <header className="page-header"><div><span className="eyebrow">{eyebrow}</span><h1>{title}</h1><p>{description}</p></div>{action}</header>;
}

export function EmptyState({ icon, title, children, action }: { icon: string; title: string; children: ReactNode; action?: ReactNode }) {
  return <div className="empty-state"><div className="empty-icon"><Icon name={icon} size={22} /></div><h2>{title}</h2><p>{children}</p>{action}</div>;
}

export function InlineError({ message, retry }: { message: string; retry?: () => void }) {
  return <div className="inline-error" role="alert"><span><strong>Operation failed.</strong> {message}</span>{retry && <button className="text-button" onClick={retry}>Try again</button>}</div>;
}

export function Spinner({ label = "Loading" }: { label?: string }) {
  return <span className="spinner-wrap" role="status"><span className="spinner" /> <span>{label}</span></span>;
}

export function Badge({ children, tone = "neutral" }: { children: ReactNode; tone?: string }) {
  return <span className={`badge badge-${tone}`}>{children}</span>;
}
