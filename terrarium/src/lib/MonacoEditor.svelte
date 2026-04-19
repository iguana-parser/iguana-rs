<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";
  import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
  import { invoke } from "@tauri-apps/api/core";

  // Set up Monaco workers
  self.MonacoEnvironment = {
    getWorker() {
      return new editorWorker();
    },
  };

  // Register iggy language and semantic tokens provider (once)
  let iggyRegistered = false;
  let onAnalyzeCallback: ((result: { success: boolean; parse_duration_ms: number; tree_construction_duration_ms: number }) => void) | undefined;
  let editorDisabled = false;

  function registerIggyLanguage() {
    if (iggyRegistered) return;
    iggyRegistered = true;

    monaco.languages.register({ id: "iggy" });

    // Language configuration — tells Monaco how to toggle comments (Cmd+/).
    monaco.languages.setLanguageConfiguration("iggy", {
      comments: { lineComment: "//" },
    });

    // Monarch baseline tokenizer — provides instant, synchronous syntax
    // highlighting so the editor is never "white". Semantic tokens from the
    // backend are layered on top and override these where they apply.
    monaco.languages.setMonarchTokensProvider("iggy", {
      keywords: ["grammar", "layout", "left", "right", "none"],
      tokenizer: {
        root: [
          [/\/\/.*$/, "comment"],
          [/@(regex|NoLayout|Layout)\b/, "decorator"],
          [/#[A-Za-z_]\w*/, "comment"],           // labels
          [/"[^"]*"/, "string"],
          [/'[^']*'/, "string"],
          [/!>>|!<</, "operator"],
          [/[=|>*+?!:\\(){}\[\]\-]/, "operator"],
          [
            /[A-Za-z_]\w*/,
            { cases: { "@keywords": "keyword", "@default": "type" } },
          ],
        ],
      },
    });

    // Define theme rules for semantic token types
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

    // Fetch legend from backend, then register provider
    invoke<{ token_types: string[] }>("get_semantic_tokens_legend").then(
      (legend) => {
        monaco.languages.registerDocumentSemanticTokensProvider("iggy", {
          getLegend() {
            return {
              tokenTypes: legend.token_types,
              tokenModifiers: [],
            };
          },
          async provideDocumentSemanticTokens(model) {
            if (editorDisabled) return { data: new Uint32Array(0) };

            // First: parse and cache the grammar
            const analyzeResult = await invoke<{
              success: boolean;
              duration_ms: number;
            }>("analyze_grammar", { source: model.getValue() });

            onAnalyzeCallback?.(analyzeResult);

            // Push diagnostics (unresolved references, etc.)
            type DiagData = { range: Range; severity: number; message: string };
            const diags = await invoke<DiagData[]>("get_diagnostics", {
              source: model.getValue(),
            });
            monaco.editor.setModelMarkers(
              model,
              "iggy",
              diags.map((d) => ({
                ...toRange(d.range),
                severity: d.severity,
                message: d.message,
              })),
            );

            // Then: extract semantic tokens from the cached parse result
            const tokens = await invoke<
              {
                delta_line: number;
                delta_start: number;
                length: number;
                token_type: number;
                token_modifiers_bitset: number;
              }[]
            >("get_semantic_tokens");

            const data = new Uint32Array(tokens.length * 5);
            for (let i = 0; i < tokens.length; i++) {
              const t = tokens[i];
              data[i * 5] = t.delta_line;
              data[i * 5 + 1] = t.delta_start;
              data[i * 5 + 2] = t.length;
              data[i * 5 + 3] = t.token_type;
              data[i * 5 + 4] = t.token_modifiers_bitset;
            }

            return { data };
          },
          releaseDocumentSemanticTokens() {},
        });
      },
    );

    type Range = { start_line: number; start_char: number; end_line: number; end_char: number };
    const toRange = (r: Range) => ({
      startLineNumber: r.start_line + 1,
      startColumn: r.start_char + 1,
      endLineNumber: r.end_line + 1,
      endColumn: r.end_char + 1,
    });

    // Document symbols (Cmd+O / quick outline). Passes the current source
    // so the backend can ensure the parse result is fresh for this version.
    monaco.languages.registerDocumentSymbolProvider("iggy", {
      displayName: "Iggy",
      async provideDocumentSymbols(model) {
        type Sym = {
          name: string;
          kind: number;
          range: Range;
          selection_range: Range;
          children: Sym[];
        };
        const symbols = await invoke<Sym[]>("get_document_symbols", {
          source: model.getValue(),
        });
        const convert = (s: Sym): monaco.languages.DocumentSymbol => ({
          name: s.name,
          detail: "",
          // LSP SymbolKind codes are 1-based; Monaco's enum is 0-based.
          kind: s.kind - 1,
          tags: [],
          range: toRange(s.range),
          selectionRange: toRange(s.selection_range),
          children: s.children.map(convert),
        });
        return symbols.map(convert);
      },
    });

    // Go to Definition (F12, F3 bound separately below)
    monaco.languages.registerDefinitionProvider("iggy", {
      async provideDefinition(model, position) {
        const loc = await invoke<{ range: Range } | null>("get_definition", {
          source: model.getValue(),
          line: position.lineNumber - 1,
          column: position.column - 1,
        });
        if (!loc) return null;
        return {
          uri: model.uri,
          range: toRange(loc.range),
        };
      },
    });

    // Find All References (Shift+F12)
    monaco.languages.registerReferenceProvider("iggy", {
      async provideReferences(model, position, context) {
        const locs = await invoke<{ range: Range }[]>("get_references", {
          source: model.getValue(),
          line: position.lineNumber - 1,
          column: position.column - 1,
          includeDeclaration: context.includeDeclaration,
        });
        return locs.map((loc) => ({
          uri: model.uri,
          range: toRange(loc.range),
        }));
      },
    });

    // Code Folding
    monaco.languages.registerFoldingRangeProvider("iggy", {
      async provideFoldingRanges(model) {
        const ranges = await invoke<{ start_line: number; end_line: number }[]>(
          "get_folding_ranges",
          { source: model.getValue() },
        );
        return ranges.map((r) => ({
          start: r.start_line + 1,
          end: r.end_line + 1,
          kind: monaco.languages.FoldingRangeKind.Region,
        }));
      },
    });
  }

  interface Props {
    value?: string;
    language?: string;
    disabled?: boolean;
    onchange?: (value: string) => void;
    onanalyze?: (result: { success: boolean; parse_duration_ms: number; tree_construction_duration_ms: number }) => void;
    onready?: (editor: monaco.editor.IStandaloneCodeEditor) => void;
  }

  let { value = $bindable(""), language = "plaintext", disabled = false, onchange, onanalyze, onready }: Props = $props();

  // Keep the module-level callback in sync with the prop
  $effect(() => {
    onAnalyzeCallback = onanalyze;
  });

  // Keep the module-level disabled flag in sync and block all interaction via CSS
  $effect(() => {
    editorDisabled = disabled;
    if (container) {
      container.style.pointerEvents = disabled ? "none" : "";
      container.style.opacity = disabled ? "0.4" : "";
    }
    editor?.updateOptions({
      lineNumbers: disabled ? "off" : "on",
    });
  });

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let ignoreChange = false;

  onMount(() => {
    if (language === "iggy") {
      registerIggyLanguage();
    }

    editorDisabled = disabled;
    if (disabled) {
      container.style.pointerEvents = "none";
      container.style.opacity = "0.4";
    }
    editor = monaco.editor.create(container, {
      value,
      language,
      lineNumbers: disabled ? "off" : "on",
      theme: language === "iggy" ? "iggy-dark" : "vs-dark",
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 14,
      fontFamily: "'Fira Code', 'Consolas', monospace",
      scrollBeyondLastLine: false,
      renderLineHighlight: "line",
      padding: { top: 8 },
      "semanticHighlighting.enabled": true,
    });

    // -- Actions ----------------------------------------------------------
    // Registered via addAction so they appear in the command palette
    // (Cmd+Shift+P) with their labels and keybindings.
    editor.addAction({
      id: "terrarium.openGrammar",
      label: "Open Grammar",
      run: () => window.dispatchEvent(new CustomEvent("terrarium-open-grammar")),
    });
    editor.addAction({
      id: "terrarium.generate",
      label: "Generate Parser",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyG],
      run: () => window.dispatchEvent(new CustomEvent("terrarium-generate")),
    });
    editor.addAction({
      id: "terrarium.parse",
      label: "Parse Input",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP],
      run: () => window.dispatchEvent(new CustomEvent("terrarium-parse")),
    });
    editor.addAction({
      id: "terrarium.mode.design",
      label: "Switch to Design Mode",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit1],
      run: () => window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "design" })),
    });
    editor.addAction({
      id: "terrarium.mode.parse",
      label: "Switch to Parse Mode",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit2],
      run: () => window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "parse" })),
    });
    editor.addAction({
      id: "terrarium.mode.debug",
      label: "Switch to Debug Mode",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit3],
      run: () => window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "debug" })),
    });
    editor.addAction({
      id: "terrarium.formatGrammar",
      label: "Format Grammar",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF],
      run: async () => {
        const model = editor.getModel();
        if (!model) return;
        const source = model.getValue();
        const formatted = await invoke<string | null>("format_grammar", { source });
        if (formatted === null || formatted === source) return;
        // Uses executeEdits (not setValue) to avoid resetting semantic tokens,
        // which would cause a white flash while tokens are re-fetched.
        editor.executeEdits("format", [
          { range: model.getFullModelRange(), text: formatted },
        ]);
      },
    });

    // -- Keybinding rules -----------------------------------------------
    // Declarative remapping for built-in Monaco actions.
    monaco.editor.addKeybindingRules([
      // Unbind Monaco built-ins that conflict with Terrarium shortcuts
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP, command: "-editor.action.quickCommand" },
      { keybinding: monaco.KeyCode.F3, command: "-editor.action.nextMatchFindAction" },
      // F3: Go to Definition
      { keybinding: monaco.KeyCode.F3, command: "editor.action.revealDefinition" },
      // Cmd+O: Quick Outline
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyO, command: "editor.action.quickOutline" },
      // Cmd+Shift+P: Command palette
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyP, command: "editor.action.quickCommand" },
      // Cmd+D: Delete line
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyD, command: "editor.action.deleteLines" },
      // Cmd+[/]: Cursor back/forward (unbind indent/outdent)
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketLeft, command: "-editor.action.outdentLines" },
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketRight, command: "-editor.action.indentLines" },
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketLeft, command: "cursorUndo" },
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.BracketRight, command: "cursorRedo" },
    ]);

    editor.onDidChangeModelContent(() => {
      if (ignoreChange) return;
      const newValue = editor.getValue();
      value = newValue;
      onchange?.(newValue);
    });

    onready?.(editor);
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
