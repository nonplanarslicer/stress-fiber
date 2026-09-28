export type Algorithm = "core" | "fiber" | "pack";
export type JsonObject = Record<string, unknown>;
export type PackStrategy = "milp" | "deepq" | "greedy";

export interface NativeBindings {
  coreRun(input: string): string;
  fiberRun(input: string): string;
  packRun(input: string): string;
}

export interface PathPoint { x: number; y: number; z: number; }

/** #42 curved layer surface (matches np-core LayerSurface). */
export interface LayerSurface {
  id: number;
  points: [number, number, number][];
  normals: [number, number, number][];
  thickness: number;
  sequence: number;
}

/** #42 TCP sample (matches np-core ToolpathPoint). */
export interface ToolpathPoint {
  xyz: [number, number, number];
  tool_vector: [number, number, number];
  h: number;
  w: number;
  feature: string;
  sequence: number;
}

export interface TcpPath { points: ToolpathPoint[]; }

export interface LoopCandidate {
  id: string;
  layer_id: number;
  points: [number, number][];
  weight: number;
  length: number;
  min_bend_radius?: number;
}

export interface PackConstraints {
  bend_radius: number;
  min_spacing: number;
  fiber_length_budget: number;
  cut_cost: number;
  restart_cost: number;
  allow_cuts: boolean;
}

export interface PackInput {
  candidates: LoopCandidate[];
  constraints: PackConstraints;
  strategy: PackStrategy;
  conflict_pairs?: [string, string][];
  weights_path?: string;
}

export interface CutEvent {
  after_tour_index: number;
  kind: string;
}

export interface PackOutput {
  selected_loop_ids: string[];
  tour_order: string[];
  cuts: CutEvent[];
  strategy: string;
  solver_status: string;
  unplaced: string[];
  notes?: string[];
  paths?: [number, number][][];
}

export interface UiResult {
  algorithm: Algorithm;
  strategy: string;
  paths: PathPoint[][];
  raw: JsonObject;
  native: boolean;
  pack?: PackOutput;
}

let native: NativeBindings | undefined;
export function configureNativeBindings(bindings: NativeBindings): void {
  native = bindings;
}

export const defaultPackConstraints: PackConstraints = {
  bend_radius: 3,
  min_spacing: 25,
  fiber_length_budget: 400,
  cut_cost: 10,
  restart_cost: 5,
  allow_cuts: true,
};

const emptyInput: Record<Algorithm, JsonObject> = {
  core: { stress: { samples: [] }, layer_height: 1, fill_spacing: 1, bead_width: 0.4, thickness: 0.2, max_overhang_deg: 45 },
  fiber: {
    field: { samples: [] },
    constraints: { min_bend_radius: 2, allow_cuts: true, restart_penalty: 1 },
  },
  pack: {
    candidates: [],
    constraints: defaultPackConstraints,
    strategy: "milp",
    conflict_pairs: [],
  },
};

/** Deterministic TypeScript mock mirroring np-pack greedy-by-weight ABI. */
export function mockPackRun(input: PackInput): PackOutput {
  const constraints = { ...defaultPackConstraints, ...input.constraints };
  const conflicts = new Set<string>();
  for (const [a, b] of input.conflict_pairs ?? []) {
    conflicts.add([a, b].sort().join("|"));
  }

  const centroid = (pts: [number, number][]): [number, number] => {
    if (!pts.length) return [0, 0];
    const n = pts.length;
    return [
      pts.reduce((s, p) => s + p[0], 0) / n,
      pts.reduce((s, p) => s + p[1], 0) / n,
    ];
  };
  const dist = (a: [number, number], b: [number, number]) =>
    Math.hypot(a[0] - b[0], a[1] - b[1]);

  const cents = input.candidates.map((c) => centroid(c.points));
  for (let i = 0; i < input.candidates.length; i++) {
    for (let j = i + 1; j < input.candidates.length; j++) {
      if (input.candidates[i].layer_id !== input.candidates[j].layer_id) continue;
      if (dist(cents[i], cents[j]) < constraints.min_spacing) {
        conflicts.add(
          [input.candidates[i].id, input.candidates[j].id].sort().join("|"),
        );
      }
    }
  }

  const byLayer = new Map<number, LoopCandidate[]>();
  for (const c of input.candidates) {
    const list = byLayer.get(c.layer_id) ?? [];
    list.push(c);
    byLayer.set(c.layer_id, list);
  }

  const selected: LoopCandidate[] = [];
  let used = 0;
  const layers = [...byLayer.keys()].sort((a, b) => a - b);
  for (const layer of layers) {
    const order = [...(byLayer.get(layer) ?? [])].sort((a, b) =>
      b.weight !== a.weight ? b.weight - a.weight : a.id.localeCompare(b.id),
    );
    for (const c of order) {
      if (
        c.min_bend_radius !== undefined &&
        c.min_bend_radius + 1e-9 < constraints.bend_radius
      ) {
        continue;
      }
      if (used + c.length > constraints.fiber_length_budget + 1e-9) continue;
      const hits = selected.some((s) =>
        conflicts.has([s.id, c.id].sort().join("|")),
      );
      if (hits) continue;
      selected.push(c);
      used += c.length;
    }
  }

  // Nearest-neighbor tour starting at highest weight.
  const remaining = [...selected].sort((a, b) =>
    b.weight !== a.weight ? b.weight - a.weight : a.id.localeCompare(b.id),
  );
  const tour: LoopCandidate[] = [];
  const cuts: CutEvent[] = [];
  if (remaining.length) {
    tour.push(remaining.shift()!);
    while (remaining.length) {
      const last = tour[tour.length - 1];
      const lc = centroid(last.points);
      let bestI = 0;
      let bestD = Infinity;
      for (let i = 0; i < remaining.length; i++) {
        const d = dist(lc, centroid(remaining[i].points));
        if (d < bestD) {
          bestD = d;
          bestI = i;
        }
      }
      if (bestD > constraints.min_spacing * 4 && constraints.allow_cuts && tour.length) {
        cuts.push({ after_tour_index: tour.length - 1, kind: "cut+restart" });
      }
      tour.push(remaining.splice(bestI, 1)[0]);
    }
  }

  const selectedIds = new Set(selected.map((c) => c.id));
  const unplaced = input.candidates
    .filter((c) => !selectedIds.has(c.id))
    .map((c) => c.id);

  const isDeepq = input.strategy === "deepq";
  return {
    selected_loop_ids: selected.map((c) => c.id),
    tour_order: tour.map((c) => c.id),
    cuts,
    strategy: isDeepq
      ? "deepq/deepq-greedy-fallback"
      : input.strategy === "greedy"
        ? "greedy"
        : "milp-greedy-stub",
    solver_status: selected.length === 0
      ? "infeasible"
      : isDeepq
        ? "policy-fallback"
        : "stub-heuristic",
    unplaced,
    notes: [
      "typescript-mock mirroring np-pack greedy stand-in",
      "NOT a full MILP; card #27/#22 gaps apply",
    ],
    paths: tour.map((c) => c.points),
  };
}

export function runAlgorithm(
  algorithm: Algorithm,
  input: JsonObject = emptyInput[algorithm],
): UiResult {
  const encoded = JSON.stringify(input);
  if (native) {
    const fn =
      algorithm === "core"
        ? native.coreRun
        : algorithm === "fiber"
          ? native.fiberRun
          : native.packRun;
    const raw = JSON.parse(fn(encoded)) as JsonObject;
    const pack =
      algorithm === "pack" ? (raw as unknown as PackOutput) : undefined;
    return {
      algorithm,
      strategy: "napi-rs",
      paths: extractPaths(raw, pack),
      raw,
      native: true,
      pack,
    };
  }

  if (algorithm === "pack") {
    const pack = mockPackRun(input as unknown as PackInput);
    return {
      algorithm,
      strategy: pack.strategy,
      paths: (pack.paths ?? []).map((poly) =>
        poly.map(([x, y]) => ({ x, y, z: 0 })),
      ),
      raw: pack as unknown as JsonObject,
      native: false,
      pack,
    };
  }

  const raw: JsonObject =
    algorithm === "fiber"
      ? { field: { samples: [] }, routing: { paths: [] }, checks: [] }
      : {
          field: null,
          layers: [] as unknown as LayerSurface[],
          fill: [],
          collision: { clear: true, notes: [] },
          tcp: { points: [] },
        };
  return {
    algorithm,
    strategy: "typescript-mock",
    paths: [],
    raw,
    native: false,
  };
}

function extractPaths(raw: JsonObject, pack?: PackOutput): PathPoint[][] {
  if (pack?.paths?.length) {
    return pack.paths.map((poly) => poly.map(([x, y]) => ({ x, y, z: 0 })));
  }
  const routing = raw.routing as { paths?: PathPoint[][] } | undefined;
  return routing?.paths ?? [];
}

export function emitGcodeStub(pack: PackOutput, candidates: LoopCandidate[]): string {
  const byId = new Map(candidates.map((c) => [c.id, c]));
  const lines: string[] = [
    "; np-pack TCP/G-code STUB",
    `; strategy=${pack.strategy} status=${pack.solver_status}`,
    "G21 ; mm",
    "G90 ; absolute",
    "F600.0",
  ];
  const cutSet = new Set(pack.cuts.map((c) => c.after_tour_index));
  pack.tour_order.forEach((id, step) => {
    const c = byId.get(id);
    lines.push(`; --- loop ${id} ---`);
    lines.push("M700 ; FIBER_ON (placeholder)");
    (c?.points ?? []).forEach((p, i) => {
      lines.push(
        i === 0
          ? `G0 X${p[0].toFixed(4)} Y${p[1].toFixed(4)}`
          : `G1 X${p[0].toFixed(4)} Y${p[1].toFixed(4)}`,
      );
    });
    lines.push("M701 ; FIBER_OFF (placeholder)");
    if (cutSet.has(step)) {
      lines.push("M702 ; FIBER_CUT (placeholder)");
      lines.push("M703 ; FIBER_RESTART (placeholder)");
    }
  });
  lines.push("M400 ; wait", "; end stub");
  return lines.join("\n") + "\n";
}
