//! #15 — matrix fill between continuous-fiber centerlines.

use crate::layer::LayerSurface;
use crate::types::{FiberHardwareProfile, FiberPath, Point3};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MatrixSegment {
    pub layer_id: u32,
    pub start: Point3,
    pub end: Point3,
}

pub fn fill_between_fibers(
    layers: &[LayerSurface],
    fibers: &[FiberPath],
    hardware: &FiberHardwareProfile,
) -> Vec<MatrixSegment> {
    let _ = (layers, fibers, hardware);
    Vec::new()
}
