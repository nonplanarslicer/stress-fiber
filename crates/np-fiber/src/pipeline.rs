//! #15 end-to-end pipeline orchestration (priority path for Family E).
//!
//! Steps (card #15):
//! 1–2 stress-weighted scalar field → 3 isocurve centerlines → 4 bend-radius
//! enforcement → 5 matrix fill → dual-extrude sync.
//!
//! #10 and #52 are optional later stages and are not run here by default.

use crate::dual_extrude::{sync_dual_extrude, DualExtrudePlan};
use crate::field::{build_stress_weighted_field, FieldInput};
use crate::isocurve::extract_centerlines;
use crate::layer::LayerSurface;
use crate::matrix_fill::{fill_between_fibers, MatrixSegment};
use crate::types::{FiberHardwareProfile, FiberPath, FiberPolyline, StressField, StressSample};
use serde::{Deserialize, Serialize};

/// Full #15 pipeline input. Hardware limits are mandatory caller params.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FiberPipelineInput {
    /// Layered surfaces from NP Rust Core (#42) — external contract.
    pub layers: Vec<LayerSurface>,
    pub stress: Vec<StressSample>,
    pub hardware: FiberHardwareProfile,
    /// Iso levels in scalar units; empty → stub extracts nothing.
    pub iso_levels: Vec<f64>,
}

/// #15 pipeline result exposed to napi / TypeScript.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FiberPipelineResult {
    pub fields: Vec<StressField>,
    pub centerlines: Vec<FiberPolyline>,
    pub fibers: Vec<FiberPath>,
    pub matrix: Vec<MatrixSegment>,
    pub dual_extrude: DualExtrudePlan,
}

/// Run the #15 adaptive stress-isocurve pipeline (stubs behind stable seams).
pub fn run_stress_isocurve_pipeline(input: &FiberPipelineInput) -> FiberPipelineResult {
    let fields = build_stress_weighted_field(&FieldInput {
        layers: &input.layers,
        stress: &input.stress,
        hardware: &input.hardware,
    });
    let centerlines = extract_centerlines(&fields, &input.iso_levels);
    let fibers = crate::bend::enforce_bend_radius(&centerlines, &input.hardware);
    let matrix = fill_between_fibers(&input.layers, &fibers, &input.hardware);
    let dual_extrude = sync_dual_extrude(&fibers, &matrix);
    FiberPipelineResult {
        fields,
        centerlines,
        fibers,
        matrix,
        dual_extrude,
    }
}
