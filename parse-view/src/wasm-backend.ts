import type { ParseOutput, ParserBackend, ParseTreeResult } from "./backend";

// The wasm wrapper's exported parse function. It returns the result envelope as
// a JSON string for any parse that ran, and throws when the parser could not run
// at all (e.g. an unknown start nonterminal). The viewer initializes the wasm
// module and passes its bound `parse` here, so this backend stays grammar-agnostic.
export type WasmParse = (input: string, startNonterminal: string) => string;

// The envelope the wasm wrapper returns, mirroring ParseOutput without the
// SPPF/GSS flags (the web view has no SPPF/GSS).
interface ResultEnvelope {
  success: boolean;
  error: string | null;
  error_info: { line: number; column: number; message: string } | null;
  duration_ms: number | null;
  tree_construction_ms: number | null;
  parse_tree: string | null;
}

// Drives a generated parser compiled to wasm, running in the same page. The wasm
// wrapper does the parse and serializes the tree, so a parse is a single call.
export class WasmBackend implements ParserBackend {
  constructor(private wasmParse: WasmParse) {}

  async parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }> {
    let envelopeJson: string;
    try {
      envelopeJson = this.wasmParse(input, startNonterminal);
    } catch (e) {
      return { error: e instanceof Error ? e.message : String(e) };
    }
    const envelope: ResultEnvelope = JSON.parse(envelopeJson);
    const output: ParseOutput = {
      success: envelope.success,
      error: envelope.error,
      error_info: envelope.error_info,
      duration_ms: envelope.duration_ms,
      tree_construction_ms: envelope.tree_construction_ms,
      has_sppf: false,
      has_gss: false,
      parse_tree: envelope.parse_tree,
    };
    return { output, treeJson: envelope.parse_tree };
  }
}
