import { useMemo, useState } from "react";
import {
  defaultPackConstraints,
  runAlgorithm,
  type PackConstraints,
  type PackStrategy,
  type UiResult,
} from "@nonplanarslicer/api";
import { AlgoPicker } from "./components/AlgoPicker";
import { ConstraintsPanel } from "./components/ConstraintsPanel";
import { ExportPanel } from "./components/ExportPanel";
import { LayerViewer } from "./components/LayerViewer";
import { PipelineBar } from "./components/PipelineBar";
import { buildDemoPackInput, demoCandidates, DEMO_NOTE } from "./data/demoLayers";

export default function App() {
  const [strategy, setStrategy] = useState<PackStrategy>("milp");
  const [constraints, setConstraints] = useState<PackConstraints>(defaultPackConstraints);
  const [result, setResult] = useState<UiResult | null>(null);

  const packInput = useMemo(
    () => buildDemoPackInput(strategy, constraints),
    [strategy, constraints],
  );

  const run = () => {
    setResult(runAlgorithm("pack", packInput as unknown as Record<string, unknown>));
  };

  const pack = result?.pack;

  return (
    <main>
      <header>
        <p className="eyebrow">NONPLANARSLICER / STRESS-FIBER</p>
        <h1>NP Pack</h1>
        <p className="lede">
          Layer-wise continuous-fiber loop packing (#27) and Deep-Q next-node
          sequencing (#22) on top of stress-aligned layers and fiber fields.
          Algorithms from NP Rust Core / NP Fiber Fields arrive via napi-rs —
          this UI owns packing, constraints, and export stubs.
        </p>
      </header>

      <PipelineBar />

      <div className="workbench">
        <aside className="sidebar">
          <AlgoPicker value={strategy} onChange={setStrategy} />
          <ConstraintsPanel value={constraints} onChange={setConstraints} />
          <div className="run-row">
            <button type="button" onClick={run}>
              Run pack
            </button>
            <span className="status">
              {result
                ? result.native
                  ? "napi-rs native"
                  : "TypeScript mock"
                : "idle"}
            </span>
          </div>
          <ExportPanel pack={pack} candidates={demoCandidates} />
        </aside>

        <section className="panel viewer-panel">
          <LayerViewer candidates={demoCandidates} pack={pack} />
          <div className="paths">
            <h2>Result</h2>
            {!pack ? (
              <p className="muted">
                Load the synthetic demo, pick #27 / #22 / greedy, set constraints,
                then Run pack. {DEMO_NOTE}
              </p>
            ) : (
              <>
                <dl className="kv">
                  <div><dt>Strategy</dt><dd>{pack.strategy}</dd></div>
                  <div><dt>Status</dt><dd>{pack.solver_status}</dd></div>
                  <div><dt>Selected</dt><dd>{pack.selected_loop_ids.join(", ") || "—"}</dd></div>
                  <div><dt>Tour</dt><dd>{pack.tour_order.join(" → ") || "—"}</dd></div>
                  <div>
                    <dt>Cuts</dt>
                    <dd>
                      {pack.cuts.length
                        ? pack.cuts
                            .map((c) => `#${c.after_tour_index} ${c.kind}`)
                            .join("; ")
                        : "none"}
                    </dd>
                  </div>
                  <div><dt>Unplaced</dt><dd>{pack.unplaced.join(", ") || "—"}</dd></div>
                </dl>
                {pack.notes?.length ? (
                  <ul className="notes">
                    {pack.notes.map((n) => (
                      <li key={n}>{n}</li>
                    ))}
                  </ul>
                ) : null}
                <pre>{JSON.stringify(pack, null, 2)}</pre>
              </>
            )}
          </div>
        </section>
      </div>

      <footer>
        Pipeline: FEA → #42 → #15|#52 → optional #10 → #27|#22 → D stub · No
        invented paper metrics · Gaps flagged from research cards
      </footer>
    </main>
  );
}
