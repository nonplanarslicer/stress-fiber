# np-fiber (Family E)

Continuous-fiber planning on layered surfaces from **NP Rust Core (#42)**.
This crate never extracts iso-layers; it consumes [`LayerSurface`](src/layer.rs).

Research cards: `papers/cards/015-*.md`, `010-*.md`, `052-*.md`.

## Priority

1. **#15** — stress-weighted scalar → isocurve centerlines → bend-radius gate → matrix fill → dual-extrude sync (**implemented**)
2. **#10** — PSL-guided hole loops (`hole_loops`, **implemented**)
3. **#52** — 2-RoSy + periodic scalar dense packing (`rosy`, **stub**)

## What #15 implements

| Stage | Module | Status |
| --- | --- | --- |
| Stress-weighted scalar φ | `field` | PCA local frame, IDW stress, magnitude→spacing from hardware, integrate `Δφ = Δu_ortho / s` |
| Isocurve centerlines | `isocurve` | Marching squares on UV grid; empty `iso_levels` → N=8 evenly spaced over `[φ_min, φ_max]` (API default, not a paper constant) |
| Bend-radius gate | `bend` | Circumradius filter; radius from `FiberHardwareProfile` |
| Matrix fill | `matrix_fill` | Zigzag hatch between ≥2 fibers; else boundary-parallel AABB pass |
| Dual-extrude sync | `dual_extrude` | `fiber_on=true` on fiber TCP, `false` on matrix |
| Orchestration | `pipeline` | `run_stress_isocurve_pipeline` (+ optional #10 holes) |

## What #10 implements

| Stage | Module | Status |
| --- | --- | --- |
| Hole targets | `hole_loops` | `HoleTarget`: boundary polyline **or** center+radius; optional `loop_count` / `stress_intensity` |
| Outward offset loops | `hole_loops` | First ring offset ≥ `hardware.width`; concentric rings stepped by stress-mapped `[min_spacing, max_spacing]` |
| Bend gate | `bend` | Drop illegal closed loops via `respects_min_bend_radius` (no invented smoothing) |
| Pipeline hook | `pipeline` | Optional `FiberPipelineInput.holes` appends closed loops before bend; empty → #15 unchanged |

Still stub: `#52` (`rosy`).

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
| `pipeline` | #15 end-to-end orchestration (+ optional #10) |
| `hole_loops` | #10 PSL-guided hole loops |
| `rosy` | #52 stub |

## Hardware params

`FiberHardwareProfile { width, min_bend_radius, min_spacing, max_spacing }` — always
caller-supplied. Cards state corpus has **no** bend-radius / spacing constants.
Loop counts come from `HoleTarget.loop_count` (default 1), never paper metrics.

## Build

```bash
cargo test -p np-fiber
```

napi TypeScript types live in `crates/np-native` (`fiber_run_pipeline`, `index.d.ts`).
