# np-core — #42 Reinforced FDM

Clean-room stress-aligned curved-layer pipeline for the nonplanarslicer monorepo.

**Does not** copy [GuoxinFang/ReinforcedFDM](https://github.com/GuoxinFang/ReinforcedFDM) Qt code; interfaces and algorithms here are independently documented stubs / approximations suitable for Fiber (E) and D handoff.

## Pipeline

```
StressIn  →  StressGrid  →  GoverningScalar φ
     →  iso LayerSurface[]  →  clamp  →  FillPolyline[] + tool vectors
     →  collision stub  →  TcpPath
```

Entry point: [`run`](src/pipeline.rs) / `np_core::run`.

## Stress ingest (`StressIn`)

JSON / Rust input accepts **either** form (serde untagged). Prefer dense grids when FEA already samples a regular mesh; use point samples for sparse probes.

### 1. Point samples (default / backward compatible)

```json
{
  "stress": {
    "samples": [
      {
        "position": [0.0, 0.0, 0.0],
        "principal_direction": [0.0, 0.0, 1.0],
        "magnitude": 12.5
      }
    ]
  },
  "layer_height": 0.2,
  "fill_spacing": 0.4,
  "bead_width": 0.4,
  "thickness": 0.2,
  "max_overhang_deg": 45.0
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `position` | `[f64; 3]` | Sample location |
| `principal_direction` | `[f64; 3]` | σ₁ direction (normalized on ingest) |
| `magnitude` | `f64` | σ₁ magnitude (relative weighting) |

Scattered samples are IDW-rasterized onto an axis-aligned grid before the field stage.

### 2. Dense grid

```json
{
  "stress": {
    "origin": [0.0, 0.0, 0.0],
    "spacing": [1.0, 1.0, 1.0],
    "dims": [8, 8, 8],
    "magnitudes": [ /* length nx*ny*nz, row-major i + j*nx + k*nx*ny */ ],
    "directions": [ /* same layout, each [dx,dy,dz] */ ]
  }
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `origin` | `[f64; 3]` | Node (0,0,0) corner |
| `spacing` | `[f64; 3]` | Cell size along XYZ |
| `dims` | `[usize; 3]` | Node counts `(nx, ny, nz)` |
| `magnitudes` | `Vec<f64>` | σ₁ magnitude per node |
| `directions` | `Vec<[f64; 3]>` | σ₁ direction per node |

**TODO:** external FEA solver integration (Abaqus/CalculiX/etc.) — this crate only ingests already-computed principal stress.

## Locked public types (crate root)

```rust
pub struct LayerSurface {
  pub id: u32,
  pub points: Vec<[f64; 3]>,
  pub normals: Vec<[f64; 3]>,
  pub thickness: f64,
  pub sequence: u32,
}
pub struct ToolpathPoint {
  pub xyz: [f64; 3],
  pub tool_vector: [f64; 3],
  pub h: f64, pub w: f64,
  pub feature: String,
  pub sequence: u32,
}
pub struct TcpPath { pub points: Vec<ToolpathPoint> }
```

`LayerSurface` is re-exported from the crate root for Fiber consumers.

## Modules

| Module | Role |
| --- | --- |
| `geom` | `Vec3`, `LayerSurface`, `ToolpathPoint`, `TcpPath`, `FillPolyline` |
| `fea` | `StressIn`, ingest → `StressGrid` |
| `field` | governing scalar φ (path-integral of σ₁) |
| `isosurface` | φ levels → `LayerSurface[]` |
| `clamp` | overhang cone + light smoothing |
| `fill` | stress-aligned stripe polylines + tool vectors |
| `collision` | Part F stub (always clear) — TODO pair #16/#68 |
| `tcp` | fill → `TcpPath` |
| `pipeline` | `CoreInput` / `CoreOutput` / `run` |

## napi

`crates/np-native` exposes `core_run(input_json) -> json` over `CoreInput` / `CoreOutput` (serde). `fiber_run` / `pack_run` are owned elsewhere and must not be wiped when touching the bridge.
