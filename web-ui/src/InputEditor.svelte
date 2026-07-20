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

  interface AmbiguityWarning {
    start: number;
    end: number;
    message: string;
  }

  interface Props {
    value?: string;
    readOnly?: boolean;
    error?: ErrorInfo | null;
    ambiguities?: AmbiguityWarning[];
    highlightSpan?: HighlightSpan | null;
    consumedUntil?: number | null;
    currentIndex?: number | null;
    placeholder?: string;
    onchange?: (value: string) => void;
    onclick?: (offset: number) => void;
    onescape?: () => void;
    initialViewState?: monaco.editor.ICodeEditorViewState | null;
    onSaveViewState?: (state: monaco.editor.ICodeEditorViewState | null) => void;
  }

  let {
    value = $bindable(""),
    readOnly = false,
    error = null,
    ambiguities = [],
    highlightSpan = null,
    consumedUntil = null,
    currentIndex = null,
    placeholder = "",
    onchange,
    onclick,
    onescape,
    initialViewState,
    onSaveViewState,
  }: Props = $props();

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let ignoreChange = false;
  let decorationIds: string[] = [];

  onMount(() => {
    // The ESM Monaco build ships no styles; load its stylesheet on first use so
    // hosts that use the plain editor never fetch it (see web-viewer/index.html).
    if (!document.querySelector("link[data-monaco-css]")) {
      const link = document.createElement("link");
      link.rel = "stylesheet";
      link.href =
        "https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs/editor/editor.main.css";
      link.setAttribute("data-monaco-css", "");
      document.head.appendChild(link);
    }

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
      colorDecorators: false,
      placeholder,
    });

    editor.onDidChangeModelContent(() => {
      if (ignoreChange) return;
      const newValue = editor.getValue();
      value = newValue;
      onchange?.(newValue);
    });

    editor.onMouseDown((e) => {
      if (!onclick) return;
      const pos = e.target.position;
      const model = editor.getModel();
      if (!pos || !model) return;
      onclick(model.getOffsetAt(pos));
    });

    editor.onKeyDown((e) => {
      if (e.keyCode === monaco.KeyCode.Escape) {
        onescape?.();
      }
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Period, () => {
      editor.trigger("keyboard", "editor.action.marker.next", null);
    });
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyL, () => {
      editor.trigger("keyboard", "editor.action.gotoLine", null);
    });

    // Restore cursor, scroll, and selection from the previous mount so a mode
    // switch returns the editor to where the user left it.
    if (initialViewState) editor.restoreViewState(initialViewState);
  });

  onDestroy(() => {
    onSaveViewState?.(editor?.saveViewState() ?? null);
    editor?.dispose();
  });

  // Exported so a host can focus the input editor, e.g. on a mode switch.
  export function focus() {
    editor?.focus();
  }

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

  // Update ambiguity markers (warnings, one per outermost Amb span)
  $effect(() => {
    const model = editor?.getModel();
    if (!model) return;

    const markers = ambiguities.map((a) => {
      const startPos = model.getPositionAt(a.start);
      const endPos = model.getPositionAt(a.end);
      return {
        startLineNumber: startPos.lineNumber,
        startColumn: startPos.column,
        endLineNumber: endPos.lineNumber,
        endColumn: endPos.column,
        severity: monaco.MarkerSeverity.Warning,
        message: a.message,
      };
    });
    monaco.editor.setModelMarkers(model, "ambiguity", markers);
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

    // Selected span highlight
    if (highlightSpan) {
      const startPos = model.getPositionAt(highlightSpan.start);
      if (highlightSpan.start < highlightSpan.end) {
        // Non-empty span: blue background
        const endPos = model.getPositionAt(highlightSpan.end);
        decorations.push({
          range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
          options: { className: "input-highlight-span" },
        });
      } else {
        // Empty span: vertical bar at position
        decorations.push({
          range: new monaco.Range(startPos.lineNumber, startPos.column, startPos.lineNumber, startPos.column),
          options: { className: "input-highlight-caret" },
        });
      }
    }

    decorationIds = editor.deltaDecorations(decorationIds, decorations);
  });

  // Scroll the highlighted span into view when it changes (e.g. selecting a node
  // in the parse tree / SPPF / s-expression). Depends only on highlightSpan, so a
  // selection reveals its text, but unrelated decoration updates do not scroll.
  $effect(() => {
    if (!editor || !highlightSpan) return;
    const model = editor.getModel();
    if (!model) return;
    const startPos = model.getPositionAt(highlightSpan.start);
    const endPos = model.getPositionAt(highlightSpan.end);
    const range = new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column);
    editor.revealRangeInCenterIfOutsideViewport(range, monaco.editor.ScrollType.Smooth);
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

  :global(.input-highlight-caret) {
    border-left: 2px solid #569cd6;
  }
</style>
