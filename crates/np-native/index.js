// Placeholder until `napi build` produces the platform .node binary.
// Desktop/Node hosts should load the built addon and re-export these names.
throw new Error(
  "@nonplanarslicer/native: native addon not built. Run `pnpm --filter @nonplanarslicer/native build`."
)
