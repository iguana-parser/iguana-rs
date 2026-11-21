import { describe, expect, it } from "vitest";

import { type WasmParseOutput, WasmBackend } from "./wasm-backend";

const highlights = [{ span: { left_extent: 0, right_extent: 1 }, group: "keyword" }];

const success: WasmParseOutput = {
  error: null,
  parse_ms: 2,
  tree_construction_ms: 1,
  parse_tree: '{"id":0}',
  highlights,
};

const failure: WasmParseOutput = {
  error: { span: { left_extent: 4, right_extent: 7 }, message: "Expected Id" },
  parse_ms: null,
  tree_construction_ms: null,
  parse_tree: null,
  highlights: null,
};

describe("WasmBackend.parse", () => {
  it("splits the reply into the summary, the tree, and the highlights", async () => {
    const backend = new WasmBackend(() => JSON.stringify(success));
    expect(await backend.parse("a", "S")).toEqual({
      summary: { error: null, parse_ms: 2, tree_construction_ms: 1 },
      unexpected_error: null,
      treeJson: '{"id":0}',
      highlights,
    });
  });

  it("returns a parse failure as a normal result", async () => {
    const backend = new WasmBackend(() => JSON.stringify(failure));
    expect(await backend.parse("a", "S")).toEqual({
      summary: { error: failure.error, parse_ms: null, tree_construction_ms: null },
      unexpected_error: null,
      treeJson: null,
      highlights: null,
    });
  });

  it("reports a throwing parser as a host-level error", async () => {
    const backend = new WasmBackend(() => {
      throw new Error("unknown start nonterminal: X");
    });
    expect(await backend.parse("a", "X")).toEqual({ error: "unknown start nonterminal: X" });
  });
});
