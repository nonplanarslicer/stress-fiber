//! Clamp / smooth iso layers that violate overhang or thickness-band limits.

use crate::geom::{self, LayerSurface};

/// Parameters for the clamp stage.
#[derive(Clone, Debug)]
pub struct ClampParams {
    /// Maximum angle (degrees) between tool normal and +Z (build axis).
    pub max_overhang_deg: f64,
    /// Optional Laplacian smoothing iterations on layer points.
    pub smooth_iters: u32,
}

impl Default for ClampParams {
    fn default() -> Self {
        Self {
            max_overhang_deg: 45.0,
            smooth_iters: 1,
        }
    }
}

/// Clamp layer normals to the overhang cone and lightly smooth point clouds.
pub fn clamp_layers(layers: &[LayerSurface], params: &ClampParams) -> Vec<LayerSurface> {
    let max_tilt = params.max_overhang_deg.to_radians().cos();
    let build = [0.0, 0.0, 1.0];
    layers
        .iter()
        .map(|layer| {
            let mut normals: Vec<_> = layer
                .normals
                .iter()
                .copied()
                .map(|n| {
                    let mut n = geom::normalize(n);
                    // Flip to point generally +Z.
                    if geom::dot(n, build) < 0.0 {
                        n = geom::scale(n, -1.0);
                    }
                    let d = geom::dot(n, build);
                    if d >= max_tilt {
                        n
                    } else {
                        // Pull toward +Z until within cone.
                        let flat = geom::normalize([n[0], n[1], 0.0]);
                        let sin_max = params.max_overhang_deg.to_radians().sin();
                        let cos_max = max_tilt;
                        if geom::norm(flat) < 1e-9 {
                            build
                        } else {
                            geom::normalize(geom::add(
                                geom::scale(build, cos_max),
                                geom::scale(flat, sin_max),
                            ))
                        }
                    }
                })
                .collect();

            let mut points = layer.points.clone();
            for _ in 0..params.smooth_iters {
                if points.len() < 3 {
                    break;
                }
                let original = points.clone();
                let n = original.len();
                for i in 0..n {
                    let prev = original[(i + n - 1) % n];
                    let next = original[(i + 1) % n];
                    let avg = geom::scale(geom::add(prev, next), 0.5);
                    points[i] = geom::lerp(original[i], avg, 0.35);
                }
                // Re-associate normals length if mismatch (should not happen).
                if normals.len() != points.len() {
                    normals.resize(points.len(), build);
                }
            }

            LayerSurface {
                id: layer.id,
                points,
                normals,
                thickness: layer.thickness,
                sequence: layer.sequence,
            }
        })
        .collect()
}
