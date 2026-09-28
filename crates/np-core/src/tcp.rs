//! TCP handoff from fill polylines.
use crate::geom::{FillPolyline,LayerSurface,TcpPath,ToolpathPoint};
pub fn emit_tcp(f:&[FillPolyline],l:&[LayerSurface],w:f64)->TcpPath{let mut p=vec![];for x in f{let h=l.iter().find(|z|z.id==x.layer_id).map(|z|z.thickness).unwrap_or(.2);for(i,q)in x.points.iter().enumerate(){p.push(ToolpathPoint{xyz:*q,tool_vector:x.tool_vectors.get(i).copied().unwrap_or([0.,0.,1.]),h,w,feature:format!("layer:{}:fill",x.layer_id),sequence:p.len() as u32});}}TcpPath{points:p}}
