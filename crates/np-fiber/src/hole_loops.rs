//! #10 — PSL-guided hole loop generation (stub; after #15).
//!
//! Card core idea: "PSL-guided curved layers with fiber loops around holes."
//! Layers themselves come from #42; this module only marks hole targets and
//! emits candidate loops under the caller-supplied bend radius.

use crate::layer::LayerSurface;
use crate::types::{FiberHardwareProfile, FiberPolyline, Point3};

/// Hole / cutout target on a layer (boundary samples in layer space).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct HoleTarget {
    pub layer_id: u32,
    pub boundary: Vec<Point3>,
}

/// #10: generate fiber loops around holes on Core layers.
///
/// Priority after #15. Stub returns no loops; bend limit must come from
/// `hardware.min_bend_radius`.
pub fn generate_hole_loops(
    layers: &[LayerSurface],
    holes: &[HoleTarget],
    hardware: &FiberHardwareProfile,
) -> Vec<FiberPolyline> {
    // TODO(#10): force closed / U-turn loops encircling hole boundaries within
    // hardware.min_bend_radius; pack by local stress without crossover.
    let _ = (layers, holes, hardware);
    Vec::new()
}
