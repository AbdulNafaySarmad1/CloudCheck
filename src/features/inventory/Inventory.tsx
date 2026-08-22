import { EmptyState, PageHeader } from "../../components/ui";
import type { InventorySummary } from "../../types";

export function Inventory({ inventory, onStart }: { inventory: InventorySummary | null; onStart: () => void }) {
  if (!inventory) return <><PageHeader eyebrow="Source analysis" title="Inventory" description="A bounded view of resources present in the selected snapshot." /><EmptyState icon="inventory" title="No inventory loaded" action={<button className="button primary" onClick={onStart}>Choose a snapshot</button>}>Inspect an AWS snapshot to build its local resource index.</EmptyState></>;
  const kinds = Object.entries(inventory.countsByKind).sort((a, b) => b[1] - a[1]);
  const regions = Object.entries(inventory.countsByRegion).sort((a, b) => b[1] - a[1]);
  return <><PageHeader eyebrow={`AWS / ${inventory.accountId}`} title="Inventory" description={`${inventory.resourceCount.toLocaleString()} normalized resources across ${regions.length} region${regions.length === 1 ? "" : "s"}.`} />
    <div className="metric-row"><div className="metric"><span>TOTAL RESOURCES</span><strong>{inventory.resourceCount.toLocaleString()}</strong></div><div className="metric"><span>RESOURCE TYPES</span><strong>{kinds.length}</strong></div><div className="metric"><span>SCOPES</span><strong>{inventory.scope.length || 1}</strong></div></div>
    <div className="split-panels"><section className="panel"><div className="panel-title"><h2>Resource composition</h2><span>COUNT</span></div>{kinds.map(([kind, count]) => <div className="bar-row" key={kind}><code>{kind}</code><div><span style={{ width: `${Math.max(3, count / inventory.resourceCount * 100)}%` }} /></div><strong>{count}</strong></div>)}</section><section className="panel"><div className="panel-title"><h2>Regional distribution</h2><span>RESOURCES</span></div>{regions.map(([region, count]) => <div className="region-row" key={region}><span><i /> <code>{region}</code></span><strong>{count}</strong></div>)}</section></div>
  </>;
}
