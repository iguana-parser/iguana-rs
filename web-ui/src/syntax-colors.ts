// The input editor's syntax colors: the highlighting that the parser returns
// with each parse, drawn as CodeMirror marks in the colors of
// syntax-theme.json.
import { type Range, StateEffect, StateField, type Text } from "@codemirror/state";
import { Decoration, type DecorationSet, EditorView } from "@codemirror/view";
import type { Highlight } from "./backend";
import { groupClass, themeGroup, themeStyles } from "./syntax-theme";
import { TextPositions } from "./text-positions";

// The highlighting of a parsed text.
export interface Syntax {
  text: string;
  highlights: Highlight[];
}

interface SyntaxState {
  syntax: Syntax | null;
  deco: DecorationSet;
}

// Sets the latest highlighting, or null to clear the colors.
export const setSyntax = StateEffect.define<Syntax | null>();

// The latest highlighting and the decorations drawn from it. While the
// document is the parsed text, also after an edit or an undo restores it, the
// decorations come from the highlighting. Otherwise the last decorations move
// with the edits.
export const syntaxColors = StateField.define<SyntaxState>({
  create: () => ({ syntax: null, deco: Decoration.none }),
  update(value, tr) {
    let { syntax, deco } = value;
    for (const e of tr.effects) if (e.is(setSyntax)) syntax = e.value;
    if (syntax == null) return { syntax, deco: Decoration.none };
    if (syntax === value.syntax && !tr.docChanged) return value;
    // The length check comes first. Most edits of a large document then skip
    // the comparison of the whole text.
    const doc = tr.state.doc;
    if (doc.length === syntax.text.length && doc.toString() === syntax.text) {
      deco = buildSyntax(syntax, doc);
    } else if (tr.docChanged) {
      deco = deco.map(tr.changes);
    }
    return { syntax, deco };
  },
  provide: (f) => EditorView.decorations.from(f, (state) => state.deco),
});

// `doc` is the document of `text`.
function buildSyntax({ text, highlights }: Syntax, doc: Text): DecorationSet {
  // The parser's offsets are in characters, and a column in a CodeMirror
  // document is in UTF-16 code units. `TextPositions.positions` converts the
  // start and the end of every span in one call, and returns a line and a
  // column for each.
  const ends = highlights.flatMap((h) => [h.span.left_extent, h.span.right_extent]);
  const positions = new TextPositions(text);
  const lineColumns = positions.positions(Uint32Array.from(ends));
  positions.free();
  const at = (end: number) => doc.line(lineColumns[2 * end] + 1).from + lineColumns[2 * end + 1];
  const ranges: Range<Decoration>[] = [];
  highlights.forEach((h, i) => {
    const from = at(2 * i);
    const to = at(2 * i + 1);
    const mark = groupMark(h.group);
    if (mark && to > from) ranges.push(mark.range(from, to));
  });
  return Decoration.set(ranges, true);
}

export const syntaxTheme = EditorView.theme(themeStyles());

// One mark per group, which every token of the group shares, or null for a
// group without a color.
const marks = new Map<string, Decoration | null>();
function groupMark(group: string): Decoration | null {
  if (!marks.has(group)) {
    const colored = themeGroup(group);
    marks.set(group, colored ? Decoration.mark({ class: groupClass(colored) }) : null);
  }
  return marks.get(group)!;
}
