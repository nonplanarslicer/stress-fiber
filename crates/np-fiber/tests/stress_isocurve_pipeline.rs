//! #15 stress → isocurve → bend → matrix pipeline integration tests.
//!
//! Hardware bend / spacing values are synthetic test inputs, never paper
//! constants (card #15 notes those are absent from the corpus).

use np_fiber::{
    enforce_bend_radius, run_stress_isocurve_pipeline, FiberHardwareProfile, FiberPipelineInput,
    FiberPolyline, LayerSurface, Point3, StressSample,
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

fn uniform_stress_along_x(layer: &LayerSurface, magnitude: f64) -> Vec<StressSample> {
    layer
        .points
        .iter()
        .step_by(2)
        .map(|p| StressSample {
            position: Point3::from_array(*p),
            principal_direction: Point3::new(1.0, 0.0, 0.0),
            magnitude,
        })
        .collect()
}

#[test]
fn happy_path_uniform_stress_yields_fields_and_printable_fibers() {
    // Flat rectangle on z=0; principal along +X → φ integrates along Y →
    // isocurves run roughly parallel to X (orthogonal to ∇φ).
    let layer = flat_grid_layer(12, 12, 0.0, 0.0, 1.0, 1.0);
    let stress = uniform_stress_along_x(&layer, 1.0);
    // Generous bend radius so straight-ish isocurves pass the gate.
    let hardware = FiberHardwareProfile::new(0.4, 0.1, 1.0, 3.0);

    let result = run_stress_isocurve_pipeline(&FiberPipelineInput {
        layers: vec![layer],
        stress,
        hardware,
        iso_levels: vec![], // auto N=8,
        holes: vec![],
    });

    assert!(
        !result.fields.is_empty() && !result.fields[0].samples.is_empty(),
        "expected non-empty stress field"
    );
    assert!(
        !result.centerlines.is_empty(),
        "expected non-empty centerlines from isocurve march"
    );
    assert!(
        !result.fibers.is_empty(),
        "expected printable fibers with generous min_bend_radius"
    );
    assert!(
        result.fibers.iter().all(|f| f.printable),
        "enforce_bend_radius only emits printable=true"
    );
    // Dual-extrude: fiber_on true on fiber paths, false on matrix.
    assert!(!result.dual_extrude.fiber_paths.is_empty());
    for path in &result.dual_extrude.fiber_paths {
        assert!(path.points.iter().all(|p| p.fiber_on));
    }
    for path in &result.dual_extrude.matrix_paths {
        assert!(path.points.iter().all(|p| !p.fiber_on));
    }
}

#[test]
fn high_bend_radius_rejects_tight_synthetic_loop() {
    // Sharp right-angle polyline (circumradius ≈ 0.707) must fail min_bend=5.
    let tight = FiberPolyline {
        id: 99,
        layer_id: 0,
        points: vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ],
        closed: false,
    };
    let hardware = FiberHardwareProfile::new(0.4, 5.0, 1.0, 3.0);
    let kept = enforce_bend_radius(&[tight], &hardware);
    assert!(
        kept.is_empty(),
        "tight corner must be filtered when min_bend_radius=5"
    );
}

#[test]
fn empty_stress_still_builds_field_and_centerlines() {
    let layer = flat_grid_layer(10, 10, 0.0, 0.0, 1.0, 1.0);
    let hardware = FiberHardwareProfile::new(0.4, 0.1, 1.5, 4.0);

    let result = run_stress_isocurve_pipeline(&FiberPipelineInput {
        layers: vec![layer],
        stress: vec![],
        hardware,
        iso_levels: vec![], // auto levels over constant-magnitude φ,
        holes: vec![],
    });

    assert_eq!(result.fields.len(), 1);
    assert!(
        !result.fields[0].samples.is_empty(),
        "empty stress must still produce a constant-magnitude field"
    );
    let vals: Vec<f64> = result.fields[0].samples.iter().map(|s| s.value).collect();
    let vmin = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let vmax = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        vmax > vmin + 1e-9,
        "φ must vary spatially so isocurves exist even without FEA stress"
    );
    assert!(
        !result.centerlines.is_empty(),
        "auto iso levels on constant-stress field should yield centerlines"
    );
}

#[test]
fn explicit_iso_levels_on_empty_stress() {
    let layer = flat_grid_layer(8, 8, 0.0, 0.0, 1.0, 1.0);
    let hardware = FiberHardwareProfile::new(0.4, 0.05, 1.0, 2.0);
    // First build once to discover φ range, then pin levels.
    let probe = run_stress_isocurve_pipeline(&FiberPipelineInput {
        layers: vec![layer.clone()],
        stress: vec![],
        hardware: hardware.clone(),
        iso_levels: vec![],
        holes: vec![],
    });
    let vals: Vec<f64> = probe.fields[0].samples.iter().map(|s| s.value).collect();
    let vmin = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let vmax = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mid = 0.5 * (vmin + vmax);

    let result = run_stress_isocurve_pipeline(&FiberPipelineInput {
        layers: vec![layer],
        stress: vec![],
        hardware,
        iso_levels: vec![mid],
        holes: vec![],
    });
    assert!(
        !result.centerlines.is_empty(),
        "explicit mid iso level should extract at least one centerline"
    );
}
