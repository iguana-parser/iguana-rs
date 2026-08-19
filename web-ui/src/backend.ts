// A character range in the input: left extent inclusive, right exclusive.
export interface Span {
  left_extent: number;
  right_extent: number;
}

// The parse failure the runtime reports: the input span the failure covers and
// a message naming what was expected. Consumers convert the span to whatever
// position form they render (Monaco markers use getPositionAt).
export interface ParseError {
  span: Span;
  message: string;
}

// The runtime's report of one parse, the shape --write-result writes and the
// wasm wrapper returns. A failure sets error; a success sets the timings.
// parse_tree is inline only in the wasm envelope; the subprocess writes the
// tree to its own file and Terrarium's backend fills the field from it.
export interface ParseOutput {
  error: ParseError | null;
  parse_ms: number | null;
  tree_construction_ms: number | null;
  parse_tree: string | null;
}

// A single parse attempt. output is the runtime's report, null when the parser
// did not run to a result (it crashed or wrote no report); unexpected_error
// then holds the detail. treeJson is the parse-tree JSON, null when the parse
// produced no tree; it matches the parser's --write-parse-tree output.
export interface ParseTreeResult {
  output: ParseOutput | null;
  unexpected_error: string | null;
  treeJson: string | null;
}

// A parser the ParseView drives, independent of how it runs. Terrarium's
// TauriBackend runs a generated parser as a subprocess; the web viewer's
// WasmBackend calls a wasm-compiled parser. parse resolves to the tree result,
// or to an error when the parser could not be run at all (distinct from a parse
// that ran and failed, which is a result whose output.error is set).
export interface ParserBackend {
  parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }>;
}
