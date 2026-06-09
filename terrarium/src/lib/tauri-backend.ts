import { commands } from "../bindings";
import type { ParserBackend, ParseTreeResult } from "./parse-view/backend";

// Drives a generated parser that Terrarium runs as a subprocess. The parse
// command runs the parser and returns the tree JSON inline, so ParseView sees
// a single call.
export class TauriBackend implements ParserBackend {
  constructor(private directory: string) {}

  async parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }> {
    const result = await commands.parse(this.directory, input, startNonterminal);
    if (result.status === "error") return { error: result.error };
    return { output: result.data, treeJson: result.data.parse_tree };
  }
}
