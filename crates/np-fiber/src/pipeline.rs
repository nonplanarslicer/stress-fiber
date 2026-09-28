//! #15 end-to-end pipeline orchestration (priority path for Family E).
//!
//! Steps (card #15):
//! 1–2 stress-weighted scalar field → 3 isocurve centerlines → 4 bend-radius
//! enforcement → 5 matrix fill → dual-extrude sync.
//!
//! Optional #10: when [`FiberPipelineInput::holes`] is non-empty, closed hole
//! loops are appended to centerlines before the bend gate. Empty holes leave
//! the #15 path unchanged. #52 remains a later stub stage.

use crate::dual_extrude::{sync_dual_extrude, DualExtrudePlan};
use crate::field::{build_stress_weighted_field, FieldInput};
use crate::hole_loops::{generate_hole_loops, HoleTarget};
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
    /// Iso levels in scalar units. Empty → [`extract_centerlines`] auto-picks
    /// [`crate::isocurve::DEFAULT_ISO_LEVEL_COUNT`] evenly spaced levels over
    /// the observed φ range (implementation default, not a paper constant).
    pub iso_levels: Vec<f64>,
    /// Optional #10 hole targets. Empty → #15-only path (no hole loops).
    #[serde(default)]
    pub holes: Vec<HoleTarget>,
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

/// Run the #15 adaptive stress-isocurve pipeline.
///
/// When `input.holes` is non-empty, #10 [`generate_hole_loops`] results are
/// appended to centerlines before the bend gate.
pub fn run_stress_isocurve_pipeline(input: &FiberPipelineInput) -> FiberPipelineResult {
    let fields = build_stress_weighted_field(&FieldInput {
        layers: &input.layers,
        stress: &input.stress,
        hardware: &input.hardware,
    });
    let mut centerlines = extract_centerlines(&fields, &input.iso_levels);
    if !input.holes.is_empty() {
        let hole_loops = generate_hole_loops(&input.layers, &input.holes, &input.hardware);
        centerlines.extend(hole_loops);
    }
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
