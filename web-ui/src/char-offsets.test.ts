import { describe, expect, it } from "vitest";

import { charIndex, utf16Offset, utf16OffsetTable } from "./char-offsets";

// "😀" is one runtime character but two UTF-16 code units, so every case
// with it exercises the difference between the two counts.
describe("utf16Offset", () => {
  it("passes ASCII text through unchanged", () => {
    expect(utf16Offset("let x", 4)).toBe(4);
  });

  it("locates a span after a non-BMP character", () => {
    // The runtime span [1, 2) of "😀x" is "x", which begins at UTF-16
    // offset 2.
    expect(utf16Offset("😀x", 1)).toBe(2);
    expect(utf16Offset("😀x", 2)).toBe(3);
  });

  it("locates an empty end-of-input span after a non-BMP character", () => {
    // A failure at the end of "a😀" has the empty span [2, 2), and the
    // UTF-16 end of the string is offset 3.
    expect(utf16Offset("a😀", 2)).toBe(3);
  });

  it("clamps an index past the end of the text", () => {
    expect(utf16Offset("ab", 5)).toBe(2);
  });
});

describe("utf16OffsetTable", () => {
  it("returns null when every character is one code unit", () => {
    expect(utf16OffsetTable("let x\ny")).toBeNull();
  });

  it("matches the single conversion at every index", () => {
    const text = "a😀x\n😀";
    const table = utf16OffsetTable(text)!;
    for (let index = 0; index < table.length; index++) {
      expect(table[index]).toBe(utf16Offset(text, index));
    }
  });

  it("ends with the end of the text", () => {
    const table = utf16OffsetTable("😀")!;
    expect(table.length).toBe(2);
    expect(table[1]).toBe(2);
  });
});

describe("charIndex", () => {
  it("passes ASCII offsets through unchanged", () => {
    expect(charIndex("let x", 4)).toBe(4);
  });

  it("counts a non-BMP character once", () => {
    expect(charIndex("😀x", 2)).toBe(1);
    expect(charIndex("😀x", 3)).toBe(2);
  });

  it("clamps an offset past the end of the text", () => {
    expect(charIndex("ab", 5)).toBe(2);
  });
});
