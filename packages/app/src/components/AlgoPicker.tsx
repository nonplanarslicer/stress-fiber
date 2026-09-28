import type { PackStrategy } from "@nonplanarslicer/api";

const GUIDE: Record<PackStrategy, { title: string; body: string }> = {
  milp: {
    title: "#27 MILP loop packing",
    body: "Prefer when layer graphs are small / offline. Exact packing under spacing, bend radius, and fiber budget. Current build: greedy-by-weight stand-in (NOT full MILP — solver encoding not in corpus).",
  },
  deepq: {
    title: "#22 Deep-Q next-node",
    body: "Prefer when graphs are large or online replan is needed. Local subgraph policy. Current build: greedy fallback (no trained weights in corpus).",
  },
  greedy: {
    title: "Greedy heuristic",
    body: "Explicit deterministic heuristic with no MILP/policy claim. Useful for UI smoke tests.",
  },
};

export function AlgoPicker({
  value,
  onChange,
}: {
  value: PackStrategy;
  onChange: (s: PackStrategy) => void;
}) {
  return (
    <fieldset className="algo-picker">
      <legend>Pack algorithm</legend>
      {(Object.keys(GUIDE) as PackStrategy[]).map((key) => (
        <label key={key} className={value === key ? "selected" : ""} title={GUIDE[key].body}>
          <input
            type="radio"
            name="pack-algo"
            value={key}
            checked={value === key}
            onChange={() => onChange(key)}
          />
          <span className="algo-title">{GUIDE[key].title}</span>
          <span className="algo-hint">{GUIDE[key].body}</span>
        </label>
      ))}
    </fieldset>
  );
}
