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

// A span of the input and its highlight group, such as "keyword" or
// "string.escape" (the runtime's Highlight).
export interface Highlight {
  span: Span;
  group: string;
}

// What a parse did, without its tree (the runtime's ParseSummary), the shape
// --write-result writes. A failure sets error; a success sets the timings.
export interface ParseSummary {
  error: ParseError | null;
  parse_ms: number | null;
  tree_construction_ms: number | null;
}

// A single parse attempt. summary is the runtime's summary, null when the
// parser did not run to a result (it crashed or wrote no summary);
// unexpected_error then holds the detail. treeJson is the parse-tree JSON, null
// when the parse produced no tree; it matches the parser's --write-parse-tree
// output. highlights is the highlighting of the input, from the tree or, when
// the parse failed, from the scanner's tokens. It is null when the backend
// does not highlight.
export interface ParseTreeResult {
  summary: ParseSummary | null;
  unexpected_error: string | null;
  treeJson: string | null;
  highlights: Highlight[] | null;
}

// A parser the ParseView drives, independent of how it runs. Terrarium's
// TauriBackend runs a generated parser as a subprocess; the web viewer's
// WasmBackend calls a wasm-compiled parser. parse resolves to the tree result,
// or to an error when the parser could not be run at all (distinct from a parse
// that ran and failed, which is a result whose summary.error is set).
export interface ParserBackend {
  parse(input: string, startNonterminal: string): Promise<ParseTreeResult | { error: string }>;
}
