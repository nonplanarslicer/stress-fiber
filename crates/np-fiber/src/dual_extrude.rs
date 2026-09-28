//! #15 — dual-extrude sync hooks (matrix + continuous fiber).
//!
//! Card stage: "sync dual extrusion (matrix + continuous fiber)."

use crate::layer::{TcpPath, ToolpathPoint};
use crate::matrix_fill::MatrixSegment;
use crate::types::FiberPath;

/// Sync plan for matrix + fiber feeds (cut/restart left to hardware policy).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DualExtrudePlan {
    pub fiber_paths: Vec<TcpPath>,
    pub matrix_paths: Vec<TcpPath>,
}

/// #15 stage 5b: sync matrix fill with continuous-fiber feed into TCP paths.
///
/// Stub builds empty TcpPath lists with stable shape for napi / D handoff.
pub fn sync_dual_extrude(fibers: &[FiberPath], matrix: &[MatrixSegment]) -> DualExtrudePlan {
    // TODO(#15): interleave fiber-on/off with matrix extrusion, insert
    // cut/restart only where the hardware profile requires it, emit ToolpathPoint
    // streams for family D.
    let fiber_paths = fibers
        .iter()
        .map(|f| TcpPath {
            id: f.id,
            points: f
                .polyline
                .points
                .iter()
                .map(|p| ToolpathPoint {
                    position: p.to_array(),
                    tool_direction: [0.0, 0.0, 1.0],
                    extrusion: 0.0,
                    fiber_on: true,
                })
                .collect(),
        })
        .collect();
    let matrix_paths = if matrix.is_empty() {
        Vec::new()
    } else {
        // Placeholder single path so the ABI stays occupied once fill exists.
        vec![TcpPath {
            id: 0,
            points: matrix
                .iter()
                .flat_map(|s| {
                    [
                        ToolpathPoint {
                            position: s.start.to_array(),
                            tool_direction: [0.0, 0.0, 1.0],
                            extrusion: 0.0,
                            fiber_on: false,
                        },
                        ToolpathPoint {
                            position: s.end.to_array(),
                            tool_direction: [0.0, 0.0, 1.0],
                            extrusion: 0.0,
                            fiber_on: false,
                        },
                    ]
                })
                .collect(),
        }]
    };
    DualExtrudePlan {
        fiber_paths,
        matrix_paths,
    }
}
