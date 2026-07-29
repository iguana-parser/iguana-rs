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
          [/!>>>|!>>|!<</, "operator"],
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
    // Adds a toolbar strip beside the editor (find, outline toggle). Off by
    // default, so a host that wants only the editor renders it full-bleed as before.
    toolbar?: boolean;
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
    toolbar = false,
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

  // The toolbar and plain layouts each bind their own element, so a host that
  // flips `toolbar` swaps the node. $state keeps the disabled-styling effect
  // above re-running against whichever node is current.
  let container: HTMLDivElement | undefined = $state();
  let editor: monaco.editor.IStandaloneCodeEditor;
  let ignoreChange = false;

  // ── Outline pane ───────────────────────────────────────────────────────────
  // A toggleable tree of the grammar's rules (and their labelled alternatives),
  // read from the same document symbols the Cmd+O quick outline uses. Clicking a
  // row reveals it in the editor. Rendered only when the `outline` prop is set.
  interface OutlineItem {
    sym: LspDocumentSymbol;
    isChild: boolean;
  }

  let outlineOpen = $state(false);
  let outlineSymbols = $state<LspDocumentSymbol[]>([]);
  let outlineExpanded = $state<Set<string>>(new Set());
  let outlineSelected = $state(-1);
  let outlineRefreshTimer: ReturnType<typeof setTimeout> | undefined;

  // The rows currently visible: every top-level symbol, plus the children of the
  // expanded ones.
  function visibleOutlineItems(): OutlineItem[] {
    const items: OutlineItem[] = [];
    for (const sym of outlineSymbols) {
      items.push({ sym, isChild: false });
      if (sym.children?.length && outlineExpanded.has(sym.name)) {
        for (const child of sym.children) items.push({ sym: child, isChild: true });
      }
    }
    return items;
  }

  function toggleOutlineNode(name: string) {
    const next = new Set(outlineExpanded);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    outlineExpanded = next;
  }

  async function refreshOutline() {
    if (!activeBackend) return;
    const source = editor?.getValue() ?? value;
    try {
      outlineSymbols = await activeBackend.documentSymbols(source);
    } catch {
      outlineSymbols = [];
    }
  }

  // Re-parse for the outline only while it is open, debounced so an editable host
  // does not parse on every keystroke.
  function scheduleOutlineRefresh() {
    if (!outlineOpen) return;
    clearTimeout(outlineRefreshTimer);
    outlineRefreshTimer = setTimeout(refreshOutline, 250);
  }

  function toggleOutline() {
    outlineOpen = !outlineOpen;
    if (outlineOpen) refreshOutline();
  }

  function revealSymbol(sym: LspDocumentSymbol) {
    if (!editor) return;
    const range = toMonacoRange(sym.selectionRange);
    editor.revealRangeInCenter(range);
    editor.setSelection(range);
  }

  // Single-letter kind glyph: rules by LSP SymbolKind (5 = Class, 11 = Interface),
  // labelled alternatives as a child mark.
  function outlineGlyph(item: OutlineItem): string {
    if (item.isChild) return "#";
    return item.sym.kind === 5 ? "S" : item.sym.kind === 11 ? "N" : "R";
  }

  // Open Monaco's own find widget (Ctrl/Cmd+F) from the toolbar, so search is
  // reachable without first clicking into the editor to give it focus.
  function openFind() {
    if (!editor) return;
    editor.focus();
    editor.getAction("actions.find")?.run();
  }

  // Open Monaco's command palette (its F1 quick-command list: find, fold, go to
  // line and symbol, and the rest of the editor's actions).
  function openCommandPalette() {
    if (!editor) return;
    editor.focus();
    editor.getAction("editor.action.quickCommand")?.run();
  }

  onMount(() => {
    if (!container) return;
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
      scheduleOutlineRefresh();
    });

    // Restore cursor, scroll, and selection from the previous mount so a mode
    // switch returns the editor to where the user left it.
    if (initialViewState) editor.restoreViewState(initialViewState);

    onready?.(editor);
  });

  onDestroy(() => {
    clearTimeout(outlineRefreshTimer);
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

{#if toolbar}
  <div class="dv-root">
    <div class="dv-toolbar">
      <button
        type="button"
        class="dv-cmd"
        onclick={openCommandPalette}
        title="Command palette"
        aria-label="Open the command palette"
      >
        <span class="dv-cmd-caret">&gt;_</span>
      </button>
      <div class="dv-toolbar-right">
        <button
          type="button"
          class="dv-tool-btn"
          onclick={openFind}
          title="Find (⌘F / Ctrl+F)"
          aria-label="Find"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.35-4.35"/></svg>
        </button>
        <button
          type="button"
          class="dv-tool-btn"
          class:active={outlineOpen}
          onclick={toggleOutline}
          title="Toggle outline"
          aria-label="Toggle outline"
          aria-pressed={outlineOpen}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>
        </button>
      </div>
    </div>
    <div class="dv-body">
      <div class="dv-editor" bind:this={container}></div>
      {#if outlineOpen}
        <aside class="dv-outline" aria-label="Outline">
          <div class="dv-outline-head"><span class="dv-outline-title">Outline</span></div>
        <div class="dv-outline-list">
          {#each visibleOutlineItems() as item, i}
            <button
              type="button"
              class="dv-outline-item"
              class:child={item.isChild}
              class:selected={i === outlineSelected}
              onmousedown={(e) => {
                e.preventDefault();
                outlineSelected = i;
                revealSymbol(item.sym);
              }}
            >
              {#if !item.isChild && item.sym.children?.length}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="dv-outline-chevron"
                  role="button"
                  tabindex="-1"
                  aria-label="Toggle {item.sym.name}"
                  onmousedown={(e) => {
                    e.stopPropagation();
                    e.preventDefault();
                    toggleOutlineNode(item.sym.name);
                  }}
                >
                  {#if outlineExpanded.has(item.sym.name)}
                    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 4.5 6 8l3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg>
                  {:else}
                    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M4.5 2.5 8 6l-3.5 3.5" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg>
                  {/if}
                </span>
              {:else if !item.isChild}
                <span class="dv-outline-chevron-blank"></span>
              {/if}
              <span class="dv-outline-glyph" class:label={item.isChild}>{outlineGlyph(item)}</span>
              <span class="dv-outline-name">{item.sym.name}</span>
            </button>
          {/each}
          {#if outlineSymbols.length === 0}
            <div class="dv-outline-empty">No symbols</div>
          {/if}
          </div>
        </aside>
      {/if}
    </div>
  </div>
{:else}
  <div class="design-view-container" bind:this={container}></div>
{/if}

<style>
  .design-view-container {
    width: 100%;
    height: 100%;
  }

  /* A top toolbar (command palette, find, outline toggle) over the editor; the
     outline pane opens on the right of the editor below it. Colours match the
     iggy-dark (vs-dark) editor theme. */
  .dv-root {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .dv-toolbar {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center; /* the command pill sits centered; the buttons float right */
    height: 40px;
    flex-shrink: 0;
    padding: 0 8px;
    background: #252526;
    border-bottom: 1px solid #3c3c3c;
  }

  .dv-toolbar-right {
    position: absolute;
    right: 8px;
    top: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  /* The command-palette entry, styled like a search input with a "> " prompt and
     the F1 hint, so it reads as "type a command here". */
  .dv-cmd {
    display: flex;
    align-items: center;
    height: 30px;
    width: 380px;
    max-width: calc(100% - 160px); /* stay clear of the right-hand buttons */
    padding: 0 10px;
    border: 1px solid #3c3c3c;
    border-radius: 5px;
    background: #1e1e1e;
    cursor: pointer;
  }

  .dv-cmd:hover {
    background: #2a2d2e;
    border-color: #4d4d4d;
  }

  .dv-cmd-caret {
    font-family: monospace;
    font-size: 13px;
    font-weight: 700;
    color: #4ec9b0;
  }

  .dv-tool-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 5px;
    background: none;
    color: #888;
    cursor: pointer;
  }

  .dv-tool-btn:hover {
    background: #2a2d2e;
    color: #d4d4d4;
  }

  .dv-tool-btn.active {
    background: #37373d;
    color: #d4d4d4;
  }

  .dv-body {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .dv-editor {
    flex: 1 1 auto;
    min-width: 0;
    height: 100%;
  }

  .dv-outline {
    display: flex;
    flex-direction: column;
    width: 220px;
    flex-shrink: 0;
    background: #1e1e1e;
    border-left: 1px solid #3c3c3c;
    overflow: hidden;
  }

  .dv-outline-head {
    display: flex;
    align-items: center;
    height: 32px;
    padding: 0 12px;
    border-bottom: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .dv-outline-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #bbbbbb;
  }

  .dv-outline-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .dv-outline-item {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding: 2px 4px;
    border: none;
    background: none;
    color: #d4d4d4;
    font-size: 13px;
    font-family: inherit;
    text-align: left;
    cursor: pointer;
  }

  .dv-outline-item:hover {
    background: #2a2d2e;
  }

  .dv-outline-item.selected,
  .dv-outline-item.selected:hover {
    background: #04395e;
  }

  .dv-outline-item.child {
    padding-left: 32px;
  }

  .dv-outline-chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    color: #888;
  }

  .dv-outline-chevron:hover {
    color: #d4d4d4;
  }

  .dv-outline-chevron-blank {
    width: 16px;
    flex-shrink: 0;
  }

  .dv-outline-glyph {
    width: 14px;
    flex-shrink: 0;
    text-align: center;
    font-size: 11px;
    font-weight: 700;
    color: #4ec9b0;
  }

  .dv-outline-glyph.label {
    color: #dcdcaa;
  }

  .dv-outline-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dv-outline-empty {
    padding: 8px 12px;
    font-size: 12px;
    color: #888;
  }

</style>
