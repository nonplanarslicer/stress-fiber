//! Lightweight iso-layer extraction.
use crate::{fea::StressGrid,field::GoverningScalar,geom::LayerSurface};
pub fn extract_iso_surfaces(f:&GoverningScalar,g:&StressGrid,_h:f64,t:f64)->Vec<LayerSurface>{if f.is_empty()||g.is_empty(){return vec![]}let pts=(0..g.dims[1]).flat_map(|j|(0..g.dims[0]).map(move|i|f.position(i,j,g.dims[2]/2))).collect::<Vec<_>>();vec![LayerSurface{id:0,points:pts.clone(),normals:vec![[0.,0.,1.];pts.len()],thickness:t,sequence:0}]}
