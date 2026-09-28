# @nonplanarslicer/app

Vite + React + TypeScript operator UI for continuous-fiber loop packing (#27 / #22).

No Next.js / Remix — plain Vite.

## Run

From the monorepo root:

```bash
pnpm install
pnpm --filter @nonplanarslicer/app dev
```

Or `pnpm dev` from root.

## Demo flow

1. Pipeline bar: FEA → #42 → #15|#52 → #10? → #27|#22 → D
2. Pick algorithm (#27 MILP vs #22 Deep-Q vs greedy) — hover for pick-guide
3. Set bend radius, spacing, budget, cut/restart
4. Run pack on synthetic demo loops (example data, not paper metrics)
5. Inspect SVG paths; export TCP/G-code stub or JSON

Uses `@nonplanarslicer/api` mocks until napi bindings are injected via `configureNativeBindings`.
