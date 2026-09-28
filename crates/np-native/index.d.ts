/* Auto / placeholder TypeScript types for @nonplanarslicer/native (Family E #15).
 * Regenerate with `napi build` when the addon is compiled for a host platform.
 */

export interface JsPoint3 {
  x: number
  y: number
  z: number
}

/** External #42 layer contract consumed by fiber (#15/#10/#52). */
export interface JsLayerSurface {
  id: number
  /** Flat xyz triplets. */
  points: Array<number>
  normals: Array<number>
  thickness: number
  sequence: number
}

export interface JsStressSample {
  position: JsPoint3
  principal_direction: JsPoint3
  magnitude: number
}

/** Caller-supplied machine limits — not paper constants. */
export interface JsFiberHardwareProfile {
  width: number
  min_bend_radius: number
  min_spacing: number
  max_spacing: number
}

export interface JsScalarFieldSample {
  position: JsPoint3
  value: number
}

export interface JsStressField {
  layer_id: number
  samples: Array<JsScalarFieldSample>
}

export type JsScalarField = JsStressField

export interface JsFiberPolyline {
  id: number
  layer_id: number
  points: Array<JsPoint3>
  closed: boolean
}

export interface JsFiberPath {
  id: number
  polyline: JsFiberPolyline
  printable: boolean
}

export interface JsToolpathPoint {
  position: JsPoint3
  tool_direction: JsPoint3
  extrusion: number
  fiber_on: boolean
}

export interface JsTcpPath {
  id: number
  points: Array<JsToolpathPoint>
}

export interface JsDualExtrudePlan {
  fiber_paths: Array<JsTcpPath>
  matrix_paths: Array<JsTcpPath>
}

export interface JsMatrixSegment {
  layer_id: number
  start: JsPoint3
  end: JsPoint3
}

export interface JsFiberPipelineInput {
  layers: Array<JsLayerSurface>
  stress: Array<JsStressSample>
  hardware: JsFiberHardwareProfile
  iso_levels: Array<number>
}

/** #15 pipeline result. */
export interface JsFiberPipelineResult {
  fields: Array<JsStressField>
  centerlines: Array<JsFiberPolyline>
  fibers: Array<JsFiberPath>
  matrix: Array<JsMatrixSegment>
  dual_extrude: JsDualExtrudePlan
}

export declare function core_run(inputJson: string): string
export declare function fiber_run(inputJson: string): string
export declare function pack_run(inputJson: string): string
export declare function fiber_run_pipeline(input: JsFiberPipelineInput): JsFiberPipelineResult
export declare function fiber_filter_bend_radius(
  polylines: Array<JsFiberPolyline>,
  minBendRadius: number,
): Array<JsFiberPolyline>
