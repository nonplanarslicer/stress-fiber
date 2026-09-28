//! Fiber-loop packing types (#27 / #22).
//!
//! JSON ABI used by `np-native::pack_run`. No invented paper metrics.

use serde::{Deserialize, Serialize};

/// A candidate fiber loop on one layer (from #15 / #10 / #52 or templates).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopCandidate {
    pub id: String,
    pub layer_id: u32,
    /// Closed or open polyline in layer XY (Z ignored if present as third coord).
    pub points: Vec<[f64; 2]>,
    /// Stress / coverage weight (caller-supplied objective proxy).
    pub weight: f64,
    /// Path length in the same units as [`PackConstraints::fiber_length_budget`].
    pub length: f64,
    /// Optional precomputed minimum bend radius on this loop.
    /// When absent, packers assume the upstream generator already filtered geometry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_bend_radius: Option<f64>,
}

/// Hardware / process packing constraints (family E).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackConstraints {
    pub bend_radius: f64,
    pub min_spacing: f64,
    pub fiber_length_budget: f64,
    pub cut_cost: f64,
    pub restart_cost: f64,
    pub allow_cuts: bool,
}

impl Default for PackConstraints {
    fn default() -> Self {
        Self {
            bend_radius: 3.0,
            min_spacing: 1.0,
            fiber_length_budget: 1_000.0,
            cut_cost: 10.0,
            restart_cost: 5.0,
            allow_cuts: true,
        }
    }
}

/// Packing / sequencing strategy requested by the caller.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackStrategy {
    /// #27 layer-wise MILP packing (currently greedy-by-weight stand-in).
    #[default]
    Milp,
    /// #22 Deep-Q next-node policy (falls back to greedy when weights missing).
    Deepq,
    /// Explicit greedy heuristic (no MILP / no policy claim).
    Greedy,
}

/// Input JSON for `pack_run` / [`crate::run`].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackInput {
    #[serde(default)]
    pub candidates: Vec<LoopCandidate>,
    #[serde(default)]
    pub constraints: PackConstraints,
    #[serde(default)]
    pub strategy: PackStrategy,
    /// Explicit conflict pairs by loop id (geometric crossover / hard exclusion).
    /// Spacing conflicts are also inferred from [`PackConstraints::min_spacing`].
    #[serde(default)]
    pub conflict_pairs: Vec<(String, String)>,
    /// Optional path to Deep-Q weights (ignored when missing / unloadable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weights_path: Option<String>,
}

/// Cut / restart event between two consecutive tour steps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CutEvent {
    /// Index into [`PackOutput::tour_order`] after which the cut occurs.
    pub after_tour_index: usize,
    pub kind: String,
}

/// Output JSON for `pack_run` / [`crate::run`].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackOutput {
    pub selected_loop_ids: Vec<String>,
    pub tour_order: Vec<String>,
    pub cuts: Vec<CutEvent>,
    pub strategy: String,
    pub solver_status: String,
    pub unplaced: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Selected loop polylines for UI preview (mirrors candidates that were kept).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<Vec<[f64; 2]>>,
}

impl PackOutput {
    pub fn stub_status(status: &str, strategy: &str) -> Self {
        Self {
            strategy: strategy.into(),
            solver_status: status.into(),
            ..Default::default()
        }
    }
}
