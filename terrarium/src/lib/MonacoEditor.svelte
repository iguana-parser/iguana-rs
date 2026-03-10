<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";
  import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";

  // Set up Monaco workers
  self.MonacoEnvironment = {
    getWorker() {
      return new editorWorker();
    },
  };

  interface Props {
    value?: string;
    language?: string;
    onchange?: (value: string) => void;
  }

  let { value = $bindable(""), language = "plaintext", onchange }: Props = $props();

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let ignoreChange = false;

  onMount(() => {
    editor = monaco.editor.create(container, {
      value,
      language,
      theme: "vs-dark",
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 14,
      fontFamily: "'Fira Code', 'Consolas', monospace",
      scrollBeyondLastLine: false,
      renderLineHighlight: "line",
      padding: { top: 8 },
    });

    editor.onDidChangeModelContent(() => {
      if (ignoreChange) return;
      const newValue = editor.getValue();
      value = newValue;
      onchange?.(newValue);
    });
  });

  onDestroy(() => {
    editor?.dispose();
  });

  // Sync external value changes into the editor
  $effect(() => {
    if (editor && value !== editor.getValue()) {
      ignoreChange = true;
      editor.setValue(value);
      ignoreChange = false;
    }
  });
</script>

<div class="monaco-container" bind:this={container}></div>

<style>
  .monaco-container {
    width: 100%;
    height: 100%;
  }
</style>
