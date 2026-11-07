// A read-only CodeMirror 6 view of an Iggy grammar, used by the website's home
// page and grammar pages. Once the iguana-lsp wasm module loads, a click on a
// reference goes to its rule, hovering a reference shows a peek of the rule,
// hovering a rule head shows its reference count, and a Go to rule button lists
// the rules. The language server also adds occurrence highlighting and folding.
//
// Colors come from the --gv-* properties in style.scss: Dark+ by default, as
// Giallo renders iggy, and a light palette on a grammar page in the light theme.
import { Compartment, EditorSelection, EditorState, StateEffect, StateField } from "@codemirror/state";
import type { Extension, SelectionRange, Text } from "@codemirror/state";
import {
  Decoration,
  EditorView,
  ViewPlugin,
  closeHoverTooltips,
  drawSelection,
  highlightActiveLine,
  hoverTooltip,
  keymap,
  lineNumbers,
  tooltips,
} from "@codemirror/view";
import type { DecorationSet, Tooltip, ViewUpdate } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import {
  HighlightStyle,
  StreamLanguage,
  bracketMatching,
  codeFolding,
  foldGutter,
  foldService,
  forceParsing,
  syntaxHighlighting,
} from "@codemirror/language";
import { highlightTree, tags } from "@lezer/highlight";

const LSP_MODULE = "/playgrounds/iggy-lsp/iguana_lsp_wasm.js";

// The parts of the iguana-lsp wasm module that the view calls. Each function
// returns its LSP result as a JSON string.
interface LspModule {
  default(): Promise<unknown>;
  document_symbols(source: string): string;
  folding(source: string): string;
  definition(source: string, line: number, character: number): string;
  references(source: string, line: number, character: number, includeDeclaration: boolean): string;
}

interface LspPosition {
  line: number;
  character: number;
}

interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

interface LspLocation {
  range: LspRange;
}

interface LspSymbol {
  name: string;
  kind: number;
  range: LspRange;
  selectionRange: LspRange;
}

interface LspFoldingRange {
  startLine: number;
  endLine: number;
}

interface TextRange {
  from: number;
  to: number;
}

interface Rule {
  symbol: LspSymbol;
  head: TextRange;
  rule: TextRange;
}

interface RuleName {
  word: SelectionRange;
  rule: Rule;
}

interface IggyState {
  mode: "string" | "class" | null;
}

const KEYWORDS = new Set(["grammar", "layout", "left", "right", "none"]);

// mode: null (top level), "string" (inside "..."), "class" (inside [...]).
const iggy = StreamLanguage.define<IggyState>({
  startState: () => ({ mode: null }),
  token(stream, state) {
    if (state.mode === "string") {
      if (stream.match(/^\\./)) return "escape";
      if (stream.eat('"')) { state.mode = null; return "string"; }
      stream.match(/^[^"\\]+/) || stream.next();
      return "string";
    }
    if (state.mode === "class") {
      if (stream.match(/^\\./)) return "escape";
      if (stream.eat("]")) { state.mode = null; return "regexp"; }
      stream.match(/^[^\]\\]+/) || stream.next();
      return "regexp";
    }
    if (stream.eatSpace()) return null;
    if (stream.match(/^\/\/.*/)) return "comment";
    if (stream.match(/^@[A-Za-z]\w*/)) return "meta";
    if (stream.match(/^#[A-Za-z]\w*/)) return "labelName";
    if (stream.eat('"')) { state.mode = "string"; return "string"; }
    if (stream.eat("[")) { state.mode = "class"; return "regexp"; }
    const word = stream.match(/^[A-Za-z_]\w*/) as RegExpMatchArray | null;
    if (word) return KEYWORDS.has(word[0]) ? "keyword" : "typeName";
    stream.next();
    return null;
  },
  languageData: { commentTokens: { line: "//" } },
});

const highlight = HighlightStyle.define([
  { tag: tags.comment, color: "var(--gv-comment)" },
  { tag: tags.keyword, color: "var(--gv-keyword)" },
  { tag: tags.typeName, color: "var(--gv-type)" },
  { tag: tags.string, color: "var(--gv-string)" },
  { tag: tags.regexp, color: "var(--gv-regexp)" },
  { tag: tags.escape, color: "var(--gv-escape)" },
  { tag: tags.meta, color: "var(--gv-meta)" },
  { tag: tags.labelName, color: "var(--gv-label)" },
]);

const theme = EditorView.theme({
  "&": {
    height: "100%",
    color: "var(--gv-text)",
    backgroundColor: "var(--bg-code)",
    fontSize: "var(--gv-font-size, 13px)",
  },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": {
    fontFamily: "var(--font-mono)",
    fontVariantLigatures: "none",
    lineHeight: "1.5",
  },
  ".cm-content": { padding: "0.5rem 0", caretColor: "var(--gv-text)" },
  ".cm-line": { padding: "0 1rem 0 0.5rem" },
  ".cm-gutters": {
    backgroundColor: "var(--bg-code)",
    color: "var(--gv-muted)",
    border: "none",
    paddingLeft: "0.4rem",
  },
  ".cm-foldGutter .cm-gutterElement": { padding: "0 2px", cursor: "pointer", color: "var(--gv-faint)" },
  ".cm-foldGutter .cm-gutterElement:hover": { color: "var(--gv-text)" },
  ".cm-fold-marker": { display: "flex", alignItems: "center", height: "100%" },
  ".cm-fold-marker--closed svg": { transform: "rotate(-90deg)" },
  ".cm-foldPlaceholder": {
    backgroundColor: "var(--gv-raised)",
    border: "1px solid var(--gv-border)",
    color: "var(--gv-muted)",
    padding: "0 0.3rem",
  },
  ".cm-activeLine": { backgroundColor: "var(--gv-active-line)" },
  ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--gv-text)" },
  ".cm-cursor": { borderLeftColor: "var(--gv-text)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
    backgroundColor: "var(--gv-selection) !important",
  },
  ".cm-matchingBracket": { backgroundColor: "var(--gv-occurrence)", outline: "none" },
  ".cm-lsp-head": {
    textDecoration: "underline dotted",
    textDecorationColor: "var(--gv-muted)",
    textUnderlineOffset: "3px",
  },
  ".cm-lsp-link": { textDecoration: "underline", textUnderlineOffset: "3px", cursor: "pointer" },
  ".cm-lsp-occurrence": { backgroundColor: "var(--gv-occurrence)", borderRadius: "2px" },
  ".cm-lsp-flash": { backgroundColor: "var(--gv-flash)" },
  ".cm-tooltip.cm-tooltip-hover": {
    backgroundColor: "var(--gv-popup)",
    border: "1px solid var(--gv-border)",
    borderRadius: "6px",
    padding: "0.35rem 0.6rem",
    maxWidth: "26rem",
    maxHeight: "14rem",
    overflow: "auto",
    boxShadow: "0 4px 14px var(--gv-shadow)",
  },
  // The cards are sized to their content inside a zero-width box (see
  // mountGrammarView), so their text must not wrap.
  ".cm-lsp-refs": {
    display: "block",
    whiteSpace: "nowrap",
    textAlign: "left",
    padding: "0.1rem 0",
    font: "inherit",
    fontFamily: "var(--font)",
    fontSize: "12px",
    color: "var(--gv-text)",
    background: "none",
    border: 0,
    cursor: "pointer",
    textDecoration: "underline",
    textDecorationColor: "var(--gv-faint)",
    textUnderlineOffset: "3px",
  },
  ".cm-lsp-refs:hover": { color: "var(--gv-strong)", textDecorationColor: "var(--gv-text)" },
  ".cm-lsp-norefs": { fontFamily: "var(--font)", fontSize: "12px", color: "var(--gv-muted)", whiteSpace: "nowrap" },
  ".cm-lsp-peek pre": {
    margin: 0,
    fontFamily: "var(--font-mono)",
    fontSize: "12px",
    lineHeight: "1.45",
    color: "var(--gv-text)",
    whiteSpace: "pre",
  },
});

// LSP positions are zero-based lines and characters; the grammars are ASCII, so
// a character is a UTF-16 code unit.
const toOffset = (doc: Text, p: LspPosition) => doc.line(p.line + 1).from + p.character;
const toLsp = (doc: Text, pos: number): LspPosition => {
  const line = doc.lineAt(pos);
  return { line: line.number - 1, character: pos - line.from };
};

function wordAt(state: EditorState, pos: number) {
  const word = state.wordAt(pos);
  return word && /^[A-Za-z_]\w*$/.test(state.sliceDoc(word.from, word.to)) ? word : null;
}

// Decorations replaced wholesale by an effect: the occurrence marks, the
// underline under the pointer, and the brief flash on a jump target.
function markField() {
  const set = StateEffect.define<DecorationSet>();
  const field = StateField.define<DecorationSet>({
    create: () => Decoration.none,
    update(deco, tr) {
      for (const e of tr.effects) if (e.is(set)) return e.value;
      return deco.map(tr.changes);
    },
    provide: (f) => EditorView.decorations.from(f),
  });
  return { set, field };
}
const occurrences = markField();
const link = markField();
const flash = markField();

function highlighted(text: string) {
  const pre = document.createElement("pre");
  let at = 0;
  const plain = (to: number) => { if (to > at) pre.append(text.slice(at, to)); };
  highlightTree(iggy.parser.parse(text), highlight, (from, to, classes) => {
    plain(from);
    const span = document.createElement("span");
    span.className = classes;
    span.textContent = text.slice(from, to);
    pre.append(span);
    at = to;
  });
  plain(text.length);
  return pre;
}

function svg(paths: string) {
  return `<svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor"
    stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
}
const CHEVRON = '<svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 6l4 4 4-4"/></svg>';
const SEARCH_ICON = svg('<circle cx="7" cy="7" r="4.5"/><path d="M10.5 10.5 14 14"/>');
const LOCK_ICON = svg('<rect x="3" y="7" width="10" height="7" rx="1.5"/><path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2"/>');

interface QuickPickItem {
  label: string;
  detail?: string;
  // The offsets of the occurrence inside the label.
  mark?: [number, number];
  rule?: Rule;
  run: () => void;
}

interface QuickPickOptions {
  placeholder: string;
  active?: number;
  onActive?: (item: QuickPickItem) => void;
  onCancel?: () => void;
}

// A filterable list over the editor, in the manner of Monaco's quick pick: type
// to narrow, arrows to move, Enter to run, Escape to return to the editor.
function quickPick(host: HTMLElement, view: EditorView) {
  const root = document.createElement("div");
  root.className = "gpq";
  root.hidden = true;
  root.innerHTML = '<input class="gpq-input" type="text" spellcheck="false" autocomplete="off"><ul class="gpq-list" role="listbox"></ul>';
  host.append(root);
  const input = root.querySelector("input")!;
  const list = root.querySelector("ul")!;
  let items: QuickPickItem[] = [];
  let shown: QuickPickItem[] = [];
  let active = 0;
  let options = {} as QuickPickOptions;

  function render() {
    const q = input.value.trim().toLowerCase();
    shown = items.filter((item) => {
      let i = 0;
      for (const c of item.label.toLowerCase()) if (c === q[i]) i++;
      return i === q.length;
    });
    active = Math.min(active, Math.max(shown.length - 1, 0));
    list.replaceChildren(...shown.map((item, i) => {
      const li = document.createElement("li");
      li.className = "gpq-item" + (i === active ? " active" : "");
      li.setAttribute("role", "option");
      const label = document.createElement("span");
      label.className = "gpq-label";
      if (item.mark) {
        // The occurrence itself, inside its line.
        const [from, to] = item.mark;
        const mark = document.createElement("mark");
        mark.textContent = item.label.slice(from, to);
        label.append(item.label.slice(0, from), mark, item.label.slice(to));
      } else {
        label.textContent = item.label;
      }
      li.append(label);
      if (item.detail) {
        const detail = document.createElement("span");
        detail.className = "gpq-detail";
        detail.textContent = item.detail;
        li.append(detail);
      }
      li.addEventListener("mousedown", (e) => { e.preventDefault(); choose(i); });
      return li;
    }));
    if (!shown.length) {
      const li = document.createElement("li");
      li.className = "gpq-empty";
      li.textContent = "No matches";
      list.append(li);
    }
    list.children[active]?.scrollIntoView({ block: "nearest" });
    if (shown[active]) options.onActive?.(shown[active]);
  }

  function close(restore: boolean) {
    if (root.hidden) return;
    root.hidden = true;
    if (restore) options.onCancel?.();
    view.focus();
  }

  function choose(i: number) {
    const item = shown[i];
    if (!item) return;
    root.hidden = true;
    item.run();
  }

  input.addEventListener("input", () => { active = 0; render(); });
  input.addEventListener("keydown", (e) => {
    if (e.key === "ArrowDown") { active = (active + 1) % Math.max(shown.length, 1); render(); }
    else if (e.key === "ArrowUp") { active = (active - 1 + shown.length) % Math.max(shown.length, 1); render(); }
    else if (e.key === "Enter") choose(active);
    else if (e.key === "Escape") close(true);
    else return;
    e.preventDefault();
  });
  input.addEventListener("blur", () => setTimeout(() => close(false), 0));

  return {
    open(newItems: QuickPickItem[], newOptions: QuickPickOptions) {
      items = newItems;
      options = newOptions;
      input.value = "";
      input.placeholder = options.placeholder;
      active = options.active ?? 0;
      root.hidden = false;
      render();
      input.focus();
    },
  };
}

function lspFeatures(
  lsp: LspModule,
  source: string,
  symbols: LspSymbol[],
  view: EditorView,
  onReferences: (rule: Rule) => void,
) {
  const doc = view.state.doc;
  const rules = new Map<string, Rule>(); // name -> { head, rule } offsets, and the symbol
  for (const s of symbols) {
    rules.set(s.name, {
      symbol: s,
      head: { from: toOffset(doc, s.selectionRange.start), to: toOffset(doc, s.selectionRange.end) },
      rule: { from: toOffset(doc, s.range.start), to: toOffset(doc, s.range.end) },
    });
  }
  const folds = new Map<number, number>(
    (JSON.parse(lsp.folding(source)) as LspFoldingRange[]).map((r) => [r.startLine + 1, r.endLine + 1]),
  );

  // A nonterminal name at pos, as a word range and its rule.
  function nameAt(state: EditorState, pos: number): RuleName | null {
    const word = wordAt(state, pos);
    const rule = word && rules.get(state.sliceDoc(word.from, word.to));
    return rule ? { word: word!, rule } : null;
  }

  // A reference is a nonterminal name anywhere but its own rule head.
  function referenceAt(state: EditorState, pos: number) {
    const name = nameAt(state, pos);
    return name && name.rule.head.from !== name.word.from ? name.word : null;
  }

  function definitionAt(state: EditorState, pos: number): TextRange | null {
    const p = toLsp(state.doc, pos);
    const loc = JSON.parse(lsp.definition(source, p.line, p.character)) as LspLocation | null;
    if (!loc) return null;
    return { from: toOffset(state.doc, loc.range.start), to: toOffset(state.doc, loc.range.end) };
  }

  function reveal(from: number, to = from) {
    view.dispatch({
      selection: EditorSelection.range(from, to),
      effects: [
        EditorView.scrollIntoView(from, { y: "start", yMargin: 24 }),
        flash.set.of(Decoration.set([Decoration.line({ class: "cm-lsp-flash" }).range(view.state.doc.lineAt(from).from)])),
      ],
    });
    setTimeout(() => view.dispatch({ effects: flash.set.of(Decoration.none) }), 900);
  }

  function goToDefinition(pos: number) {
    const target = definitionAt(view.state, pos);
    if (!target) return;
    reveal(target.from);
    view.focus();
  }

  function references(pos: number): TextRange[] {
    const name = nameAt(view.state, pos);
    if (!name) return [];
    const p = toLsp(view.state.doc, name.word.from);
    return (JSON.parse(lsp.references(source, p.line, p.character, true)) as LspLocation[]).map((r) => ({
      from: toOffset(view.state.doc, r.range.start),
      to: toOffset(view.state.doc, r.range.end),
    }));
  }

  // How many places refer to a rule, its own head not counted. Counted on first
  // hover: a large grammar takes too long to count every rule up front.
  const referenceCounts = new Map<string, number>();
  function referenceCount(rule: Rule) {
    if (!referenceCounts.has(rule.symbol.name)) {
      const p = toLsp(doc, rule.head.from);
      referenceCounts.set(rule.symbol.name, (JSON.parse(lsp.references(source, p.line, p.character, false)) as LspLocation[]).length);
    }
    return referenceCounts.get(rule.symbol.name)!;
  }

  let pressed: { x: number; y: number } | null = null;

  // posAtCoords snaps to the nearest position, so the pointer can be past the
  // end of a line and still land on its last word; check it is over the word.
  function overWord(v: EditorView, word: TextRange, x: number) {
    const start = v.coordsAtPos(word.from);
    const end = v.coordsAtPos(word.to, -1);
    return !!start && !!end && x >= start.left && x <= end.right;
  }

  const occurrencePlugin = ViewPlugin.fromClass(class {
    update(u: ViewUpdate) {
      if (!u.selectionSet && !u.docChanged) return;
      const refs = references(u.state.selection.main.head);
      const deco = Decoration.set(
        refs.map((r) => Decoration.mark({ class: "cm-lsp-occurrence" }).range(r.from, r.to)),
        true,
      );
      queueMicrotask(() => u.view.dispatch({ effects: occurrences.set.of(deco) }));
    }
  });

  // A rule head offers its references; a click on the count lists them.
  function headCard(v: EditorView, name: RuleName): Tooltip {
    return {
      pos: name.word.from,
      end: name.word.to,
      above: false,
      create() {
        const count = referenceCount(name.rule);
        const dom = document.createElement("div");
        dom.className = "cm-lsp-peek";
        if (count === 0) {
          const none = document.createElement("span");
          none.className = "cm-lsp-norefs";
          none.textContent = "No references";
          dom.append(none);
        } else {
          const button = document.createElement("button");
          button.type = "button";
          button.className = "cm-lsp-refs";
          button.textContent = count === 1 ? "1 reference" : `${count} references`;
          button.addEventListener("mousedown", (event) => {
            event.preventDefault();
            v.dispatch({ effects: closeHoverTooltips });
            onReferences(name.rule);
          });
          dom.append(button);
        }
        return { dom };
      },
    };
  }

  const peek = hoverTooltip((v, pos) => {
    const name = nameAt(v.state, pos);
    if (name && name.rule.head.from === name.word.from) return headCard(v, name);
    const word = referenceAt(v.state, pos);
    if (!word) return null;
    const target = definitionAt(v.state, word.from);
    const rule = target && [...rules.values()].find((r) => r.head.from === target.from);
    if (!rule) return null;
    return {
      pos: word.from,
      end: word.to,
      above: false,
      create() {
        const dom = document.createElement("div");
        dom.className = "cm-lsp-peek";
        const lines = v.state.sliceDoc(rule.rule.from, rule.rule.to).split("\n");
        dom.append(highlighted(lines.length > 12 ? lines.slice(0, 12).join("\n") + "\n  …" : lines.join("\n")));
        return { dom };
      },
    };
  }, { hoverTime: 250 });

  const extensions = [
    occurrences.field,
    link.field,
    flash.field,
    occurrencePlugin,
    peek,
    EditorView.domEventHandlers({
      // A reference under the pointer is underlined as a link; a rule head,
      // which has a card but no click, gets a dotted underline instead.
      mousemove(event, v) {
        const pos = v.posAtCoords({ x: event.clientX, y: event.clientY });
        const name = pos == null ? null : nameAt(v.state, pos);
        let mark: (TextRange & { cls: string }) | null = null;
        if (name && overWord(v, name.word, event.clientX)) {
          const head = name.rule.head.from === name.word.from;
          mark = { ...name.word, cls: head ? "cm-lsp-head" : "cm-lsp-link" };
        }
        const current = v.state.field(link.field);
        let same = false;
        current.between(0, v.state.doc.length, (from, to) => { same = !!mark && from === mark.from && to === mark.to; });
        if (same || (!mark && current.size === 0)) return;
        v.dispatch({ effects: link.set.of(mark
          ? Decoration.set([Decoration.mark({ class: mark.cls }).range(mark.from, mark.to)])
          : Decoration.none) });
      },
      mouseleave(_event, v) {
        if (v.state.field(link.field).size) v.dispatch({ effects: link.set.of(Decoration.none) });
      },
      // CodeMirror redraws the word under the pointer between mousedown and
      // mouseup, so the browser never fires click; a press that ends where it
      // began on a reference counts as one.
      mousedown(event) {
        pressed = event.button === 0 && !event.shiftKey ? { x: event.clientX, y: event.clientY } : null;
        return false;
      },
      mouseup(event, v) {
        const start = pressed;
        pressed = null;
        if (!start || Math.abs(event.clientX - start.x) > 3 || Math.abs(event.clientY - start.y) > 3) return false;
        const pos = v.posAtCoords({ x: event.clientX, y: event.clientY });
        const word = pos == null ? null : referenceAt(v.state, pos);
        if (!word || !overWord(v, word, event.clientX)) return false;
        setTimeout(() => goToDefinition(pos!));
        return false;
      },
    }),
  ];

  const foldRanges = foldService.of((state, lineStart) => {
    const line = state.doc.lineAt(lineStart);
    const end = folds.get(line.number);
    return end ? { from: line.to, to: state.doc.line(end).to } : null;
  });

  return { extensions, foldRanges, rules, nameAt, references, reveal };
}

// The fold gutter only computes its markers when the document or viewport
// changes, so new ranges come with a new gutter rather than into the old one.
function folding(ranges: Extension) {
  return [
    ranges,
    codeFolding(),
    // An SVG chevron rather than a text glyph, which sits on the font's
    // baseline and so lower than the line's middle.
    foldGutter({ markerDOM: (open) => {
      const el = document.createElement("span");
      el.className = "cm-fold-marker" + (open ? "" : " cm-fold-marker--closed");
      el.innerHTML = CHEVRON;
      return el;
    } }),
  ];
}

interface GrammarViewOptions {
  source: string;
  name: string;
  ruleButtonHost: Element;
  tooltipHost: Element;
  panel?: HTMLElement | null;
}

// Mounts the view in `host`. `ruleButtonHost` receives the Go to rule button,
// `tooltipHost` the hover cards (an ancestor that has the --gv-* colors and
// does not clip), and `panel`, when given, delays the language server until it
// is first shown.
function mountGrammarView(host: HTMLElement, { source, name, ruleButtonHost, tooltipHost, panel }: GrammarViewOptions) {
  host.classList.add("gv-host");
  // CodeMirror gives the card container the editor's theme classes, whose
  // height: 100% would stretch it over the page; a zero-size box holds it.
  const cards = document.createElement("div");
  cards.className = "gv-cards";
  tooltipHost.append(cards);
  const lspSlot = new Compartment();
  const foldSlot = new Compartment();
  const view = new EditorView({
    parent: host,
    state: EditorState.create({
      doc: source,
      extensions: [
        lineNumbers(),
        // Mounted before the language server loads, so the code does not shift
        // when it does; the ranges arrive with a fresh gutter then.
        foldSlot.of(folding([])),
        lspSlot.of([]),
        tooltips({ parent: cards }),
        drawSelection(),
        highlightActiveLine(),
        bracketMatching(),
        keymap.of(defaultKeymap),
        iggy,
        syntaxHighlighting(highlight),
        theme,
        EditorState.readOnly.of(true),
        EditorView.contentAttributes.of({ "aria-label": `${name} grammar (read-only)`, "aria-readonly": "true" }),
      ],
    }),
  });

  // CodeMirror tokenizes near the viewport and the rest in idle chunks, so a
  // fast scroll can outrun it and show uncolored lines. The grammar never
  // changes and is small, so tokenize all of it once, up front.
  forceParsing(view, view.state.doc.length, 1000);

  // Pinned to the corner, since it describes the whole editor.
  const readOnly = document.createElement("span");
  readOnly.className = "gp-readonly";
  readOnly.innerHTML = `${LOCK_ICON}<span>read-only</span>`;
  host.append(readOnly);

  const pick = quickPick(host, view);
  let features: ReturnType<typeof lspFeatures> | null = null;

  const ruleButton = document.createElement("button");
  ruleButton.type = "button";
  ruleButton.className = "gp-rule-search";
  ruleButton.disabled = true;
  ruleButton.innerHTML = `${SEARCH_ICON}<span>Go to rule</span>`;
  ruleButton.addEventListener("click", openRules);
  ruleButtonHost.append(ruleButton);

  function openRules() {
    if (!features) return;
    const start = view.state.selection.main;
    const scrollTop = view.scrollDOM.scrollTop;
    const items = [...features.rules.values()].map((r): QuickPickItem => ({
      label: r.symbol.name,
      detail: `${r.symbol.kind === 10 ? "lexical" : "syntax"} · line ${r.symbol.selectionRange.start.line + 1}`,
      rule: r,
      run: () => { features!.reveal(r.head.from, r.head.to); view.focus(); },
    }));
    pick.open(items, {
      placeholder: "Go to rule",
      // Preview the rule under the highlighted entry, as Monaco does.
      onActive: (item) => view.dispatch({
        effects: EditorView.scrollIntoView(item.rule!.head.from, { y: "start", yMargin: 24 }),
      }),
      onCancel: () => {
        view.dispatch({ selection: start });
        view.scrollDOM.scrollTop = scrollTop;
      },
    });
  }

  function openReferences(rule: Rule) {
    const word = rule.symbol.name;
    const refs = features!.references(rule.head.from).filter((r) => r.from !== rule.head.from);
    const items = refs.map((r): QuickPickItem => {
      const line = view.state.doc.lineAt(r.from);
      const indent = line.text.length - line.text.trimStart().length;
      const column = r.from - line.from;
      return {
        label: line.text.trim(),
        mark: [column - indent, column - indent + (r.to - r.from)],
        detail: `${line.number}:${column + 1}`,
        run: () => { features!.reveal(r.from, r.to); view.focus(); },
      };
    });
    pick.open(items, {
      placeholder: `${refs.length === 1 ? "1 reference" : `${refs.length} references`} to ${word}`,
    });
  }

  let started = false;
  const start = () => {
    if (panel) ruleButton.hidden = panel.hidden;
    if (started || panel?.hidden) return;
    started = true;
    import(/* @vite-ignore */ LSP_MODULE)
      .then(async (lsp: LspModule) => {
        await lsp.default();
        const symbols = JSON.parse(lsp.document_symbols(source)) as LspSymbol[];
        features = lspFeatures(lsp, source, symbols, view, openReferences);
        view.dispatch({ effects: [
          lspSlot.reconfigure(features.extensions),
          foldSlot.reconfigure(folding(features.foldRanges)),
        ] });
        ruleButton.disabled = false;
      })
      .catch((error) => console.error("Could not load the grammar language server", error));
  };
  if (panel) new MutationObserver(start).observe(panel, { attributes: true, attributeFilter: ["hidden"] });
  start();
}

// The hero's Grammar pane: the grammar is rendered into the page at build time,
// Go to rule sits at the end of the tab row, and the language server waits for
// the pane to be shown.
for (const pane of document.querySelectorAll<HTMLElement>(".hero .gp-grammar")) {
  const source = pane.querySelector("pre code")?.textContent?.replace(/\n$/, "");
  if (source == null) continue;
  const widget = pane.closest(".gp-widget")!;
  pane.replaceChildren();
  pane.removeAttribute("tabindex");
  pane.classList.add("gp-grammar--cm");
  mountGrammarView(pane, {
    source,
    name: /^\s*grammar\s+(\w+)/m.exec(source)?.[1] ?? "grammar",
    ruleButtonHost: widget.querySelector(".gp-tabbar")!,
    tooltipHost: widget,
    panel: pane.closest<HTMLElement>(".gp-pane"),
  });
}

// A grammar page: the grammar is fetched. In the page's Grammar | Playground
// widget, Go to rule sits in the tab row and the language server waits for the
// Grammar tab, as in the hero; on its own, Go to rule sits in a row above the
// editor and the language server starts right away.
for (const root of document.querySelectorAll<HTMLElement>(".grammar-view[data-src]")) {
  const widget = root.closest(".gp-widget");
  const box = document.createElement("div");
  box.className = "grammar-view-box";
  box.textContent = "Loading the grammar…";
  let bar = widget?.querySelector(".gp-tabbar");
  if (bar) {
    root.replaceChildren(box);
  } else {
    bar = document.createElement("div");
    bar.className = "grammar-view-bar";
    root.replaceChildren(bar, box);
  }
  fetch(root.dataset.src!)
    .then((response) => {
      if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
      return response.text();
    })
    .then((source) => {
      box.textContent = "";
      mountGrammarView(box, {
        source: source.replace(/\n$/, ""),
        name: root.dataset.name ?? "grammar",
        ruleButtonHost: bar!,
        tooltipHost: widget ?? root,
        panel: widget ? root.closest<HTMLElement>(".gp-pane") : undefined,
      });
    })
    .catch((error) => {
      box.textContent = "Could not load the grammar.";
      console.error("Could not load " + root.dataset.src, error);
    });
}
