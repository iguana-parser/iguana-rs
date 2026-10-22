<script lang="ts">
  // A lightweight, dependency-free stand-in for the Monaco InputEditor, used when
  // a host wants a small footprint (the website hero embeds many of these, and a
  // full Monaco per iframe is heavy). It is a transparent <textarea> layered over
  // a highlighted <pre>, so it stays editable while reading like a code sample.
  //
  // Highlighting is intentionally generic: the viewer parses any grammar's wasm,
  // so it cannot know the input language's real keywords. It colours by lexical
  // shape (strings, numbers, punctuation) plus a small set of keywords common
  // across languages — enough to look like code, not a bare textarea.
  //
  // It honours the same selection-sync contract as InputEditor (highlightSpan,
  // onclick, onescape), so ParseView drives text<->node selection identically in
  // both editors. Like InputEditor, it numbers the lines and marks a parse error
  // with a squiggle; the error's line number turns red. The Monaco-only extras
  // (ambiguity markers, the message on hover, and scroll-the-span-into-view) are
  // still dropped in this mode.
  import type { ParseError } from "./backend";
  import { utf16Offset } from "./char-offsets";

  interface HighlightSpan {
    start: number;
    end: number;
  }

  interface Props {
    value: string;
    placeholder?: string;
    readOnly?: boolean;
    error?: ParseError | null;
    highlightSpan?: HighlightSpan | null;
    onchange?: (value: string) => void;
    onclick?: (offset: number) => void;
    onescape?: () => void;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    readOnly = false,
    error = null,
    highlightSpan = null,
    onchange,
    onclick,
    onescape,
  }: Props = $props();

  let ta: HTMLTextAreaElement | undefined;
  let pre: HTMLPreElement | undefined;
  let sel: HTMLPreElement | undefined;
  let diag: HTMLPreElement | undefined;
  // The textarea's vertical scroll, which the line numbers follow.
  let scrollTop = $state(0);

  // Safari autocorrects a textarea unless the attribute says otherwise, and a
  // code input should not have it. Svelte's typings list autocorrect on input
  // alone, so it reaches the element through a spread.
  const AUTOCORRECT_OFF = { autocorrect: "off" };

  const COMMON_KEYWORDS = new Set([
    "if", "then", "else", "let", "in", "fun", "case", "of", "match", "with",
    "where", "for", "while", "do", "return", "true", "false", "null", "nil",
    "and", "or", "not", "class", "def", "fn", "func", "function", "var", "val",
    "const", "type", "data", "import", "module", "print", "this", "super",
  ]);

  const escapeHtml = (s: string) =>
    s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

  function highlight(src: string): string {
    // string | number | word | whitespace | punctuation-run
    const re =
      /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')|(\d+(?:\.\d+)?)|([A-Za-z_]\w*)|(\s+)|([^\sA-Za-z0-9_]+)/g;
    let out = "";
    let m: RegExpExecArray | null;
    while ((m = re.exec(src)) !== null) {
      if (m[1]) out += `<span class="ple-str">${escapeHtml(m[1])}</span>`;
      else if (m[2]) out += `<span class="ple-num">${escapeHtml(m[2])}</span>`;
      else if (m[3])
        out += COMMON_KEYWORDS.has(m[3])
          ? `<span class="ple-kw">${escapeHtml(m[3])}</span>`
          : escapeHtml(m[3]);
      else if (m[4]) out += escapeHtml(m[4]);
      else if (m[5]) out += `<span class="ple-op">${escapeHtml(m[5])}</span>`;
    }
    // trailing newline so the last line keeps its height in the <pre>
    return out + "\n";
  }

  let highlighted = $derived(highlight(value));

  // The selection layer sits behind the coloured text and paints the highlighted
  // character range (or a caret bar for an empty span). It renders the full text
  // transparently so its metrics match the highlight layer exactly and the two
  // stay pixel-aligned as they scroll. Mirrors InputEditor's highlightSpan.
  let selectionMarkup = $derived.by(() => {
    if (!highlightSpan) return escapeHtml(value) + "\n";
    const len = value.length;
    const s = Math.max(0, Math.min(highlightSpan.start, len));
    const e = Math.max(s, Math.min(highlightSpan.end, len));
    const before = escapeHtml(value.slice(0, s));
    const after = escapeHtml(value.slice(e));
    const mid =
      e > s
        ? `<span class="ple-sel-range">${escapeHtml(value.slice(s, e))}</span>`
        : `<span class="ple-caret"></span>`;
    return before + mid + after + "\n";
  });

  // The parse error as UTF-16 offsets into the text (the runtime counts
  // characters). An empty span marks the character after it, and a failure at a
  // line break or at the end of the input, which has no character to underline,
  // marks one column past the end of the line.
  let errorRange = $derived.by(() => {
    if (!error) return null;
    const start = utf16Offset(value, error.span.left_extent);
    let end = Math.max(start, utf16Offset(value, error.span.right_extent));
    if (end === start && start < value.length && value[start] !== "\n") {
      end = utf16Offset(value, error.span.left_extent + 1);
    }
    return { start, end, line: value.slice(0, start).split("\n").length };
  });

  // The error layer, like the selection layer, renders the text transparently
  // and paints only the squiggle under the error range. A range with nothing
  // visible to underline (empty, or only line breaks) gets a squiggled space at
  // its start, and its line breaks stay in place after it.
  let errorMarkup = $derived.by(() => {
    if (!errorRange) return "";
    const { start, end } = errorRange;
    const text = value.slice(start, end);
    const invisible = /^\n*$/.test(text);
    return (
      escapeHtml(value.slice(0, start)) +
      `<span class="ple-err">${invisible ? "&nbsp;" : escapeHtml(text)}</span>` +
      escapeHtml(value.slice(invisible ? start : end)) +
      "\n"
    );
  });

  let lineCount = $derived(value.split("\n").length);

  // Exported so ParseView can focus the input the way it does the Monaco editor.
  export function focus() {
    ta?.focus();
  }

  function oninput(e: Event) {
    onchange?.((e.target as HTMLTextAreaElement).value);
  }

  // A click places the caret; report its offset so ParseView can select the
  // deepest node at that position (the text->node half of the sync). This mirrors
  // InputEditor's onMouseDown handler.
  function onclickInput() {
    if (ta) onclick?.(ta.selectionStart);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onescape?.();
  }

  // Keep the highlighted, selection, and error layers and the line numbers
  // aligned with the textarea as it scrolls, and after a layer's markup changes:
  // a layer emptied while a parse runs loses its scroll position, and the
  // markup that returns would otherwise sit at the top until the next scroll.
  $effect(() => {
    void highlighted;
    void selectionMarkup;
    void errorMarkup;
    onscroll();
  });

  function onscroll() {
    if (!ta) return;
    for (const layer of [pre, sel, diag]) {
      if (!layer) continue;
      layer.scrollTop = ta.scrollTop;
      layer.scrollLeft = ta.scrollLeft;
    }
    scrollTop = ta.scrollTop;
  }
</script>

<div class="ple">
  <!-- The line numbers move by transform rather than by scrolling, because a
       box without the textarea's horizontal scrollbar cannot scroll as far. -->
  <div class="ple-gutter" aria-hidden="true">
    <div style="transform: translateY({-scrollTop}px)">
      {#each { length: lineCount } as _, i}
        <div class:ple-gutter-err={errorRange?.line === i + 1}>{i + 1}</div>
      {/each}
    </div>
  </div>
  <div class="ple-body">
    <pre class="ple-sel" aria-hidden="true" bind:this={sel}>{@html selectionMarkup}</pre>
    <pre class="ple-pre" aria-hidden="true" bind:this={pre}><code>{@html highlighted}</code></pre>
    <pre class="ple-diag" aria-hidden="true" bind:this={diag}>{@html errorMarkup}</pre>
    <textarea
      class="ple-ta"
      bind:this={ta}
      bind:value
      {placeholder}
      readonly={readOnly}
      wrap="off"
      spellcheck="false"
      autocapitalize="off"
      autocomplete="off"
      {...AUTOCORRECT_OFF}
      oninput={oninput}
      onclick={onclickInput}
      onkeydown={onkeydown}
      onscroll={onscroll}
    ></textarea>
  </div>
</div>

<style>
  .ple {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--iguana-bg, #1e1e1e);
    /* The host's code font, else the components' shared list (CODE_FONT in
       theme.ts). Ligatures off either way. */
    font-family: var(--iguana-font-mono, "Fira Code", ui-monospace, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace);
    font-variant-ligatures: none;
    font-size: 13px;
    line-height: 1.5;
  }

  /* The line numbers sit in their own column, so text scrolled sideways is
     clipped at the column's edge instead of passing under the numbers. */
  .ple-gutter {
    flex-shrink: 0;
    overflow: hidden;
    padding: 12px 0 8px 10px;
    min-width: 2ch;
    text-align: right;
    color: var(--iguana-text-muted, #6e7681);
    user-select: none;
  }
  .ple-gutter-err {
    color: var(--iguana-error, #f14c4c);
  }

  .ple-body {
    position: relative;
    flex: 1;
    min-width: 0;
  }

  /* The highlighted, selection, error, and textarea layers must share identical
     text metrics and padding, or the caret and the highlight will drift from the
     rendered glyphs. */
  .ple-pre,
  .ple-sel,
  .ple-diag,
  .ple-ta {
    margin: 0;
    padding: 12px 10px 8px 14px;
    border: 0;
    font: inherit;
    line-height: inherit;
    white-space: pre;
    tab-size: 2;
    box-sizing: border-box;
    width: 100%;
    height: 100%;
    overflow: auto;
  }

  /* Selection layer: behind the coloured text, transparent glyphs so only its
     background (the highlighted span) and caret bar show through. */
  .ple-sel {
    position: absolute;
    inset: 0;
    pointer-events: none;
    color: transparent;
  }
  .ple-sel :global(.ple-sel-range) {
    background: var(--iguana-selection, #264f78);
  }
  .ple-sel :global(.ple-caret) {
    border-left: 2px solid var(--iguana-accent, #569cd6);
  }

  .ple-pre {
    position: absolute;
    inset: 0;
    pointer-events: none;
    color: var(--iguana-text, #d4d4d4);
  }

  /* Error layer: above the coloured text, transparent glyphs, and a wavy
     underline in the error color under the error range. */
  .ple-diag {
    position: absolute;
    inset: 0;
    pointer-events: none;
    color: transparent;
  }
  .ple-diag :global(.ple-err) {
    text-decoration: underline wavy var(--iguana-error, #f14c4c);
    text-decoration-skip-ink: none;
    text-decoration-thickness: 1px;
    text-underline-offset: 3px;
  }
  .ple-pre code {
    font: inherit;
  }

  .ple-ta {
    position: relative;
    background: transparent;
    color: transparent;
    caret-color: var(--iguana-text, #d4d4d4);
    resize: none;
    outline: none;
  }
  .ple-ta::placeholder {
    color: var(--iguana-text-muted, #6a6a6a);
  }

  /* The highlighted tokens are injected via {@html}, so they don't get Svelte's
     scoping class — target them with :global, kept under the scoped .ple-pre. */
  .ple-pre :global(.ple-kw) {
    color: var(--iguana-syntax-keyword, #569cd6);
  }
  .ple-pre :global(.ple-str) {
    color: var(--iguana-syntax-string, #ce9178);
  }
  .ple-pre :global(.ple-num) {
    color: var(--iguana-syntax-number, #b5cea8);
  }
  .ple-pre :global(.ple-op) {
    color: var(--iguana-syntax-operator, #c8c8c8);
  }
</style>
