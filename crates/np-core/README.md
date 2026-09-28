# np-core — #42 Reinforced FDM

Clean-room stress-aligned curved-layer pipeline. It ingests principal-stress samples, builds a governing scalar, emits curved layer surfaces, stress-aligned fill, and a TCP handoff stub. External FEA and collision solvers remain documented seams.

`np-native` exposes `core_run(input_json) -> json`; Fiber consumes the locked `LayerSurface` contract.
