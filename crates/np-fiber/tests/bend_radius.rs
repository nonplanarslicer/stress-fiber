//! Bend-radius filter unit test.
use np_fiber::{filter_by_bend_radius, local_bend_radius, FiberPolyline, Point3};
fn poly(points: &[[f64; 3]]) -> FiberPolyline {
    FiberPolyline { id: 1, layer_id: 0, points: points.iter().copied().map(Point3::from_array).collect(), closed: false }
}
#[test]
fn rejects_polyline_tighter_than_given_radius() {
    let sharp = poly(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
    let r = local_bend_radius(sharp.points[0], sharp.points[1], sharp.points[2]);
    assert!((r - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9, "got r={r}");
    let kept = filter_by_bend_radius(vec![sharp], 1.0);
    assert!(kept.is_empty());
}
#[test]
fn accepts_polyline_looser_than_given_radius() {
    let gentle = poly(&[[0.0, 0.0, 0.0], [10.0, 0.1, 0.0], [20.0, 0.0, 0.0]]);
    let r = local_bend_radius(gentle.points[0], gentle.points[1], gentle.points[2]);
    assert!(r > 1.0);
    let kept = filter_by_bend_radius(vec![gentle], 1.0);
    assert_eq!(kept.len(), 1);
}
#[test]
fn short_polyline_always_passes() {
    let short = poly(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
    assert_eq!(filter_by_bend_radius(vec![short], 100.0).len(), 1);
}
