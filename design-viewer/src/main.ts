import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";

console.info("[iguana] loading the editor from CDN (cached after first load)...");

// Monaco loads from the CDN (see the importmap), so its editor worker does too.
// A worker script must be same-origin, so wrap the cross-origin module in a blob
// (the blob URL is same-origin to this page) that imports it. The design view is
// read-only iggy, so only the base editor worker is ever requested.
self.MonacoEnvironment = {
  getWorker() {
    const workerUrl = "https://esm.sh/monaco-editor@0.55.1/esm/vs/editor/editor.worker.js";
    const blob = new Blob([`import ${JSON.stringify(workerUrl)};`], { type: "text/javascript" });
    return new Worker(URL.createObjectURL(blob), { type: "module" });
  },
};

const app = mount(App, { target: document.getElementById("app")! });

export default app;
