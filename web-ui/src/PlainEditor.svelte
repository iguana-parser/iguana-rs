<script lang="ts">
  // A lightweight, dependency-free stand-in for the Monaco InputEditor, used when
  // a host wants a small footprint (the website hero embeds many of these, and a
  // full Monaco per iframe is heavy). It is a transparent <textarea> layered over
  // a highlighted <pre>, so it stays editable while reading like a code sample.
  //
  // Highlighting is intentionally generic: the viewer parses any grammar's wasm,
  // so it cannot know the input language's real keywords. It colours by lexical
  // shape (strings, numbers, punctuation) plus a small set of keywords common
  // across languages — enough to look like code, not a bare textarea. It is not a
  // full editor: the parse-error / ambiguity / span decorations Monaco draws are
  // dropped in this mode.
  interface Props {
    value: string;
    placeholder?: string;
    readOnly?: boolean;
    onchange?: (value: string) => void;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    readOnly = false,
    onchange,
  }: Props = $props();

  let ta: HTMLTextAreaElement | undefined;
  let pre: HTMLPreElement | undefined;

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

  // Exported so ParseView can focus the input the way it does the Monaco editor.
  export function focus() {
    ta?.focus();
  }

  function oninput(e: Event) {
    onchange?.((e.target as HTMLTextAreaElement).value);
  }

  // Keep the highlighted layer aligned with the textarea as it scrolls.
  function onscroll() {
    if (pre && ta) {
      pre.scrollTop = ta.scrollTop;
      pre.scrollLeft = ta.scrollLeft;
    }
  }
</script>

<div class="ple">
  <pre class="ple-pre" aria-hidden="true" bind:this={pre}><code>{@html highlighted}</code></pre>
  <textarea
    class="ple-ta"
    bind:this={ta}
    bind:value
    {placeholder}
    {readOnly}
    wrap="off"
    spellcheck="false"
    autocapitalize="off"
    autocomplete="off"
    autocorrect="off"
    oninput={oninput}
    onscroll={onscroll}
  ></textarea>
</div>

<style>
  .ple {
    position: relative;
    height: 100%;
    overflow: hidden;
    background: #1e1e1e;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 13px;
    line-height: 1.5;
  }

  /* The highlighted layer and the textarea must share identical text metrics and
     padding, or the caret will drift from the rendered glyphs. */
  .ple-pre,
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
