// Parse error location and message, mirroring the parser's --write-result JSON.
export interface ParseErrorInfo {
  line: number;
  column: number;
  message: string;
}

// The result of a parse that ran, independent of the host. It mirrors the shape
// Terrarium's parse command returns and the wasm wrapper's envelope, so both
// backends produce it without a translation layer. has_sppf/has_gss are debug
// artifacts the parse view does not show; the web backend leaves them false.
export interface ParseOutput {
  success: boolean;
  error: string | null;
  error_info: ParseErrorInfo | null;
  duration_ms: number | null;
  tree_construction_ms: number | null;
  has_sppf: boolean;
  has_gss: boolean;
  parse_tree: string | null;
}

// A single parse: the result metadata plus the parse-tree JSON, fetched in the
// same step. treeJson is null when the parse produced no tree (a parse failure
// with no partial tree). The JSON matches the parser's --write-parse-tree output.
export interface ParseTreeResult {
  output: ParseOutput;
  treeJson: string | null;
}

// A parser the ParseView drives, independent of how it runs. Terrarium's
// TauriBackend runs a generated parser as a subprocess; the web viewer's
// WasmBackend calls a wasm-compiled parser. parse resolves to the tree result,
// or to an error when the parser could not be run at all (distinct from a parse
// that ran and failed, which is a successful call with output.success == false).
export interface ParserBackend {
  parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }>;
}
