<script lang="ts">
  import { onMount } from "svelte";
  import { ParseView, WasmBackend, type ParserBackend, type WasmParse } from "@iguana-parser/parse-view";

  // The grammar-derived data the bundle ships alongside the wasm module. The
  // viewer is grammar-independent and reads it all at runtime.
  interface Manifest {
    grammar: string;
    layout_name: string | null;
    sample_input: string;
    start_nonterminals: string[];
  }

  let status = $state<"loading" | "ready" | "error">("loading");
  let errorMessage = $state("");

  let backend = $state<ParserBackend | null>(null);
  let grammarName = $state<string | null>(null);
  let nonterminals = $state<string[]>([]);
  let startNonterminal = $state<string | null>(null);
  let inputText = $state("");
  let leftPanelWidth = $state(420);

  onMount(async () => {
    try {
      const base = import.meta.env.BASE_URL;
      const manifest: Manifest = await (await fetch(`${base}manifest.json`)).json();

      // The wasm bundle is a static sibling of the viewer, not a source
      // dependency, so it loads at runtime. The @vite-ignore keeps Vite from
      // trying to resolve it at build time.
      const wasm = await import(/* @vite-ignore */ `${base}wasm/pkg/iggy_wasm.js`);
      await wasm.default();

      backend = new WasmBackend(wasm.parse as WasmParse);
      grammarName = manifest.grammar;
      nonterminals = manifest.start_nonterminals;
      startNonterminal = manifest.start_nonterminals[0] ?? null;
      inputText = manifest.sample_input;
      status = "ready";
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
      status = "error";
    }
  });

  function startVerticalDrag(e: MouseEvent) {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = leftPanelWidth;
    function onMove(ev: MouseEvent) {
      leftPanelWidth = Math.max(240, Math.min(900, startWidth + ev.clientX - startX));
    }
    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }
</script>

{#if status === "ready"}
  <div class="viewer-root">
    <ParseView
      {backend}
      parserName={grammarName}
      buildStatus="success"
      {nonterminals}
      bind:startNonterminal
      bind:inputText
      {leftPanelWidth}
      {startVerticalDrag}
    />
  </div>
{:else if status === "loading"}
  <div class="viewer-message">Loading parser…</div>
{:else}
  <div class="viewer-message viewer-error">Could not load the parser: {errorMessage}</div>
{/if}

<style>
  .viewer-root {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .viewer-message {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    font-size: 0.95rem;
    color: #9aa0a6;
  }

  .viewer-error {
    color: #e05050;
  }
</style>
