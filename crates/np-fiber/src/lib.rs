//! Family E continuous-fiber crate for nonplanarslicer.
//!
//! Priority: **#15** stress-weighted isocurves → bend gate → matrix fill →
//! dual-extrude; then **#10** hole loops; then **#52** 2-RoSy dense packing.
//!
//! Layer iso-surfaces come from NP Rust Core (#42) via [`layer::LayerSurface`].
//! This crate never reimplements iso-layer extraction.
//!
//! Research cards: `papers/cards/015-field-based-toolpaths-cfrtpc.md`,
//! `010-spatial-printing-continuous-fiber.md`,
//! `052-high-density-spatial-fiber-toolpaths.md`.

pub mod bend;
pub mod dual_extrude;
pub mod field;
pub mod hole_loops;
pub mod isocurve;
pub mod layer;
pub mod matrix_fill;
pub mod pipeline;
pub mod rosy;
pub mod types;

pub use bend::{
    enforce_bend_radius, filter_by_bend_radius, local_bend_radius, respects_min_bend_radius,
};
pub use dual_extrude::{sync_dual_extrude, DualExtrudePlan};
pub use field::{build_stress_weighted_field, FieldInput};
pub use hole_loops::{generate_hole_loops, HoleTarget};
pub use isocurve::extract_centerlines;
pub use layer::{LayerSurface, TcpPath, ToolpathPoint};
pub use matrix_fill::{fill_between_fibers, MatrixSegment};
pub use pipeline::{run_stress_isocurve_pipeline, FiberPipelineInput, FiberPipelineResult};
pub use rosy::{
    build_periodic_scalar, build_rosy_field, extract_dense_fibers, RoSyField, RoSySample,
};
pub use types::{
    FiberHardwareProfile, FiberPath, FiberPolyline, Point3, ScalarField, ScalarFieldSample,
    StressField, StressSample,
};

use serde::{Deserialize, Serialize};

/// Backward-compatible JSON entry used by `fiber_run` in np-native.
///
/// Prefer [`FiberPipelineInput`] / [`run_stress_isocurve_pipeline`] for #15.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FiberInput {
    #[serde(default)]
    pub layers: Vec<LayerSurface>,
    #[serde(default)]
    pub stress: Vec<StressSample>,
    pub hardware: FiberHardwareProfile,
    #[serde(default)]
    pub iso_levels: Vec<f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FiberOutput {
    pub pipeline: FiberPipelineResult,
}

/// JSON-friendly runner for the native bridge.
pub fn run(input: &FiberInput) -> FiberOutput {
    let pipeline = run_stress_isocurve_pipeline(&FiberPipelineInput {
        layers: input.layers.clone(),
        stress: input.stress.clone(),
        hardware: input.hardware.clone(),
        iso_levels: input.iso_levels.clone(),
    });
    FiberOutput { pipeline }
}
