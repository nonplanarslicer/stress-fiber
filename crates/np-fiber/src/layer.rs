//! External layer / TCP contract from NP Rust Core (#42).
//!
//! Do **not** implement iso-surface extraction here. Family E consumes
//! [`LayerSurface`] produced by Core.

use serde::{Deserialize, Serialize};

/// Locked curved-layer handoff from NP Rust Core (#42).
pub use np_core::LayerSurface;

/// Fiber-aware TCP sample for dual-extrude sync (#15) before D posing.
///
/// Distinct from `np_core::ToolpathPoint` (Part-F xyz/tool_vector/h/w/feature):
/// this record carries `fiber_on` for matrix+fiber feed sync. Map to Core/D
/// types at the kinematics adapter boundary.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ToolpathPoint {
    pub position: [f64; 3],
    pub tool_direction: [f64; 3],
    pub extrusion: f64,
    /// Continuous-fiber feed engaged at this sample.
    pub fiber_on: bool,
}

/// Ordered TCP path (matrix and/or fiber) handed toward family D.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TcpPath {
    pub id: u32,
    pub points: Vec<ToolpathPoint>,
}
