import { commands } from "../bindings";
import type { ParserBackend, ParseTreeResult } from "@iguana-parser/web-ui";

// Drives a generated parser that Terrarium runs as a subprocess. The parse
// command runs the parser and returns the tree JSON inline, so ParseView sees
// a single call.
export class TauriBackend implements ParserBackend {
  constructor(private directory: string) {}

  async parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }> {
    const run = await commands.parse(this.directory, input, startNonterminal);
    if (run.status === "error") return { error: run.error };
    return {
      summary: run.data.summary,
      unexpected_error: run.data.unexpected_error,
      treeJson: run.data.parse_tree,
      highlights: null,
    };
  }
}
