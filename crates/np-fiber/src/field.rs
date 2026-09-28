//! #15 — stress-weighted scalar field on layered surfaces.
//!
//! Card stage: "Build a stress-weighted scalar field on each printable layer
//! so isocurve spacing tightens where stress is high."
//!
//! Consumes [`LayerSurface`](crate::layer::LayerSurface) from #42; does **not**
//! extract iso-layers.
//!
//! Algorithm (clean-room):
//! 1. PCA local frame on each layer (fallback XY if nearly planar in Z).
//! 2. IDW-interpolate stress magnitude + principal direction onto layer /
//!    denser UV-grid samples; project direction into the tangent plane.
//! 3. Empty stress → magnitude 1.0 and default in-plane direction (PCA e0).
//! 4. Map magnitude → local spacing in `[min_spacing, max_spacing]` (caller
//!    hardware params) and integrate φ along the axis orthogonal to the
//!    principal direction: `Δφ = Δu_ortho / s`.

use crate::types::{
    FiberHardwareProfile, LayerSurface, Point3, ScalarFieldSample, StressField, StressSample,
};

/// Input for #15 scalar-field construction.
#[derive(Clone, Debug)]
pub struct FieldInput<'a> {
    pub layers: &'a [LayerSurface],
    pub stress: &'a [StressSample],
    pub hardware: &'a FiberHardwareProfile,
}

/// Orthonormal tangent frame for a layer (origin + two in-plane axes + normal).
#[derive(Clone, Copy, Debug)]
pub(crate) struct LocalFrame {
    pub origin: Point3,
    pub e0: Point3,
    pub e1: Point3,
    pub normal: Point3,
}

impl LocalFrame {
    pub fn to_uv(self, p: Point3) -> (f64, f64) {
        let d = Point3::new(p.x - self.origin.x, p.y - self.origin.y, p.z - self.origin.z);
        (dot(d, self.e0), dot(d, self.e1))
    }

    pub fn from_uv(self, u: f64, v: f64) -> Point3 {
        Point3::new(
            self.origin.x + u * self.e0.x + v * self.e1.x,
            self.origin.y + u * self.e0.y + v * self.e1.y,
            self.origin.z + u * self.e0.z + v * self.e1.z,
        )
    }

    pub fn project_dir(self, d: Point3) -> Point3 {
        // Remove normal component, then re-normalize in the tangent plane.
        let n_comp = dot(d, self.normal);
        let mut t = Point3::new(
            d.x - n_comp * self.normal.x,
            d.y - n_comp * self.normal.y,
            d.z - n_comp * self.normal.z,
        );
        let len = (t.x * t.x + t.y * t.y + t.z * t.z).sqrt();
        if len < 1e-12 {
            return self.e0;
        }
        t.x /= len;
        t.y /= len;
        t.z /= len;
        t
    }
}

fn dot(a: Point3, b: Point3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn cross(a: Point3, b: Point3) -> Point3 {
    Point3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

fn norm(a: Point3) -> f64 {
    (a.x * a.x + a.y * a.y + a.z * a.z).sqrt()
}

fn normalize(a: Point3) -> Point3 {
    let n = norm(a);
    if n < 1e-15 {
        return Point3::new(0.0, 0.0, 0.0);
    }
    Point3::new(a.x / n, a.y / n, a.z / n)
}

/// Build a local 2-D frame from a point cloud via PCA (largest two eigenvectors
/// span the tangent plane). Falls back to world XY when the cloud is degenerate
/// or essentially flat in Z.
pub(crate) fn build_local_frame(points: &[Point3]) -> LocalFrame {
    if points.is_empty() {
        return LocalFrame {
            origin: Point3::default(),
            e0: Point3::new(1.0, 0.0, 0.0),
            e1: Point3::new(0.0, 1.0, 0.0),
            normal: Point3::new(0.0, 0.0, 1.0),
        };
    }

    let n = points.len() as f64;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for p in points {
        cx += p.x;
        cy += p.y;
        cz += p.z;
    }
    cx /= n;
    cy /= n;
    cz /= n;
    let origin = Point3::new(cx, cy, cz);

    // Covariance (symmetric 3×3).
    let mut c = [[0.0_f64; 3]; 3];
    for p in points {
        let d = [p.x - cx, p.y - cy, p.z - cz];
        for i in 0..3 {
            for j in 0..3 {
                c[i][j] += d[i] * d[j];
            }
        }
    }
    for row in &mut c {
        for v in row.iter_mut() {
            *v /= n;
        }
    }

    // Extent along Z relative to XY — prefer XY fallback when nearly planar in Z.
    let ext_x = (c[0][0]).sqrt();
    let ext_y = (c[1][1]).sqrt();
    let ext_z = (c[2][2]).sqrt();
    let xy_ext = ext_x.max(ext_y);
    if xy_ext > 1e-12 && ext_z < 1e-6 * xy_ext {
        return LocalFrame {
            origin,
            e0: Point3::new(1.0, 0.0, 0.0),
            e1: Point3::new(0.0, 1.0, 0.0),
            normal: Point3::new(0.0, 0.0, 1.0),
        };
    }

    let e0 = power_iteration(&c, 24);
    // Deflate largest direction and take next.
    let mut c2 = c;
    for i in 0..3 {
        for j in 0..3 {
            c2[i][j] -= e0[i] * e0[j] * rayleigh(&c, &e0);
        }
    }
    let mut e1 = power_iteration(&c2, 24);
    // Orthonormalize against e0.
    let proj = e1[0] * e0[0] + e1[1] * e0[1] + e1[2] * e0[2];
    e1 = [e1[0] - proj * e0[0], e1[1] - proj * e0[1], e1[2] - proj * e0[2]];
    let e1n = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
    if e1n < 1e-12 {
        // Degenerate: pick any orthogonal.
        let helper = if e0[0].abs() < 0.9 {
            Point3::new(1.0, 0.0, 0.0)
        } else {
            Point3::new(0.0, 1.0, 0.0)
        };
        let e0p = Point3::new(e0[0], e0[1], e0[2]);
        let e1p = normalize(cross(e0p, helper));
        let normal = normalize(cross(e0p, e1p));
        return LocalFrame {
            origin,
            e0: e0p,
            e1: e1p,
            normal,
        };
    }
    e1 = [e1[0] / e1n, e1[1] / e1n, e1[2] / e1n];
    let e0p = Point3::new(e0[0], e0[1], e0[2]);
    let e1p = Point3::new(e1[0], e1[1], e1[2]);
    let normal = normalize(cross(e0p, e1p));
    LocalFrame {
        origin,
        e0: e0p,
        e1: e1p,
        normal,
    }
}

fn rayleigh(c: &[[f64; 3]; 3], v: &[f64; 3]) -> f64 {
    let mut cv = [0.0; 3];
    for i in 0..3 {
        cv[i] = c[i][0] * v[0] + c[i][1] * v[1] + c[i][2] * v[2];
    }
    cv[0] * v[0] + cv[1] * v[1] + cv[2] * v[2]
}

fn power_iteration(c: &[[f64; 3]; 3], iters: usize) -> [f64; 3] {
    let mut v = [1.0, 0.3, 0.1];
    for _ in 0..iters {
        let mut w = [
            c[0][0] * v[0] + c[0][1] * v[1] + c[0][2] * v[2],
            c[1][0] * v[0] + c[1][1] * v[1] + c[1][2] * v[2],
            c[2][0] * v[0] + c[2][1] * v[1] + c[2][2] * v[2],
        ];
        let n = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if n < 1e-15 {
            return [1.0, 0.0, 0.0];
        }
        w[0] /= n;
        w[1] /= n;
        w[2] /= n;
        v = w;
    }
    v
}

/// Inverse-distance-weighted interpolate of magnitude + principal direction.
fn idw_stress(
    p: Point3,
    stress: &[StressSample],
    frame: LocalFrame,
) -> (f64, Point3) {
    const EPS: f64 = 1e-9;
    let mut w_sum = 0.0;
    let mut m_acc = 0.0;
    let mut dx = 0.0;
    let mut dy = 0.0;
    let mut dz = 0.0;
    for s in stress {
        let d = p.distance(s.position);
        let w = 1.0 / (d * d + EPS);
        w_sum += w;
        m_acc += w * s.magnitude;
        dx += w * s.principal_direction.x;
        dy += w * s.principal_direction.y;
        dz += w * s.principal_direction.z;
    }
    if w_sum < 1e-30 {
        return (1.0, frame.e0);
    }
    let m = m_acc / w_sum;
    let dir = frame.project_dir(Point3::new(dx / w_sum, dy / w_sum, dz / w_sum));
    (m, dir)
}

fn map_magnitude_to_spacing(m: f64, m_min: f64, m_max: f64, hw: &FiberHardwareProfile) -> f64 {
    let lo = hw.min_spacing.min(hw.max_spacing);
    let hi = hw.min_spacing.max(hw.max_spacing);
    if (m_max - m_min).abs() < 1e-15 {
        return 0.5 * (lo + hi);
    }
    let t = ((m - m_min) / (m_max - m_min)).clamp(0.0, 1.0);
    // High magnitude → min_spacing (denser isocurves).
    hi + t * (lo - hi)
}

/// #15 stage 1–2: build stress-weighted scalar fields per layer.
///
/// Spacing bounds come from `hardware.min_spacing` / `max_spacing` (caller
/// params). Returns one [`StressField`] per layer with φ samples on layer
/// points plus a denser UV grid for robust isocurve marching.
pub fn build_stress_weighted_field(input: &FieldInput<'_>) -> Vec<StressField> {
    input
        .layers
        .iter()
        .map(|layer| build_field_for_layer(layer, input.stress, input.hardware))
        .collect()
}

fn build_field_for_layer(
    layer: &LayerSurface,
    stress: &[StressSample],
    hardware: &FiberHardwareProfile,
) -> StressField {
    let layer_pts: Vec<Point3> = layer.points.iter().copied().map(Point3::from_array).collect();
    if layer_pts.is_empty() {
        return StressField {
            layer_id: layer.id,
            samples: Vec::new(),
        };
    }

    let frame = build_local_frame(&layer_pts);

    // Per-layer-point magnitude + tangent principal direction.
    let mut mags = Vec::with_capacity(layer_pts.len());
    let mut dirs = Vec::with_capacity(layer_pts.len());
    if stress.is_empty() {
        for _ in &layer_pts {
            mags.push(1.0);
            dirs.push(frame.e0);
        }
    } else {
        for p in &layer_pts {
            let (m, d) = idw_stress(*p, stress, frame);
            mags.push(m);
            dirs.push(d);
        }
    }

    let m_min = mags.iter().cloned().fold(f64::INFINITY, f64::min);
    let m_max = mags.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // UV AABB of layer points.
    let mut u_min = f64::INFINITY;
    let mut u_max = f64::NEG_INFINITY;
    let mut v_min = f64::INFINITY;
    let mut v_max = f64::NEG_INFINITY;
    let mut uvs = Vec::with_capacity(layer_pts.len());
    for p in &layer_pts {
        let (u, v) = frame.to_uv(*p);
        uvs.push((u, v));
        u_min = u_min.min(u);
        u_max = u_max.max(u);
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }
    if !u_min.is_finite() {
        u_min = 0.0;
        u_max = 1.0;
        v_min = 0.0;
        v_max = 1.0;
    }
    // Pad slightly so the grid covers the boundary.
    let pad_u = ((u_max - u_min) * 0.02).max(1e-6);
    let pad_v = ((v_max - v_min) * 0.02).max(1e-6);
    u_min -= pad_u;
    u_max += pad_u;
    v_min -= pad_v;
    v_max += pad_v;

    // Grid resolution from sample density (aim ~sqrt(n) cells per side, clamp).
    let n_pts = layer_pts.len().max(4);
    let side = ((n_pts as f64).sqrt() * 2.0).ceil() as usize;
    let nu = side.clamp(8, 64);
    let nv = side.clamp(8, 64);
    let du = (u_max - u_min) / (nu as f64 - 1.0).max(1.0);
    let dv = (v_max - v_min) / (nv as f64 - 1.0).max(1.0);

    // Mean principal direction in UV → choose primary integration (ortho) axis.
    let mut mean_du = 0.0;
    let mut mean_dv = 0.0;
    for d in &dirs {
        mean_du += dot(*d, frame.e0);
        mean_dv += dot(*d, frame.e1);
    }
    let mean_len = (mean_du * mean_du + mean_dv * mean_dv).sqrt();
    let (pd_u, pd_v) = if mean_len > 1e-12 {
        (mean_du / mean_len, mean_dv / mean_len)
    } else {
        (1.0, 0.0)
    };
    // Ortho in UV (rotate 90° CCW).
    let (ortho_u, ortho_v) = (-pd_v, pd_u);

    // Build φ on the regular UV grid by integrating Δφ = Δortho / s.
    // At each grid node, IDW magnitude from layer samples (or constant).
    let mut grid_m = vec![1.0_f64; nu * nv];
    let mut grid_dir = vec![frame.e0; nu * nv];
    for j in 0..nv {
        for i in 0..nu {
            let u = u_min + i as f64 * du;
            let v = v_min + j as f64 * dv;
            let p = frame.from_uv(u, v);
            let idx = j * nu + i;
            if stress.is_empty() {
                // IDW magnitude from layer-point constants (all 1) — keep default dir.
                grid_m[idx] = 1.0;
                grid_dir[idx] = frame.e0;
            } else {
                let (m, d) = idw_stress(p, stress, frame);
                grid_m[idx] = m;
                grid_dir[idx] = d;
            }
        }
    }
    // If no stress, still allow m_min/m_max from layer (all 1).
    let g_m_min = grid_m.iter().cloned().fold(m_min, f64::min);
    let g_m_max = grid_m.iter().cloned().fold(m_max, f64::max);

    let mut grid_s = vec![0.0_f64; nu * nv];
    for idx in 0..nu * nv {
        grid_s[idx] = map_magnitude_to_spacing(grid_m[idx], g_m_min, g_m_max, hardware);
    }

    // Integrate φ along the ortho axis on the grid.
    // φ(u,v) ≈ ∫ (ortho_u du + ortho_v dv) / s  from a reference corner,
    // implemented as a separable sweep along the dominant ortho component.
    let mut grid_phi = vec![0.0_f64; nu * nv];
    // Seed corner, then sweep in +i and +j using local path increments projected on ortho.
    for j in 0..nv {
        for i in 0..nu {
            if i == 0 && j == 0 {
                grid_phi[0] = 0.0;
                continue;
            }
            let mut candidates = Vec::new();
            if i > 0 {
                let prev = j * nu + (i - 1);
                let s_avg = 0.5 * (grid_s[prev] + grid_s[j * nu + i]).max(1e-12);
                let d_ortho = ortho_u * du; // moving +u
                candidates.push(grid_phi[prev] + d_ortho / s_avg);
            }
            if j > 0 {
                let prev = (j - 1) * nu + i;
                let s_avg = 0.5 * (grid_s[prev] + grid_s[j * nu + i]).max(1e-12);
                let d_ortho = ortho_v * dv; // moving +v
                candidates.push(grid_phi[prev] + d_ortho / s_avg);
            }
            grid_phi[j * nu + i] = candidates.iter().sum::<f64>() / candidates.len() as f64;
        }
    }

    // Sample φ back onto layer points (bilinear from grid) and emit denser grid samples.
    let mut samples = Vec::with_capacity(layer_pts.len() + nu * nv);
    for (k, p) in layer_pts.iter().enumerate() {
        let (u, v) = uvs[k];
        let phi = sample_grid_bilinear(&grid_phi, nu, nv, u_min, v_min, du, dv, u, v);
        samples.push(ScalarFieldSample {
            position: *p,
            value: phi,
            direction: Some(dirs[k]),
        });
    }
    for j in 0..nv {
        for i in 0..nu {
            let u = u_min + i as f64 * du;
            let v = v_min + j as f64 * dv;
            let p = frame.from_uv(u, v);
            let idx = j * nu + i;
            samples.push(ScalarFieldSample {
                position: p,
                value: grid_phi[idx],
                direction: Some(grid_dir[idx]),
            });
        }
    }

    StressField {
        layer_id: layer.id,
        samples,
    }
}

fn sample_grid_bilinear(
    grid: &[f64],
    nu: usize,
    nv: usize,
    u_min: f64,
    v_min: f64,
    du: f64,
    dv: f64,
    u: f64,
    v: f64,
) -> f64 {
    if nu == 0 || nv == 0 || du <= 0.0 || dv <= 0.0 {
        return 0.0;
    }
    let fu = ((u - u_min) / du).clamp(0.0, (nu - 1) as f64);
    let fv = ((v - v_min) / dv).clamp(0.0, (nv - 1) as f64);
    let i0 = fu.floor() as usize;
    let j0 = fv.floor() as usize;
    let i1 = (i0 + 1).min(nu - 1);
    let j1 = (j0 + 1).min(nv - 1);
    let tu = fu - i0 as f64;
    let tv = fv - j0 as f64;
    let a = grid[j0 * nu + i0];
    let b = grid[j0 * nu + i1];
    let c = grid[j1 * nu + i0];
    let d = grid[j1 * nu + i1];
    let ab = a * (1.0 - tu) + b * tu;
    let cd = c * (1.0 - tu) + d * tu;
    ab * (1.0 - tv) + cd * tv
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn empty_stress_yields_constant_magnitude_field() {
        let layer = LayerSurface {
            id: 0,
            points: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            thickness: 0.2,
            sequence: 0,
        };
        let hw = FiberHardwareProfile::new(0.4, 2.0, 1.0, 4.0);
        let fields = build_stress_weighted_field(&FieldInput {
            layers: &[layer],
            stress: &[],
            hardware: &hw,
        });
        assert_eq!(fields.len(), 1);
        assert!(!fields[0].samples.is_empty());
        // φ should vary along the ortho (Y) axis for default e0=+X.
        let vals: Vec<f64> = fields[0].samples.iter().map(|s| s.value).collect();
        let vmin = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let vmax = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(vmax > vmin + 1e-6, "φ must span a range for isocurves");
    }
}
