//! #10 — PSL-guided hole loop generation (stub; after #15).
//!
//! Card core idea: "PSL-guided curved layers with fiber loops around holes."
//! Layers themselves come from #42; this module only marks hole targets and
//! emits candidate loops under the caller-supplied bend radius.

use crate::layer::LayerSurface;
use crate::types::{FiberHardwareProfile, FiberPolyline, Point3};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct HoleTarget {
    pub layer_id: u32,
    pub boundary: Vec<Point3>,
}

pub fn generate_hole_loops(layers: &[LayerSurface], holes: &[HoleTarget], hardware: &FiberHardwareProfile) -> Vec<FiberPolyline> {
    let _ = (layers, holes, hardware);
    Vec::new()
}
