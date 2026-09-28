# np-pack

Layer-wise continuous-fiber **loop packing (#27)** and **Deep-Q next-node planning (#22)** for the nonplanarslicer stress-fiber monorepo.

Owned by **NP Pack UI**. Upstream field / routing stay in `np-core` / `np-fiber`; this crate owns packing + sequencing and the JSON ABI for `np-native::pack_run`.

**Stand-in flags:** #27 (`MilpBackend`) ships a **deterministic greedy-by-weight stand-in — NOT a full MILP**. #22 (`DeepQPolicy`) is a **policy interface that falls back to greedy when weights are missing**. No invented paper metrics; no trained Deep-Q weights in-tree.

## I/O

**In (`PackInput`)**

| Field | Meaning |
| --- | --- |
| `candidates[]` | `LoopCandidate` — id, layer_id, points, weight, length |
| `constraints` | bend_radius, min_spacing, fiber_length_budget, cut_cost, restart_cost, allow_cuts |
| `strategy` | `milp` \\| `deepq` \\| `greedy` |
| `conflict_pairs` | optional hard exclusions by loop id |
| `weights_path` | optional Deep-Q weights (ignored when missing) |

**Out (`PackOutput`)**

| Field | Meaning |
| --- | --- |
| `selected_loop_ids` | loops kept by the packer |
| `tour_order` | deposition sequence |
| `cuts` | cut/restart events along the tour |
| `strategy` / `solver_status` | which path ran and how |
| `unplaced` | rejected candidate ids |
| `paths` | selected polylines for UI preview |
| `notes` | gap / stub labels |

**Stand-ins only; no proprietary solvers or invented paper metrics.**
