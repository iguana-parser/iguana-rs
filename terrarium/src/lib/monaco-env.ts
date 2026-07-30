import editorWorker from "monaco-editor/editor/editor.worker.js?worker";

// Monaco needs its editor web worker configured before any editor mounts. The
// iggy editor now lives in the shared web-ui package, which leaves worker setup
// to the host, so the page imports this module for its side effect.
self.MonacoEnvironment = {
  getWorker() {
    return new editorWorker();
  },
};
