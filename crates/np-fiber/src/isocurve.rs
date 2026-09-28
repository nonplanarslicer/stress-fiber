//! #15 — isocurve centerline extraction from stress-weighted scalar fields.
//!
//! Card stage: "Extract isocurves as fiber centerlines; clip to printable
//! region and merge short fragments."
//!
//! When `iso_levels` is empty, this module auto-picks **N = 8** evenly spaced
//! levels between the observed `[φ_min, φ_max]` on each field. That N is an
//! implementation convenience for empty-input callers, not a paper constant.

use crate::field::{build_local_frame, LocalFrame};
use crate::types::{FiberPolyline, Point3, StressField};

/// Default number of evenly spaced iso levels when the caller passes an empty
/// `iso_levels` slice. Documented API default — not a research-card constant.
pub const DEFAULT_ISO_LEVEL_COUNT: usize = 8;

/// #15 stage 3: extract adaptive-density isocurves as fiber centerlines.
pub fn extract_centerlines(fields: &[StressField], iso_levels: &[f64]) -> Vec<FiberPolyline> {
    let mut out = Vec::new();
    let mut next_id = 0u32;
    for field in fields {
        let polys = extract_field_centerlines(field, iso_levels, &mut next_id);
        out.extend(polys);
    }
    out
}

fn extract_field_centerlines(
    field: &StressField,
    iso_levels: &[f64],
    next_id: &mut u32,
) -> Vec<FiberPolyline> {
    if field.samples.len() < 3 {
        return Vec::new();
    }

    let positions: Vec<Point3> = field.samples.iter().map(|s| s.position).collect();
    let frame = build_local_frame(&positions);

    let mut u_min = f64::INFINITY;
    let mut u_max = f64::NEG_INFINITY;
    let mut v_min = f64::INFINITY;
    let mut v_max = f64::NEG_INFINITY;
    for p in &positions {
        let (u, v) = frame.to_uv(*p);
        u_min = u_min.min(u);
        u_max = u_max.max(u);
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }
    if !u_min.is_finite() || (u_max - u_min) < 1e-12 || (v_max - v_min) < 1e-12 {
        return Vec::new();
    }

    // Typical sample spacing ≈ median nearest-neighbour in UV (approx via grid step).
    let n = positions.len();
    let side = ((n as f64).sqrt() * 1.5).ceil() as usize;
    let nu = side.clamp(12, 96);
    let nv = side.clamp(12, 96);
    let du = (u_max - u_min) / (nu as f64 - 1.0);
    let dv = (v_max - v_min) / (nv as f64 - 1.0);
    let typical_spacing = (du * du + dv * dv).sqrt();

    // Rasterize φ onto regular UV grid via IDW from field samples.
    let mut grid = vec![0.0_f64; nu * nv];
    let mut occupied = vec![false; nu * nv];
    for j in 0..nv {
        for i in 0..nu {
            let u = u_min + i as f64 * du;
            let v = v_min + j as f64 * dv;
            if let Some(phi) = idw_phi(u, v, field, frame) {
                grid[j * nu + i] = phi;
                occupied[j * nu + i] = true;
            }
        }
    }
    // Fill any empty cells from neighbours (simple average) so marching works.
    fill_holes(&mut grid, &occupied, nu, nv);

    let (phi_min, phi_max) = grid_range(&grid);
    if !phi_min.is_finite() || (phi_max - phi_min).abs() < 1e-12 {
        return Vec::new();
    }

    let levels: Vec<f64> = if iso_levels.is_empty() {
        let n_levels = DEFAULT_ISO_LEVEL_COUNT;
        (0..n_levels)
            .map(|k| {
                let t = (k as f64 + 0.5) / n_levels as f64;
                phi_min + t * (phi_max - phi_min)
            })
            .collect()
    } else {
        iso_levels
            .iter()
            .copied()
            .filter(|l| *l >= phi_min - 1e-9 && *l <= phi_max + 1e-9)
            .collect()
    };

    // Expand AABB slightly for clipping.
    let expand = typical_spacing * 0.5;
    let aabb = (
        u_min - expand,
        u_max + expand,
        v_min - expand,
        v_max + expand,
    );

    let mut segments: Vec<(Point3, Point3)> = Vec::new();
    for level in levels {
        let segs = marching_squares(&grid, nu, nv, u_min, v_min, du, dv, level, frame);
        for (a, b) in segs {
            let (ua, va) = frame.to_uv(a);
            let (ub, vb) = frame.to_uv(b);
            if inside_aabb(ua, va, aabb) && inside_aabb(ub, vb, aabb) {
                segments.push((a, b));
            }
        }
    }

    let mut polylines = stitch_segments(&segments, typical_spacing * 0.35);
    let min_len = 2.0 * typical_spacing;
    polylines.retain(|pts| polyline_length(pts) >= min_len);

    let mut result = Vec::new();
    for pts in polylines {
        if pts.len() < 2 {
            continue;
        }
        result.push(FiberPolyline {
            id: *next_id,
            layer_id: field.layer_id,
            points: pts,
            closed: false,
        });
        *next_id += 1;
    }
    result
}

fn inside_aabb(u: f64, v: f64, aabb: (f64, f64, f64, f64)) -> bool {
    u >= aabb.0 && u <= aabb.1 && v >= aabb.2 && v <= aabb.3
}

fn idw_phi(u: f64, v: f64, field: &StressField, frame: LocalFrame) -> Option<f64> {
    const EPS: f64 = 1e-9;
    let mut w_sum = 0.0;
    let mut acc = 0.0;
    let mut nearest = f64::INFINITY;
    for s in &field.samples {
        let (su, sv) = frame.to_uv(s.position);
        let d2 = (su - u) * (su - u) + (sv - v) * (sv - v);
        nearest = nearest.min(d2.sqrt());
        let w = 1.0 / (d2 + EPS);
        w_sum += w;
        acc += w * s.value;
    }
    if w_sum < 1e-30 {
        return None;
    }
    Some(acc / w_sum)
}

fn fill_holes(grid: &mut [f64], occupied: &[bool], nu: usize, nv: usize) {
    for j in 0..nv {
        for i in 0..nu {
            let idx = j * nu + i;
            if occupied[idx] {
                continue;
            }
            let mut sum = 0.0;
            let mut n = 0;
            for (di, dj) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let ii = i as i32 + di;
                let jj = j as i32 + dj;
                if ii >= 0 && jj >= 0 && (ii as usize) < nu && (jj as usize) < nv {
                    let nidx = jj as usize * nu + ii as usize;
                    if occupied[nidx] {
                        sum += grid[nidx];
                        n += 1;
                    }
                }
            }
            if n > 0 {
                grid[idx] = sum / n as f64;
            }
        }
    }
}

fn grid_range(grid: &[f64]) -> (f64, f64) {
    let mut mn = f64::INFINITY;
    let mut mx = f64::NEG_INFINITY;
    for &v in grid {
        if v.is_finite() {
            mn = mn.min(v);
            mx = mx.max(v);
        }
    }
    (mn, mx)
}

/// Classic marching squares on a regular UV grid; returns 3-D segment endpoints.
fn marching_squares(
    grid: &[f64],
    nu: usize,
    nv: usize,
    u_min: f64,
    v_min: f64,
    du: f64,
    dv: f64,
    level: f64,
    frame: LocalFrame,
) -> Vec<(Point3, Point3)> {
    let mut segs = Vec::new();
    for j in 0..nv.saturating_sub(1) {
        for i in 0..nu.saturating_sub(1) {
            let v00 = grid[j * nu + i];
            let v10 = grid[j * nu + i + 1];
            let v01 = grid[(j + 1) * nu + i];
            let v11 = grid[(j + 1) * nu + i + 1];
            let mut code = 0u8;
            if v00 >= level {
                code |= 1;
            }
            if v10 >= level {
                code |= 2;
            }
            if v11 >= level {
                code |= 4;
            }
            if v01 >= level {
                code |= 8;
            }
            if code == 0 || code == 15 {
                continue;
            }

            let u0 = u_min + i as f64 * du;
            let v0 = v_min + j as f64 * dv;
            let p = |u: f64, v: f64| frame.from_uv(u, v);
            let lerp = |a: f64, b: f64, va: f64, vb: f64| -> f64 {
                let denom = vb - va;
                if denom.abs() < 1e-15 {
                    return 0.5 * (a + b);
                }
                let t = ((level - va) / denom).clamp(0.0, 1.0);
                a + t * (b - a)
            };

            // Edge midpoints: bottom, right, top, left.
            let bottom = p(lerp(u0, u0 + du, v00, v10), v0);
            let right = p(u0 + du, lerp(v0, v0 + dv, v10, v11));
            let top = p(lerp(u0, u0 + du, v01, v11), v0 + dv);
            let left = p(u0, lerp(v0, v0 + dv, v00, v01));

            // Ambiguous cases 5 and 10: pick the pair that connects consistently
            // (average of diagonals — use the "asymmetric" convention).
            match code {
                1 | 14 => segs.push((left, bottom)),
                2 | 13 => segs.push((bottom, right)),
                3 | 12 => segs.push((left, right)),
                4 | 11 => segs.push((right, top)),
                6 | 9 => segs.push((bottom, top)),
                7 | 8 => segs.push((left, top)),
                5 => {
                    segs.push((left, bottom));
                    segs.push((right, top));
                }
                10 => {
                    segs.push((bottom, right));
                    segs.push((left, top));
                }
                _ => {}
            }
        }
    }
    segs
}

fn stitch_segments(segments: &[(Point3, Point3)], eps: f64) -> Vec<Vec<Point3>> {
    if segments.is_empty() {
        return Vec::new();
    }
    let eps2 = (eps.max(1e-9)).powi(2);
    let mut used = vec![false; segments.len()];
    let mut polylines: Vec<Vec<Point3>> = Vec::new();

    for start in 0..segments.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let mut chain = vec![segments[start].0, segments[start].1];

        // Extend forward.
        loop {
            let tip = *chain.last().unwrap();
            let mut found = None;
            for (i, seg) in segments.iter().enumerate() {
                if used[i] {
                    continue;
                }
                if dist2(tip, seg.0) <= eps2 {
                    found = Some((i, seg.1));
                    break;
                }
                if dist2(tip, seg.1) <= eps2 {
                    found = Some((i, seg.0));
                    break;
                }
            }
            match found {
                Some((i, next)) => {
                    used[i] = true;
                    chain.push(next);
                }
                None => break,
            }
        }

        // Extend backward.
        loop {
            let tip = chain[0];
            let mut found = None;
            for (i, seg) in segments.iter().enumerate() {
                if used[i] {
                    continue;
                }
                if dist2(tip, seg.0) <= eps2 {
                    found = Some((i, seg.1));
                    break;
                }
                if dist2(tip, seg.1) <= eps2 {
                    found = Some((i, seg.0));
                    break;
                }
            }
            match found {
                Some((i, next)) => {
                    used[i] = true;
                    chain.insert(0, next);
                }
                None => break,
            }
        }

        // Forward/backward growth already consumes a connected component, so
        // each chain is maximal — just collect it.
        polylines.push(chain);
    }
    polylines
}

fn dist2(a: Point3, b: Point3) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    dx * dx + dy * dy + dz * dz
}

fn polyline_length(pts: &[Point3]) -> f64 {
    let mut len = 0.0;
    for w in pts.windows(2) {
        len += w[0].distance(w[1]);
    }
    len
}
