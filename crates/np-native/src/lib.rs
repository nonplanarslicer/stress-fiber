//! napi-rs bridge for NP Core / Fiber / Pack.
//!
//! Fiber exports expose TypeScript-friendly types for the #15 pipeline.
//! Core and Pack JSON entry points are preserved for backward compatibility.

use napi_derive::napi;

// ---------------------------------------------------------------------------
// Existing JSON ABI (unchanged for Core / Pack / legacy fiber_run)
// ---------------------------------------------------------------------------

#[napi]
pub fn core_run(input_json: String) -> String {
    let input: np_core::CoreInput = serde_json::from_str(&input_json).unwrap_or_default();
    serde_json::to_string(&np_core::run(&input)).unwrap_or_else(|_| "{}".into())
}

#[napi]
pub fn fiber_run(input_json: String) -> String {
    // Hardware is required; if JSON is incomplete, return an empty pipeline result.
    match serde_json::from_str::<np_fiber::FiberInput>(&input_json) {
        Ok(input) => serde_json::to_string(&np_fiber::run(&input)).unwrap_or_else(|_| "{}".into()),
        Err(_) => serde_json::to_string(&np_fiber::FiberOutput::default())
            .unwrap_or_else(|_| "{}".into()),
    }
}

#[napi]
pub fn pack_run(input_json: String) -> String {
    let input: np_pack::PackInput = serde_json::from_str(&input_json).unwrap_or_default();
    serde_json::to_string(&np_pack::run(&input)).unwrap_or_else(|_| "{}".into())
}

// ---------------------------------------------------------------------------
// Family B / #42 Core TypeScript-friendly types (locked Part-F contract)
// ---------------------------------------------------------------------------

/// `np_core::ToolpathPoint` — xyz / tool_vector / h / w / feature / sequence.
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsCoreToolpathPoint {
    pub xyz: Vec<f64>,
    pub tool_vector: Vec<f64>,
    pub h: f64,
    pub w: f64,
    pub feature: String,
    pub sequence: u32,
}

/// `np_core::TcpPath`.
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsCoreTcpPath {
    pub points: Vec<JsCoreToolpathPoint>,
}

fn vec3_to_flat(v: [f64; 3]) -> Vec<f64> {
    vec![v[0], v[1], v[2]]
}

fn layer_to_js(l: &np_core::LayerSurface) -> JsLayerSurface {
    JsLayerSurface {
        id: l.id,
        points: l.points.iter().flat_map(|p| [p[0], p[1], p[2]]).collect(),
        normals: l.normals.iter().flat_map(|p| [p[0], p[1], p[2]]).collect(),
        thickness: l.thickness,
        sequence: l.sequence,
    }
}

/// Typed #42 runner: same pipeline as `core_run`, returning structured layers + TCP.
#[napi]
pub fn core_run_typed(input_json: String) -> JsCorePipelineResult {
    let input: np_core::CoreInput = serde_json::from_str(&input_json).unwrap_or_default();
    let out = np_core::run(&input);
    JsCorePipelineResult {
        layers: out.layers.iter().map(layer_to_js).collect(),
        tcp: JsCoreTcpPath {
            points: out
                .tcp
                .points
                .iter()
                .map(|p| JsCoreToolpathPoint {
                    xyz: vec3_to_flat(p.xyz),
                    tool_vector: vec3_to_flat(p.tool_vector),
                    h: p.h,
                    w: p.w,
                    feature: p.feature.clone(),
                    sequence: p.sequence,
                })
                .collect(),
        },
        layer_count: out.layers.len() as u32,
        tcp_point_count: out.tcp.points.len() as u32,
    }
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsCorePipelineResult {
    pub layers: Vec<JsLayerSurface>,
    pub tcp: JsCoreTcpPath,
    pub layer_count: u32,
    pub tcp_point_count: u32,
}

// ---------------------------------------------------------------------------
// Family E / #15 TypeScript-friendly types (napi objects)
// ---------------------------------------------------------------------------

/// External #42 layer contract (mirrors `np_core::LayerSurface`, re-exported by np-fiber).
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsLayerSurface {
    pub id: u32,
    /// Flat xyz triplets: [x0,y0,z0, x1,y1,z1, ...]
    pub points: Vec<f64>,
    pub normals: Vec<f64>,
    pub thickness: f64,
    pub sequence: u32,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsPoint3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsStressSample {
    pub position: JsPoint3,
    pub principal_direction: JsPoint3,
    pub magnitude: f64,
}

/// Hardware profile params — caller-supplied; not paper constants.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct JsFiberHardwareProfile {
    pub width: f64,
    pub min_bend_radius: f64,
    pub min_spacing: f64,
    pub max_spacing: f64,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsScalarFieldSample {
    pub position: JsPoint3,
    pub value: f64,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsStressField {
    pub layer_id: u32,
    pub samples: Vec<JsScalarFieldSample>,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsFiberPolyline {
    pub id: u32,
    pub layer_id: u32,
    pub points: Vec<JsPoint3>,
    pub closed: bool,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsFiberPath {
    pub id: u32,
    pub polyline: JsFiberPolyline,
    pub printable: bool,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsToolpathPoint {
    pub position: JsPoint3,
    pub tool_direction: JsPoint3,
    pub extrusion: f64,
    pub fiber_on: bool,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsTcpPath {
    pub id: u32,
    pub points: Vec<JsToolpathPoint>,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsDualExtrudePlan {
    pub fiber_paths: Vec<JsTcpPath>,
    pub matrix_paths: Vec<JsTcpPath>,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsMatrixSegment {
    pub layer_id: u32,
    pub start: JsPoint3,
    pub end: JsPoint3,
}

/// #15 pipeline input from TypeScript / Vite host.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct JsFiberPipelineInput {
    pub layers: Vec<JsLayerSurface>,
    pub stress: Vec<JsStressSample>,
    pub hardware: JsFiberHardwareProfile,
    pub iso_levels: Vec<f64>,
}

/// #15 pipeline result for TypeScript.
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsFiberPipelineResult {
    pub fields: Vec<JsStressField>,
    pub centerlines: Vec<JsFiberPolyline>,
    pub fibers: Vec<JsFiberPath>,
    pub matrix: Vec<JsMatrixSegment>,
    pub dual_extrude: JsDualExtrudePlan,
}

fn flat_to_points(flat: &[f64]) -> Vec<[f64; 3]> {
    flat.chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect()
}

fn pjs(p: np_fiber::Point3) -> JsPoint3 {
    JsPoint3 {
        x: p.x,
        y: p.y,
        z: p.z,
    }
}

fn pfrom(p: &JsPoint3) -> np_fiber::Point3 {
    np_fiber::Point3::new(p.x, p.y, p.z)
}

fn arr_to_js(a: [f64; 3]) -> JsPoint3 {
    JsPoint3 {
        x: a[0],
        y: a[1],
        z: a[2],
    }
}

/// #15: run stress-weighted isocurve → bend → matrix → dual-extrude pipeline.
#[napi]
pub fn fiber_run_pipeline(input: JsFiberPipelineInput) -> JsFiberPipelineResult {
    let layers: Vec<np_fiber::LayerSurface> = input
        .layers
        .iter()
        .map(|l| np_fiber::LayerSurface {
            id: l.id,
            points: flat_to_points(&l.points),
            normals: flat_to_points(&l.normals),
            thickness: l.thickness,
            sequence: l.sequence,
        })
        .collect();
    let stress: Vec<np_fiber::StressSample> = input
        .stress
        .iter()
        .map(|s| np_fiber::StressSample {
            position: pfrom(&s.position),
            principal_direction: pfrom(&s.principal_direction),
            magnitude: s.magnitude,
        })
        .collect();
    let hardware = np_fiber::FiberHardwareProfile::new(
        input.hardware.width,
        input.hardware.min_bend_radius,
        input.hardware.min_spacing,
        input.hardware.max_spacing,
    );
    let result = np_fiber::run_stress_isocurve_pipeline(&np_fiber::FiberPipelineInput {
        layers,
        stress,
        hardware,
        iso_levels: input.iso_levels,
    });
    convert_pipeline_result(result)
}

/// #15 bend gate exposed for TS unit tests / UI previews.
#[napi]
pub fn fiber_filter_bend_radius(
    polylines: Vec<JsFiberPolyline>,
    min_bend_radius: f64,
) -> Vec<JsFiberPolyline> {
    let native: Vec<np_fiber::FiberPolyline> = polylines
        .into_iter()
        .map(|p| np_fiber::FiberPolyline {
            id: p.id,
            layer_id: p.layer_id,
            points: p.points.iter().map(pfrom).collect(),
            closed: p.closed,
        })
        .collect();
    np_fiber::filter_by_bend_radius(native, min_bend_radius)
        .into_iter()
        .map(|p| JsFiberPolyline {
            id: p.id,
            layer_id: p.layer_id,
            points: p.points.into_iter().map(pjs).collect(),
            closed: p.closed,
        })
        .collect()
}

fn convert_pipeline_result(result: np_fiber::FiberPipelineResult) -> JsFiberPipelineResult {
    JsFiberPipelineResult {
        fields: result
            .fields
            .into_iter()
            .map(|f| JsStressField {
                layer_id: f.layer_id,
                samples: f
                    .samples
                    .into_iter()
                    .map(|s| JsScalarFieldSample {
                        position: pjs(s.position),
                        value: s.value,
                    })
                    .collect(),
            })
            .collect(),
        centerlines: result
            .centerlines
            .into_iter()
            .map(|p| JsFiberPolyline {
                id: p.id,
                layer_id: p.layer_id,
                points: p.points.into_iter().map(pjs).collect(),
                closed: p.closed,
            })
            .collect(),
        fibers: result
            .fibers
            .into_iter()
            .map(|f| JsFiberPath {
                id: f.id,
                layer_id: f.layer_id,
                points: f.points.into_iter().map(pjs).collect(),
                closed: p.closed,
            })
            .collect(),
        matrix: result
            .matrix
            .into_iter()
            .map(|m| JsMatrixSegment {
                layer_id: m.layer_id,
                start: pjs(m.start),
                end: pjs(m.end),
            })
            .collect(),
        dual_extrude: JsDualExtrudePlan {
            fiber_paths: result
                .dual_extrude
                .fiber_paths
                .into_iter()
                .map(|t| JsTcpPath {
                    id: t.id,
                    points: t
                        .points
                        .into_iter()
                        .map(|q| JsToolpathPoint {
                            position: arr_to_js(q.position),
                            tool_direction: arr_to_js(q.tool_direction),
                            extrusion: q.extrusion,
                            fiber_on: q.fiber_on,
                        })
                        .collect(),
                })
                .collect(),
            matrix_paths: result
                .dual_extrude
                .matrix_paths
                .into_iter()
                .map(|t| JsTcpPath {
                    id: t.id,
                    points: t
                        .points
                        .into_iter()
                        .map(|q| JsToolpathPoint {
                            position: arr_to_js(q.position),
                            tool_direction: arr_to_js(q.tool_direction),
                            extrusion: q.extrusion,
                            fiber_on: q.fiber_on,
                        })
                        .collect(),
                })
                .collect(),
        },
    }
}
