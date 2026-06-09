import type { ParseOutput } from "../../bindings";

// A single parse: the result metadata plus the parse-tree JSON, fetched in the
// same step. treeJson is null when the parse produced no tree (a parse failure
// with no partial tree). The JSON matches the parser's --write-parse-tree output.
export interface ParseTreeResult {
  output: ParseOutput;
  treeJson: string | null;
}

// A parser the ParseView drives, independent of how it runs. Terrarium's
// TauriBackend runs a generated parser as a subprocess; the web viewer's backend
// calls a wasm-compiled parser. parse resolves to the tree result, or to an error
// when the parser could not be run at all (distinct from a parse that ran and
// failed, which is a successful call with output.success == false).
export interface ParserBackend {
  parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }>;
}
