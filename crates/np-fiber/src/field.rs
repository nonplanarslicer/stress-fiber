//! #15 — stress-weighted scalar field on layered surfaces.
//!
//! Card stage: "Build a stress-weighted scalar field on each printable layer
//! so isocurve spacing tightens where stress is high."
//!
//! Consumes [`LayerSurface`](crate::layer::LayerSurface) from #42; does **not**
//! extract iso-layers.

use crate::types::{FiberHardwareProfile, LayerSurface, ScalarFieldSample, StressField, StressSample};

/// Input for #15 scalar-field construction.
#[derive(Clone, Debug)]
pub struct FieldInput<'a> {
    pub layers: &'a [LayerSurface],
    pub stress: &'a [StressSample],
    pub hardware: &'a FiberHardwareProfile,
}

/// #15 stage 1–2: build stress-weighted scalar fields per layer.
///
/// Spacing bounds come from `hardware.min_spacing` / `max_spacing` (caller
/// params). Algorithm body is a TODO stub with stable output shape.
pub fn build_stress_weighted_field(input: &FieldInput<'_>) -> Vec<StressField> {
    // TODO(#15): interpolate FEA principals onto each LayerSurface and weight
    // a scalar so isocurve density tracks stress magnitude within
    // [min_spacing, max_spacing].
    let _ = input.hardware;
    input
        .layers
        .iter()
        .map(|layer| {
            let samples = if input.stress.is_empty() {
                layer
                    .points
                    .iter()
                    .map(|p| ScalarFieldSample {
                        position: crate::types::Point3::from_array(*p),
                        value: 0.0,
                        direction: None,
                    })
                    .collect()
            } else {
                input
                    .stress
                    .iter()
                    .cloned()
                    .map(|s| ScalarFieldSample {
                        position: s.position,
                        value: s.magnitude,
                        direction: Some(s.principal_direction),
                    })
                    .collect()
            };
            StressField {
                layer_id: layer.id,
                samples,
            }
        })
        .collect()
}
