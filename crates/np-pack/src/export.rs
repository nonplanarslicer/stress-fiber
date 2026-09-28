//! TCP / G-code stub emitter for packed fiber tours.
use crate::types::{LoopCandidate, PackOutput};
#[derive(Clone, Debug)]
pub struct ExportOptions { pub program_name: String, pub feed_mm_min: f64 }
impl Default for ExportOptions { fn default() -> Self { Self { program_name: "np-pack-stub".into(), feed_mm_min: 600.0 } } }
pub fn emit_gcode_stub(candidates: &[LoopCandidate], output: &PackOutput, opts: &ExportOptions) -> String {
    let by_id: std::collections::HashMap<&str, &LoopCandidate> = candidates.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut out = format!("; np-pack TCP/G-code STUB — {}\n; NOT machine-validated.\n; strategy={} status={}\nG21 ; mm\nG90 ; absolute\nF{:.1}\n", opts.program_name, output.strategy, output.solver_status, opts.feed_mm_min);
    let cut_set: std::collections::HashSet<usize> = output.cuts.iter().map(|c| c.after_tour_index).collect();
    for (step, id) in output.tour_order.iter().enumerate() { if let Some(c) = by_id.get(id.as_str()) { out.push_str(&format!("; --- loop {} ---\nM700 ; FIBER_ON (placeholder)\n", id)); for (i,p) in c.points.iter().enumerate() { out.push_str(&format!("{} X{:.4} Y{:.4}\n", if i==0 {"G0"} else {"G1"}, p[0], p[1])); } out.push_str("M701 ; FIBER_OFF (placeholder)\n"); if cut_set.contains(&step) { out.push_str("M702 ; FIBER_CUT (placeholder)\nM703 ; FIBER_RESTART (placeholder)\n"); } } }
    out.push_str("M400 ; wait\n; end stub\n"); out
}
