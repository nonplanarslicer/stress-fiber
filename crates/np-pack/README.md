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
| `strategy` | `milp` \| `deepq` \| `greedy` |
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

## Deep-Q weights (v1)

```json
{ "version": 1, "state_dim": N, "action_dim": M, "weights": [/* N*M row-major */] }
```

`weights_path` on `PackInput`: missing/unreadable → greedy fallback; valid → linear Q stand-in (`deepq-linear` / `weights-linear`). **Not** a trained Deep-Q net.

## Ownership

- **Spacing → conflict pairs:** owned here (centroid distance, same-layer + explicit pairs).
- **Interlayer stacking:** TODO (card #27) until real MILP encoding.

## Algorithms

- **#27 MILP** — pluggable `MilpBackend`; shipped implementation is **deterministic greedy-by-weight**, layer-wise, respecting conflicts, centroid spacing, bend radius, and length budget. **NOT a full MILP.** Card gap: solver choice, variable encoding, timing not in corpus.
- **#22 Deep-Q** — `NextNodePolicy` + `DeepQPolicy`; missing/unreadable weights → `GreedyHeuristicPolicy` (`policy-fallback`); v1 JSON load → linear Q stand-in. Card gap: architecture, reward, dataset / trained net not in corpus.

No proprietary solvers. No invented paper metrics.

## Test

```bash
cargo test -p np-pack
```
