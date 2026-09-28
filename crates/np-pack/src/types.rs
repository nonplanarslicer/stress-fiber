//! Fiber-loop packing types (#27 / #22).
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopCandidate { pub id: String, pub layer_id: u32, pub points: Vec<[f64; 2]>, pub weight: f64, pub length: f64, #[serde(default, skip_serializing_if = "Option::is_none")] pub min_bend_radius: Option<f64> }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackConstraints { pub bend_radius: f64, pub min_spacing: f64, pub fiber_length_budget: f64, pub cut_cost: f64, pub restart_cost: f64, pub allow_cuts: bool }
impl Default for PackConstraints { fn default() -> Self { Self { bend_radius: 3.0, min_spacing: 1.0, fiber_length_budget: 1_000.0, cut_cost: 10.0, restart_cost: 5.0, allow_cuts: true } } }
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackStrategy { #[default] Milp, Deepq, Greedy }
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackInput { #[serde(default)] pub candidates: Vec<LoopCandidate>, #[serde(default)] pub constraints: PackConstraints, #[serde(default)] pub strategy: PackStrategy, #[serde(default)] pub conflict_pairs: Vec<(String, String)>, #[serde(default, skip_serializing_if = "Option::is_none")] pub weights_path: Option<String> }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CutEvent { pub after_tour_index: usize, pub kind: String }
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackOutput { pub selected_loop_ids: Vec<String>, pub tour_order: Vec<String>, pub cuts: Vec<CutEvent>, pub strategy: String, pub solver_status: String, pub unplaced: Vec<String>, #[serde(default, skip_serializing_if = "Vec::is_empty")] pub notes: Vec<String>, #[serde(default, skip_serializing_if = "Vec::is_empty")] pub paths: Vec<Vec<[f64; 2]>> }
impl PackOutput { pub fn stub_status(status: &str, strategy: &str) -> Self { Self { strategy: strategy.into(), solver_status: status.into(), ..Default::default() } } }
