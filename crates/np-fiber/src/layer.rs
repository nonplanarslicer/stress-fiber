//! External layer / TCP contract from NP Rust Core (#42).

use serde::{Deserialize, Serialize};
pub use np_core::LayerSurface;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ToolpathPoint { pub position: [f64; 3], pub tool_direction: [f64; 3], pub extrusion: f64, pub fiber_on: bool }
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TcpPath { pub id: u32, pub points: Vec<ToolpathPoint> }
