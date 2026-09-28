//! np-pack — continuous-fiber loop packing (#27) and Deep-Q next-node planning (#22).
//!
//! Owned by NP Pack UI. Upstream field / routing algorithms live in `np-core` and
//! `np-fiber`; this crate owns packing + sequencing policy and the JSON ABI used
//! by `np-native::pack_run`.
//!
//! ## Gaps (from research cards — do not invent metrics)
//! - #27: solver choice, variable encoding, timing not in corpus → greedy stand-in
//! - #22: architecture, reward, dataset, weights not in corpus → greedy fallback

pub mod deepq;
pub mod export;
pub mod milp;
pub mod tour;
pub mod types;

pub use deepq::{load_weights_file, plan_tour, DeepQPolicy, DeepQWeightsFile, GreedyHeuristicPolicy, NextNodePolicy, TourState, WeightsLoadError};
pub use export::{emit_gcode_stub, ExportOptions};
pub use milp::{pack_select, GreedyPackStub, MilpBackend, PackError, Selection};
pub use tour::{sequence_selection, TourOutcome};
pub use types::{
    CutEvent, LoopCandidate, PackConstraints, PackInput, PackOutput, PackStrategy,
};

/// Entry point for `np-native::pack_run` (JSON in / JSON out via serde).
pub fn run(input: &PackInput) -> PackOutput {
    if input.candidates.is_empty() {
        return PackOutput {
            strategy: strategy_label(&input.strategy),
            solver_status: "infeasible".into(),
            notes: vec!["no candidates".into()],
            ..Default::default()
        };
    }

    let selection = match pack_select(
        &input.candidates,
        &input.constraints,
        &input.conflict_pairs,
        &input.strategy,
    ) {
        Ok(s) => s,
        Err(PackError::Empty) => {
            return PackOutput {
                strategy: strategy_label(&input.strategy),
                solver_status: "infeasible".into(),
                notes: vec!["no candidates".into()],
                ..Default::default()
            };
        }
        Err(e) => {
            return PackOutput {
                strategy: strategy_label(&input.strategy),
                solver_status: "error".into(),
                notes: vec![e.to_string()],
                ..Default::default()
            };
        }
    };

    let tour = sequence_selection(
        &input.candidates,
        &selection,
        &input.constraints,
        &input.strategy,
        input.weights_path.as_deref(),
    );

    let selected_loop_ids: Vec<String> = selection
        .selected_indices
        .iter()
        .map(|&i| input.candidates[i].id.clone())
        .collect();
    let tour_order: Vec<String> = tour
        .order_indices
        .iter()
        .map(|&i| input.candidates[i].id.clone())
        .collect();
    let unplaced: Vec<String> = selection
        .unplaced_indices
        .iter()
        .map(|&i| input.candidates[i].id.clone())
        .collect();
    let paths: Vec<Vec<[f64; 2]>> = tour
        .order_indices
        .iter()
        .map(|&i| input.candidates[i].points.clone())
        .collect();

    let strategy = match input.strategy {
        PackStrategy::Deepq => format!("deepq/{}", tour.policy_name),
        PackStrategy::Milp => selection.strategy_label.clone(),
        PackStrategy::Greedy => "greedy".into(),
    };

    PackOutput {
        selected_loop_ids,
        tour_order,
        cuts: tour.cuts,
        strategy,
        solver_status: tour.solver_status,
        unplaced,
        notes: tour.notes,
        paths,
    }
}

fn strategy_label(s: &PackStrategy) -> String {
    match s {
        PackStrategy::Milp => "milp-greedy-stub".into(),
        PackStrategy::Deepq => "deepq-greedy-fallback".into(),
        PackStrategy::Greedy => "greedy".into(),
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn demo_input(strategy: PackStrategy) -> PackInput {
        PackInput {
            candidates: vec![
                LoopCandidate {
                    id: "L0".into(),
                    layer_id: 0,
                    points: vec![[10.0, 10.0], [40.0, 10.0], [40.0, 40.0], [10.0, 40.0], [10.0, 10.0]],
                    weight: 8.0,
                    length: 120.0,
                    min_bend_radius: Some(5.0),
                },
                LoopCandidate {
                    id: "L1".into(),
                    layer_id: 0,
                    points: vec![[60.0, 10.0], [90.0, 10.0], [90.0, 40.0], [60.0, 40.0], [60.0, 10.0]],
                    weight: 5.0,
                    length: 120.0,
                    min_bend_radius: Some(5.0),
                },
                LoopCandidate {
                    id: "L2".into(),
                    layer_id: 0,
                    points: vec![[12.0, 12.0], [38.0, 12.0], [38.0, 38.0], [12.0, 38.0], [12.0, 12.0]],
                    weight: 9.0,
                    length: 104.0,
                    min_bend_radius: Some(5.0),
                },
                LoopCandidate {
                    id: "L3".into(),
                    layer_id: 1,
                    points: vec![[20.0, 60.0], [50.0, 60.0], [50.0, 90.0], [20.0, 90.0], [20.0, 60.0]],
                    weight: 4.0,
                    length: 120.0,
                    min_bend_radius: Some(5.0),
                },
            ],
            constraints: PackConstraints {
                bend_radius: 3.0,
                min_spacing: 25.0,
                fiber_length_budget: 400.0,
                cut_cost: 10.0,
                restart_cost: 5.0,
                allow_cuts: true,
            },
            strategy,
            conflict_pairs: vec![("L0".into(), "L2".into())],
            weights_path: None,
        }
    }

    #[test]
    fn milp_strategy_runs_greedy_stub() {
        let out = run(&demo_input(PackStrategy::Milp));
        assert!(!out.selected_loop_ids.is_empty());
        assert!(out.selected_loop_ids.contains(&"L0".into()) || out.selected_loop_ids.contains(&"L2".into()));
        assert!(!out.selected_loop_ids.contains(&"L0".into()) || !out.selected_loop_ids.contains(&"L2".into()));
        assert_eq!(out.solver_status, "stub-heuristic");
        assert!(out.strategy.contains("milp") || out.strategy.contains("greedy"));
        assert_eq!(out.tour_order.len(), out.selected_loop_ids.len());
        assert!(!out.paths.is_empty());
    }

    #[test]
    fn deepq_strategy_falls_back() {
        let out = run(&demo_input(PackStrategy::Deepq));
        assert_eq!(out.solver_status, "policy-fallback");
        assert!(out.strategy.contains("deepq"));
        assert!(!out.tour_order.is_empty());
    }
}
