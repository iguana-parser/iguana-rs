// Conversions between the runtime's character indexes and JavaScript string
// offsets. The runtime counts one index per Unicode scalar value (a Rust
// char), while JavaScript strings and Monaco offsets count UTF-16 code units,
// where a character outside the Basic Multilingual Plane takes two. Both
// walks stop at the end of the text, so an index past the end clamps.

// The UTF-16 offset of the character at `index`.
export function utf16Offset(text: string, index: number): number {
  let offset = 0;
  for (let remaining = index; remaining > 0 && offset < text.length; remaining--) {
    offset += text.codePointAt(offset)! > 0xffff ? 2 : 1;
  }
  return offset;
}

// The UTF-16 offset of each character in `text`, with one trailing entry for
// the end of the text so an end-of-input index resolves too. Returns null
// when the text holds no surrogate half, where the index and the offset are
// the same number; callers use null as the fast path instead of a table.
export function utf16OffsetTable(text: string): Uint32Array | null {
  if (!/[\uD800-\uDBFF]/.test(text)) return null;
  const offsets: number[] = [];
  let offset = 0;
  while (offset < text.length) {
    offsets.push(offset);
    offset += text.codePointAt(offset)! > 0xffff ? 2 : 1;
  }
  offsets.push(text.length);
  return Uint32Array.from(offsets);
}

// The character index at the UTF-16 offset `offset`. An offset inside a
// surrogate pair counts as past the character it splits.
export function charIndex(text: string, offset: number): number {
  let index = 0;
  for (let at = 0; at < offset && at < text.length; index++) {
    at += text.codePointAt(at)! > 0xffff ? 2 : 1;
  }
  return index;
}
