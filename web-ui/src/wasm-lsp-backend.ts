import type {
  LspBackend,
  LspDiagnostic,
  LspDocumentSymbol,
  LspFoldingRange,
  LspLocation,
} from "./lsp-backend";

// The iguana-lsp-wasm module's exported functions. Each runs the analysis and
// returns its result as an LSP-shaped JSON string. The viewer initializes the
// wasm module and passes its bound functions here, so this backend stays
// grammar-agnostic.
export interface LspWasm {
  semantic_tokens_legend(): string;
  semantic_tokens(source: string): string;
  diagnostics(source: string): string;
  document_symbols(source: string): string;
  folding(source: string): string;
  definition(source: string, line: number, character: number): string;
  references(
    source: string,
    line: number,
    character: number,
    includeDeclaration: boolean,
  ): string;
}

// Drives iguana-lsp compiled to wasm, running in the same page. The wasm module
// already emits LSP-shaped JSON, so each method parses its string and returns
// it. The analysis is synchronous, but the LspBackend contract is async to match
// Terrarium's Tauri-backed implementation.
export class WasmLspBackend implements LspBackend {
  constructor(private wasm: LspWasm) {}

  async semanticTokensLegend(): Promise<string[]> {
    return JSON.parse(this.wasm.semantic_tokens_legend()).tokenTypes;
  }

  async semanticTokens(source: string): Promise<Uint32Array> {
    return new Uint32Array(JSON.parse(this.wasm.semantic_tokens(source)).data);
  }

  async diagnostics(source: string): Promise<LspDiagnostic[]> {
    return JSON.parse(this.wasm.diagnostics(source));
  }

  async documentSymbols(source: string): Promise<LspDocumentSymbol[]> {
    return JSON.parse(this.wasm.document_symbols(source));
  }

  async folding(source: string): Promise<LspFoldingRange[]> {
    return JSON.parse(this.wasm.folding(source));
  }

  async definition(
    source: string,
    line: number,
    character: number,
  ): Promise<LspLocation | null> {
    return JSON.parse(this.wasm.definition(source, line, character));
  }

  async references(
    source: string,
    line: number,
    character: number,
    includeDeclaration: boolean,
  ): Promise<LspLocation[]> {
    return JSON.parse(this.wasm.references(source, line, character, includeDeclaration));
  }
}
