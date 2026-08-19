<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";

  import type { ParseError } from "./backend";
  import { charIndex, utf16OffsetTable } from "./char-offsets";

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
    error?: ParseError | null;
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

    // Monaco normalizes mixed line endings to one EOL per model, so the bound
    // value follows the model text. The parser then parses the exact string
    // the editor shows, and the runtime's character indexes line up with it.
    value = editor.getValue();

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
      onclick(charIndex(model.getValue(), model.getOffsetAt(pos)));
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

  // Sync external value changes into the editor. Reading the value back picks
  // up Monaco's line-ending normalization (see the mount-time sync); the
  // re-run this triggers finds the two equal and stops.
  $effect(() => {
    if (editor && value !== editor.getValue()) {
      ignoreChange = true;
      editor.setValue(value);
      ignoreChange = false;
      value = editor.getValue();
    }
  });

  // Sync readOnly
  $effect(() => {
    editor?.updateOptions({ readOnly });
  });

  // The runtime reports positions as character indexes, so every marker and
  // decoration converts through the offset table before asking Monaco for a
  // position; the click handler converts the other way. The table is rebuilt
  // only when the model text changes, so resolving many markers stays linear
  // in the marker count instead of rescanning the input per marker.
  let offsetTable: { versionId: number; offsets: Uint32Array | null } | null = null;

  function positionAt(model: monaco.editor.ITextModel, index: number): monaco.Position {
    const versionId = model.getVersionId();
    if (offsetTable === null || offsetTable.versionId !== versionId) {
      offsetTable = { versionId, offsets: utf16OffsetTable(model.getValue()) };
    }
    const { offsets } = offsetTable;
    const offset = offsets === null ? index : offsets[Math.min(index, offsets.length - 1)];
    return model.getPositionAt(offset);
  }

  // Update error markers
  $effect(() => {
    const model = editor?.getModel();
    if (!model) return;

    if (error) {
      const startPos = positionAt(model, error.span.left_extent);
      const endPos = positionAt(model, error.span.right_extent);
      // An empty span (a failure at the end of the input) still marks one
      // column; Monaco tolerates a column past the end of the line.
      const emptySpan = endPos.lineNumber === startPos.lineNumber && endPos.column === startPos.column;
      monaco.editor.setModelMarkers(model, "parse-error", [
        {
          startLineNumber: startPos.lineNumber,
          startColumn: startPos.column,
          endLineNumber: endPos.lineNumber,
          endColumn: emptySpan ? startPos.column + 1 : endPos.column,
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
      const startPos = positionAt(model, a.start);
      const endPos = positionAt(model, a.end);
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
      const startPos = positionAt(model, 0);
      const endPos = positionAt(model, consumedUntil);
      decorations.push({
        range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
        options: { className: "input-consumed" },
      });
    }

    // Current character (blue background)
    if (currentIndex !== null) {
      const startPos = positionAt(model, currentIndex);
      const endPos = positionAt(model, currentIndex + 1);
      decorations.push({
        range: new monaco.Range(startPos.lineNumber, startPos.column, endPos.lineNumber, endPos.column),
        options: { className: "input-current" },
      });
    }

    // Selected span highlight
    if (highlightSpan) {
      const startPos = positionAt(model, highlightSpan.start);
      if (highlightSpan.start < highlightSpan.end) {
        // Non-empty span: blue background
        const endPos = positionAt(model, highlightSpan.end);
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
    const startPos = positionAt(model, highlightSpan.start);
    const endPos = positionAt(model, highlightSpan.end);
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
