# Ownership

- **NP Rust Core** owns `crates/np-core` (stress field, iso layers, fill, TCP-facing path data).
- **NP Fiber Fields** owns `crates/np-fiber` (continuous-fiber direction, bend/cut/restart constraints).
- **NP Pack UI** owns `crates/np-pack`, `packages/app`, and the product README (packing, UI, and operator-facing workflow).
- **Shared native bridge**: `crates/np-native` is coordinated by all three owners. Keep its exported JSON/API surface backward compatible and review cross-family changes together.

The algorithms here are clean-room interfaces and scaffolding. Specialists should replace placeholders with validated implementations and tests without inventing performance claims.
