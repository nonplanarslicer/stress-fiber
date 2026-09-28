//! Governing stress-aligned scalar field.
use serde::{Deserialize,Serialize};use crate::fea::StressGrid;use crate::geom::{self,Vec3};
#[derive(Clone,Debug,Serialize,Deserialize)] pub struct GoverningScalar{pub origin:Vec3,pub spacing:Vec3,pub dims:[usize;3],pub values:Vec<f64>,pub gradients:Vec<Vec3>}
impl GoverningScalar{pub fn is_empty(&self)->bool{self.values.is_empty()}pub fn index(&self,i:usize,j:usize,k:usize)->usize{i+j*self.dims[0]+k*self.dims[0]*self.dims[1]}pub fn position(&self,i:usize,j:usize,k:usize)->Vec3{[self.origin[0]+i as f64*self.spacing[0],self.origin[1]+j as f64*self.spacing[1],self.origin[2]+k as f64*self.spacing[2]]}pub fn range(&self)->(f64,f64){self.values.iter().fold((f64::INFINITY,f64::NEG_INFINITY),|(a,b),v|(a.min(*v),b.max(*v)))}}
pub fn build_governing_scalar(g:&StressGrid)->GoverningScalar{let values=g.magnitudes.clone();GoverningScalar{origin:g.origin,spacing:g.spacing,dims:g.dims,gradients:vec![[0.,0.,1.];values.len()],values}}
