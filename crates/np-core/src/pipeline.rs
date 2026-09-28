//! End-to-end #42 pipeline: stress → field → iso → clamp → fill → TCP.

use serde::{Deserialize, Serialize};

use crate::clamp::{self, ClampParams};
use crate::collision::{self, CollisionReport};
use crate::fea::{self, StressIn};
use crate::field::{self, GoverningScalar};
use crate::fill;
use crate::geom::{FillPolyline, LayerSurface, TcpPath};
use crate::isosurface;
use crate::tcp;

fn default_layer_height() -> f64 { 0.2 }
fn default_fill_spacing() -> f64 { 0.4 }
fn default_bead_width() -> f64 { 0.4 }
fn default_thickness() -> f64 { 0.2 }
fn default_max_overhang() -> f64 { 45.0 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoreInput {
    #[serde(default)] pub stress: StressIn,
    #[serde(default = "default_layer_height")] pub layer_height: f64,
    #[serde(default = "default_fill_spacing")] pub fill_spacing: f64,
    #[serde(default = "default_bead_width")] pub bead_width: f64,
    #[serde(default = "default_thickness")] pub thickness: f64,
    #[serde(default = "default_max_overhang")] pub max_overhang_deg: f64,
}
impl Default for CoreInput { fn default() -> Self { Self { stress: StressIn::default(), layer_height: default_layer_height(), fill_spacing: default_fill_spacing(), bead_width: default_bead_width(), thickness: default_thickness(), max_overhang_deg: default_max_overhang() } } }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CoreOutput { pub field: Option<GoverningScalarSummary>, pub layers: Vec<LayerSurface>, pub fill: Vec<FillPolyline>, pub collision: CollisionReport, pub tcp: TcpPath }
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GoverningScalarSummary { pub dims: [usize;3], pub origin: [f64;3], pub spacing: [f64;3], pub phi_min: f64, pub phi_max: f64, pub node_count: usize }
impl From<&GoverningScalar> for GoverningScalarSummary { fn from(f:&GoverningScalar)->Self { let (phi_min,phi_max)=f.range(); Self{dims:f.dims,origin:f.origin,spacing:f.spacing,phi_min,phi_max,node_count:f.values.len()} } }

pub fn run(input: &CoreInput) -> CoreOutput {
    let grid = fea::ingest(&input.stress);
    if grid.is_empty() { return CoreOutput::default(); }
    let field = field::build_governing_scalar(&grid);
    let raw_layers = isosurface::extract_iso_surfaces(&field, &grid, input.layer_height.max(1e-9), input.thickness.max(1e-9));
    let layers = clamp::clamp_layers(&raw_layers, &ClampParams { max_overhang_deg: input.max_overhang_deg, smooth_iters: 1 });
    let fill = fill::fill_layers(&layers, &grid, input.fill_spacing.max(1e-9));
    let collision = collision::check_collision_stub(&layers, &fill);
    let tcp = tcp::emit_tcp(&fill, &layers, input.bead_width.max(1e-9));
    CoreOutput { field: Some(GoverningScalarSummary::from(&field)), layers, fill, collision, tcp }
}
