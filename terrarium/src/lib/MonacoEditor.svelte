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

  function registerIggyLanguage() {
    if (iggyRegistered) return;
    iggyRegistered = true;

    monaco.languages.register({ id: "iggy" });


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
            // First: parse and cache the grammar
            const analyzeResult = await invoke<{
              success: boolean;
              duration_ms: number;
            }>("analyze_grammar", { source: model.getValue() });

            onAnalyzeCallback?.(analyzeResult);

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
  }

  interface Props {
    value?: string;
    language?: string;
    onchange?: (value: string) => void;
    onanalyze?: (result: { success: boolean; parse_duration_ms: number; tree_construction_duration_ms: number }) => void;
    onready?: (editor: monaco.editor.IStandaloneCodeEditor) => void;
  }

  let { value = $bindable(""), language = "plaintext", onchange, onanalyze, onready }: Props = $props();

  // Keep the module-level callback in sync with the prop
  $effect(() => {
    onAnalyzeCallback = onanalyze;
  });

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let ignoreChange = false;

  onMount(() => {
    if (language === "iggy") {
      registerIggyLanguage();
    }

    editor = monaco.editor.create(container, {
      value,
      language,
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

    // -- Custom commands ------------------------------------------------
    // Register named commands so addKeybindingRules can reference them.
    monaco.editor.addCommand({
      id: "terrarium.generate",
      run: () => window.dispatchEvent(new CustomEvent("terrarium-generate")),
    });
    monaco.editor.addCommand({
      id: "terrarium.parse",
      run: () => window.dispatchEvent(new CustomEvent("terrarium-parse")),
    });
    monaco.editor.addCommand({
      id: "terrarium.mode.design",
      run: () => window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "design" })),
    });
    monaco.editor.addCommand({
      id: "terrarium.mode.parse",
      run: () => window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "parse" })),
    });
    monaco.editor.addCommand({
      id: "terrarium.mode.debug",
      run: () => window.dispatchEvent(new CustomEvent("terrarium-mode", { detail: "debug" })),
    });
    monaco.editor.addCommand({
      id: "terrarium.formatGrammar",
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
    // Declarative remapping via the public API (replaces the private
    // _standaloneKeybindingService hack). A null command unbinds Monaco's
    // built-in action for that key.
    monaco.editor.addKeybindingRules([
      // Cmd+G: Generate (unbind Monaco "Find Next")
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyG, command: "terrarium.generate" },
      // Cmd+P: Parse (unbind Monaco "Quick Command")
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP, command: "terrarium.parse" },
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyP, command: "-editor.action.quickCommand" },
      // Cmd+1/2/3: Switch mode
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit1, command: "terrarium.mode.design" },
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit2, command: "terrarium.mode.parse" },
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit3, command: "terrarium.mode.debug" },
      // F3: Go to Definition (unbind Monaco "Find Next Match")
      { keybinding: monaco.KeyCode.F3, command: "editor.action.revealDefinition" },
      { keybinding: monaco.KeyCode.F3, command: "-editor.action.nextMatchFindAction" },
      // Cmd+O: Quick Outline
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyO, command: "editor.action.quickOutline" },
      // Cmd+D: Delete line
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyD, command: "editor.action.deleteLines" },
      // Cmd+Shift+F: Format grammar
      { keybinding: monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF, command: "terrarium.formatGrammar" },
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
