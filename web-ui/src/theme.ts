import type * as monaco from "monaco-editor";

// Cytoscape's typings do not export the stylesheet type (see graph-styles.ts).
type Stylesheet = any;

// The host's theme: CSS custom properties a host sets on :root (a viewer's
// theme.css, or the web viewer's own palette). Each use falls back to the
// component's own value, Terrarium's VS Code dark palette, so a host that sets
// nothing looks as before.
//
//   --iguana-font-ui, --iguana-font-mono    interface and code fonts
//   --iguana-bg                             editor and result pane
//   --iguana-bg-raised                      bars, menus, tooltips, controls
//   --iguana-bg-hover                       a hovered row
//   --iguana-border                         borders and dividers, a hovered button
//   --iguana-border-strong                  control outlines, graph edges
//   --iguana-text, --iguana-text-muted,
//   --iguana-text-strong                    text: body, secondary, emphasized
//   --iguana-accent                         active tab, focus, the input caret mark
//   --iguana-selection, --iguana-flash      a selected node or span, a revealed row
//   --iguana-error                          a parse error
//   --iguana-warning                        an ambiguity in the input editors
//   --iguana-syntax-nonterminal, -token, -keyword, -string, -number, -operator,
//   -reference, -reference-hover, -paren, -comment, -ambiguity
//                                           syntax colors in the editor and views
//
// CSS reads them with var(--iguana-..., <fallback>). Monaco and Cytoscape take
// colors as options instead, so they read them through themeValue and toHex and
// redraw on onThemeChange.
export function themeValue(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

// The code font when the host sets none: Fira Code where it is installed, else
// the platform's monospace font (SF Mono on macOS through ui-monospace or its
// name, Consolas on Windows). Every component uses this list, so the input and
// the tree next to it always share a font. CSS cannot read this constant, so
// the components' stylesheets spell out the same list.
export const CODE_FONT =
  '"Fira Code", ui-monospace, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace';

export function codeFont(): string {
  return themeValue("--iguana-font-mono", CODE_FONT);
}

// Calls `callback` whenever the theme may have changed: the root's attributes
// (a host switching data-theme, or setting properties inline) or the system
// color scheme. Returns the cleanup.
export function onThemeChange(callback: () => void): () => void {
  const observer = new MutationObserver(callback);
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme", "class", "style"],
  });
  const scheme = window.matchMedia("(prefers-color-scheme: dark)");
  scheme.addEventListener("change", callback);
  return () => {
    observer.disconnect();
    scheme.removeEventListener("change", callback);
  };
}

export function isLight(): boolean {
  return getComputedStyle(document.documentElement).colorScheme.trim() === "light";
}

// A CSS color in hex: #rrggbb, or #rrggbbaa when it is translucent. Null when
// the browser cannot parse it. Monaco takes theme colors only in hex, and the
// graph blends colors channel by channel, so both read the host's colors
// through this. A canvas context parses the color: it serializes an opaque
// fillStyle as #rrggbb and a translucent one as rgba().
let colorContext: CanvasRenderingContext2D | null | undefined;
export function toHex(color: string): string | null {
  colorContext ??= document.createElement("canvas").getContext("2d");
  if (!colorContext || !color) return null;
  // An unparseable color leaves fillStyle unchanged, so it reads back as the
  // value set before it, which differs between the two passes.
  const parse = (previous: string) => {
    colorContext!.fillStyle = previous;
    colorContext!.fillStyle = color;
    return String(colorContext!.fillStyle);
  };
  const parsed = parse("#000000");
  if (parsed !== parse("#ffffff")) return null;
  if (parsed.startsWith("#")) return parsed;
  const rgba = parsed.match(/^rgba\((\d+), (\d+), (\d+), ([\d.]+)\)$/);
  if (!rgba) return null;
  const byte = (value: number) => Math.round(value).toString(16).padStart(2, "0");
  return "#" + [1, 2, 3].map((i) => byte(Number(rgba[i]))).join("") + byte(Number(rgba[4]) * 255);
}

// The host's value of a theme property in hex, or undefined when the host does
// not set it or sets something other than a color.
function themeHex(name: string): string | undefined {
  return toHex(themeValue(name, "")) ?? undefined;
}

// Monaco theme colors from the host's theme. A color the host does not set
// keeps the base theme's own.
function editorColors(colors: Record<string, string | undefined>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(colors).filter((entry): entry is [string, string] => entry[1] !== undefined),
  );
}

// Monaco's colors for the editor surface, the text, and the selection, shared by
// the input editor and the grammar editor.
function surfaceColors(): Record<string, string | undefined> {
  const bg = themeHex("--iguana-bg");
  const text = themeHex("--iguana-text");
  return {
    "editor.background": bg,
    "editorGutter.background": bg,
    "editor.foreground": text,
    "editorCursor.foreground": text,
    "editorLineNumber.foreground": themeHex("--iguana-text-muted"),
    "editorLineNumber.activeForeground": text,
    "editor.selectionBackground": themeHex("--iguana-selection"),
    "editorError.foreground": themeHex("--iguana-error"),
    "editorWarning.foreground": themeHex("--iguana-warning"),
  };
}

// vs-dark, or vs under a light color scheme, recolored from the host's theme, so
// the input editor sits on the same surface as the rest of the view. Monaco
// themes are global to the page, so redefining it restyles every editor using
// it.
export function defineInputTheme(editorApi: typeof monaco.editor): string {
  editorApi.defineTheme("iguana-input", {
    base: isLight() ? "vs" : "vs-dark",
    inherit: true,
    rules: [],
    colors: editorColors(surfaceColors()),
  });
  return "iguana-input";
}

// The grammar editor's theme: the iggy token colors (Dark+ by default) and the
// surface, each recolored where the host's theme sets the matching property.
export function defineGrammarTheme(editorApi: typeof monaco.editor): string {
  // Monaco takes token colors as hex without the leading #.
  const token = (name: string, fallback: string) =>
    (themeHex(name) ?? fallback).replace(/^#/, "").slice(0, 6);
  const selection = themeHex("--iguana-selection")?.slice(0, 7) ?? "#264f78";
  editorApi.defineTheme("iggy", {
    base: isLight() ? "vs" : "vs-dark",
    inherit: true,
    rules: [
      { token: "keyword", foreground: token("--iguana-syntax-keyword", "#569cd6"), fontStyle: "bold" },
      { token: "type", foreground: token("--iguana-syntax-nonterminal", "#4ec9b0") },
      { token: "string", foreground: token("--iguana-syntax-string", "#ce9178") },
      { token: "regexp", foreground: "d16969" },
      { token: "operator", foreground: token("--iguana-syntax-operator", "#d4d4d4") },
      { token: "decorator", foreground: "dcdcaa" },
      { token: "comment", foreground: token("--iguana-syntax-comment", "#808080") }, // labels
    ],
    colors: editorColors({
      ...surfaceColors(),
      "editor.inactiveSelectionBackground": selection,
      "editor.selectionHighlightBackground": `${selection}4d`,
      "editor.wordHighlightBackground": `${selection}4d`,
      "editor.wordHighlightStrongBackground": `${selection}4d`,
      "editor.rangeHighlightBackground": selection,
      "editor.rangeHighlightBorder": "#00000000",
    }),
  });
  return "iggy";
}

// Monaco measures glyph widths when it lays out. A web font that finishes
// loading afterwards leaves the caret and selection drawn against the fallback
// font's widths, so measure again whenever a font load completes. Returns the
// listener's cleanup.
export function remeasureOnFontLoad(editorApi: typeof monaco.editor): () => void {
  const remeasure = () => editorApi.remeasureFonts();
  document.fonts.addEventListener("loadingdone", remeasure);
  return () => document.fonts.removeEventListener("loadingdone", remeasure);
}

// `color` blended into `base` by `amount` (0 to 1). `base` is #rrggbb, and
// `color` is any CSS color; one the browser cannot parse is returned as it is.
function mix(color: string, base: string, amount: number): string {
  const hex = toHex(color)?.slice(0, 7);
  if (!hex) return color;
  color = hex;
  const channel = (hex: string, i: number) => parseInt(hex.slice(1 + 2 * i, 3 + 2 * i), 16);
  let out = "#";
  for (let i = 0; i < 3; i++) {
    const value = Math.round(channel(base, i) + (channel(color, i) - channel(base, i)) * amount);
    out += value.toString(16).padStart(2, "0");
  }
  return out;
}

// The parse-tree graph's stylesheet recolored from the host's theme. Cytoscape
// applies rules in order, so each color replaces the one in its own rule
// rather than being appended, which would override the selection and highlight
// rules that follow. Each property the host sets recolors the rules that use
// it, and a rule whose properties the host leaves unset keeps its default
// color. Node fills are the kind's syntax color blended into the background,
// so a fill changes when the host sets either. A translucent background has
// nothing opaque to blend into, so fills then blend into the default one.
export function themeGraphStyles(styles: Stylesheet[]): Stylesheet[] {
  // The host's value of a property in a form Cytoscape reads, or undefined when
  // the host sets none or sets something that is not a color. Cytoscape does
  // not parse every CSS color (space-separated rgb(), hsl() in some forms,
  // #rrggbbaa), so each one goes through toHex: opaque colors as #rrggbb,
  // translucent ones as rgba().
  const host = (name: string) => {
    const hex = toHex(themeValue(name, ""));
    if (!hex) return undefined;
    if (hex.length === 7) return hex;
    const [r, g, b, a] = [1, 3, 5, 7].map((i) => parseInt(hex.slice(i, i + 2), 16));
    return `rgba(${r}, ${g}, ${b}, ${Math.round((a / 255) * 1000) / 1000})`;
  };
  const hostBg = themeHex("--iguana-bg");
  const bg = hostBg?.length === 7 ? hostBg : "#1e1e1e";
  const overlay: Record<string, object> = {};
  const add = (selector: string, style: Record<string, string>) => {
    overlay[selector] = { ...overlay[selector], ...style };
  };
  const line = (color: string) => ({ "line-color": color, "target-arrow-color": color });

  const text = host("--iguana-text");
  if (text) add("node", { color: text });
  const raised = host("--iguana-bg-raised");
  if (raised) add("node", { "background-color": raised });

  // The node kinds, each with its syntax property and the default color its
  // fills blend from.
  const kinds = [
    {
      name: "--iguana-syntax-nonterminal",
      fallback: "#4ec9b0",
      rule: "node.nonterminal, node[kind='Nonterminal']",
      selected: "node.nonterminal.selected, node[kind='Nonterminal'].selected",
      edge: "edge.edge-selected-nonterminal",
    },
    {
      name: "--iguana-syntax-token",
      fallback: "#ce9178",
      rule: "node.token, node[kind='Token']",
      selected: "node.token.selected, node[kind='Token'].selected",
      edge: "edge.edge-selected-terminal",
    },
    {
      name: "--iguana-syntax-ambiguity",
      fallback: "#e05050",
      rule: "node.ambiguous, node.amb",
      selected: "node.ambiguous.selected, node.amb.selected",
      edge: "edge.edge-selected-ambiguous",
    },
  ];
  for (const kind of kinds) {
    const color = host(kind.name);
    if (!color && !hostBg) continue;
    const stroke = color ?? kind.fallback;
    add(kind.rule, { "background-color": mix(stroke, bg, 0.18), "border-color": stroke });
    add(kind.selected, { "background-color": mix(stroke, bg, 0.35), "border-color": stroke });
    if (color) add(kind.edge, line(color));
  }
  const ambiguity = host("--iguana-syntax-ambiguity");
  if (ambiguity) add("edge.edge-ambiguous", line(ambiguity));

  const edge = host("--iguana-border-strong");
  if (edge) add("edge", line(edge));
  const reference = host("--iguana-syntax-reference");
  if (reference) add("edge.shared", line(reference));
  const muted = host("--iguana-text-muted");
  if (muted) add("edge.edge-clicked", line(muted));

  return styles.map((rule) =>
    overlay[rule.selector] ? { ...rule, style: { ...rule.style, ...overlay[rule.selector] } } : rule,
  );
}
