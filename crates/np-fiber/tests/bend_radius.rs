//! Bend-radius filter unit test.
//!
//! Radius comes from test input (synthetic hardware profile), never a paper
//! constant — see card #15 notes on missing corpus constants.

use np_fiber::{filter_by_bend_radius, local_bend_radius, FiberPolyline, Point3};

fn poly(points: &[[f64; 3]]) -> FiberPolyline {
    FiberPolyline {
        id: 1,
        layer_id: 0,
        points: points.iter().copied().map(Point3::from_array).collect(),
        closed: false,
    }
}

#[test]
fn rejects_polyline_tighter_than_given_radius() {
    // Sharp right-angle: circumradius of (0,0)-(1,0)-(1,1) is 0.5*sqrt(2) ≈ 0.707
    let sharp = poly(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
    let r = local_bend_radius(sharp.points[0], sharp.points[1], sharp.points[2]);
    assert!((r - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9, "got r={r}");

    // Hardware profile for this test only — not a paper metric.
    let min_bend_radius = 1.0;
    let kept = filter_by_bend_radius(vec![sharp], min_bend_radius);
    assert!(kept.is_empty(), "curve with r≈0.707 must fail min_bend_radius=1.0");
}

#[test]
fn accepts_polyline_looser_than_given_radius() {
    // Nearly collinear: very large bend radius.
    let gentle = poly(&[
        [0.0, 0.0, 0.0],
        [10.0, 0.1, 0.0],
        [20.0, 0.0, 0.0],
    ]);
    let min_bend_radius = 1.0; // test input, not a paper constant
    let r = local_bend_radius(gentle.points[0], gentle.points[1], gentle.points[2]);
    assert!(r > min_bend_radius, "expected gentle curve r={r} > {min_bend_radius}");
    let kept = filter_by_bend_radius(vec![gentle], min_bend_radius);
    assert_eq!(kept.len(), 1);
}

#[test]
fn short_polyline_always_passes() {
    let short = poly(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
    let kept = filter_by_bend_radius(vec![short], 100.0);
    assert_eq!(kept.len(), 1);
}
