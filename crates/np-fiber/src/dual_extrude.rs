//! #15 — dual-extrude sync hooks (matrix + continuous fiber).

use crate::layer::{TcpPath, ToolpathPoint};
use crate::matrix_fill::MatrixSegment;
use crate::types::FiberPath;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DualExtrudePlan { pub fiber_paths: Vec<TcpPath>, pub matrix_paths: Vec<TcpPath> }

pub fn sync_dual_extrude(fibers: &[FiberPath], matrix: &[MatrixSegment]) -> DualExtrudePlan {
    let fiber_paths = fibers.iter().filter(|f| f.printable).map(|f| {
        let mut points = Vec::with_capacity(f.polyline.points.len());
        for (i, p) in f.polyline.points.iter().enumerate() {
            let extrusion = if i == 0 { 0.0 } else { f.polyline.points[i - 1].distance(*p) };
            points.push(ToolpathPoint { position: p.to_array(), tool_direction: [0.0, 0.0, 1.0], extrusion, fiber_on: true });
        }
        TcpPath { id: f.id, points }
    }).collect();
    let mut layer_ids: Vec<u32> = matrix.iter().map(|s| s.layer_id).collect(); layer_ids.sort_unstable(); layer_ids.dedup();
    let matrix_paths = layer_ids.into_iter().enumerate().map(|(idx, layer_id)| {
        let mut points = Vec::new();
        for seg in matrix.iter().filter(|s| s.layer_id == layer_id) {
            let len = seg.start.distance(seg.end);
            points.push(ToolpathPoint { position: seg.start.to_array(), tool_direction: [0.0, 0.0, 1.0], extrusion: 0.0, fiber_on: false });
            points.push(ToolpathPoint { position: seg.end.to_array(), tool_direction: [0.0, 0.0, 1.0], extrusion: len, fiber_on: false });
        }
        TcpPath { id: idx as u32, points }
    }).filter(|p| !p.points.is_empty()).collect();
    DualExtrudePlan { fiber_paths, matrix_paths }
}
