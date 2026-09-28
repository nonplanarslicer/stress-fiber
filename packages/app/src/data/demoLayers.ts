/**
 * Synthetic demo loops for the UI — EXAMPLE DATA ONLY.
 * Not paper metrics; not from arXiv 2404.11404 / 2408.09198 results.
 */
import type { LoopCandidate, PackConstraints, PackInput } from "@nonplanarslicer/api";
import { defaultPackConstraints } from "@nonplanarslicer/api";

export const DEMO_NOTE =
  "Synthetic example loops for UI demo — not paper metrics.";

export const demoCandidates: LoopCandidate[] = [
  {
    id: "L0",
    layer_id: 0,
    points: [
      [20, 20],
      [80, 20],
      [80, 70],
      [20, 70],
      [20, 20],
    ],
    weight: 8,
    length: 200,
    min_bend_radius: 5,
  },
  {
    id: "L1",
    layer_id: 0,
    points: [
      [110, 25],
      [170, 25],
      [170, 75],
      [110, 75],
      [110, 25],
    ],
    weight: 5,
    length: 200,
    min_bend_radius: 5,
  },
  {
    id: "L2",
    layer_id: 0,
    points: [
      [30, 30],
      [70, 30],
      [70, 60],
      [30, 60],
      [30, 30],
    ],
    weight: 9,
    length: 160,
    min_bend_radius: 5,
  },
  {
    id: "L3",
    layer_id: 1,
    points: [
      [40, 100],
      [100, 100],
      [100, 150],
      [40, 150],
      [40, 100],
    ],
    weight: 4,
    length: 200,
    min_bend_radius: 5,
  },
  {
    id: "L4",
    layer_id: 1,
    points: [
      [130, 105],
      [180, 105],
      [180, 145],
      [130, 145],
      [130, 105],
    ],
    weight: 6,
    length: 160,
    min_bend_radius: 5,
  },
];

/** L0 conflicts with nested L2 (explicit pair); spacing also applies. */
export const demoConflictPairs: [string, string][] = [["L0", "L2"]];

export function buildDemoPackInput(
  strategy: PackInput["strategy"],
  constraints: PackConstraints = defaultPackConstraints,
): PackInput {
  return {
    candidates: demoCandidates,
    constraints,
    strategy,
    conflict_pairs: demoConflictPairs,
  };
}
