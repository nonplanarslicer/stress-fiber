import type { PackConstraints } from "@nonplanarslicer/api";

type NumKey = {
  [K in keyof PackConstraints]: PackConstraints[K] extends number ? K : never;
}[keyof PackConstraints];

const FIELDS: { key: NumKey; label: string; hint: string; step: number }[] = [
  { key: "bend_radius", label: "Bend radius", hint: "Min legal fiber bend radius", step: 0.5 },
  { key: "min_spacing", label: "Min spacing", hint: "Centerline spacing / collision order", step: 1 },
  { key: "fiber_length_budget", label: "Fiber length budget", hint: "Total spool / layer budget", step: 10 },
  { key: "cut_cost", label: "Cut cost", hint: "Penalty for a cut event", step: 1 },
  { key: "restart_cost", label: "Restart cost", hint: "Penalty for fiber re-feed", step: 1 },
];

export function ConstraintsPanel({
  value,
  onChange,
}: {
  value: PackConstraints;
  onChange: (c: PackConstraints) => void;
}) {
  return (
    <section className="constraints panel-block">
      <h2>Constraints</h2>
      <div className="constraints-grid">
        {FIELDS.map((f) => (
          <label key={f.key}>
            <span>{f.label}</span>
            <input
              type="number"
              step={f.step}
              value={value[f.key]}
              title={f.hint}
              onChange={(e) =>
                onChange({ ...value, [f.key]: Number(e.target.value) })
              }
            />
          </label>
        ))}
        <label className="check">
          <input
            type="checkbox"
            checked={value.allow_cuts}
            onChange={(e) => onChange({ ...value, allow_cuts: e.target.checked })}
          />
          Allow cuts / restarts
        </label>
      </div>
    </section>
  );
}
