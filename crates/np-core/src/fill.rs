//! Stress-aligned fill polylines + tool vectors on clamped layer surfaces.

use crate::fea::StressGrid;
use crate::geom::{self, FillPolyline, LayerSurface, Vec3};

/// Generate parallel stripe fills on each layer.
///
/// Feed direction is the in-plane projection of the mean principal stress on
/// that layer (fallback: +X). Tool vectors copy (clamped) surface normals.
pub fn fill_layers(
    layers: &[LayerSurface],
    grid: &StressGrid,
    spacing: f64,
) -> Vec<FillPolyline> {
    let spacing = spacing.max(1e-6);
    let mut out = Vec::new();
    for layer in layers {
        if layer.points.len() < 2 {
            continue;
        }
        let feed = mean_inplane_feed(layer, grid);
        let normal_ref = mean_normal(layer);
        let binormal = geom::normalize(geom::cross(normal_ref, feed));
        // Project points into (feed, binormal) plane coords.
        let origin = layer.points[0];
        let mut u_vals = Vec::with_capacity(layer.points.len());
        let mut v_vals = Vec::with_capacity(layer.points.len());
        for p in &layer.points {
            let d = geom::sub(*p, origin);
            u_vals.push(geom::dot(d, feed));
            v_vals.push(geom::dot(d, binormal));
        }
        let (u_min, u_max) = min_max(&u_vals);
        let (v_min, v_max) = min_max(&v_vals);
        if (u_max - u_min).abs() < 1e-9 && (v_max - v_min).abs() < 1e-9 {
            continue;
        }

        let mut stripe_v = v_min;
        let mut flip = false;
        while stripe_v <= v_max + 1e-9 {
            let mut pts: Vec<Vec3> = Vec::new();
            let mut tvs: Vec<Vec3> = Vec::new();
            let mut feeds: Vec<Vec3> = Vec::new();
            // Sample along feed at ~spacing.
            let mut u = u_min;
            while u <= u_max + 1e-9 {
                let q = geom::add(
                    origin,
                    geom::add(geom::scale(feed, u), geom::scale(binormal, stripe_v)),
                );
                // Keep samples near the layer point cloud (radius ~ 1.5 * spacing).
                if let Some((p, n)) = nearest_on_layer(layer, q, spacing * 1.75) {
                    pts.push(p);
                    tvs.push(n);
                    feeds.push(feed);
                }
                u += spacing;
            }
            if pts.len() >= 2 {
                if flip {
                    pts.reverse();
                    tvs.reverse();
                    feeds.reverse();
                }
                out.push(FillPolyline {
                    layer_id: layer.id,
                    points: pts,
                    tool_vectors: tvs,
                    feed_directions: feeds,
                });
                flip = !flip;
            }
            stripe_v += spacing;
        }
    }
    out
}

fn mean_normal(layer: &LayerSurface) -> Vec3 {
    let mut acc = [0.0, 0.0, 0.0];
    for n in &layer.normals {
        acc = geom::add(acc, *n);
    }
    geom::normalize(acc)
}

fn mean_inplane_feed(layer: &LayerSurface, grid: &StressGrid) -> Vec3 {
    let n_ref = mean_normal(layer);
    let mut acc = [0.0, 0.0, 0.0];
    if !grid.is_empty() {
        for p in layer.points.iter().step_by(layer.points.len().max(1) / 8 + 1) {
            let d = sample_direction(grid, *p);
            // Remove normal component → in-plane.
            let inplane = geom::sub(d, geom::scale(n_ref, geom::dot(d, n_ref)));
            acc = geom::add(acc, inplane);
        }
    }
    if geom::norm(acc) < 1e-9 {
        // Fallback: in-plane +X.
        let x = [1.0, 0.0, 0.0];
        let inplane = geom::sub(x, geom::scale(n_ref, geom::dot(x, n_ref)));
        if geom::norm(inplane) < 1e-9 {
            geom::normalize(geom::cross(n_ref, [0.0, 1.0, 0.0]))
        } else {
            geom::normalize(inplane)
        }
    } else {
        geom::normalize(acc)
    }
}

fn sample_direction(grid: &StressGrid, p: Vec3) -> Vec3 {
    // Nearest-node lookup.
    let mut best = 0usize;
    let mut best_d = f64::INFINITY;
    for k in 0..grid.dims[2] {
        for j in 0..grid.dims[1] {
            for i in 0..grid.dims[0] {
                let q = grid.position(i, j, k);
                let d2 = geom::dot(geom::sub(p, q), geom::sub(p, q));
                if d2 < best_d {
                    best_d = d2;
                    best = grid.index(i, j, k);
                }
            }
        }
    }
    grid.directions[best]
}

fn nearest_on_layer(layer: &LayerSurface, q: Vec3, radius: f64) -> Option<(Vec3, Vec3)> {
    let r2 = radius * radius;
    let mut best_i = None;
    let mut best_d = f64::INFINITY;
    for (i, p) in layer.points.iter().enumerate() {
        let d2 = geom::dot(geom::sub(q, *p), geom::sub(q, *p));
        if d2 <= r2 && d2 < best_d {
            best_d = d2;
            best_i = Some(i);
        }
    }
    best_i.map(|i| {
        let n = layer
            .normals
            .get(i)
            .copied()
            .unwrap_or([0.0, 0.0, 1.0]);
        (layer.points[i], geom::normalize(n))
    })
}

fn min_max(vals: &[f64]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &v in vals {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if !lo.is_finite() {
        (0.0, 0.0)
    } else {
        (lo, hi)
    }
}
