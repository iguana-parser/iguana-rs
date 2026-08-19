import type { ParseOutput, ParserBackend, ParseTreeResult } from "./backend";

// The wasm wrapper's exported parse function. It returns the runtime's
// ParseOutput as a JSON string for any parse that ran, and throws when the
// parser could not run at all (e.g. an unknown start nonterminal). The viewer
// initializes the wasm module and passes its bound `parse` here, so this
// backend stays grammar-agnostic.
export type WasmParse = (input: string, startNonterminal: string) => string;

// Drives a generated parser compiled to wasm, running in the same page. The wasm
// wrapper does the parse and serializes the tree, so a parse is a single call.
export class WasmBackend implements ParserBackend {
  constructor(private wasmParse: WasmParse) {}

  async parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }> {
    let outputJson: string;
    try {
      outputJson = this.wasmParse(input, startNonterminal);
    } catch (e) {
      return { error: e instanceof Error ? e.message : String(e) };
    }
    const output: ParseOutput = JSON.parse(outputJson);
    // The wasm wrapper throws on an unexpected error (caught above), so an
    // output it returns is always a normal parse result.
    return { output, unexpected_error: null, treeJson: output.parse_tree };
  }
}
