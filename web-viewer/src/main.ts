import { mount } from "svelte";
import cytoscape from "cytoscape";
import tidytree from "cytoscape-tidytree";
import App from "./App.svelte";
import "./app.css";

console.info("[iguana] loading editor and graph libraries from CDN (cached after first load)...");

// The parse view lays the parse-tree graph out with cytoscape-tidytree and edits
// input in a Monaco editor. Both expect the host to register them, the same setup
// Terrarium does. Do it once, before any parse view mounts.
cytoscape.use(tidytree);

// Monaco loads from the CDN (see the importmap), so its editor worker does too.
// A worker script must be same-origin, so wrap the cross-origin module in a blob
// (the blob URL is same-origin to this page) that imports it. The input editor
// is plaintext, so only the base editor worker is ever requested.
self.MonacoEnvironment = {
  getWorker() {
    const workerUrl = "https://esm.sh/monaco-editor@0.56.0/esm/vs/editor/editor.worker.js";
    const blob = new Blob([`import ${JSON.stringify(workerUrl)};`], { type: "text/javascript" });
    return new Worker(URL.createObjectURL(blob), { type: "module" });
  },
};

const app = mount(App, { target: document.getElementById("app")! });

export default app;
