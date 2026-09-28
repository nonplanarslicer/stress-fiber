//! #27 — Layer-wise MILP packing of fiber loops (arxiv.org/abs/2404.11404).
//!
//! **Implementation gap (card):** solver choice, variable encoding, and timing
//! are not in the research corpus. A pluggable [`MilpBackend`] is reserved for a
//! real MILP solver. The shipped implementation is a **deterministic
//! greedy-by-weight** stand-in that respects conflict pairs, centroid spacing,
//! bend-radius legality, and fiber-length budget — clearly labeled
//! **NOT a full MILP**.
//!
//! **Ownership:** this crate owns `min_spacing` → conflict-pair inference.
//! Interlayer stacking rules stay TODO (card #27) until a real MILP encoding lands.

use crate::types::{LoopCandidate, PackConstraints, PackStrategy};
use std::collections::{HashMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum PackError {
    #[error("no candidates")]
    Empty,
    #[error("infeasible: {0}")]
    Infeasible(String),
}

/// Pluggable MILP backend. Production solvers implement this.
pub trait MilpBackend {
    fn select(
        &self,
        candidates: &[LoopCandidate],
        constraints: &PackConstraints,
        conflict_pairs: &[(String, String)],
    ) -> Result<Selection, PackError>;
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub selected_indices: Vec<usize>,
    pub unplaced_indices: Vec<usize>,
    pub strategy_label: String,
    pub solver_status: String,
    pub notes: Vec<String>,
}

/// Deterministic greedy-by-weight selection. **NOT a full MILP.**
pub struct GreedyPackStub;

impl MilpBackend for GreedyPackStub {
    fn select(
        &self,
        candidates: &[LoopCandidate],
        constraints: &PackConstraints,
        conflict_pairs: &[(String, String)],
    ) -> Result<Selection, PackError> {
        greedy_select(candidates, constraints, conflict_pairs)
    }
}

pub fn pack_select(
    candidates: &[LoopCandidate],
    constraints: &PackConstraints,
    conflict_pairs: &[(String, String)],
    strategy: &PackStrategy,
) -> Result<Selection, PackError> {
    let _ = strategy; // milp | greedy both use the stub until a real backend lands
    GreedyPackStub.select(candidates, constraints, conflict_pairs)
}

fn centroid(pts: &[[f64; 2]]) -> [f64; 2] {
    if pts.is_empty() {
        return [0.0, 0.0];
    }
    let n = pts.len() as f64;
    [
        pts.iter().map(|p| p[0]).sum::<f64>() / n,
        pts.iter().map(|p| p[1]).sum::<f64>() / n,
    ]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

fn build_conflict_set(
    candidates: &[LoopCandidate],
    constraints: &PackConstraints,
    conflict_pairs: &[(String, String)],
) -> HashSet<(usize, usize)> {
    let id_to_idx: HashMap<&str, usize> = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.as_str(), i))
        .collect();

    let mut set = HashSet::new();
    for (a, b) in conflict_pairs {
        if let (Some(&i), Some(&j)) = (id_to_idx.get(a.as_str()), id_to_idx.get(b.as_str())) {
            if i != j {
                set.insert(if i < j { (i, j) } else { (j, i) });
            }
        }
    }

    // Ownership (FNPS): np-pack owns spacing → conflict_pairs inference.
    // Centroid distance is a working stand-in for continuous spacing MILP vars.
    let cents: Vec<[f64; 2]> = candidates.iter().map(|c| centroid(&c.points)).collect();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            // Same-layer only. Interlayer stacking rules: TODO (card #27) until real MILP.
            if candidates[i].layer_id != candidates[j].layer_id {
                continue;
            }
            if dist(cents[i], cents[j]) < constraints.min_spacing {
                set.insert((i, j));
            }
        }
    }
    set
}

fn bend_ok(c: &LoopCandidate, bend_radius: f64) -> bool {
    match c.min_bend_radius {
        Some(r) => r + f64::EPSILON >= bend_radius,
        None => true, // upstream filtered
    }
}

fn greedy_select(
    candidates: &[LoopCandidate],
    constraints: &PackConstraints,
    conflict_pairs: &[(String, String)],
) -> Result<Selection, PackError> {
    if candidates.is_empty() {
        return Err(PackError::Empty);
    }

    let conflicts = build_conflict_set(candidates, constraints, conflict_pairs);

    // Process layer-wise: within each layer, greedy by weight desc, id asc.
    let mut layers: Vec<u32> = candidates.iter().map(|c| c.layer_id).collect();
    layers.sort_unstable();
    layers.dedup();

    let mut selected: Vec<usize> = Vec::new();
    let mut used_length = 0.0_f64;

    for layer in layers {
        let mut order: Vec<usize> = candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.layer_id == layer)
            .map(|(i, _)| i)
            .collect();
        order.sort_by(|&a, &b| {
            let ca = &candidates[a];
            let cb = &candidates[b];
            cb.weight
                .partial_cmp(&ca.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| ca.id.cmp(&cb.id))
        });

        for &idx in &order {
            let c = &candidates[idx];
            if !bend_ok(c, constraints.bend_radius) {
                continue;
            }
            if used_length + c.length > constraints.fiber_length_budget + f64::EPSILON {
                continue;
            }
            let hits = selected.iter().any(|&s| {
                let (lo, hi) = if s < idx { (s, idx) } else { (idx, s) };
                conflicts.contains(&(lo, hi))
            });
            if hits {
                continue;
            }
            selected.push(idx);
            used_length += c.length;
        }
    }

    selected.sort_unstable();
    let selected_set: HashSet<usize> = selected.iter().copied().collect();
    let unplaced: Vec<usize> = (0..candidates.len())
        .filter(|i| !selected_set.contains(i))
        .collect();

    let status = if selected.is_empty() {
        "infeasible"
    } else {
        "stub-heuristic"
    };

    Ok(Selection {
        selected_indices: selected,
        unplaced_indices: unplaced,
        strategy_label: "milp-greedy-stub".into(),
        solver_status: status.into(),
        notes: vec![
            "greedy-by-weight layer-wise stand-in — NOT a full MILP".into(),
            "card #27 gap: solver choice, variable encoding, timing not in corpus".into(),
            "min_spacing enforced via centroid distance + explicit conflict_pairs".into(),
            "TODO card #27: interlayer stacking rules not encoded until real MILP".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(id: &str, layer: u32, weight: f64, length: f64, xy: [f64; 2]) -> LoopCandidate {
        LoopCandidate {
            id: id.into(),
            layer_id: layer,
            points: vec![xy, [xy[0] + 1.0, xy[1]], [xy[0] + 1.0, xy[1] + 1.0], xy],
            weight,
            length,
            min_bend_radius: Some(5.0),
        }
    }

    #[test]
    fn greedy_picks_highest_weight_respecting_conflict() {
        let candidates = vec![
            cand("a", 0, 10.0, 5.0, [0.0, 0.0]),
            cand("b", 0, 8.0, 5.0, [50.0, 0.0]),
            cand("c", 0, 3.0, 5.0, [100.0, 0.0]),
        ];
        let constraints = PackConstraints {
            bend_radius: 3.0,
            min_spacing: 1.0,
            fiber_length_budget: 20.0,
            cut_cost: 1.0,
            restart_cost: 1.0,
            allow_cuts: true,
        };
        let sel = greedy_select(&candidates, &constraints, &[("a".into(), "b".into())]).unwrap();
        let ids: Vec<&str> = sel
            .selected_indices
            .iter()
            .map(|&i| candidates[i].id.as_str())
            .collect();
        assert_eq!(ids, vec!["a", "c"]);
        assert_eq!(sel.solver_status, "stub-heuristic");
    }

    #[test]
    fn spacing_infers_conflict() {
        let candidates = vec![
            cand("a", 0, 10.0, 2.0, [0.0, 0.0]),
            cand("b", 0, 9.0, 2.0, [0.1, 0.0]), // too close
        ];
        let constraints = PackConstraints {
            min_spacing: 5.0,
            fiber_length_budget: 100.0,
            ..PackConstraints::default()
        };
        let sel = greedy_select(&candidates, &constraints, &[]).unwrap();
        assert_eq!(sel.selected_indices, vec![0]);
    }

    #[test]
    fn length_budget_stops_selection() {
        let candidates = vec![
            cand("a", 0, 10.0, 8.0, [0.0, 0.0]),
            cand("b", 0, 9.0, 8.0, [50.0, 0.0]),
        ];
        let constraints = PackConstraints {
            fiber_length_budget: 10.0,
            min_spacing: 1.0,
            ..PackConstraints::default()
        };
        let sel = greedy_select(&candidates, &constraints, &[]).unwrap();
        assert_eq!(sel.selected_indices, vec![0]);
    }
}
