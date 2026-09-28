//! External layer / TCP contract from NP Rust Core (#42).
//!
//! Do **not** implement iso-surface extraction here. Family E consumes
//! [`LayerSurface`](src/layer.rs).

use serde::{Deserialize, Serialize};

/// Locked curved-layer handoff from NP Rust Core (#42).
pub use np_core::LayerSurface;

/// Fiber-aware TCP sample for dual-extrude sync (#15) before D posing.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ToolpathPoint {
    pub position: [f64; 3],
    pub tool_direction: [f64; 3],
    pub extrusion: f64,
    pub fiber_on: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TcpPath {
    pub id: u32,
    pub points: Vec<ToolpathPoint>,
}
