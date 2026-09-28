//! Sequence selected loops into a deposition tour.

use crate::deepq::{plan_tour, DeepQPolicy, GreedyHeuristicPolicy, NextNodePolicy};
use crate::milp::Selection;
use crate::types::{CutEvent, LoopCandidate, PackConstraints, PackStrategy};

pub struct TourOutcome {
    pub order_indices: Vec<usize>,
    pub cuts: Vec<CutEvent>,
    pub travel_cost: f64,
    pub solver_status: String,
    pub notes: Vec<String>,
    pub policy_name: String,
}

pub fn sequence_selection(
    candidates: &[LoopCandidate],
    selection: &Selection,
    constraints: &PackConstraints,
    strategy: &PackStrategy,
    weights_path: Option<&str>,
) -> TourOutcome {
    let selected = &selection.selected_indices;
    let (policy_name, order, cuts, travel, status, mut notes): (
        String,
        Vec<usize>,
        Vec<CutEvent>,
        f64,
        String,
        Vec<String>,
    ) = match strategy {
        PackStrategy::Deepq => {
            let policy = DeepQPolicy::new(weights_path);
            let name = policy.name().to_string();
            let (o, c, t, s, mut n) = plan_tour(candidates, selected, constraints, &policy);
            if let Some(p) = weights_path {
                n.push(format!("weights_path={p} loaded={}", policy.weights_loaded()));
            }
            if let Some(note) = policy.load_note() {
                n.push(note.to_string());
            }
            if policy.weights_loaded() {
                n.push("linear Q(s,a) stand-in over loaded v1 weights — NOT a trained Deep-Q net".into());
            }
            (name, o, c, t, s, n)
        }
        PackStrategy::Milp | PackStrategy::Greedy => {
            let policy = GreedyHeuristicPolicy;
            let name = policy.name().to_string();
            let (o, c, t, s, n) = plan_tour(candidates, selected, constraints, &policy);
            (name, o, c, t, s, n)
        }
    };

    // Map local selected-order indices → candidate indices.
    let order_indices: Vec<usize> = order.iter().map(|&si| selected[si]).collect();

    // Prefer pack solver_status for milp path; for deepq keep policy status.
    let solver_status = match strategy {
        PackStrategy::Deepq => status,
        PackStrategy::Milp | PackStrategy::Greedy => {
            if selection.solver_status == "infeasible" {
                selection.solver_status.clone()
            } else if order_indices.is_empty() {
                "infeasible".into()
            } else {
                selection.solver_status.clone()
            }
        }
    };

    notes.extend(selection.notes.iter().cloned());
    notes.push(format!("tour_policy={policy_name}"));

    TourOutcome {
        order_indices,
        cuts,
        travel_cost: travel,
        solver_status,
        notes,
        policy_name,
    }
}
