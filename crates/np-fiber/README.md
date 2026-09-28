# np-fiber (Family E)

Continuous-fiber planning on layered surfaces from **NP Rust Core (#42)**.
This crate never extracts iso-layers; it consumes [`LayerSurface`](src/layer.rs).

Research cards: `papers/cards/015-*.md`, `010-*.md`, `052-*.md`.

## Priority

1. **#15** — stress-weighted scalar → isocurve centerlines → bend-radius gate → matrix fill → dual-extrude sync
2. **#10** — PSL-guided hole loops (`hole_loops`, stub)
3. **#52** — 2-RoSy + periodic scalar dense packing (`rosy`, stub)

## Module map

| Module | Role |
| --- | --- |
| `layer` | Locked `LayerSurface` (+ fiber `ToolpathPoint` / `TcpPath`) |
| `types` | `StressField`, `FiberPolyline`, `FiberPath`, `FiberHardwareProfile` |
| `field` | #15 stress-weighted scalar |
| `isocurve` | #15 centerline extraction |
| `bend` | Bend-radius enforcement (radius from hardware profile) |
| `matrix_fill` | Matrix fill between fibers |
| `dual_extrude` | Matrix + fiber feed sync hooks |
| `pipeline` | #15 end-to-end orchestration |
| `hole_loops` | #10 stub |
| `rosy` | #52 stub |

## Hardware params

`FiberHardwareProfile { width, min_bend_radius, min_spacing, max_spacing }` — always
caller-supplied. Cards state corpus has **no** bend-radius / spacing constants.

## Build

```bash
cargo test -p np-fiber
```

napi TypeScript types live in `crates/np-native` (`fiber_run_pipeline`, `index.d.ts`).
