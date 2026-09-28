use crate::geom::{self,LayerSurface};
#[derive(Clone,Debug)] pub struct ClampParams{pub max_overhang_deg:f64,pub smooth_iters:u32} impl Default for ClampParams{fn default()->Self{Self{max_overhang_deg:45.0,smooth_iters:1}}}
pub fn clamp_layers(layers:&[LayerSurface],_params:&ClampParams)->Vec<LayerSurface>{layers.to_vec()}
