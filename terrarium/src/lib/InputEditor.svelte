<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";

  interface ErrorInfo {
    line: number;
    column: number;
    message: string;
  }

  interface HighlightSpan {
    start: number;
    end: number;
  }

  interface Props {
    value?: string;
    readOnly?: boolean;
    error?: ErrorInfo | null;
    highlightSpan?: HighlightSpan | null;
    consumedUntil?: number | null;
    currentIndex?: number | null;
    placeholder?: string;
    onchange?: (value: string) => void;
  }

  let {
    value = $bindable(""),
    readOnly = false,
    error = null,
    highlightSpan = null,
    consumedUntil = null,
    currentIndex = null,
    placeholder = "",
    onchange,
  }: Props = $props();

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let ignoreChange = false;
  let decorationIds: string[] = [];

  onMount(() => {
    editor = monaco.editor.create(container, {
      value,
      language: "plaintext",
      theme: "vs-dark",
      readOnly,
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 13,
      fontFamily: "'Fira Code', 'Consolas', monospace",
      scrollBeyondLastLine: false,
      renderLineHighlight: "none",
      lineNumbers: "on",
      renderWhitespace: "all",
      padding: { top: 8 },
      placeholder,
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

  // Sync readOnly
  $effect(() => {
    editor?.updateOptions({ readOnly });
  });

  // Update error markers
  $effect(() => {
    const model = editor?.getModel();
    if (!model) return;

    if (error) {
      // line_column from iguana runtime is 0-based; Monaco is 1-based
      const lineNumber = error.line + 1;
      const column = error.column + 1;
      const lineLength = model.getLineLength(lineNumber) || 1;
      monaco.editor.setModelMarkers(model, "parse-error", [
        {
          startLineNumber: lineNumber,
          startColumn: column,
          endLineNumber: lineNumber,
          endColumn: lineLength + 1,
          severity: monaco.MarkerSeverity.Error,
          message: `Parse error: ${error.message}`,
        },
      ]);
    } else {
      monaco.editor.setModelMarkers(model, "parse-error", []);
    }
  });

  // Update decorations (highlight span, consumed, current)
  $effect(() => {
    if (!editor) return;
    const model = editor.getModel();
    if (!model) return;

    const decorations: monaco.editor.IModelDeltaDecoration[] = [];

    // Consumed characters (green background)
    if (consumedUntil !== null && consumedUntil > 0) {
      const startPos = model.getPositionAt(0);
      const endPos = model.getPositionAt(consumedUntil);
      decorations.push({
        range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
        options: { className: "input-consumed" },
      });
    }

    // Current character (blue background)
    if (currentIndex !== null) {
      const startPos = model.getPositionAt(currentIndex);
      const endPos = model.getPositionAt(currentIndex + 1);
      decorations.push({
        range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
        options: { className: "input-current" },
      });
    }

    // Selected span highlight (blue background)
    if (highlightSpan) {
      const startPos = model.getPositionAt(highlightSpan.start);
      const endPos = model.getPositionAt(highlightSpan.end);
      decorations.push({
        range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
        options: { className: "input-highlight-span" },
      });
    }

    decorationIds = editor.deltaDecorations(decorationIds, decorations);
  });
</script>

<div class="input-editor-container" bind:this={container}></div>

<style>
  .input-editor-container {
    width: 100%;
    height: 100%;
  }

  :global(.input-consumed) {
    background-color: rgba(106, 153, 85, 0.3);
  }

  :global(.input-current) {
    background-color: #264f78;
  }

  :global(.input-highlight-span) {
    background-color: #264f78;
  }
</style>
