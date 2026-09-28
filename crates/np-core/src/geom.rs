//! Geometry aliases and locked public path/layer types for #42 → Fiber/D handoff.

use serde::{Deserialize, Serialize};

/// World-space point or direction (metres unless the host remaps units).
pub type Vec3 = [f64; 3];

/// Curved printable layer surface (iso-surface of the governing scalar).
///
/// Re-exported at the crate root for Fiber (family E) consumers.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayerSurface {
    pub id: u32,
    pub points: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub thickness: f64,
    pub sequence: u32,
}

/// Single TCP / toolpath sample with Part-F style per-point record.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolpathPoint {
    pub xyz: Vec3,
    pub tool_vector: Vec3,
    pub h: f64,
    pub w: f64,
    pub feature: String,
    pub sequence: u32,
}

/// Ordered tool-centre-point path stub for D / kinematics adapters.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TcpPath {
    pub points: Vec<ToolpathPoint>,
}

/// One stress-aligned fill polyline on a layer (open or closed).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FillPolyline {
    pub layer_id: u32,
    pub points: Vec<Vec3>,
    /// Tool / nozzle axis at each point (typically surface normal).
    pub tool_vectors: Vec<Vec3>,
    /// In-plane feed direction (stress-aligned when available).
    pub feed_directions: Vec<Vec3>,
}

#[inline]
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
pub fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
pub fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}

#[inline]
pub fn normalize(a: Vec3) -> Vec3 {
    let n = norm(a);
    if n < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        scale(a, 1.0 / n)
    }
}

#[inline]
pub fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    add(a, scale(sub(b, a), t))
}
