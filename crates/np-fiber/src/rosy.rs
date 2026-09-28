//! #52 — 2-RoSy + periodic scalar dense packing (stub; after #15 and #10).
//!
//! Card core idea: "2-RoSy field plus periodic scalar for evenly spaced fibers."

use crate::layer::LayerSurface;
use crate::types::{FiberHardwareProfile, FiberPolyline, Point3, StressField};

/// Undirected line-field sample (2-RoSy has no preferred arrow).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoSySample {
    pub position: Point3,
    /// Line direction (θ ~ θ+π identified).
    pub line_direction: Point3,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoSyField {
    pub layer_id: u32,
    pub samples: Vec<RoSySample>,
}

/// #52 stage: optimize / accept a 2-RoSy orientation on Core layers.
pub fn build_rosy_field(layers: &[LayerSurface], seed_dirs: &[RoSySample]) -> Vec<RoSyField> {
    // TODO(#52): optimize 2-RoSy on printable volume / curved layers.
    let _ = (layers, seed_dirs);
    Vec::new()
}

/// #52 stage: periodic scalar whose gradient is orthogonal to RoSy direction.
pub fn build_periodic_scalar(rosy: &[RoSyField], spacing: f64) -> Vec<StressField> {
    // TODO(#52): build periodic scalar for even fiber slots; `spacing` from
    // FiberHardwareProfile, not a paper constant.
    let _ = (rosy, spacing);
    Vec::new()
}

/// #52: extract dense evenly spaced spatial fiber centerlines.
pub fn extract_dense_fibers(
    fields: &[StressField],
    hardware: &FiberHardwareProfile,
) -> Vec<FiberPolyline> {
    // TODO(#52): extract dense isocurves; enforce bend-radius + collision with
    // prior fibers using hardware.min_bend_radius / min_spacing.
    let _ = (fields, hardware);
    Vec::new()
}
