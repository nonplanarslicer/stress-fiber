//! Collision / sequencing gate stub (Part F).
//!
//! Real multi-axis collision ordering pairs with #16 / #68 and is deliberately
//! external to this crate. The stub always reports clear so the TCP path can
//! still be emitted for D adapters.

use serde::{Deserialize, Serialize};

use crate::geom::{FillPolyline, LayerSurface};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CollisionReport {
    pub clear: bool,
    pub notes: Vec<String>,
}

/// Placeholder gate: accepts any layer/fill set.
///
/// TODO(#42 / Part F): wire #16/#68 collision-free ordering before machine emit.
pub fn check_collision_stub(layers: &[LayerSurface], fills: &[FillPolyline]) -> CollisionReport {
    let _ = (layers, fills);
    CollisionReport {
        clear: true,
        notes: vec![
            "collision stub: always clear; pair with #16/#68 for production sequencing".into(),
        ],
    }
}
