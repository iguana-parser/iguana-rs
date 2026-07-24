<script lang="ts">
  import { onMount } from "svelte";
  import { DesignView, WasmLspBackend, type LspBackend, type LspWasm } from "@iguana-parser/web-ui";

  let status = $state<"loading" | "ready" | "error">("loading");
  let errorMessage = $state("");
  let backend = $state<LspBackend | null>(null);
  let source = $state("");

  const log = (msg: string) => console.info(`[iguana] ${msg}`);

  onMount(async () => {
    try {
      const base = import.meta.env.BASE_URL;

      // The grammar to show is named by the `?src` query as a URL to its .iggy
      // source, so this one deployed app serves every grammar page; only the
      // source differs.
      const src = new URLSearchParams(location.search).get("src");
      if (!src) throw new Error("missing ?src=<grammar.iggy> query parameter");

      log(`loading grammar source: ${src}`);
      const res = await fetch(src);
      if (!res.ok) throw new Error(`could not load ${src}: ${res.status}`);
      source = await res.text();

      // The lsp-wasm module analyzes iggy itself, so it is grammar-independent:
      // one bundle, loaded once, drives every grammar page. It sits next to this
      // app, so it loads at runtime (the @vite-ignore keeps Vite from resolving
      // it at build time). Called with no argument, its init fetches the wasm
      // sibling of the module.
      log("loading lsp-wasm...");
      const lspWasm = await import(/* @vite-ignore */ `${base}wasm/pkg/iguana_lsp_wasm.js`);
      await lspWasm.default();

      backend = new WasmLspBackend(lspWasm as LspWasm);
      status = "ready";
      log("ready");
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
      console.error("[iguana] failed to load the design view:", e);
      status = "error";
    }
  });
</script>

{#if status === "ready" && backend}
  <div class="design-root">
    <DesignView value={source} readOnly={true} {backend} toolbar />
  </div>
{:else if status === "loading"}
  <div class="viewer-message">Loading grammar…</div>
{:else}
  <div class="viewer-message viewer-error">Could not load the grammar: {errorMessage}</div>
{/if}

<style>
  .design-root {
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
