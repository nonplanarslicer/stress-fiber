//! #42 Reinforced FDM — stress-aligned curved layers (clean-room).
//!
//! Pipeline: FEA/stress ingest → governing scalar → iso [`LayerSurface`]s →
//! clamp → stress-aware fill + tool vectors → [`TcpPath`] stub for D.
//!
//! External FEA and Part-F collision MILP solvers stay behind documented TODOs.
//! Public layer / TCP types are re-exported at the crate root for Fiber handoff.

pub mod clamp;
pub mod collision;
pub mod fea;
pub mod field;
pub mod fill;
pub mod geom;
pub mod isosurface;
pub mod pipeline;
pub mod tcp;

// Locked public contract + Fiber-facing re-exports.
pub use geom::{FillPolyline, LayerSurface, TcpPath, ToolpathPoint, Vec3};
pub use fea::{PrincipalStressSample, StressGrid, StressIn};
pub use field::GoverningScalar;
pub use pipeline::{CoreInput, CoreOutput, GoverningScalarSummary};

/// Run the #42 pipeline (`pipeline::run`).
pub fn run(input: &CoreInput) -> CoreOutput {
    pipeline::run(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_empty_output() {
        let out = run(&CoreInput::default());
        assert!(out.layers.is_empty());
        assert!(out.tcp.points.is_empty());
    }

    #[test]
    fn point_samples_produce_layers_and_tcp() {
        let mut samples = Vec::new();
        for z in 0..5 {
            for y in 0..5 {
                for x in 0..5 {
                    samples.push(PrincipalStressSample {
                        position: [x as f64 * 0.5, y as f64 * 0.5, z as f64 * 0.5],
                        principal_direction: [0.0, 0.0, 1.0],
                        magnitude: 1.0 + z as f64,
                    });
                }
            }
        }
        let input = CoreInput {
            stress: StressIn::PointSamples { samples },
            layer_height: 0.5,
            fill_spacing: 0.5,
            bead_width: 0.4,
            thickness: 0.2,
            max_overhang_deg: 45.0,
        };
        let out = run(&input);
        assert!(
            !out.layers.is_empty(),
            "expected at least one LayerSurface"
        );
        assert_eq!(out.layers[0].points.len(), out.layers[0].normals.len());
        assert!(out.field.is_some());
    }

    #[test]
    fn dense_grid_ingest_runs() {
        let dims = [4usize, 4, 4];
        let n = dims[0] * dims[1] * dims[2];
        let magnitudes = vec![1.0; n];
        let directions = vec![[0.0, 0.0, 1.0]; n];
        let input = CoreInput {
            stress: StressIn::DenseGrid {
                origin: [0.0, 0.0, 0.0],
                spacing: [1.0, 1.0, 1.0],
                dims,
                magnitudes,
                directions,
            },
            layer_height: 1.0,
            fill_spacing: 1.0,
            bead_width: 0.4,
            thickness: 0.2,
            max_overhang_deg: 60.0,
        };
        let out = run(&input);
        assert!(out.field.is_some());
        assert!(!out.layers.is_empty());
    }
}
