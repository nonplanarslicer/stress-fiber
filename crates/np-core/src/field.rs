//! Governing scalar field aligned with principal stress (#42 core idea).
//!
//! External continuum FEA remains out of scope. Given an ingested stress grid we
//! build a discrete scalar φ whose gradient tracks σ₁ in a least-squares sense:
//! at each node, φ is the path integral of the principal direction from the
//! grid origin along axis-aligned edges (Dijkstra-style sweep from the corner).

use serde::{Deserialize, Serialize};

use crate::fea::StressGrid;
use crate::geom::{self, Vec3};

/// Discrete governing scalar sampled on the same nodes as [`StressGrid`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoverningScalar {
    pub origin: Vec3,
    pub spacing: Vec3,
    pub dims: [usize; 3],
    /// Scalar φ per node (same layout as the stress grid).
    pub values: Vec<f64>,
    /// Analytic / finite-difference gradient ∇φ ≈ principal direction.
    pub gradients: Vec<Vec3>,
}

impl GoverningScalar {
    pub fn is_empty(&self) -> bool {
        self.values.is_empty() || self.dims[0] * self.dims[1] * self.dims[2] == 0
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

    pub fn range(&self) -> (f64, f64) {
        let mut min_v = f64::INFINITY;
        let mut max_v = f64::NEG_INFINITY;
        for &v in &self.values {
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
        if !min_v.is_finite() {
            (0.0, 0.0)
        } else {
            (min_v, max_v)
        }
    }
}

/// Build the stress-aligned governing scalar on `grid`.
///
/// Algorithm (clean-room, documented approximation of Reinforced FDM):
/// 1. Seed φ(0,0,0) = 0.
/// 2. Propagate along +i, +j, +k edges: φ_b = φ_a + σ̂₁(a)·(p_b − p_a).
///    When multiple parents exist, average the candidate values (LS-ish).
/// 3. Recompute ∇φ with central differences; blend lightly with σ̂₁ so iso
///    normals stay stress-aware even on coarse grids.
pub fn build_governing_scalar(grid: &StressGrid) -> GoverningScalar {
    if grid.is_empty() {
        return GoverningScalar {
            origin: grid.origin,
            spacing: grid.spacing,
            dims: [0, 0, 0],
            values: Vec::new(),
            gradients: Vec::new(),
        };
    }

    let n = grid.len();
    let mut values = vec![0.0_f64; n];
    let mut counts = vec![0_u32; n];
    values[0] = 0.0;
    counts[0] = 1;

    let push = |values: &mut [f64], counts: &mut [u32], idx: usize, candidate: f64| {
        if counts[idx] == 0 {
            values[idx] = candidate;
            counts[idx] = 1;
        } else {
            let c = counts[idx] as f64;
            values[idx] = (values[idx] * c + candidate) / (c + 1.0);
            counts[idx] += 1;
        }
    };

    // Sweep increasing i,j,k so every node sees at least one parent.
    for k in 0..grid.dims[2] {
        for j in 0..grid.dims[1] {
            for i in 0..grid.dims[0] {
                let idx = grid.index(i, j, k);
                if counts[idx] == 0 {
                    // Should not happen after seed + ordered sweep, but be safe.
                    values[idx] = 0.0;
                    counts[idx] = 1;
                }
                let dir = grid.directions[idx];
                let p = grid.position(i, j, k);
                if i + 1 < grid.dims[0] {
                    let q = grid.position(i + 1, j, k);
                    let cand = values[idx] + geom::dot(dir, geom::sub(q, p));
                    push(&mut values, &mut counts, grid.index(i + 1, j, k), cand);
                }
                if j + 1 < grid.dims[1] {
                    let q = grid.position(i, j + 1, k);
                    let cand = values[idx] + geom::dot(dir, geom::sub(q, p));
                    push(&mut values, &mut counts, grid.index(i, j + 1, k), cand);
                }
                if k + 1 < grid.dims[2] {
                    let q = grid.position(i, j, k + 1);
                    let cand = values[idx] + geom::dot(dir, geom::sub(q, p));
                    push(&mut values, &mut counts, grid.index(i, j, k + 1), cand);
                }
            }
        }
    }

    // Finite-difference gradients, blended with principal direction.
    let mut gradients = vec![[0.0, 0.0, 1.0]; n];
    for k in 0..grid.dims[2] {
        for j in 0..grid.dims[1] {
            for i in 0..grid.dims[0] {
                let idx = grid.index(i, j, k);
                let gx = if i == 0 {
                    (values[grid.index(i + 1, j, k)] - values[idx]) / grid.spacing[0].max(1e-12)
                } else if i + 1 == grid.dims[0] {
                    (values[idx] - values[grid.index(i - 1, j, k)]) / grid.spacing[0].max(1e-12)
                } else {
                    (values[grid.index(i + 1, j, k)] - values[grid.index(i - 1, j, k)])
                        / (2.0 * grid.spacing[0].max(1e-12))
                };
                let gy = if j == 0 {
                    (values[grid.index(i, j + 1, k)] - values[idx]) / grid.spacing[1].max(1e-12)
                } else if j + 1 == grid.dims[1] {
                    (values[idx] - values[grid.index(i, j - 1, k)]) / grid.spacing[1].max(1e-12)
                } else {
                    (values[grid.index(i, j + 1, k)] - values[grid.index(i, j - 1, k)])
                        / (2.0 * grid.spacing[1].max(1e-12))
                };
                let gz = if k == 0 {
                    (values[grid.index(i, j, k + 1)] - values[idx]) / grid.spacing[2].max(1e-12)
                } else if k + 1 == grid.dims[2] {
                    (values[idx] - values[grid.index(i, j, k - 1)]) / grid.spacing[2].max(1e-12)
                } else {
                    (values[grid.index(i, j, k + 1)] - values[grid.index(i, j, k - 1)])
                        / (2.0 * grid.spacing[2].max(1e-12))
                };
                let fd = geom::normalize([gx, gy, gz]);
                let blended = geom::normalize(geom::add(fd, grid.directions[idx]));
                gradients[idx] = blended;
            }
        }
    }

    GoverningScalar {
        origin: grid.origin,
        spacing: grid.spacing,
        dims: grid.dims,
        values,
        gradients,
    }
}
