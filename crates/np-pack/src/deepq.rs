//! #22 — Deep-Q next-node planner on local subgraphs (arxiv.org/abs/2408.09198).
//!
//! **Implementation gap (card):** network architecture, reward, and dataset are
//! not in the research corpus; no trained model weights ship with this crate.
//! [`DeepQPolicy`] accepts an optional weights path and falls back to
//! [`GreedyHeuristicPolicy`] when missing or unreadable.
//!
//! ## Weights file format (v1)
//!
//! JSON, flat row-major `state_dim × action_dim`:
//! ```json
//! { "version": 1, "state_dim": N, "action_dim": M, "weights": [/* N*M floats */] }
//! ```

use crate::types::{CutEvent, LoopCandidate, PackConstraints};
use serde::Deserialize;
use std::path::Path;

/// Deep-Q weights file schema (version 1).
#[derive(Clone, Debug, Deserialize)]
pub struct DeepQWeightsFile {
    pub version: u32,
    pub state_dim: usize,
    pub action_dim: usize,
    pub weights: Vec<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum WeightsLoadError {
    #[error("weights path missing")]
    MissingPath,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported version {0} (expected 1)")]
    BadVersion(u32),
    #[error("weights length {got} != state_dim*action_dim ({expect})")]
    BadLength { got: usize, expect: usize },
    #[error("state_dim and action_dim must be > 0")]
    BadDims,
}

/// Load and validate a v1 Deep-Q weights JSON file.
pub fn load_weights_file(path: &Path) -> Result<DeepQWeightsFile, WeightsLoadError> {
    let raw = std::fs::read_to_string(path)?;
    let w: DeepQWeightsFile = serde_json::from_str(&raw)?;
    if w.version != 1 {
        return Err(WeightsLoadError::BadVersion(w.version));
    }
    if w.state_dim == 0 || w.action_dim == 0 {
        return Err(WeightsLoadError::BadDims);
    }
    let expect = w.state_dim.saturating_mul(w.action_dim);
    if w.weights.len() != expect {
        return Err(WeightsLoadError::BadLength {
            got: w.weights.len(),
            expect,
        });
    }
    Ok(w)
}

/// Local observation for next-node choice.
#[derive(Clone, Debug)]
pub struct TourState<'a> {
    pub candidates: &'a [LoopCandidate],
    pub selected: &'a [usize],
    pub visited: &'a [bool],
    pub current: Option<usize>,
    pub fiber_on: bool,
    pub constraints: &'a PackConstraints,
}

/// Policy interface for choosing the next selected-loop index (into `selected`).
pub trait NextNodePolicy {
    fn next_node(&self, state: &TourState<'_>) -> Option<usize>;
    fn name(&self) -> &str;
}

/// Greedy heuristic: prefer nearest unvisited centroid; start at highest weight.
pub struct GreedyHeuristicPolicy;

impl NextNodePolicy for GreedyHeuristicPolicy {
    fn name(&self) -> &str {
        "greedy-heuristic"
    }

    fn next_node(&self, state: &TourState<'_>) -> Option<usize> {
        let selected = state.selected;
        if selected.is_empty() {
            return None;
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

        // First node: highest weight among unvisited.
        if state.current.is_none() {
            let mut best: Option<(usize, f64, &str)> = None;
            for (si, &ci) in selected.iter().enumerate() {
                if state.visited[si] {
                    continue;
                }
                let c = &state.candidates[ci];
                let key = (c.weight, c.id.as_str());
                best = match best {
                    None => Some((si, key.0, key.1)),
                    Some((_, w, id)) if key.0 > w || (key.0 == w && key.1 < id) => {
                        Some((si, key.0, key.1))
                    }
                    other => other,
                };
            }
            return best.map(|(si, _, _)| si);
        }

        let cur_ci = selected[state.current.unwrap()];
        let cur_c = centroid(&state.candidates[cur_ci].points);
        let mut best: Option<(usize, f64)> = None;
        for (si, &ci) in selected.iter().enumerate() {
            if state.visited[si] {
                continue;
            }
            let d = dist(cur_c, centroid(&state.candidates[ci].points));
            let score = -d;
            best = match best {
                None => Some((si, score)),
                Some((_, s)) if score > s => Some((si, score)),
                other => other,
            };
        }
        best.map(|(si, _)| si)
    }
}

/// Deep-Q policy shell. Loads optional v1 JSON weights; falls back to greedy if absent/unreadable.
///
/// When weights load, next-node uses a **linear** Q(s,a) = s · W[:,a] (row-major) over a
/// tiny hand-built state vector — **not** a trained deep network. Missing architecture
/// in corpus; this only exercises the load path + argmax over remaining actions.
pub struct DeepQPolicy {
    weights_path: Option<String>,
    weights: Option<DeepQWeightsFile>,
    load_note: Option<String>,
    fallback: GreedyHeuristicPolicy,
}

impl DeepQPolicy {
    pub fn new(weights_path: Option<&str>) -> Self {
        let path = weights_path.map(str::to_string);
        let (weights, load_note) = match path.as_deref() {
            None => (None, Some("no weights_path — greedy fallback".into())),
            Some(p) => match load_weights_file(Path::new(p)) {
                Ok(w) => (Some(w), None),
                Err(e) => (
                    None,
                    Some(format!("weights unreadable ({e}) — greedy fallback")),
                ),
            },
        };
        Self {
            weights_path: path,
            weights,
            load_note,
            fallback: GreedyHeuristicPolicy,
        }
    }

    pub fn weights_loaded(&self) -> bool {
        self.weights.is_some()
    }

    pub fn weights_path(&self) -> Option<&str> {
        self.weights_path.as_deref()
    }

    pub fn load_note(&self) -> Option<&str> {
        self.load_note.as_deref()
    }

    /// Build a deterministic state vector of length `state_dim` (pad/truncate).
    fn encode_state(state: &TourState<'_>, state_dim: usize) -> Vec<f64> {
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
        let mut feats = Vec::with_capacity(8);
        let cur = state
            .current
            .map(|si| centroid(&state.candidates[state.selected[si]].points))
            .unwrap_or([0.0, 0.0]);
        feats.push(cur[0]);
        feats.push(cur[1]);
        feats.push(if state.fiber_on { 1.0 } else { 0.0 });
        let remaining = state.visited.iter().filter(|&&v| !v).count() as f64;
        feats.push(remaining);
        feats.push(state.constraints.min_spacing);
        feats.push(state.constraints.bend_radius);
        feats.push(state.selected.len() as f64);
        feats.push(state.current.map(|c| c as f64).unwrap_or(-1.0));
        feats.resize(state_dim, 0.0);
        feats.truncate(state_dim);
        feats
    }

    fn linear_q_next(&self, state: &TourState<'_>, w: &DeepQWeightsFile) -> Option<usize> {
        let s = Self::encode_state(state, w.state_dim);
        let mut best: Option<(usize, f64)> = None;
        for (si, _) in state.selected.iter().enumerate() {
            if state.visited[si] {
                continue;
            }
            // Map selected-slot index into action column (mod action_dim).
            let a = si % w.action_dim;
            let mut q = 0.0;
            for i in 0..w.state_dim {
                q += s[i] * w.weights[i * w.action_dim + a];
            }
            // Tie-break: prefer lower si for determinism.
            best = match best {
                None => Some((si, q)),
                Some((bsi, bq)) if q > bq + f64::EPSILON || ((q - bq).abs() <= f64::EPSILON && si < bsi) => {
                    Some((si, q))
                }
                other => other,
            };
        }
        best.map(|(si, _)| si)
    }
}

impl NextNodePolicy for DeepQPolicy {
    fn name(&self) -> &str {
        if self.weights.is_some() {
            "deepq-linear"
        } else {
            "deepq-greedy-fallback"
        }
    }

    fn next_node(&self, state: &TourState<'_>) -> Option<usize> {
        if let Some(ref w) = self.weights {
            self.linear_q_next(state, w)
                .or_else(|| self.fallback.next_node(state))
        } else {
            self.fallback.next_node(state)
        }
    }
}

/// Run a tour over already-selected candidate indices.
pub fn plan_tour(
    candidates: &[LoopCandidate],
    selected: &[usize],
    constraints: &PackConstraints,
    policy: &dyn NextNodePolicy,
) -> (Vec<usize>, Vec<CutEvent>, f64, String, Vec<String>) {
    let n = selected.len();
    let mut visited = vec![false; n];
    let mut order_local: Vec<usize> = Vec::with_capacity(n);
    let mut cuts: Vec<CutEvent> = Vec::new();
    let mut current: Option<usize> = None;
    let mut fiber_on = false;
    let mut travel = 0.0_f64;

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

    while order_local.len() < n {
        let state = TourState {
            candidates,
            selected,
            visited: &visited,
            current,
            fiber_on,
            constraints,
        };
        let Some(next) = policy.next_node(&state) else {
            break;
        };
        if visited[next] {
            break;
        }

        if let Some(cur) = current {
            let d = dist(
                centroid(&candidates[selected[cur]].points),
                centroid(&candidates[selected[next]].points),
            );
            travel += d;
            // Jump threshold: > 4× min_spacing ⇒ cut/restart if allowed.
            let jump = d > constraints.min_spacing * 4.0;
            if jump && fiber_on {
                if constraints.allow_cuts {
                    cuts.push(CutEvent {
                        after_tour_index: order_local.len().saturating_sub(1),
                        kind: "cut+restart".into(),
                    });
                    travel += constraints.cut_cost + constraints.restart_cost;
                }
                // else: still sequence; forced continuous deposition when cuts disallowed
            }
        }

        order_local.push(next);
        visited[next] = true;
        current = Some(next);
        fiber_on = true;
    }

    let status = if order_local.is_empty() {
        "infeasible"
    } else if policy.name().contains("fallback") {
        "policy-fallback"
    } else if policy.name().contains("linear") {
        "weights-linear" // v1 JSON loaded; linear Q stand-in, NOT a trained Deep-Q net
    } else if policy.name().contains("greedy") {
        "stub-heuristic"
    } else {
        "feasible"
    };

    let notes = vec![
        format!("policy={}", policy.name()),
        "card #22 gap: architecture/reward/dataset/weights not in corpus".into(),
    ];

    (order_local, cuts, travel, status.into(), notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn sample() -> Vec<LoopCandidate> {
        vec![
            LoopCandidate {
                id: "a".into(),
                layer_id: 0,
                points: vec![[0.0, 0.0], [1.0, 0.0]],
                weight: 5.0,
                length: 2.0,
                min_bend_radius: Some(5.0),
            },
            LoopCandidate {
                id: "b".into(),
                layer_id: 0,
                points: vec![[10.0, 0.0], [11.0, 0.0]],
                weight: 10.0,
                length: 2.0,
                min_bend_radius: Some(5.0),
            },
            LoopCandidate {
                id: "c".into(),
                layer_id: 0,
                points: vec![[5.0, 0.0], [6.0, 0.0]],
                weight: 1.0,
                length: 2.0,
                min_bend_radius: Some(5.0),
            },
        ]
    }

    #[test]
    fn greedy_tour_starts_at_highest_weight() {
        let c = sample();
        let selected = vec![0, 1, 2];
        let (order, _, _, status, _) =
            plan_tour(&c, &selected, &PackConstraints::default(), &GreedyHeuristicPolicy);
        assert_eq!(status, "stub-heuristic");
        assert_eq!(order[0], 1); // candidate b
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn deepq_without_weights_falls_back() {
        let c = sample();
        let selected = vec![0, 1, 2];
        let policy = DeepQPolicy::new(None);
        assert!(!policy.weights_loaded());
        let (order, _, _, status, _) =
            plan_tour(&c, &selected, &PackConstraints::default(), &policy);
        assert_eq!(status, "policy-fallback");
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn deepq_bad_path_falls_back() {
        let policy = DeepQPolicy::new(Some("/no/such/weights.json"));
        assert!(!policy.weights_loaded());
        assert!(policy.load_note().unwrap().contains("unreadable"));
    }

    #[test]
    fn deepq_loads_v1_json_and_runs_linear() {
        let dir = std::env::temp_dir();
        let path = dir.join("np-pack-deepq-v1-test.json");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            // state_dim=4, action_dim=3 → 12 weights; bias action 1 (index of high-weight loop slot)
            write!(
                f,
                r#"{{"version":1,"state_dim":4,"action_dim":3,"weights":[0,1,0, 0,1,0, 0,1,0, 0,1,0]}}"#
            )
            .unwrap();
        }
        let policy = DeepQPolicy::new(Some(path.to_str().unwrap()));
        assert!(policy.weights_loaded());
        assert_eq!(policy.name(), "deepq-linear");
        let c = sample();
        let selected = vec![0, 1, 2];
        let (order, _, _, status, _) =
            plan_tour(&c, &selected, &PackConstraints::default(), &policy);
        assert_eq!(status, "weights-linear");
        assert_eq!(order.len(), 3);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_length_mismatch() {
        let dir = std::env::temp_dir();
        let path = dir.join("np-pack-deepq-bad-len.json");
        std::fs::write(
            &path,
            r#"{"version":1,"state_dim":2,"action_dim":2,"weights":[1.0]}"#,
        )
        .unwrap();
        let err = load_weights_file(&path).unwrap_err();
        assert!(matches!(err, WeightsLoadError::BadLength { .. }));
        let _ = std::fs::remove_file(&path);
    }
}
