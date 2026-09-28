//! #15 — isocurve centerline extraction from stress-weighted scalar fields.
//!
//! Card stage: "Extract isocurves as fiber centerlines; clip to printable
//! region and merge short fragments."

use crate::types::{FiberPolyline, StressField};

/// #15 stage 3: extract adaptive-density isocurves as fiber centerlines.
///
/// Stub returns no polylines until the marching / stripe extractor lands.
pub fn extract_centerlines(fields: &[StressField], iso_levels: &[f64]) -> Vec<FiberPolyline> {
    // TODO(#15): march isocurves of each StressField at `iso_levels`, clip to
    // layer domain, merge short fragments into FiberPolyline centerlines.
    let _ = (fields, iso_levels);
    Vec::new()
}
