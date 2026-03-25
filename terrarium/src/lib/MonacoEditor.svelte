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
      colors: {},
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
  }

  interface Props {
    value?: string;
    language?: string;
    onchange?: (value: string) => void;
    onanalyze?: (result: { success: boolean; parse_duration_ms: number; tree_construction_duration_ms: number }) => void;
  }

  let { value = $bindable(""), language = "plaintext", onchange, onanalyze }: Props = $props();

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

    // Override Cmd+G / Ctrl+G: Monaco uses it for "Find Next", but Terrarium
    // uses it globally for "Generate Parser". Dispatch a custom event that the
    // global handler in +page.svelte listens for.
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyG, () => {
      window.dispatchEvent(new CustomEvent("terrarium-generate"));
    });

    // Cmd+D / Ctrl+D: Delete current line
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyD, () => {
      editor.getAction("editor.action.deleteLines")?.run();
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
