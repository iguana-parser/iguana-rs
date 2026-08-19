import { describe, expect, it } from "vitest";

import type { ParseOutput } from "./backend";
import { WasmBackend } from "./wasm-backend";

const success: ParseOutput = {
  error: null,
  parse_ms: 2,
  tree_construction_ms: 1,
  parse_tree: '{"id":0}',
};

const failure: ParseOutput = {
  error: { span: { left_extent: 4, right_extent: 7 }, message: "Expected Id" },
  parse_ms: null,
  tree_construction_ms: null,
  parse_tree: null,
};

describe("WasmBackend.parse", () => {
  it("returns the runtime output and the inline tree on success", async () => {
    const backend = new WasmBackend(() => JSON.stringify(success));
    expect(await backend.parse("a", "S")).toEqual({
      output: success,
      unexpected_error: null,
      treeJson: success.parse_tree,
    });
  });

  it("returns a parse failure as a normal result", async () => {
    const backend = new WasmBackend(() => JSON.stringify(failure));
    expect(await backend.parse("a", "S")).toEqual({
      output: failure,
      unexpected_error: null,
      treeJson: null,
    });
  });

  it("reports a throwing parser as a host-level error", async () => {
    const backend = new WasmBackend(() => {
      throw new Error("unknown start nonterminal: X");
    });
    expect(await backend.parse("a", "X")).toEqual({ error: "unknown start nonterminal: X" });
  });
});
