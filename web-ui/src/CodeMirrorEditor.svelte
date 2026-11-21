<script lang="ts">
  // A CodeMirror 6 input editor, used when a host asks for editor="codemirror":
  // a real editor (undo history, selection, line numbers) at a fraction of
  // Monaco's size. It has InputEditor's selection-sync contract (highlightSpan,
  // onclick, onescape) and, like InputEditor, marks the parse error and each
  // ambiguity with a squiggle whose message shows on hover. The error's line
  // number is also red.
  import { onMount } from "svelte";
  import { Compartment, EditorState, RangeSet, StateEffect, StateField } from "@codemirror/state";
  import {
    Decoration,
    type DecorationSet,
    EditorView,
    GutterMarker,
    gutterLineClass,
    hoverTooltip,
    keymap,
    WidgetType,
    lineNumbers,
    placeholder as placeholderExt,
    drawSelection,
  } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import type { ParseError } from "./backend";
  import { TextPositions } from "./text-positions";
  import { type Syntax, setSyntax, syntaxColors, syntaxTheme } from "./syntax-colors";
  import { CODE_FONT } from "./theme";

  interface HighlightSpan {
    start: number;
    end: number;
  }

  interface AmbiguityWarning {
    start: number;
    end: number;
    message: string;
  }

  interface Props {
    value: string;
    placeholder?: string;
    readOnly?: boolean;
    error?: ParseError | null;
    ambiguities?: AmbiguityWarning[];
    highlightSpan?: HighlightSpan | null;
    // The highlighting of the last parse (`syntaxColors`).
    syntax?: Syntax | null;
    onchange?: (value: string) => void;
    onclick?: (offset: number) => void;
    onescape?: () => void;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    readOnly = false,
    error = null,
    ambiguities = [],
    highlightSpan = null,
    syntax = null,
    onchange,
    onclick,
    onescape,
  }: Props = $props();

  let host: HTMLDivElement | undefined;
  let view: EditorView | undefined;
  // The extensions that follow the readOnly and placeholder props.
  const readOnlyConf = new Compartment();
  const placeholderConf = new Compartment();

  const WAVE =
    "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='6' height='4'%3E" +
    "%3Cpath d='M0 3 Q1.5 0 3 3 T6 3' fill='none' stroke='black' stroke-width='1'/%3E%3C/svg%3E\")";

  const theme = EditorView.theme({
    "&": {
      height: "100%",
      color: "var(--iguana-text, #d4d4d4)",
      backgroundColor: "var(--iguana-bg, #1e1e1e)",
      fontSize: "13px",
    },
    "&.cm-focused": { outline: "none" },
    ".cm-scroller": {
      fontFamily: `var(--iguana-font-mono, ${CODE_FONT})`,
      fontVariantLigatures: "none",
      lineHeight: "1.5",
    },
    ".cm-content": { padding: "12px 0 8px", caretColor: "var(--iguana-text, #d4d4d4)" },
    ".cm-line": { padding: "0 10px 0 4px" },
    ".cm-gutters": {
      backgroundColor: "var(--iguana-bg, #1e1e1e)",
      color: "var(--iguana-text-muted, #6e7681)",
      border: "none",
      paddingLeft: "6px",
    },
    ".cm-cursor": { borderLeftColor: "var(--iguana-text, #d4d4d4)" },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
      backgroundColor: "var(--iguana-selection, #264f78) !important",
    },
    ".cm-placeholder": { color: "var(--iguana-text-muted, #6a6a6a)" },
    ".cm-lineNumbers .cme-gutter-err": { color: "var(--iguana-error, #f14c4c)" },
    ".cme-span": { backgroundColor: "var(--iguana-selection, #264f78)" },
    // An empty span's caret, drawn without moving the text around it.
    ".cme-caret": {
      borderLeft: "2px solid var(--iguana-accent, #569cd6)",
      marginLeft: "-1px",
      marginRight: "-1px",
    },
    ".cme-err": {
      textDecoration: "underline wavy var(--iguana-error, #f14c4c)",
      textDecorationSkipInk: "none",
      textDecorationThickness: "1px",
      textUnderlineOffset: "3px",
    },
    ".cme-amb": {
      textDecoration: "underline wavy var(--iguana-warning, #cca700)",
      textDecorationSkipInk: "none",
      textDecorationThickness: "1px",
      textUnderlineOffset: "3px",
    },
    // A wave drawn as a mask over the mark's color: Safari does not paint a text
    // underline under the lone space this mark would otherwise be.
    ".cme-eol": {
      display: "inline-block",
      width: "1ch",
      height: "4px",
      verticalAlign: "-4px",
      maskImage: WAVE,
      WebkitMaskImage: WAVE,
      maskSize: "6px 4px",
      WebkitMaskSize: "6px 4px",
      maskRepeat: "repeat-x",
      WebkitMaskRepeat: "repeat-x",
    },
    ".cme-eol.cme-err": { backgroundColor: "var(--iguana-error, #f14c4c)" },
    ".cme-eol.cme-amb": { backgroundColor: "var(--iguana-warning, #cca700)" },
    // CodeMirror wraps the hover messages in its own tooltip box.
    ".cm-tooltip.cm-tooltip-hover": {
      backgroundColor: "var(--iguana-bg-raised, #252526)",
      color: "var(--iguana-text, #d4d4d4)",
      border: "1px solid var(--iguana-border, #454545)",
      borderRadius: "3px",
      maxWidth: "480px",
    },
    ".cme-tooltip": { padding: "4px 8px", fontSize: "12px" },
  });

  // The tree-selected span, the parse error, and the ambiguities, as one
  // decoration set that the component replaces whenever a prop changes, plus
  // the error's line in the gutter and the messages that show on hover.
  interface Message {
    from: number;
    to: number;
    text: string;
  }
  interface Marks {
    deco: DecorationSet;
    gutter: RangeSet<GutterMarker>;
    messages: Message[];
  }
  const setMarks = StateEffect.define<Marks>();
  const marks = StateField.define<Marks>({
    create: () => ({ deco: Decoration.none, gutter: RangeSet.empty, messages: [] }),
    update(value, tr) {
      for (const e of tr.effects) if (e.is(setMarks)) return e.value;
      if (!tr.docChanged) return value;
      return {
        deco: value.deco.map(tr.changes),
        gutter: value.gutter.map(tr.changes),
        messages: value.messages.map((m) => ({
          ...m,
          from: tr.changes.mapPos(m.from),
          to: tr.changes.mapPos(m.to),
        })),
      };
    },
    provide: (f) => [
      EditorView.decorations.from(f, (m) => m.deco),
      gutterLineClass.from(f, (m) => m.gutter),
    ],
  });

  // A mark at a line break or at the end of the input has no character to
  // underline, so a squiggle one column wide past the end of the line shows it.
  class EndOfLineMark extends WidgetType {
    kind: string;
    constructor(kind: string) {
      super();
      this.kind = kind;
    }
    toDOM() {
      const span = document.createElement("span");
      span.className = `cme-eol ${this.kind}`;
      return span;
    }
    eq(other: EndOfLineMark) {
      return other.kind === this.kind;
    }
  }

  class SpanCaret extends WidgetType {
    toDOM() {
      const span = document.createElement("span");
      span.className = "cme-caret";
      return span;
    }
    eq() {
      return true;
    }
  }

  class ErrorLine extends GutterMarker {
    elementClass = "cme-gutter-err";
  }
  const errorLine = new ErrorLine();

  function buildMarks(state: EditorState): Marks {
    const doc = state.doc.toString();
    const len = doc.length;
    const ranges = [];
    let gutter: RangeSet<GutterMarker> = RangeSet.empty;
    // The parser's offsets are in characters, and a column in a CodeMirror
    // document is in UTF-16 code units. `at` gives the document offset of a
    // parser offset.
    const positions = new TextPositions(doc);
    const at = (offset: number) => {
      const [line, column] = positions.position(offset);
      return state.doc.line(line + 1).from + column;
    };
    if (highlightSpan) {
      const s = at(highlightSpan.start);
      const e = Math.max(s, at(highlightSpan.end));
      if (e > s) ranges.push(Decoration.mark({ class: "cme-span" }).range(s, e));
      else ranges.push(Decoration.widget({ widget: new SpanCaret(), side: -1 }).range(s));
    }
    // A squiggle of `kind` under the characters `start` to `end`, with `text`
    // shown on hover. An empty span marks the character after it, and a span
    // with nothing visible to underline (empty at a line break or at the end of
    // the input, or only line breaks) gets the end-of-line squiggle at its start.
    const messages: Message[] = [];
    const squiggle = (start: number, end: number, kind: string, text: string) => {
      const s = at(start);
      let e = Math.max(s, at(end));
      if (e === s && s < len && doc[s] !== "\n") e = at(start + 1);
      if (/^\n*$/.test(doc.slice(s, e))) {
        ranges.push(Decoration.widget({ widget: new EndOfLineMark(kind), side: 1 }).range(s));
        messages.push({ from: s, to: s, text });
      } else {
        ranges.push(Decoration.mark({ class: kind }).range(s, e));
        messages.push({ from: s, to: e, text });
      }
      return s;
    };
    if (error) {
      const s = squiggle(error.span.left_extent, error.span.right_extent, "cme-err", `Parse error: ${error.message}`);
      gutter = RangeSet.of(errorLine.range(state.doc.lineAt(s).from));
    }
    for (const a of ambiguities) squiggle(a.start, a.end, "cme-amb", a.message);
    positions.free();
    return { deco: Decoration.set(ranges, true), gutter, messages };
  }

  // The messages of the marks under the pointer, in one tooltip. A mark's last
  // character counts from its left side only, so the character after a mark
  // does not show it; an end-of-line squiggle sits at a single position.
  const messageTooltip = hoverTooltip((view, pos, side) => {
    const hits = view.state.field(marks).messages.filter(
      (m) => (m.from === m.to ? pos === m.from : m.from <= pos && (pos < m.to || (pos === m.to && side < 0))),
    );
    if (hits.length === 0) return null;
    return {
      pos: Math.min(...hits.map((m) => m.from)),
      end: Math.max(...hits.map((m) => m.to)),
      above: true,
      create() {
        const dom = document.createElement("div");
        dom.className = "cme-tooltip";
        for (const m of hits) {
          const line = document.createElement("div");
          line.textContent = m.text;
          dom.appendChild(line);
        }
        return { dom };
      },
    };
  });

  export function focus() {
    view?.focus();
  }

  onMount(() => {
    view = new EditorView({
      parent: host!,
      state: EditorState.create({
        doc: value,
        extensions: [
          lineNumbers(),
          history(),
          drawSelection(),
          keymap.of([
            { key: "Escape", run: () => (onescape?.(), false) },
            ...defaultKeymap,
            ...historyKeymap,
          ]),
          syntaxColors,
          syntaxTheme,
          theme,
          marks,
          messageTooltip,
          placeholderConf.of(placeholderExt(placeholder)),
          readOnlyConf.of(EditorState.readOnly.of(readOnly)),
          EditorView.updateListener.of((u) => {
            if (!u.docChanged) return;
            value = u.state.doc.toString();
            onchange?.(value);
          }),
          EditorView.domEventHandlers({
            click: (_e, v) => {
              if (!onclick) return false;
              const head = v.state.selection.main.head;
              const line = v.state.doc.lineAt(head);
              const positions = new TextPositions(v.state.doc.toString());
              onclick(positions.offset(line.number - 1, head - line.from));
              positions.free();
              return false;
            },
          }),
        ],
      }),
    });
    return () => view?.destroy();
  });

  // A value set from outside (a sample, a reset) replaces the document.
  $effect(() => {
    const v = value;
    if (view && v !== view.state.doc.toString()) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } });
    }
  });

  $effect(() => {
    const effects = [
      readOnlyConf.reconfigure(EditorState.readOnly.of(readOnly)),
      placeholderConf.reconfigure(placeholderExt(placeholder)),
    ];
    view?.dispatch({ effects });
  });

  $effect(() => {
    const current = syntax;
    view?.dispatch({ effects: setSyntax.of(current) });
  });

  $effect(() => {
    void value;
    void highlightSpan;
    void error;
    void ambiguities;
    if (view) view.dispatch({ effects: setMarks.of(buildMarks(view.state)) });
  });
</script>

<div class="cme" bind:this={host}></div>

<style>
  .cme {
    height: 100%;
    overflow: hidden;
  }
  .cme :global(.cm-editor) {
    height: 100%;
  }
</style>
