# np-pack

Layer-wise continuous-fiber **loop packing (#27)** and **Deep-Q next-node planning (#22)** for the nonplanarslicer stress-fiber monorepo.

Owned by **NP Pack UI**. Upstream field / routing stay in `np-core` / `np-fiber`; this crate owns packing + sequencing and the JSON ABI for `np-native::pack_run`.

## Algorithms

- **#27 MILP** — shipped implementation is deterministic greedy-by-weight, respecting conflicts, centroid spacing, bend radius, and length budget. **NOT a full MILP.**
- **#22 Deep-Q** — missing weights fall back to a greedy heuristic. No proprietary solvers or invented paper metrics.

## Test

```bash
cargo test -p np-pack
```
