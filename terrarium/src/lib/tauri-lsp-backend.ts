import { commands, type AnalyzeResult, type DocumentSymbolData, type RangeData } from "../bindings";
import type {
  LspBackend,
  LspDiagnostic,
  LspDocumentSymbol,
  LspFoldingRange,
  LspLocation,
  LspRange,
} from "@iguana-parser/web-ui";

// DesignView uses the editor's own model URI for locations, so this value is
// never read; it only fills the LspLocation shape.
const PLACEHOLDER_URI = "file:///grammar.iggy";

// Maps Terrarium's range shape (zero-based line and character) to the LSP range
// the LspBackend contract uses.
function toLspRange(r: RangeData): LspRange {
  return {
    start: { line: r.start_line, character: r.start_char },
    end: { line: r.end_line, character: r.end_char },
  };
}

// Drives iguana-lsp through Terrarium's Tauri commands, so the DesignView editor
// reaches grammar intelligence without a Tauri dependency of its own.
export class TauriLspBackend implements LspBackend {
  // `onAnalyze` receives the parse timing on each semantic-token request, which
  // is the parse trigger, so the design-mode status bar can report it.
  constructor(private onAnalyze?: (result: AnalyzeResult) => void) {}

  async semanticTokensLegend(): Promise<string[]> {
    return (await commands.getSemanticTokensLegend()).token_types;
  }

  async semanticTokens(source: string): Promise<Uint32Array> {
    // analyze_grammar parses and caches; get_semantic_tokens reads that cache,
    // so the parse runs first. It also yields the timing for onAnalyze.
    this.onAnalyze?.(await commands.analyzeGrammar(source));
    const tokens = await commands.getSemanticTokens();
    const data = new Uint32Array(tokens.length * 5);
    for (let i = 0; i < tokens.length; i++) {
      const t = tokens[i];
      data[i * 5] = t.delta_line;
      data[i * 5 + 1] = t.delta_start;
      data[i * 5 + 2] = t.length;
      data[i * 5 + 3] = t.token_type;
      data[i * 5 + 4] = t.token_modifiers_bitset;
    }
    return data;
  }

  async diagnostics(source: string): Promise<LspDiagnostic[]> {
    const diags = await commands.getDiagnostics(source);
    return diags.map((d) => ({
      range: toLspRange(d.range),
      severity: d.severity,
      message: d.message,
    }));
  }

  async documentSymbols(source: string): Promise<LspDocumentSymbol[]> {
    const convert = (s: DocumentSymbolData): LspDocumentSymbol => ({
      name: s.name,
      kind: s.kind,
      range: toLspRange(s.range),
      selectionRange: toLspRange(s.selection_range),
      children: s.children.map(convert),
    });
    return (await commands.getDocumentSymbols(source)).map(convert);
  }

  async folding(source: string): Promise<LspFoldingRange[]> {
    const ranges = await commands.getFoldingRanges(source);
    return ranges.map((r) => ({ startLine: r.start_line, endLine: r.end_line }));
  }

  async definition(source: string, line: number, character: number): Promise<LspLocation | null> {
    const loc = await commands.getDefinition(source, line, character);
    if (!loc) return null;
    return { uri: PLACEHOLDER_URI, range: toLspRange(loc.range) };
  }

  async references(
    source: string,
    line: number,
    character: number,
    includeDeclaration: boolean,
  ): Promise<LspLocation[]> {
    const locs = await commands.getReferences(source, line, character, includeDeclaration);
    return locs.map((loc) => ({ uri: PLACEHOLDER_URI, range: toLspRange(loc.range) }));
  }
}
