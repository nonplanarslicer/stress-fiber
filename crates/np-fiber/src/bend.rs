//! Bend-radius enforcement for continuous-fiber polylines.
//!
//! Used by #15 (after isocurves), #10 (hole loops), and #52 (dense packing).
//! The minimum radius is always taken from
//! [`FiberHardwareProfile::min_bend_radius`](crate::types::FiberHardwareProfile)
//! — never from invented paper constants.

use crate::types::{FiberHardwareProfile, FiberPath, FiberPolyline, Point3};

/// Discrete curvature radius at vertex `i` (open polyline), using circumradius
/// of points `(i-1, i, i+1)`. Returns `f64::INFINITY` for collinear / degenerate.
pub fn local_bend_radius(a: Point3, b: Point3, c: Point3) -> f64 {
    let ab = b.distance(a);
    let bc = c.distance(b);
    let ca = a.distance(c);
    if ab < f64::EPSILON || bc < f64::EPSILON || ca < f64::EPSILON {
        return f64::INFINITY;
    }
    // Triangle area via cross product of AB × BC.
    let abv = (b.x - a.x, b.y - a.y, b.z - a.z);
    let bcv = (c.x - b.x, c.y - b.y, c.z - b.z);
    let cx = abv.1 * bcv.2 - abv.2 * bcv.1;
    let cy = abv.2 * bcv.0 - abv.0 * bcv.2;
    let cz = abv.0 * bcv.1 - abv.1 * bcv.0;
    let area2 = (cx * cx + cy * cy + cz * cz).sqrt(); // 2 * area
    if area2 < f64::EPSILON {
        return f64::INFINITY;
    }
    // Circumradius R = abc / (4Δ); area2 = |AB×BC| = 2Δ ⇒ R = abc / (2 * area2)
    (ab * bc * ca) / (2.0 * area2)
}

/// True when every interior vertex respects `min_bend_radius`.
pub fn respects_min_bend_radius(poly: &FiberPolyline, min_bend_radius: f64) -> bool {
    if poly.points.len() < 3 {
        return true;
    }
    let n = poly.points.len();
    let last = if poly.closed { n } else { n - 1 };
    for i in 1..last {
        let a = poly.points[(i + n - 1) % n];
        let b = poly.points[i % n];
        let c = poly.points[(i + 1) % n];
        let r = local_bend_radius(a, b, c);
        if r < min_bend_radius {
            return false;
        }
    }
    true
}

/// #15 stage 4 (bend gate): keep or reject polylines by hardware min bend radius.
///
/// Rejects curves tighter than `hardware.min_bend_radius`. Does not invent a
/// smoothing radius; callers that need smoothing must supply their own policy.
pub fn enforce_bend_radius(
    polylines: &[FiberPolyline],
    hardware: &FiberHardwareProfile,
) -> Vec<FiberPath> {
    polylines
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, polyline)| {
            let printable = respects_min_bend_radius(&polyline, hardware.min_bend_radius);
            FiberPath {
                id: polyline.id.max(i as u32),
                polyline,
                printable,
            }
        })
        .filter(|p| p.printable)
        .collect()
}

/// Filter helper used by unit tests and pipeline: drop illegal segments only.
pub fn filter_by_bend_radius(
    polylines: Vec<FiberPolyline>,
    min_bend_radius: f64,
) -> Vec<FiberPolyline> {
    polylines
        .into_iter()
        .filter(|p| respects_min_bend_radius(p, min_bend_radius))
        .collect()
}
