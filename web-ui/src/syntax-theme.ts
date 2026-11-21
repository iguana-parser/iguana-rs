// The colors of the highlight groups, in syntax-theme.json: a dark and a
// light color per group, and italic for some. A group without a color in the
// theme takes its parent's color, the group without its last segment, or
// keeps the text color (`themeGroup`).
import theme from "./syntax-theme.json";

interface GroupStyle {
  dark: string;
  light: string;
  italic?: boolean;
}

export const THEME: Record<string, GroupStyle> = theme;

// The theme's group that colors `group`: the group itself, or the nearest
// group it falls back to, or null.
export function themeGroup(group: string): string | null {
  for (let g = group; ; g = g.slice(0, g.lastIndexOf("."))) {
    if (g in THEME) return g;
    if (!g.includes(".")) return null;
  }
}

// The group of an LSP semantic token type, the inverse of the language
// server's naming: "ruleLexical" is the group "rule.lexical".
export function groupOfTokenType(type: string): string {
  return type.replace(/[A-Z]/g, (c) => "." + c.toLowerCase());
}

// The LSP semantic token type of a group: "rule.lexical" is "ruleLexical".
export function tokenTypeOfGroup(group: string): string {
  return group.replace(/\.([a-z])/g, (_, c: string) => c.toUpperCase());
}

// The CSS class of a theme group: "rule.lexical" is "hl-rule-lexical".
export function groupClass(group: string): string {
  return "hl-" + group.replaceAll(".", "-");
}

// A CSS rule for each theme group's class. The color is light-dark(), which
// takes the light or the dark color from the element's color-scheme.
export function themeStyles(): Record<string, { color: string; fontStyle?: string }> {
  return Object.fromEntries(
    Object.entries(THEME).map(([group, style]) => [
      `.${groupClass(group)}`,
      {
        color: `light-dark(${style.light}, ${style.dark})`,
        ...(style.italic ? { fontStyle: "italic" } : {}),
      },
    ]),
  );
}
