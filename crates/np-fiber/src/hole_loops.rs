//! #10 — PSL-guided hole loop generation.
//!
//! Card core idea: "PSL-guided curved layers with fiber loops around holes."
//! Layers themselves come from #42; this module marks hole targets and emits
//! closed fiber loops under the caller-supplied bend radius / spacing.
//!
//! Clean-room stages (card #10):
//! 1. Resolve each [`HoleTarget`] boundary (polyline or center+radius circle).
//! 2. Offset outward by ≥ `hardware.width`, then concentric rings stepped by
//!    stress-mapped spacing in `[min_spacing, max_spacing]` (or `min_spacing`
//!    when no stress). Loop count is explicit [`HoleTarget::loop_count`] or 1 —
//!    never an invented paper count.
//! 3. Drop loops that violate [`crate::bend::respects_min_bend_radius`].
//! 4. Emit [`FiberPolyline`] with `closed: true`.

use crate::bend::respects_min_bend_radius;
use crate::field::build_local_frame;
use crate::layer::LayerSurface;
use crate::types::{FiberHardwareProfile, FiberPolyline, Point3};

/// Implementation default for synthesizing a circular hole boundary when the
/// caller supplies center+radius. Not a paper constant.
pub const DEFAULT_CIRCLE_SEGMENTS: usize = 32;

/// Hole / cutout target on a layer.
///
/// Prefer an explicit closed `boundary`, or `center` + `radius` for circular
/// holes. `loop_count` controls concentric packing; when absent a single loop
/// is emitted. `stress_intensity` (typically in `[0, 1]`) only affects the
/// spacing between concentric rings when `loop_count > 1`.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct HoleTarget {
    pub layer_id: u32,
    /// Closed polyline boundary in 3D (first≠last OK; treated as closed).
    #[serde(default)]
    pub boundary: Vec<Point3>,
    /// Circular hole center when `boundary` is empty.
    #[serde(default)]
    pub center: Option<Point3>,
    /// Circular hole radius when `boundary` is empty (with `center`).
    #[serde(default)]
    pub radius: Option<f64>,
    /// Optional local stress intensity for denser concentric packing.
    /// High → spacing nearer `hardware.min_spacing`; low → `max_spacing`.
    #[serde(default)]
    pub stress_intensity: Option<f64>,
    /// Explicit concentric loop count. Prefer this over inventing paper counts.
    /// `None` → a single loop.
    #[serde(default)]
    pub loop_count: Option<u32>,
}

impl HoleTarget {
    /// Circular hole helper.
    pub fn circle(layer_id: u32, center: Point3, radius: f64) -> Self {
        Self {
            layer_id,
            boundary: Vec::new(),
            center: Some(center),
            radius: Some(radius),
            stress_intensity: None,
            loop_count: None,
        }
    }

    /// Closed polyline boundary helper.
    pub fn from_boundary(layer_id: u32, boundary: Vec<Point3>) -> Self {
        Self {
            layer_id,
            boundary,
            center: None,
            radius: None,
            stress_intensity: None,
            loop_count: None,
        }
    }
}

/// #10: generate closed fiber loops around holes on Core layers.
///
/// Offset distance for ring `k` (0-based):
/// `hardware.width + k * spacing`, where `spacing` is stress-mapped into
/// `[hardware.min_spacing, hardware.max_spacing]` when `stress_intensity` is
/// set, else `hardware.min_spacing`. Illegal bend-radius loops are dropped
/// (no invented smoothing).
pub fn generate_hole_loops(
    layers: &[LayerSurface],
    holes: &[HoleTarget],
    hardware: &FiberHardwareProfile,
) -> Vec<FiberPolyline> {
    if holes.is_empty() {
        return Vec::new();
    }
    let width = hardware.width.abs().max(0.0);
    let mut out = Vec::new();
    let mut next_id = 0u32;

    for hole in holes {
        let Some(boundary) = resolve_boundary(layers, hole) else {
            continue;
        };
        if boundary.len() < 3 {
            continue;
        }

        let frame_pts: Vec<Point3> = layers
            .iter()
            .find(|l| l.id == hole.layer_id)
            .map(|l| l.points.iter().copied().map(Point3::from_array).collect())
            .unwrap_or_else(|| boundary.clone());
        let frame = build_local_frame(&frame_pts);

        let n_loops = hole.loop_count.unwrap_or(1).max(1);
        let spacing = concentric_spacing(hardware, hole.stress_intensity);

        for k in 0..n_loops {
            let offset = width + k as f64 * spacing;
            let points = offset_closed_polyline(&boundary, &frame, offset);
            if points.len() < 3 {
                continue;
            }
            let poly = FiberPolyline {
                id: next_id,
                layer_id: hole.layer_id,
                points,
                closed: true,
            };
            next_id = next_id.saturating_add(1);
            if respects_min_bend_radius(&poly, hardware.min_bend_radius) {
                out.push(poly);
            }
        }
    }
    out
}

/// Optional #10 stage entry (same as [`generate_hole_loops`]).
///
/// Call separately after #15 centerlines, or pass holes via
/// [`crate::pipeline::FiberPipelineInput::holes`] so they append before the
/// bend gate. Empty holes leave the #15 path unchanged.
pub fn run_hole_loop_stage(
    layers: &[LayerSurface],
    holes: &[HoleTarget],
    hardware: &FiberHardwareProfile,
) -> Vec<FiberPolyline> {
    generate_hole_loops(layers, holes, hardware)
}

fn concentric_spacing(hardware: &FiberHardwareProfile, stress: Option<f64>) -> f64 {
    let min_s = hardware.min_spacing.abs().max(1e-9);
    let max_s = hardware.max_spacing.abs().max(min_s);
    match stress {
        Some(s) => {
            let t = s.clamp(0.0, 1.0);
            // High stress → denser packing (min_spacing).
            max_s + (min_s - max_s) * t
        }
        None => min_s,
    }
}

fn resolve_boundary(layers: &[LayerSurface], hole: &HoleTarget) -> Option<Vec<Point3>> {
    if hole.boundary.len() >= 3 {
        return Some(dedup_closed(&hole.boundary));
    }
    let center = hole.center?;
    let radius = hole.radius.filter(|r| *r > 1e-12)?;
    let frame_pts: Vec<Point3> = layers
        .iter()
        .find(|l| l.id == hole.layer_id)
        .map(|l| l.points.iter().copied().map(Point3::from_array).collect())
        .unwrap_or_else(|| vec![center]);
    let frame = build_local_frame(&frame_pts);
    Some(synthesize_circle(center, radius, &frame, DEFAULT_CIRCLE_SEGMENTS))
}

fn dedup_closed(pts: &[Point3]) -> Vec<Point3> {
    if pts.len() >= 2 && pts[0].distance(*pts.last().unwrap()) < 1e-9 {
        pts[..pts.len() - 1].to_vec()
    } else {
        pts.to_vec()
    }
}

fn synthesize_circle(
    center: Point3,
    radius: f64,
    frame: &crate::field::LocalFrame,
    n: usize,
) -> Vec<Point3> {
    let n = n.max(3);
    let (cu, cv) = frame.to_uv(center);
    (0..n)
        .map(|i| {
            let theta = std::f64::consts::TAU * (i as f64) / (n as f64);
            frame.from_uv(cu + radius * theta.cos(), cv + radius * theta.sin())
        })
        .collect()
}

/// Offset a closed polyline outward (away from its UV centroid) by `distance`.
fn offset_closed_polyline(
    boundary: &[Point3],
    frame: &crate::field::LocalFrame,
    distance: f64,
) -> Vec<Point3> {
    let uv: Vec<(f64, f64)> = boundary.iter().map(|p| frame.to_uv(*p)).collect();
    let n = uv.len();
    if n < 3 {
        return Vec::new();
    }
    let mut cu = 0.0;
    let mut cv = 0.0;
    for &(u, v) in &uv {
        cu += u;
        cv += v;
    }
    cu /= n as f64;
    cv /= n as f64;

    let mut out = Vec::with_capacity(n);
    for &(u, v) in &uv {
        let du = u - cu;
        let dv = v - cv;
        let len = (du * du + dv * dv).sqrt();
        let (ou, ov) = if len > 1e-12 {
            (u + distance * du / len, v + distance * dv / len)
        } else {
            // Degenerate vertex at centroid — push along +e0.
            (u + distance, v)
        };
        out.push(frame.from_uv(ou, ov));
    }
    out
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::types::FiberHardwareProfile;

    fn flat_layer() -> LayerSurface {
        let mut points = Vec::new();
        let mut normals = Vec::new();
        for j in 0..8 {
            for i in 0..8 {
                points.push([i as f64, j as f64, 0.0]);
                normals.push([0.0, 0.0, 1.0]);
            }
        }
        LayerSurface {
            id: 0,
            points,
            normals,
            thickness: 0.2,
            sequence: 0,
        }
    }

    #[test]
    fn empty_holes_yield_no_loops() {
        let hw = FiberHardwareProfile::new(0.4, 0.1, 0.5, 2.0);
        let loops = generate_hole_loops(&[flat_layer()], &[], &hw);
        assert!(loops.is_empty());
    }
}
