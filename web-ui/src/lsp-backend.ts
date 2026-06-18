// LSP analysis results for an iggy grammar, in the shape Monaco's providers
// consume. Positions are zero-based (line and character), matching the LSP wire
// format that the language server and the wasm wrapper both emit; the editor
// converts to Monaco's one-based positions in one place.

export interface LspPosition {
  line: number;
  character: number;
}

export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

// A span in a document. The design viewer holds a single grammar, so `uri` is a
// fixed placeholder and only the range is used.
export interface LspLocation {
  uri: string;
  range: LspRange;
}

export interface LspDiagnostic {
  range: LspRange;
  // LSP DiagnosticSeverity: 1 = error, 2 = warning, 3 = info, 4 = hint.
  severity: number;
  message: string;
}

export interface LspDocumentSymbol {
  name: string;
  // LSP SymbolKind, one-based (e.g. 5 = Class, 9 = Constructor, 10 = Enum).
  kind: number;
  range: LspRange;
  selectionRange: LspRange;
  // Absent rather than empty when a symbol has no nested symbols, as the LSP
  // format and the wasm backend both omit it.
  children?: LspDocumentSymbol[];
}

export interface LspFoldingRange {
  startLine: number;
  endLine: number;
}

// The grammar intelligence the DesignView editor drives, independent of how it
// runs. Terrarium's backend reaches iguana-lsp through Tauri commands; the web
// viewer's WasmLspBackend calls the same functions compiled to wasm. Each method
// takes the current source, so the backend holds no document state: the
// read-only viewer parses once and the editor reuses the result per source
// version.
export interface LspBackend {
  // The semantic-token type names, ordered so a token's type index references
  // one. Registered once with Monaco as the legend.
  semanticTokensLegend(): Promise<string[]>;
  // Semantic tokens as the flat five-integer delta encoding Monaco expects.
  semanticTokens(source: string): Promise<Uint32Array>;
  diagnostics(source: string): Promise<LspDiagnostic[]>;
  documentSymbols(source: string): Promise<LspDocumentSymbol[]>;
  definition(source: string, line: number, character: number): Promise<LspLocation | null>;
  references(
    source: string,
    line: number,
    character: number,
    includeDeclaration: boolean,
  ): Promise<LspLocation[]>;
  folding(source: string): Promise<LspFoldingRange[]>;
}
