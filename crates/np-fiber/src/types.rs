//! Shared geometry and hardware-profile types for Family E.
//!
//! Bend-radius / spacing / width values always come from
//! [`FiberHardwareProfile`] (caller / machine config). Never hard-code paper
//! constants — cards #15/#10/#52 explicitly say those numbers are absent from
//! the research corpus and must come from hardware.

use serde::{Deserialize, Serialize};

pub use crate::layer::{LayerSurface, TcpPath, ToolpathPoint};

/// 3-D point (f64 to match LayerSurface).
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn from_array(a: [f64; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }

    pub fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    pub fn distance(self, other: Point3) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Stress-weighted or periodic scalar field sample (#15 / #52).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ScalarFieldSample {
    pub position: Point3,
    /// Scalar value whose isocurves become fiber centerlines.
    pub value: f64,
    /// Optional principal / design direction (unit-ish); unused by pure #15 scalar path.
    pub direction: Option<Point3>,
}

/// Stress-weighted scalar field on a layer (#15 stage: build field).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StressField {
    pub layer_id: u32,
    pub samples: Vec<ScalarFieldSample>,
}

/// Alias kept for napi / TS naming.
pub type ScalarField = StressField;

/// Continuous-fiber centerline polyline.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FiberPolyline {
    pub id: u32,
    pub layer_id: u32,
    pub points: Vec<Point3>,
    /// Closed loop (e.g. #10 hole loop) when true.
    pub closed: bool,
}

/// Ordered fiber path after bend / spacing filters (toolpath-ready).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FiberPath {
    pub id: u32,
    pub polyline: FiberPolyline,
    /// True when the path survived bend-radius and spacing gates.
    pub printable: bool,
}

/// Machine / process limits supplied by the caller — **not** paper constants.
///
/// Card #15 notes: "No full-text algorithm constants in corpus — bend-radius /
/// spacing numbers must come from hardware profile, not this card."
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FiberHardwareProfile {
    /// Deposited fiber / tow width.
    pub width: f64,
    /// Minimum printable bend radius (length units of the part).
    pub min_bend_radius: f64,
    /// Minimum centerline spacing between adjacent fibers.
    pub min_spacing: f64,
    /// Maximum centerline spacing (adaptive density upper bound for #15).
    pub max_spacing: f64,
}

impl FiberHardwareProfile {
    pub fn new(width: f64, min_bend_radius: f64, min_spacing: f64, max_spacing: f64) -> Self {
        Self {
            width,
            min_bend_radius,
            min_spacing,
            max_spacing,
        }
    }
}

/// Optional FEA / design stress sample for weighting the #15 scalar field.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StressSample {
    pub position: Point3,
    pub principal_direction: Point3,
    pub magnitude: f64,
}
