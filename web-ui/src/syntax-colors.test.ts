import { EditorState } from "@codemirror/state";
import { describe, expect, it } from "vitest";

import type { Highlight } from "./backend";
import { setSyntax, syntaxColors } from "./syntax-colors";

// The text and CSS class of each colored range.
function colored(state: EditorState): [string, string][] {
  const ranges: [string, string][] = [];
  state.field(syntaxColors).deco.between(0, state.doc.length, (from, to, deco) => {
    ranges.push([state.sliceDoc(from, to), deco.spec.class]);
  });
  return ranges;
}

const highlight = (left: number, right: number, group: string): Highlight => ({
  span: { left_extent: left, right_extent: right },
  group,
});

// `"😀" x`: the emoji is one character and two UTF-16 code units.
const text = '"😀" x';
const syntax = { text, highlights: [highlight(0, 3, "string"), highlight(4, 5, "variable")] };

function parsed() {
  const state = EditorState.create({ doc: text, extensions: [syntaxColors] });
  return state.update({ effects: setSyntax.of(syntax) }).state;
}

describe("syntaxColors", () => {
  it("colors the parsed text at UTF-16 offsets", () => {
    expect(colored(parsed())).toEqual([
      ['"😀"', "hl-string"],
      ["x", "hl-variable"],
    ]);
  });

  it("moves the colors with an edit", () => {
    const state = parsed().update({ changes: { from: 0, insert: "y " } }).state;
    expect(colored(state)).toEqual([
      ['"😀"', "hl-string"],
      ["x", "hl-variable"],
    ]);
  });

  it("restores the colors when the document is the parsed text again", () => {
    // Replacing the text drops the colors inside it. Putting the parsed
    // text back draws them from the parse result again.
    const replaced = parsed().update({ changes: { from: 0, to: text.length, insert: "z" } }).state;
    expect(colored(replaced)).toEqual([]);
    const restored = replaced.update({ changes: { from: 0, to: 1, insert: text } }).state;
    expect(colored(restored)).toEqual([
      ['"😀"', "hl-string"],
      ["x", "hl-variable"],
    ]);
  });

  it("colors a group with the nearest group of the theme, and leaves one without a color plain", () => {
    const state = EditorState.create({ doc: "x + y", extensions: [syntaxColors] });
    const groups = [highlight(0, 1, "variable.parameter"), highlight(2, 3, "operator"), highlight(4, 5, "string.escape")];
    expect(colored(state.update({ effects: setSyntax.of({ text: "x + y", highlights: groups }) }).state)).toEqual([
      ["x", "hl-variable"],
      ["y", "hl-string-escape"],
    ]);
  });

  it("colors a text of several lines", () => {
    // The emoji on the first line does not move the colors on the second.
    const lines = '"😀"\nx\n';
    const state = EditorState.create({ doc: lines, extensions: [syntaxColors] });
    const highlights = [highlight(0, 3, "string"), highlight(4, 5, "variable")];
    expect(colored(state.update({ effects: setSyntax.of({ text: lines, highlights }) }).state)).toEqual([
      ['"😀"', "hl-string"],
      ["x", "hl-variable"],
    ]);
  });

  it("keeps the moved colors when a result for an older text arrives", () => {
    const edited = parsed().update({ changes: { from: 0, insert: "y " } }).state;
    const late = { text, highlights: [highlight(4, 5, "string")] };
    expect(colored(edited.update({ effects: setSyntax.of(late) }).state)).toEqual([
      ['"😀"', "hl-string"],
      ["x", "hl-variable"],
    ]);
  });

  it("clears the colors without a parse result", () => {
    expect(colored(parsed().update({ effects: setSyntax.of(null) }).state)).toEqual([]);
  });
});
