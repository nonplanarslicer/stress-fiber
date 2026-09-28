//! Stress-aligned fill stub.
use crate::{fea::StressGrid,geom::{FillPolyline,LayerSurface}};
pub fn fill_layers(layers:&[LayerSurface],_grid:&StressGrid,_spacing:f64)->Vec<FillPolyline>{layers.iter().map(|l|FillPolyline{layer_id:l.id,points:l.points.clone(),tool_vectors:l.normals.clone(),feed_directions:vec![[1.,0.,0.];l.points.len()]}).collect()}
