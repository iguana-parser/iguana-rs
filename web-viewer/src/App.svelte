<script lang="ts">
  import { onMount } from "svelte";
  import { ParseView, WasmBackend, type ParserBackend, type WasmParse } from "@iguana-parser/web-ui";

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
  let orientation = $state<"horizontal" | "vertical">("horizontal");
  let leftPanelWidth = $state(420);

  // The editor and graph libraries load from a CDN at runtime (see the
  // importmap in index.html), so they are fetched on first load and served from
  // the browser cache after that. Each step logs to the console so a slow first
  // load is legible rather than a silent blank page.
  const log = (msg: string) => console.info(`[iguana] ${msg}`);

  onMount(async () => {
    try {
      // Layout is a deploy-time choice passed in the URL: the hero embeds the
      // viewer with ?layout=vertical (input over result); the grammar pages use
      // the default horizontal split.
      const params = new URLSearchParams(location.search);
      if (params.get("layout") === "vertical") {
        orientation = "vertical";
        leftPanelWidth = 180; // a shorter input pane reads better when stacked
      }

      const base = import.meta.env.BASE_URL;

      log("loading manifest...");
      const manifest: Manifest = await (await fetch(`${base}manifest.json`)).json();
      log(`grammar: ${manifest.grammar}`);

      // The wasm bundle is a static sibling of the viewer, not a source
      // dependency, so it loads at runtime. The viewer is grammar-independent,
      // so the module name is fixed (`iguana generate --wasm` builds it with
      // `wasm-pack --out-name parser`). The @vite-ignore keeps Vite from
      // trying to resolve it at build time.
      log("loading wasm parser...");
      const wasm = await import(/* @vite-ignore */ `${base}wasm/pkg/parser.js`);
      await wasm.default();

      backend = new WasmBackend(wasm.parse as WasmParse);
      grammarName = manifest.grammar;
      nonterminals = manifest.start_nonterminals;
      startNonterminal = manifest.start_nonterminals[0] ?? null;
      inputText = manifest.sample_input;
      status = "ready";
      log("ready");
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
      console.error("[iguana] failed to load the parser:", e);
      status = "error";
    }
  });

  // Drag the divider. Horizontal resizes the input panel by width (clientX),
  // vertical by height (clientY); leftPanelWidth holds the size along either axis.
  function startVerticalDrag(e: MouseEvent) {
    e.preventDefault();
    const vertical = orientation === "vertical";
    const start = vertical ? e.clientY : e.clientX;
    const startSize = leftPanelWidth;
    const min = vertical ? 100 : 240;
    const max = vertical ? 520 : 900;
    function onMove(ev: MouseEvent) {
      const pos = vertical ? ev.clientY : ev.clientX;
      leftPanelWidth = Math.max(min, Math.min(max, startSize + pos - start));
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
      {orientation}
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
