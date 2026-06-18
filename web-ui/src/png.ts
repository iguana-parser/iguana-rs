import type { Core } from "cytoscape";

/**
 * Exports a Cytoscape graph as a PNG via a browser download. This is the
 * default the parse view uses when the host does not inject its own exporter
 * (Terrarium injects a native save dialog through `onExportPng`).
 */
export async function downloadPng(graph: Core | null, defaultName: string): Promise<void> {
  if (!graph) return;

  const blob = await graph.png({ output: "blob", bg: "#1e1e1e", scale: 2 });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `${defaultName}.png`;
  anchor.click();
  URL.revokeObjectURL(url);
}
