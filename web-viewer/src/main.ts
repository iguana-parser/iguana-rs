import { mount } from "svelte";
import cytoscape from "cytoscape";
import tidytree from "cytoscape-tidytree";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import App from "./App.svelte";
import "./app.css";

// The parse view lays the parse-tree graph out with cytoscape-tidytree and edits
// input in a Monaco editor. Both expect the host to register them, the same setup
// Terrarium does. Do it once, before any parse view mounts.
cytoscape.use(tidytree);
self.MonacoEnvironment = {
  getWorker() {
    return new editorWorker();
  },
};

const app = mount(App, { target: document.getElementById("app")! });

export default app;
