const STAGES = [
  { id: "fea", label: "FEA" },
  { id: "42", label: "#42" },
  { id: "field", label: "#15|#52" },
  { id: "10", label: "#10?", optional: true },
  { id: "pack", label: "#27|#22", active: true },
  { id: "d", label: "D stub" },
] as const;

export function PipelineBar() {
  return (
    <nav className="pipeline" aria-label="Chaining pipeline">
      {STAGES.map((s, i) => (
        <span key={s.id} className="pipeline-stage-wrap">
          {i > 0 && <span className="pipeline-arrow" aria-hidden>→</span>}
          <span
            className={[
              "pipeline-stage",
              "optional" in s && s.optional ? "optional" : "",
              "active" in s && s.active ? "active" : "",
            ]
              .filter(Boolean)
              .join(" ")}
            title={
              s.id === "10"
                ? "Optional hole-loop / routing (#10)"
                : s.id === "pack"
                  ? "Pack / sequence — this UI"
                  : undefined
            }
          >
            {s.label}
          </span>
        </span>
      ))}
    </nav>
  );
}
