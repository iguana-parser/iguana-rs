import { describe, expect, it } from "vitest";

import { TextPositions } from "./text-positions";

// "😀" is one character but two UTF-16 code units, so every case with it
// exercises the difference between the two columns. The runtime's own tests
// cover the conversion; these check that the wasm module loads and answers.
describe("TextPositions", () => {
  it("gives a column in UTF-16", () => {
    const text = new TextPositions("a😀x\nb");
    expect(Array.from(text.position(2))).toEqual([0, 3]);
    expect(Array.from(text.position(4))).toEqual([1, 0]);
    expect(text.offset(0, 3)).toBe(2);
    // A column in the middle of a surrogate pair is the column of its
    // character.
    expect(text.offset(0, 2)).toBe(1);
  });

  it("puts the end of the text after a final newline on the next line", () => {
    const text = new TextPositions("a\n");
    expect(Array.from(text.position(2))).toEqual([1, 0]);
    expect(Array.from(text.position(9))).toEqual([1, 0]);
  });

  it("puts a position past the end of its line at the end of the line", () => {
    const text = new TextPositions("ab\nc");
    expect(text.offset(0, 9)).toBe(2);
    expect(text.offset(1, 9)).toBe(4);
    expect(text.offset(5, 0)).toBe(4);
  });
});
