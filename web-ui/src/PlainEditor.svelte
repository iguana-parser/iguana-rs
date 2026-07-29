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
  // both editors. The Monaco-only extras (parse-error / ambiguity markers and
  // scroll-the-span-into-view) are still dropped in this mode.
  interface HighlightSpan {
    start: number;
    end: number;
  }

  interface Props {
    value: string;
    placeholder?: string;
    readOnly?: boolean;
    highlightSpan?: HighlightSpan | null;
    onchange?: (value: string) => void;
    onclick?: (offset: number) => void;
    onescape?: () => void;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    readOnly = false,
    highlightSpan = null,
    onchange,
    onclick,
    onescape,
  }: Props = $props();

  let ta: HTMLTextAreaElement | undefined;
  let pre: HTMLPreElement | undefined;
  let sel: HTMLPreElement | undefined;

  // Safari autocorrects a textarea unless the attribute says otherwise, and a
  // code input should not have it. Svelte's typings list autocorrect on input
  // alone, so it reaches the element through a spread.
  const AUTOCORRECT_OFF = { autocorrect: "off" };

  const COMMON_KEYWORDS = new Set([
    "if", "then", "else", "let", "in", "fun", "case", "of", "match", "with",
    "where", "for", "while", "do", "return", "true", "false", "null", "and",
    "or", "not", "class", "def", "fn", "func", "function", "var", "val",
    "const", "type", "data", "import", "module",
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

  // Keep the highlighted and selection layers aligned with the textarea as it
  // scrolls.
  function onscroll() {
    if (!ta) return;
    if (pre) {
      pre.scrollTop = ta.scrollTop;
      pre.scrollLeft = ta.scrollLeft;
    }
    if (sel) {
      sel.scrollTop = ta.scrollTop;
      sel.scrollLeft = ta.scrollLeft;
    }
  }
</script>

<div class="ple">
  <pre class="ple-sel" aria-hidden="true" bind:this={sel}>{@html selectionMarkup}</pre>
  <pre class="ple-pre" aria-hidden="true" bind:this={pre}><code>{@html highlighted}</code></pre>
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

<style>
  .ple {
    position: relative;
    height: 100%;
    overflow: hidden;
    background: #1e1e1e;
    /* Match Terrarium's Monaco editor: Fira Code where installed, ligatures off. */
    font-family: "Fira Code", Consolas, "Liberation Mono", Menlo, monospace;
    font-variant-ligatures: none;
    font-size: 13px;
    line-height: 1.5;
  }

  /* The highlighted, selection, and textarea layers must share identical text
     metrics and padding, or the caret and the highlight will drift from the
     rendered glyphs. */
  .ple-pre,
  .ple-sel,
  .ple-ta {
    margin: 0;
    padding: 12px 10px 8px;
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
    background: #264f78;
  }
  .ple-sel :global(.ple-caret) {
    border-left: 2px solid #569cd6;
  }

  .ple-pre {
    position: absolute;
    inset: 0;
    pointer-events: none;
    color: #d4d4d4;
  }
  .ple-pre code {
    font: inherit;
  }

  .ple-ta {
    position: relative;
    background: transparent;
    color: transparent;
    caret-color: #d4d4d4;
    resize: none;
    outline: none;
  }
  .ple-ta::placeholder {
    color: #6a6a6a;
  }

  /* The highlighted tokens are injected via {@html}, so they don't get Svelte's
     scoping class — target them with :global, kept under the scoped .ple-pre. */
  .ple-pre :global(.ple-kw) {
    color: #569cd6;
  }
  .ple-pre :global(.ple-str) {
    color: #ce9178;
  }
  .ple-pre :global(.ple-num) {
    color: #b5cea8;
  }
  .ple-pre :global(.ple-op) {
    color: #c8c8c8;
  }
</style>
