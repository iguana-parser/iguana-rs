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
  let views = $state<Array<"tree" | "graph" | "sexpr">>(["tree", "graph", "sexpr"]);
  let editorMode = $state<"monaco" | "plain">("monaco");
  let parseTrigger = $state<"button" | "auto">("auto");
  let flashOnReveal = $state(false);

  // Min/max extent of the input pane along the split axis, shared by the divider
  // drag, the ?input= URL override, and window resizes. The result pane keeps at
  // least its own minimum: side by side, the width its tab row needs (the three
  // tabs, Show, and the theme toggle, about 340px with their margins); stacked,
  // 100px.
  const INPUT_MIN = 250;
  const RESULT_MIN = 360;
  function paneBounds(axis: "horizontal" | "vertical") {
    return axis === "vertical"
      ? { min: 100, max: Math.max(100, Math.min(520, window.innerHeight - 100)) }
      : { min: INPUT_MIN, max: Math.max(INPUT_MIN, window.innerWidth - RESULT_MIN) };
  }

  // Side by side, the input pane takes half the window, and keeps half as the
  // window resizes, until the user drags the divider or the URL sizes the pane.
  // A pane sized either way keeps the chosen size, within the bounds of the
  // window as it resizes, so the result pane never ends up outside it and the
  // input pane grows back when the window does.
  let halfSplit = true;
  let chosenSize = 0;
  function fitSplit() {
    if (halfSplit && orientation === "horizontal") {
      leftPanelWidth = Math.round(window.innerWidth / 2);
    } else {
      const { min, max } = paneBounds(orientation);
      leftPanelWidth = Math.max(min, Math.min(max, chosenSize || leftPanelWidth));
    }
  }

  // The color theme. A ?theme= in the URL decides it: dark, light, auto, which
  // follows the system's light or dark setting, or inherit, which follows the
  // embedding page as it changes. inherit takes the color-scheme the page gives
  // the iframe element, so a page can theme one embed differently from the rest
  // (a dark band, a breakpoint); where that is neither light nor dark, it takes
  // the page's data-theme attribute. inherit reads the page, so it needs the
  // page on the same origin; elsewhere the viewer stays dark.
  //
  // Without a ?theme=, an embedded viewer is dark, and a standalone one (iguana
  // try) follows the system until the user picks a theme with the toggle in the
  // result tabs. The browser remembers that choice. The theme is set before the
  // first render, so the page does not flash the other palette.
  let theme = $state<"dark" | "light">("dark");

  function setTheme(next: "dark" | "light") {
    theme = next;
    const root = document.documentElement;
    if (root.dataset.theme === next) return;
    // Controls with a hover transition (the divider, the buttons) would fade
    // from the old palette while everything else switches at once, so
    // transitions are off while the new palette is applied. Reading the layout
    // applies it at once; afterwards the transitions have no change to animate.
    // The attribute is not one the components' theme observers watch.
    root.toggleAttribute("data-theme-switching", true);
    root.dataset.theme = next;
    void document.body.offsetHeight;
    root.removeAttribute("data-theme-switching");
  }

  // Follows the system's setting as it changes. Returns the cleanup.
  function followSystemTheme(): () => void {
    const systemDark = window.matchMedia("(prefers-color-scheme: dark)");
    const sync = () => setTheme(systemDark.matches ? "dark" : "light");
    sync();
    systemDark.addEventListener("change", sync);
    return () => systemDark.removeEventListener("change", sync);
  }

  // The theme the user picked with the toggle. Storage can be unavailable (a
  // private window, blocked site data), and the viewer then follows the system.
  const THEME_KEY = "iguana-viewer-theme";

  function savedTheme(): "dark" | "light" | null {
    try {
      const value = localStorage.getItem(THEME_KEY);
      return value === "dark" || value === "light" ? value : null;
    } catch {
      return null;
    }
  }

  function toggleTheme() {
    stopFollowingSystem?.();
    stopFollowingSystem = undefined;
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next);
    try {
      localStorage.setItem(THEME_KEY, next);
    } catch {
      // The choice then lasts for this page only.
    }
  }

  function followHostTheme() {
    if (window.parent === window) return;
    try {
      const hostWindow = window.parent;
      const host = hostWindow.document.documentElement;
      const frame = window.frameElement;
      const sync = () => {
        const scheme = frame ? hostWindow.getComputedStyle(frame).colorScheme.split(/\s+/) : [];
        const light = scheme.includes("light");
        const dark = scheme.includes("dark");
        if (light !== dark) setTheme(light ? "light" : "dark");
        else setTheme(host.getAttribute("data-theme") === "light" ? "light" : "dark");
      };
      sync();
      // A page can change the frame's color-scheme through any element's class
      // or style on the way down to the iframe, or the iframe's own, so the
      // whole page is watched. The check is one style read.
      new MutationObserver(sync).observe(host, {
        attributes: true,
        attributeFilter: ["data-theme", "class", "style"],
        subtree: true,
      });
      // The page's media queries can change it without any attribute changing:
      // at a width breakpoint or with the system theme.
      hostWindow.addEventListener("resize", sync);
      hostWindow.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", sync);
    } catch {
      // A page on another origin cannot be read; keep the default.
    }
  }

  const themeParam = new URLSearchParams(location.search).get("theme");
  // The toggle shows only where the user's choice decides the theme: a
  // standalone viewer whose URL does not set one.
  const showThemeToggle = window.parent === window && themeParam === null;
  let stopFollowingSystem: (() => void) | undefined;
  if (themeParam === "light" || themeParam === "dark") setTheme(themeParam);
  else if (themeParam === "inherit") followHostTheme();
  else if (themeParam === "auto") followSystemTheme();
  else if (showThemeToggle) {
    const saved = savedTheme();
    if (saved) setTheme(saved);
    else stopFollowingSystem = followSystemTheme();
  }

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
        leftPanelWidth = chosenSize = 180; // a shorter input pane reads better when stacked
      }
      // The embedder can size the input pane from the URL — its width in the
      // horizontal split, its height in the vertical one — with ?input=<px>. The
      // hero passes a short value so the stacked parse tree gets more room. It is
      // clamped to the same range the divider drag allows.
      const inputParam = Number(params.get("input"));
      if (Number.isFinite(inputParam) && inputParam > 0) {
        const { min, max } = paneBounds(orientation);
        leftPanelWidth = Math.max(min, Math.min(max, inputParam));
        chosenSize = inputParam;
        halfSplit = false;
      }
      fitSplit();
      window.addEventListener("resize", fitSplit);

      // Result views to expose (comma-separated subset of tree,graph,sexpr) and
      // whether to use the lightweight editor — both trim the embed's footprint.
      const viewsParam = params.get("views");
      if (viewsParam) {
        const allowed = ["tree", "graph", "sexpr"] as const;
        const chosen = viewsParam
          .split(",")
          .map((s) => s.trim())
          .filter((v): v is (typeof allowed)[number] =>
            (allowed as readonly string[]).includes(v),
          );
        if (chosen.length) views = chosen;
      }
      if (params.get("editor") === "plain") editorMode = "plain";
      // Parsing follows the input as it is edited; ?parse=button parses only when
      // the user presses ▶.
      if (params.get("parse") === "button") parseTrigger = "button";
      // A reveal in the tree only jumps to the row; ?flash=on also flashes it.
      if (params.get("flash") === "on") flashOnReveal = true;

      // With the relative build base, BASE_URL is "./". A dynamic import
      // resolves a relative specifier against the importing module (under
      // assets/), not the page, so the base is made absolute against the
      // document URL before use.
      const base = new URL(import.meta.env.BASE_URL, document.baseURI).href;

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
    const { min, max } = paneBounds(orientation);
    halfSplit = false;
    function onMove(ev: MouseEvent) {
      const pos = vertical ? ev.clientY : ev.clientX;
      leftPanelWidth = chosenSize = Math.max(min, Math.min(max, startSize + pos - start));
    }
    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }
</script>

{#snippet themeToggle()}
  <button
    class="theme-toggle"
    onclick={toggleTheme}
    title={theme === "dark" ? "Light theme" : "Dark theme"}
    aria-label={theme === "dark" ? "Switch to the light theme" : "Switch to the dark theme"}
  >
    <!-- The website's sun and moon, drawn lighter: lucide's moon is a heavy crescent
         at this size. -->
    {#if theme === "dark"}
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="4.2"/><path d="M12 1.8v2.4M12 19.8v2.4M4.2 12H1.8M22.2 12h-2.4M5.6 5.6 3.9 3.9M20.1 20.1l-1.7-1.7M18.4 5.6l1.7-1.7M3.9 20.1l1.7-1.7"/></svg>
    {:else}
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20.5 14.8A8.2 8.2 0 0 1 9.2 3.5 7.3 7.3 0 1 0 20.5 14.8z"/></svg>
    {/if}
  </button>
{/snippet}

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
      {views}
      editor={editorMode}
      {parseTrigger}
      {flashOnReveal}
      exportable={false}
      tabActions={showThemeToggle ? themeToggle : undefined}
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

  /* Sized and colored like the Show button beside it. */
  .theme-toggle {
    display: inline-flex;
    align-items: center;
    padding: 4px 6px;
    background: var(--iguana-bg-raised, #2d2d2d);
    color: var(--iguana-text-muted, #888);
    border: none;
    border-radius: 3px;
    cursor: pointer;
  }
  .theme-toggle:hover {
    background: var(--iguana-border, #3c3c3c);
    color: var(--iguana-text-strong, #fff);
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
