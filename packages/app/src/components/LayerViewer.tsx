import type { LoopCandidate, PackOutput } from "@nonplanarslicer/api";
import { DEMO_NOTE } from "../data/demoLayers";

const COLORS = ["#66e0ba", "#7ab8ff", "#e0b466", "#e066a8", "#a8e066"];

export function LayerViewer({
  candidates,
  pack,
}: {
  candidates: LoopCandidate[];
  pack: PackOutput | undefined;
}) {
  const selected = new Set(pack?.selected_loop_ids ?? []);
  const tourIndex = new Map((pack?.tour_order ?? []).map((id, i) => [id, i]));

  const allPts = candidates.flatMap((c) => c.points);
  const xs = allPts.map((p) => p[0]);
  const ys = allPts.map((p) => p[1]);
  const minX = Math.min(...xs, 0) - 10;
  const maxX = Math.max(...xs, 200) + 10;
  const minY = Math.min(...ys, 0) - 10;
  const maxY = Math.max(...ys, 200) + 10;
  const w = maxX - minX;
  const h = maxY - minY;

  return (
    <div className="layer-viewer">
      <svg viewBox={`${minX} ${minY} ${w} ${h}`} role="img" aria-label="Layer loop preview">
        <rect x={minX} y={minY} width={w} height={h} className="viewer-bg" />
        {candidates.map((c, i) => {
          const d =
            c.points.map((p, pi) => `${pi === 0 ? "M" : "L"}${p[0]},${p[1]}`).join(" ") +
            " Z";
          const isSel = selected.has(c.id);
          const order = tourIndex.get(c.id);
          return (
            <g key={c.id} opacity={pack && !isSel ? 0.25 : 1}>
              <path
                d={d}
                fill="none"
                stroke={COLORS[i % COLORS.length]}
                strokeWidth={isSel ? 2.5 : 1.2}
                strokeDasharray={isSel ? undefined : "4 3"}
              />
              {order !== undefined && (
                <text
                  x={c.points[0][0]}
                  y={c.points[0][1] - 3}
                  className="loop-label"
                >
                  {order + 1}:{c.id}
                </text>
              )}
              {order === undefined && (
                <text x={c.points[0][0]} y={c.points[0][1] - 3} className="loop-label muted">
                  {c.id}
                </text>
              )}
            </g>
          );
        })}
      </svg>
      <p className="demo-note">{DEMO_NOTE}</p>
    </div>
  );
}
