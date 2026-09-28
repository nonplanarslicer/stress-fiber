//! #15 — matrix fill between continuous-fiber centerlines.
//!
//! Card stage: "Assign matrix fill between fiber curves."
//!
//! Strategy:
//! - ≥2 printable fibers on a layer → zigzag hatch spanning the UV AABB of
//!   those fibers, stepped by `hardware.min_spacing` along the mean fiber
//!   direction (parallel / offset hatch).
//! - <2 printable fibers on a layer → a single boundary-parallel pass along
//!   the layer point AABB (documented fallback so the dual-extrude stage still
//!   sees matrix work when fibers are sparse).

use crate::field::build_local_frame;
use crate::layer::LayerSurface;
use crate::types::{FiberHardwareProfile, FiberPath, Point3};

/// Matrix (thermoplastic) fill segment between / around fibers.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MatrixSegment {
    pub layer_id: u32,
    pub start: Point3,
    pub end: Point3,
}

/// #15 stage 5a: generate matrix fill between accepted fiber paths.
///
/// Spacing / width come from `hardware` (caller params), not paper constants.
pub fn fill_between_fibers(
    layers: &[LayerSurface],
    fibers: &[FiberPath],
    hardware: &FiberHardwareProfile,
) -> Vec<MatrixSegment> {
    let spacing = hardware.min_spacing.abs().max(1e-9);
    let mut out = Vec::new();
    for layer in layers {
        let layer_fibers: Vec<&FiberPath> = fibers
            .iter()
            .filter(|f| f.printable && f.polyline.layer_id == layer.id)
            .collect();
        if layer_fibers.len() >= 2 {
            out.extend(hatch_between_fibers(layer, &layer_fibers, spacing));
        } else {
            out.extend(boundary_parallel_pass(layer, spacing));
        }
    }
    out
}

fn hatch_between_fibers(
    layer: &LayerSurface,
    fibers: &[&FiberPath],
    spacing: f64,
) -> Vec<MatrixSegment> {
    let mut pts = Vec::new();
    for f in fibers {
        pts.extend(f.polyline.points.iter().copied());
    }
    if pts.len() < 2 {
        return boundary_parallel_pass(layer, spacing);
    }
    let frame = build_local_frame(&pts);

    // Mean fiber tangent in UV → hatch lines run parallel to fibers,
    // stepped along the orthogonal axis.
    let mut tu = 0.0;
    let mut tv = 0.0;
    let mut n_seg = 0usize;
    for f in fibers {
        for w in f.polyline.points.windows(2) {
            let (u0, v0) = frame.to_uv(w[0]);
            let (u1, v1) = frame.to_uv(w[1]);
            tu += u1 - u0;
            tv += v1 - v0;
            n_seg += 1;
        }
    }
    if n_seg == 0 {
        return boundary_parallel_pass(layer, spacing);
    }
    let tlen = (tu * tu + tv * tv).sqrt();
    let (dir_u, dir_v) = if tlen > 1e-12 {
        (tu / tlen, tv / tlen)
    } else {
        (1.0, 0.0)
    };
    let (ortho_u, ortho_v) = (-dir_v, dir_u);

    let mut u_min = f64::INFINITY;
    let mut u_max = f64::NEG_INFINITY;
    let mut v_min = f64::INFINITY;
    let mut v_max = f64::NEG_INFINITY;
    for p in &pts {
        let (u, v) = frame.to_uv(*p);
        u_min = u_min.min(u);
        u_max = u_max.max(u);
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }

    // Project AABB corners onto ortho / along-fiber axes.
    let corners = [
        (u_min, v_min),
        (u_max, v_min),
        (u_min, v_max),
        (u_max, v_max),
    ];
    let mut o_min = f64::INFINITY;
    let mut o_max = f64::NEG_INFINITY;
    let mut a_min = f64::INFINITY;
    let mut a_max = f64::NEG_INFINITY;
    for &(u, v) in &corners {
        let o = u * ortho_u + v * ortho_v;
        let a = u * dir_u + v * dir_v;
        o_min = o_min.min(o);
        o_max = o_max.max(o);
        a_min = a_min.min(a);
        a_max = a_max.max(a);
    }
    if o_max - o_min < spacing * 0.5 {
        return boundary_parallel_pass(layer, spacing);
    }

    let mut segs = Vec::new();
    let mut flip = false;
    let mut o = o_min + spacing * 0.5;
    while o <= o_max - spacing * 0.25 {
        let start_a = if flip { a_max } else { a_min };
        let end_a = if flip { a_min } else { a_max };
        let s_u = ortho_u * o + dir_u * start_a;
        let s_v = ortho_v * o + dir_v * start_a;
        let e_u = ortho_u * o + dir_u * end_a;
        let e_v = ortho_v * o + dir_v * end_a;
        segs.push(MatrixSegment {
            layer_id: layer.id,
            start: frame.from_uv(s_u, s_v),
            end: frame.from_uv(e_u, e_v),
        });
        flip = !flip;
        o += spacing;
    }
    segs
}

/// Fallback when fewer than two fibers are present on the layer: one zigzag
/// pass covering the layer point AABB, stepped by `spacing`.
fn boundary_parallel_pass(layer: &LayerSurface, spacing: f64) -> Vec<MatrixSegment> {
    if layer.points.is_empty() {
        return Vec::new();
    }
    let pts: Vec<Point3> = layer.points.iter().copied().map(Point3::from_array).collect();
    let frame = build_local_frame(&pts);
    let mut u_min = f64::INFINITY;
    let mut u_max = f64::NEG_INFINITY;
    let mut v_min = f64::INFINITY;
    let mut v_max = f64::NEG_INFINITY;
    for p in &pts {
        let (u, v) = frame.to_uv(*p);
        u_min = u_min.min(u);
        u_max = u_max.max(u);
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }
    if !u_min.is_finite() || (u_max - u_min) < 1e-12 || (v_max - v_min) < 1e-12 {
        return Vec::new();
    }

    let mut segs = Vec::new();
    let mut flip = false;
    let mut v = v_min;
    // Prefer lines parallel to e0 (u), stepped in v — matches default fiber
    // direction when stress is empty / principal along PCA e0.
    while v <= v_max + 1e-12 {
        let (s, e) = if flip {
            (frame.from_uv(u_max, v), frame.from_uv(u_min, v))
        } else {
            (frame.from_uv(u_min, v), frame.from_uv(u_max, v))
        };
        segs.push(MatrixSegment {
            layer_id: layer.id,
            start: s,
            end: e,
        });
        flip = !flip;
        v += spacing;
        if segs.len() > 10_000 {
            break;
        }
    }
    segs
}
