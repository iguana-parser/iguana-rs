<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";
  import type { LspBackend, LspDocumentSymbol, LspRange } from "./lsp-backend";

  // The Monaco language providers are registered once per page, not per editor,
  // so they read the active instance's backend and disabled flag from module
  // scope. A page mounts a single DesignView, so last-mount-wins is moot here.
  let activeBackend: LspBackend | undefined;
  let activeDisabled = false;
  let iggyRegistered = false;

  // Registers the iggy language, its baseline tokenizer and theme, and the LSP
  // providers (all backed by `activeBackend`). Runs once; later mounts reuse it.
  function registerIggyLanguage() {
    if (iggyRegistered) return;
    iggyRegistered = true;

    monaco.languages.register({ id: "iggy" });

    // Language configuration tells Monaco how to toggle comments (Cmd+/).
    monaco.languages.setLanguageConfiguration("iggy", {
      comments: { lineComment: "//" },
    });

    // Monarch baseline tokenizer: instant, synchronous syntax highlighting so
    // the editor is never blank. Semantic tokens from the backend layer on top
    // and override these where they apply.
    monaco.languages.setMonarchTokensProvider("iggy", {
      keywords: ["grammar", "layout", "left", "right", "none"],
      tokenizer: {
        root: [
          [/\/\/.*$/, "comment"],
          [/@(regex|NoLayout|Layout)\b/, "decorator"],
          [/#[A-Za-z_]\w*/, "comment"], // labels
          [/"[^"]*"/, "string"],
          [/'[^']*'/, "string"],
          [/!>>|!<</, "operator"],
          [/[=|>*+?!:\\(){}\[\]\-]/, "operator"],
          [/[A-Za-z_]\w*/, { cases: { "@keywords": "keyword", "@default": "type" } }],
        ],
      },
    });

    // Theme rules for the semantic token types.
    monaco.editor.defineTheme("iggy-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [
        { token: "keyword", foreground: "569cd6", fontStyle: "bold" },
        { token: "type", foreground: "4ec9b0" },
        { token: "string", foreground: "ce9178" },
        { token: "regexp", foreground: "d16969" },
        { token: "operator", foreground: "d4d4d4" },
        { token: "decorator", foreground: "dcdcaa" },
        { token: "comment", foreground: "808080" }, // labels
      ],
      colors: {
        "editor.inactiveSelectionBackground": "#264f78",
        "editor.selectionHighlightBackground": "#264f784d",
        "editor.wordHighlightBackground": "#264f784d",
        "editor.wordHighlightStrongBackground": "#264f784d",
        "editor.rangeHighlightBackground": "#264f78",
        "editor.rangeHighlightBorder": "#00000000",
      },
    });

    // Fetch the legend from the backend, then register the semantic tokens
    // provider against it.
    activeBackend?.semanticTokensLegend().then((tokenTypes) => {
      monaco.languages.registerDocumentSemanticTokensProvider("iggy", {
        getLegend() {
          return { tokenTypes, tokenModifiers: [] };
        },
        async provideDocumentSemanticTokens(model) {
          if (activeDisabled || !activeBackend) return { data: new Uint32Array(0) };
          const source = model.getValue();

          // Diagnostics (unresolved references, etc.) share this parse trigger,
          // which fires on load and on every edit.
          const diags = await activeBackend.diagnostics(source);
          monaco.editor.setModelMarkers(
            model,
            "iggy",
            diags.map((d) => ({
              ...toMonacoRange(d.range),
              severity: d.severity,
              message: d.message,
            })),
          );

          const data = await activeBackend.semanticTokens(source);
          return { data };
        },
        releaseDocumentSemanticTokens() {},
      });
    });

    // Document symbols (Cmd+O quick outline).
    monaco.languages.registerDocumentSymbolProvider("iggy", {
      displayName: "Iggy",
      async provideDocumentSymbols(model) {
        if (!activeBackend) return [];
        const symbols = await activeBackend.documentSymbols(model.getValue());
        const convert = (s: LspDocumentSymbol): monaco.languages.DocumentSymbol => ({
          name: s.name,
          detail: "",
          // LSP SymbolKind codes are one-based; Monaco's enum is zero-based.
          kind: s.kind - 1,
          tags: [],
          range: toMonacoRange(s.range),
          selectionRange: toMonacoRange(s.selectionRange),
          children: (s.children ?? []).map(convert),
        });
        return symbols.map(convert);
      },
    });

    // Go to Definition (F12).
    monaco.languages.registerDefinitionProvider("iggy", {
      async provideDefinition(model, position) {
        if (!activeBackend) return null;
        const loc = await activeBackend.definition(
          model.getValue(),
          position.lineNumber - 1,
          position.column - 1,
        );
        if (!loc) return null;
        return { uri: model.uri, range: toMonacoRange(loc.range) };
      },
    });

    // Find All References (Shift+F12).
    monaco.languages.registerReferenceProvider("iggy", {
      async provideReferences(model, position, context) {
        if (!activeBackend) return [];
        const locs = await activeBackend.references(
          model.getValue(),
          position.lineNumber - 1,
          position.column - 1,
          context.includeDeclaration,
        );
        return locs.map((loc) => ({ uri: model.uri, range: toMonacoRange(loc.range) }));
      },
    });

    // Code folding.
    monaco.languages.registerFoldingRangeProvider("iggy", {
      async provideFoldingRanges(model) {
        if (!activeBackend) return [];
        const ranges = await activeBackend.folding(model.getValue());
        return ranges.map((r) => ({
          start: r.startLine + 1,
          end: r.endLine + 1,
          kind: monaco.languages.FoldingRangeKind.Region,
        }));
      },
    });
  }

  // Convert an LSP range (zero-based line and character) to a Monaco range
  // (one-based line and column).
  function toMonacoRange(r: LspRange) {
    return {
      startLineNumber: r.start.line + 1,
      startColumn: r.start.character + 1,
      endLineNumber: r.end.line + 1,
      endColumn: r.end.character + 1,
    };
  }

  interface Props {
    value?: string;
    backend: LspBackend;
    // Dims the editor and blocks interaction, for a host that has no grammar
    // loaded yet. Distinct from `readOnly`, which keeps the editor interactive.
    disabled?: boolean;
    readOnly?: boolean;
    onchange?: (value: string) => void;
    // Receives the editor once created, so a host can add its own actions and
    // keybindings (the shared component registers none of its own).
    onready?: (editor: monaco.editor.IStandaloneCodeEditor) => void;
    initialViewState?: monaco.editor.ICodeEditorViewState | null;
    onSaveViewState?: (state: monaco.editor.ICodeEditorViewState | null) => void;
  }

  let {
    value = $bindable(""),
    backend,
    disabled = false,
    readOnly = false,
    onchange,
    onready,
    initialViewState,
    onSaveViewState,
  }: Props = $props();

  // Keep the module-level backend the global providers read in sync with this
  // instance's prop.
  $effect(() => {
    activeBackend = backend;
  });

  // Keep the module-level disabled flag in sync and block interaction via CSS.
  $effect(() => {
    activeDisabled = disabled;
    if (container) {
      container.style.pointerEvents = disabled ? "none" : "";
      container.style.opacity = disabled ? "0.4" : "";
    }
    editor?.updateOptions({ lineNumbers: disabled ? "off" : "on" });
  });

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let ignoreChange = false;

  onMount(() => {
    activeBackend = backend;
    registerIggyLanguage();

    activeDisabled = disabled;
    if (disabled) {
      container.style.pointerEvents = "none";
      container.style.opacity = "0.4";
    }
    editor = monaco.editor.create(container, {
      value,
      language: "iggy",
      lineNumbers: disabled ? "off" : "on",
      theme: "iggy-dark",
      readOnly,
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 14,
      fontFamily: "'Fira Code', 'Consolas', monospace",
      scrollBeyondLastLine: false,
      renderLineHighlight: "line",
      padding: { top: 8 },
      "semanticHighlighting.enabled": true,
      colorDecorators: false,
    });

    editor.onDidChangeModelContent(() => {
      if (ignoreChange) return;
      const newValue = editor.getValue();
      value = newValue;
      onchange?.(newValue);
    });

    // Restore cursor, scroll, and selection from the previous mount so a mode
    // switch returns the editor to where the user left it.
    if (initialViewState) editor.restoreViewState(initialViewState);

    onready?.(editor);
  });

  onDestroy(() => {
    onSaveViewState?.(editor?.saveViewState() ?? null);
    editor?.dispose();
  });

  // Sync external value changes into the editor.
  $effect(() => {
    if (editor && value !== editor.getValue()) {
      ignoreChange = true;
      editor.setValue(value);
      ignoreChange = false;
    }
  });

  // Sync readOnly.
  $effect(() => {
    editor?.updateOptions({ readOnly });
  });
</script>

<div class="design-view-container" bind:this={container}></div>

<style>
  .design-view-container {
    width: 100%;
    height: 100%;
  }
</style>
