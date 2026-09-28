//! TCP path emission from fill polylines (D / kinematics handoff stub).

use crate::geom::{FillPolyline, LayerSurface, TcpPath, ToolpathPoint};

/// Convert fill polylines into a flat [`TcpPath`] with per-point records.
pub fn emit_tcp(
    fills: &[FillPolyline],
    layers: &[LayerSurface],
    bead_width: f64,
) -> TcpPath {
    let mut points = Vec::new();
    let mut seq = 0u32;
    for fill in fills {
        let thickness = layers
            .iter()
            .find(|l| l.id == fill.layer_id)
            .map(|l| l.thickness)
            .unwrap_or(0.2);
        for (i, xyz) in fill.points.iter().enumerate() {
            let tool_vector = fill
                .tool_vectors
                .get(i)
                .copied()
                .unwrap_or([0.0, 0.0, 1.0]);
            points.push(ToolpathPoint {
                xyz: *xyz,
                tool_vector,
                h: thickness,
                w: bead_width,
                feature: format!("layer:{}:fill", fill.layer_id),
                sequence: seq,
            });
            seq = seq.saturating_add(1);
        }
    }
    TcpPath { points }
}
