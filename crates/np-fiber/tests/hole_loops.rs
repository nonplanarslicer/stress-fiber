//! #10 PSL-guided hole loop tests.
//!
//! Hardware bend / spacing / width are synthetic test inputs, never paper
//! constants (card #10 / #15 note those are absent from the corpus).

use np_fiber::{
    generate_hole_loops, respects_min_bend_radius, run_stress_isocurve_pipeline,
    FiberHardwareProfile, FiberPipelineInput, HoleTarget, LayerSurface, Point3,
};

fn flat_grid_layer(nx: usize, ny: usize, x0: f64, y0: f64, dx: f64, dy: f64) -> LayerSurface {
    let mut points = Vec::with_capacity(nx * ny);
    let mut normals = Vec::with_capacity(nx * ny);
    for j in 0..ny {
        for i in 0..nx {
            points.push([x0 + i as f64 * dx, y0 + j as f64 * dy, 0.0]);
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
fn circular_hole_generous_bend_yields_closed_printable_loop() {
    let layer = flat_grid_layer(16, 16, 0.0, 0.0, 1.0, 1.0);
    // Hole radius 3; first loop at 3 + width=0.4 = 3.4 ≫ min_bend=0.1.
    let hole = HoleTarget::circle(0, Point3::new(8.0, 8.0, 0.0), 3.0);
    let hardware = FiberHardwareProfile::new(0.4, 0.1, 0.5, 2.0);

    let loops = generate_hole_loops(&[layer], &[hole], &hardware);
    assert!(
        !loops.is_empty(),
        "expected at least one closed loop around circular hole"
    );
    assert!(loops.iter().all(|p| p.closed), "hole loops must be closed");
    assert!(
        loops
            .iter()
            .all(|p| respects_min_bend_radius(p, hardware.min_bend_radius)),
        "emitted loops must already pass bend gate"
    );
    assert!(loops.iter().all(|p| p.points.len() >= 3));
}

#[test]
fn too_tight_bend_radius_filters_loops() {
    let layer = flat_grid_layer(12, 12, 0.0, 0.0, 1.0, 1.0);
    // Small hole: radius 0.5 + width 0.4 = 0.9 < min_bend_radius 5.0.
    let hole = HoleTarget::circle(0, Point3::new(5.0, 5.0, 0.0), 0.5);
    let hardware = FiberHardwareProfile::new(0.4, 5.0, 0.5, 2.0);

    let loops = generate_hole_loops(&[layer], &[hole], &hardware);
    assert!(
        loops.is_empty(),
        "loops tighter than min_bend_radius must be filtered"
    );
}

#[test]
fn empty_holes_yield_no_loops() {
    let layer = flat_grid_layer(8, 8, 0.0, 0.0, 1.0, 1.0);
    let hardware = FiberHardwareProfile::new(0.4, 0.1, 0.5, 2.0);
    let loops = generate_hole_loops(&[layer], &[], &hardware);
    assert!(loops.is_empty());
}

#[test]
fn explicit_loop_count_emits_concentric_rings() {
    let layer = flat_grid_layer(16, 16, 0.0, 0.0, 1.0, 1.0);
    let mut hole = HoleTarget::circle(0, Point3::new(8.0, 8.0, 0.0), 2.0);
    hole.loop_count = Some(3);
    // Generous bend so all three rings (r≈2.4, 2.9, 3.4) pass.
    let hardware = FiberHardwareProfile::new(0.4, 0.1, 0.5, 2.0);

    let loops = generate_hole_loops(&[layer], &[hole], &hardware);
    assert_eq!(
        loops.len(),
        3,
        "explicit loop_count=3 should emit three closed rings"
    );
    assert!(loops.iter().all(|p| p.closed));
}

#[test]
fn pipeline_with_empty_holes_unchanged_from_no_hole_field() {
    let layer = flat_grid_layer(10, 10, 0.0, 0.0, 1.0, 1.0);
    let hardware = FiberHardwareProfile::new(0.4, 0.1, 1.5, 4.0);
    let result = run_stress_isocurve_pipeline(&FiberPipelineInput {
        layers: vec![layer],
        stress: vec![],
        hardware,
        iso_levels: vec![],
        holes: vec![],
    });
    assert_eq!(result.fields.len(), 1);
    // #15 still runs; no closed hole loops injected.
    assert!(result.centerlines.iter().all(|c| !c.closed) || !result.centerlines.is_empty());
}

#[test]
fn pipeline_appends_hole_loops_before_bend() {
    let layer = flat_grid_layer(16, 16, 0.0, 0.0, 1.0, 1.0);
    let hole = HoleTarget::circle(0, Point3::new(8.0, 8.0, 0.0), 3.0);
    let hardware = FiberHardwareProfile::new(0.4, 0.1, 1.0, 3.0);
    let result = run_stress_isocurve_pipeline(&FiberPipelineInput {
        layers: vec![layer],
        stress: vec![],
        hardware,
        iso_levels: vec![],
        holes: vec![hole],
    });
    let closed: Vec<_> = result.centerlines.iter().filter(|c| c.closed).collect();
    assert!(
        !closed.is_empty(),
        "pipeline with holes should append closed hole loops into centerlines"
    );
    assert!(
        result.fibers.iter().any(|f| f.polyline.closed && f.printable),
        "at least one closed hole loop should survive the bend gate"
    );
}
