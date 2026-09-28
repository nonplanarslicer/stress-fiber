import { emitGcodeStub, type LoopCandidate, type PackOutput } from "@nonplanarslicer/api";

export function ExportPanel({
  pack,
  candidates,
}: {
  pack: PackOutput | undefined;
  candidates: LoopCandidate[];
}) {
  const download = (kind: "gcode" | "json") => {
    if (!pack) return;
    const body =
      kind === "gcode"
        ? emitGcodeStub(pack, candidates)
        : JSON.stringify(pack, null, 2);
    const blob = new Blob([body], {
      type: kind === "gcode" ? "text/plain" : "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = kind === "gcode" ? "np-pack-stub.gcode" : "np-pack-result.json";
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <section className="export panel-block">
      <h2>Export</h2>
      <p className="muted">
        TCP / G-code stub with placeholder fiber feed commands (M700–M703). Not machine-validated.
      </p>
      <div className="export-actions">
        <button type="button" disabled={!pack} onClick={() => download("gcode")}>
          Download G-code stub
        </button>
        <button type="button" className="ghost" disabled={!pack} onClick={() => download("json")}>
          Download JSON
        </button>
      </div>
    </section>
  );
}
