//! Stress / FEA ingest for #42.
//!
//! External FEA solvers are deliberately out of scope. This module only accepts
//! already-computed principal-stress data in one of two layouts (see README).

use serde::{Deserialize, Serialize};

use crate::geom::{self, Vec3};

/// One principal-stress sample (σ₁ direction + magnitude).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrincipalStressSample {
    pub position: Vec3,
    /// Unit (or near-unit) maximum-principal direction.
    pub principal_direction: Vec3,
    pub magnitude: f64,
}

/// Stress field input accepted by the #42 pipeline.
///
/// Untagged so existing JSON `{ "samples": [...] }` keeps working as
/// [`StressIn::PointSamples`], while dense grids use explicit grid keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StressIn {
    /// Scattered FEA / probe samples of principal stress.
    PointSamples {
        samples: Vec<PrincipalStressSample>,
    },
    /// Regular 3-D grid of principal-stress samples (row-major `i + j*nx + k*nx*ny`).
    DenseGrid {
        origin: Vec3,
        spacing: Vec3,
        dims: [usize; 3],
        /// Principal magnitude per voxel/node.
        magnitudes: Vec<f64>,
        /// Principal direction per voxel/node (same layout as `magnitudes`).
        directions: Vec<Vec3>,
    },
}

impl Default for StressIn {
    fn default() -> Self {
        StressIn::PointSamples {
            samples: Vec::new(),
        }
    }
}

/// Internal regular grid used by field / iso / fill stages.
#[derive(Clone, Debug)]
pub struct StressGrid {
    pub origin: Vec3,
    pub spacing: Vec3,
    pub dims: [usize; 3],
    pub magnitudes: Vec<f64>,
    pub directions: Vec<Vec3>,
}

impl StressGrid {
    pub fn len(&self) -> usize {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        i + j * self.dims[0] + k * self.dims[0] * self.dims[1]
    }

    pub fn position(&self, i: usize, j: usize, k: usize) -> Vec3 {
        [
            self.origin[0] + i as f64 * self.spacing[0],
            self.origin[1] + j as f64 * self.spacing[1],
            self.origin[2] + k as f64 * self.spacing[2],
        ]
    }

    pub fn is_empty(&self) -> bool {
        self.dims[0] == 0 || self.dims[1] == 0 || self.dims[2] == 0 || self.magnitudes.is_empty()
    }
}

/// Normalize either ingest form into a dense [`StressGrid`].
///
/// Point samples are rasterized onto an axis-aligned bounding box via inverse
/// distance weighting (IDW). Empty input yields an empty grid.
pub fn ingest(stress: &StressIn) -> StressGrid {
    match stress {
        StressIn::DenseGrid {
            origin,
            spacing,
            dims,
            magnitudes,
            directions,
        } => {
            let expected = dims[0].saturating_mul(dims[1]).saturating_mul(dims[2]);
            if expected == 0 || magnitudes.len() < expected || directions.len() < expected {
                return StressGrid {
                    origin: *origin,
                    spacing: *spacing,
                    dims: [0, 0, 0],
                    magnitudes: Vec::new(),
                    directions: Vec::new(),
                };
            }
            StressGrid {
                origin: *origin,
                spacing: *spacing,
                dims: *dims,
                magnitudes: magnitudes[..expected].to_vec(),
                directions: directions[..expected]
                    .iter()
                    .copied()
                    .map(geom::normalize)
                    .collect(),
            }
        }
        StressIn::PointSamples { samples } => rasterize_samples(samples),
    }
}

fn rasterize_samples(samples: &[PrincipalStressSample]) -> StressGrid {
    if samples.is_empty() {
        return StressGrid {
            origin: [0.0, 0.0, 0.0],
            spacing: [1.0, 1.0, 1.0],
            dims: [0, 0, 0],
            magnitudes: Vec::new(),
            directions: Vec::new(),
        };
    }

    let mut min_b = samples[0].position;
    let mut max_b = samples[0].position;
    for s in samples.iter().skip(1) {
        for a in 0..3 {
            min_b[a] = min_b[a].min(s.position[a]);
            max_b[a] = max_b[a].max(s.position[a]);
        }
    }

    // Target ~12–16 nodes along the longest axis (keeps iso cheap for the stub).
    let extent = geom::sub(max_b, min_b);
    let longest = extent[0].max(extent[1]).max(extent[2]).max(1e-6);
    let target = 12usize;
    let cell = (longest / target as f64).max(1e-6);
    let dims = [
        ((extent[0] / cell).ceil() as usize + 1).max(2),
        ((extent[1] / cell).ceil() as usize + 1).max(2),
        ((extent[2] / cell).ceil() as usize + 1).max(2),
    ];
    let spacing = [
        if dims[0] > 1 {
            extent[0] / (dims[0] - 1) as f64
        } else {
            cell
        },
        if dims[1] > 1 {
            extent[1] / (dims[1] - 1) as f64
        } else {
            cell
        },
        if dims[2] > 1 {
            extent[2] / (dims[2] - 1) as f64
        } else {
            cell
        },
    ];

    let n = dims[0] * dims[1] * dims[2];
    let mut magnitudes = vec![0.0; n];
    let mut directions = vec![[0.0, 0.0, 1.0]; n];
    let power = 2.0_f64;

    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                let p = [
                    min_b[0] + i as f64 * spacing[0],
                    min_b[1] + j as f64 * spacing[1],
                    min_b[2] + k as f64 * spacing[2],
                ];
                let mut w_sum = 0.0;
                let mut m_acc = 0.0;
                let mut d_acc = [0.0, 0.0, 0.0];
                let mut exact: Option<&PrincipalStressSample> = None;
                for s in samples {
                    let d2 = geom::dot(geom::sub(p, s.position), geom::sub(p, s.position));
                    if d2 < 1e-18 {
                        exact = Some(s);
                        break;
                    }
                    let w = 1.0 / d2.powf(power / 2.0);
                    w_sum += w;
                    m_acc += w * s.magnitude;
                    let dir = geom::normalize(s.principal_direction);
                    d_acc = geom::add(d_acc, geom::scale(dir, w));
                }
                let idx = i + j * dims[0] + k * dims[0] * dims[1];
                if let Some(s) = exact {
                    magnitudes[idx] = s.magnitude;
                    directions[idx] = geom::normalize(s.principal_direction);
                } else if w_sum > 0.0 {
                    magnitudes[idx] = m_acc / w_sum;
                    directions[idx] = geom::normalize(geom::scale(d_acc, 1.0 / w_sum));
                }
            }
        }
    }

    StressGrid {
        origin: min_b,
        spacing,
        dims,
        magnitudes,
        directions,
    }
}
