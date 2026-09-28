//! Geometry aliases and locked public path/layer types for #42 → Fiber/D handoff.

use serde::{Deserialize, Serialize};
pub type Vec3 = [f64; 3];
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayerSurface { pub id: u32, pub points: Vec<Vec3>, pub normals: Vec<Vec3>, pub thickness: f64, pub sequence: u32 }
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolpathPoint { pub xyz: Vec3, pub tool_vector: Vec3, pub h: f64, pub w: f64, pub feature: String, pub sequence: u32 }
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TcpPath { pub points: Vec<ToolpathPoint> }
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FillPolyline { pub layer_id: u32, pub points: Vec<Vec3>, pub tool_vectors: Vec<Vec3>, pub feed_directions: Vec<Vec3> }
#[inline] pub fn add(a: Vec3,b: Vec3)->Vec3{[a[0]+b[0],a[1]+b[1],a[2]+b[2]]}
#[inline] pub fn sub(a: Vec3,b: Vec3)->Vec3{[a[0]-b[0],a[1]-b[1],a[2]-b[2]]}
#[inline] pub fn scale(a: Vec3,s:f64)->Vec3{[a[0]*s,a[1]*s,a[2]*s]}
#[inline] pub fn dot(a: Vec3,b: Vec3)->f64{a[0]*b[0]+a[1]*b[1]+a[2]*b[2]}
#[inline] pub fn cross(a: Vec3,b: Vec3)->Vec3{[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]}
#[inline] pub fn norm(a: Vec3)->f64{dot(a,a).sqrt()}
#[inline] pub fn normalize(a: Vec3)->Vec3{let n=norm(a);if n<1e-12{[0.,0.,1.]}else{scale(a,1./n)}}
#[inline] pub fn lerp(a: Vec3,b: Vec3,t:f64)->Vec3{add(a,scale(sub(b,a),t))}
