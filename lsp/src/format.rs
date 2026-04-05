// Iggy Grammar Formatter
//
// Formatting rules:
// - Two-space indentation for `=`, `|`, and `>`
// - `=` for the first priority level, `>` for subsequent ones
// - `|` between alternatives within the same priority level
// - Labels (`#Name`) are left-aligned to a column determined by the longest
//   alternative in the rule (per nonterminal head, not global)
// - Maximum line width: 100 characters
// - Lines exceeding the limit wrap to the next line with 6-space continuation
//   indent (2 for rule + 4 extra); the label sits at the alignment column on
//   the last line
// - Blank line between every rule
// - `@regex` annotation on the line before the regex rule head
// - `@NoLayout` / `@Layout(X)` annotation on the line before the syntax rule head
// - Regex rules are single-line (head = body postconditions)
// - Character classes have no internal spaces
// - Comments are emitted from Layout nodes during the tree walk
// - Final newline at end of file

use crate::ParseResult;
use iggy::parse_tree::*;
use iguana_runtime::input::Input;
use iguana_runtime::sppf::Span;

const MAX_LINE_WIDTH: usize = 100;
const RULE_PREFIX: &str = "  = ";
const ALT_PREFIX: &str = "  | ";
const PRIO_PREFIX: &str = "  > ";
const CONT_INDENT: &str = "      "; // 6 spaces for continuation lines

/// Format an iggy grammar from its parse result.
/// Returns `None` if the parse result has no tree (parse failure).
pub fn format(result: &ParseResult) -> Option<String> {
    let tree = result.tree.as_ref()?;
    let ParseTree::StartGrammar(start_grammar) = tree else {
        return None;
    };
    let f = Formatter::new(&result.input);
    Some(f.format_grammar(&start_grammar.start))
}

struct Formatter<'a> {
    input: &'a Input,
}

/// An alternative formatted into parts, before label alignment.
struct FormattedAlt {
    lines: Vec<String>,
    label: Option<String>,
}

impl<'a> Formatter<'a> {
    fn new(input: &'a Input) -> Self {
        Self { input }
    }

    fn text(&self, span: Span) -> String {
        self.input.substring(span.left_extent, span.right_extent)
    }

    fn is_same_line(&self, pos_a: u32, pos_b: u32) -> bool {
        let (line_a, _) = self.input.line_column(pos_a);
        let (line_b, _) = self.input.line_column(pos_b);
        line_a == line_b
    }

    /// Emit comments from a layout node. Trailing comments (same line as
    /// the previous token) get a space prefix. Standalone comments get
    /// their own line.
    fn emit_comments(&self, out: &mut String, layout: &Layout, previous: Span) {
        for comment in layout.line_comments() {
            if self.is_same_line(comment.span().left_extent, previous.right_extent) {
                out.push(' ');
                out.push_str(&self.text(comment.span()));
            } else {
                out.push('\n');
                out.push_str(&self.text(comment.span()));
            }
        }
    }

    fn format_grammar(&self, grammar: &Grammar) -> String {
        let mut out = String::new();

        // grammar Name
        out.push_str("grammar ");
        out.push_str(&self.text(grammar.name.span()));
        self.emit_comments(&mut out, &grammar.layout_3, grammar.name.span());

        // layout Def
        if let Some(layout_def) = grammar.layout_def.value() {
            out.push_str("\n\n");
            out.push_str("layout ");
            out.push_str(&self.text(layout_def.identifier.span()));
            self.emit_comments(&mut out, &grammar.layout_5, layout_def.identifier.span());
        }

        // Rules
        for rule in grammar.rules.rules() {
            out.push_str("\n\n");
            self.format_rule(&mut out, &rule);
        }

        out.push('\n');
        out
    }

    fn format_rule(&self, out: &mut String, rule: &Rule) {
        match rule {
            Rule::SyntaxRule { syntax_rule, .. } => self.format_syntax_rule(out, syntax_rule),
            Rule::RegexRule { regex_rule, .. } => self.format_regex_rule(out, regex_rule),
        }
    }

    fn format_syntax_rule(&self, out: &mut String, rule: &SyntaxRule) {
        if let Some(annotation) = rule.annotation.value() {
            self.format_annotation(out, annotation);
            out.push('\n');
        }

        out.push_str(&self.text(rule.head.span()));

        // Pass 1: format alternatives, keep references to originals
        let priority_levels: Vec<_> = rule.priority_levels.priority_levels().collect();
        let mut formatted_alts: Vec<(FormattedAlt, &Alternative)> = Vec::new();

        for (pi, pl) in priority_levels.iter().enumerate() {
            let prefix_first = if pi == 0 { RULE_PREFIX } else { PRIO_PREFIX };
            let alternatives: Vec<_> = pl.alternatives.alternatives().collect();

            let assoc_str = pl.associativity.value().map(|a| match a {
                Associativity::Alt0 { .. } => "left ",
                Associativity::Alt1 { .. } => "right ",
                Associativity::Alt2 { .. } => "none ",
            });

            for (ai, alt) in alternatives.iter().enumerate() {
                let prefix = if ai == 0 { prefix_first } else { ALT_PREFIX };
                let mut symbols_str = String::new();
                if ai == 0 && assoc_str.is_some() {
                    symbols_str.push_str(assoc_str.unwrap());
                }
                let syms: Vec<_> = alt.symbols.symbols().collect();
                for (si, sym) in syms.iter().enumerate() {
                    if si > 0 {
                        symbols_str.push(' ');
                    }
                    self.format_symbol(&mut symbols_str, sym);
                }
                let label = alt.label.value().map(|t| self.text(t.span()));
                let lines = wrap_line(prefix, &symbols_str, CONT_INDENT, MAX_LINE_WIDTH);
                formatted_alts.push((FormattedAlt { lines, label }, alt));
            }
        }

        // Compute label alignment column
        let has_any_label = formatted_alts.iter().any(|(fa, _)| fa.label.is_some());
        let label_column = if has_any_label {
            formatted_alts
                .iter()
                .map(|(fa, _)| fa.lines.last().unwrap().len())
                .max()
                .unwrap_or(0)
                + 1
        } else {
            0
        };

        // Pass 2: emit with alignment, visit layout nodes for comments
        let mut prev_span = rule.head.span();
        for (fa, alt) in &formatted_alts {
            out.push('\n');
            for (i, line) in fa.lines.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                out.push_str(line);
            }
            if let Some(ref label) = fa.label {
                let last_len = fa.lines.last().unwrap().len();
                let padding = if label_column > last_len {
                    label_column - last_len
                } else {
                    1
                };
                for _ in 0..padding {
                    out.push(' ');
                }
                out.push_str(label);
                prev_span = alt.label.value().unwrap().span();
            } else {
                let syms: Vec<_> = alt.symbols.symbols().collect();
                if let Some(last_sym) = syms.last() {
                    prev_span = last_sym.span();
                }
            }
            self.emit_comments(out, &alt.layout, prev_span);
        }
    }

    fn format_annotation(&self, out: &mut String, annotation: &Annotation) {
        match annotation {
            Annotation::NoLayout { .. } => out.push_str("@NoLayout"),
            Annotation::Layout { identifier, .. } => {
                out.push_str("@Layout(");
                out.push_str(&self.text(identifier.span()));
                out.push(')');
            }
        }
    }

    fn format_symbol(&self, out: &mut String, symbol: &Symbol) {
        match symbol {
            Symbol::Identifier { identifier, .. } => {
                out.push_str(&self.text(identifier.span()));
            }
            Symbol::Lit { string, .. } => {
                out.push_str(&self.text(string.span()));
            }
            Symbol::Group { symbols, .. } => {
                out.push('(');
                let syms: Vec<_> = symbols.symbols().collect();
                for (i, s) in syms.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    self.format_symbol(out, s);
                }
                out.push(')');
            }
            Symbol::Alt { first, rest, .. } => {
                out.push('(');
                self.format_symbol(out, first);
                for s in rest.symbols() {
                    out.push_str(" | ");
                    self.format_symbol(out, s);
                }
                out.push(')');
            }
            Symbol::Star { symbol, .. } => {
                self.format_symbol(out, symbol);
                out.push('*');
            }
            Symbol::Plus { symbol, .. } => {
                self.format_symbol(out, symbol);
                out.push('+');
            }
            Symbol::Opt { symbol, .. } => {
                self.format_symbol(out, symbol);
                out.push('?');
            }
            Symbol::StarSep { symbol, sep, .. } => {
                out.push_str("{ ");
                self.format_symbol(out, symbol);
                out.push(' ');
                self.format_symbol(out, sep);
                out.push_str(" }*");
            }
            Symbol::PlusSep { symbol, sep, .. } => {
                out.push_str("{ ");
                self.format_symbol(out, symbol);
                out.push(' ');
                self.format_symbol(out, sep);
                out.push_str(" }+");
            }
            Symbol::Except { symbol, excepts, .. } => {
                self.format_symbol(out, symbol);
                for id in excepts.identifiers() {
                    out.push_str(" \\ ");
                    out.push_str(&self.text(id.span()));
                }
            }
            Symbol::FollowRestriction { symbol, restrictions, .. } => {
                self.format_symbol(out, symbol);
                for id in restrictions.identifiers() {
                    out.push_str(" !>> ");
                    out.push_str(&self.text(id.span()));
                }
            }
            Symbol::PrecedeRestriction { identifier, symbol, .. } => {
                out.push_str(&self.text(identifier.span()));
                out.push_str(" !<< ");
                self.format_symbol(out, symbol);
            }
            Symbol::Exclude { symbol, labels, .. } => {
                self.format_symbol(out, symbol);
                for id in labels.identifiers() {
                    out.push('!');
                    out.push_str(&self.text(id.span()));
                }
            }
            Symbol::Labeled { label, symbol, .. } => {
                out.push_str(&self.text(label.span()));
                out.push(':');
                self.format_symbol(out, symbol);
            }
        }
    }

    fn format_regex_rule(&self, out: &mut String, rule: &RegexRule) {
        out.push_str("@regex\n");
        out.push_str(&self.text(rule.identifier.span()));

        if let Some(pre) = rule.pre_condition.value() {
            out.push_str(" = ");
            out.push_str(&self.text(pre.identifier.span()));
            out.push_str(" !<< ");
        } else {
            out.push_str(" = ");
        }

        let mut first_alt = true;
        for regex_group in rule.body.regexes() {
            if !first_alt {
                out.push_str(" | ");
            }
            first_alt = false;
            let mut first_regex = true;
            for regex in regex_group {
                if !first_regex {
                    out.push(' ');
                }
                first_regex = false;
                self.format_regex(out, regex);
            }
        }

        for pc in rule.post_conditions.post_conditions() {
            match pc {
                PostCondition::Except { identifier, .. } => {
                    out.push_str(" \\ ");
                    out.push_str(&self.text(identifier.span()));
                }
                PostCondition::FollowRestriction { identifier, .. } => {
                    out.push_str(" !>> ");
                    out.push_str(&self.text(identifier.span()));
                }
            }
        }
    }

    fn format_regex(&self, out: &mut String, regex: &Regex) {
        match regex {
            Regex::Plus { regex, .. } => {
                self.format_regex(out, regex);
                out.push('+');
            }
            Regex::Star { regex, .. } => {
                self.format_regex(out, regex);
                out.push('*');
            }
            Regex::Opt { regex, .. } => {
                self.format_regex(out, regex);
                out.push('?');
            }
            Regex::Alt { first, rest, .. } => {
                out.push('(');
                self.format_regex(out, first);
                for r in rest.regexes() {
                    out.push_str(" | ");
                    self.format_regex(out, r);
                }
                out.push(')');
            }
            Regex::Group { regexes, .. } => {
                out.push('(');
                for r in regexes.regexes() {
                    self.format_regex(out, r);
                }
                out.push(')');
            }
            Regex::CharClass { char_class, .. } => {
                self.format_char_class(out, char_class);
            }
            Regex::Char { char, .. } => {
                out.push_str(&self.text(char.span()));
            }
            Regex::String { string, .. } => {
                out.push_str(&self.text(string.span()));
            }
            Regex::Identifier { identifier, .. } => {
                out.push_str(&self.text(identifier.span()));
            }
        }
    }

    fn format_char_class(&self, out: &mut String, cc: &CharClass) {
        if cc.neg.value().is_some() {
            out.push('!');
        }
        out.push('[');
        for re in cc.range_elements.range_elements() {
            match re {
                RangeElement::Alt0 { range, .. } => {
                    out.push_str(&self.text(range.start.span()));
                    out.push('-');
                    out.push_str(&self.text(range.end.span()));
                }
                RangeElement::Alt1 { range_char, .. } => {
                    out.push_str(&self.text(range_char.span()));
                }
            }
        }
        out.push(']');
    }
}

/// Wrap a line of symbols if it exceeds `max_width`.
fn wrap_line(
    prefix: &str,
    symbols: &str,
    cont_indent: &str,
    max_width: usize,
) -> Vec<String> {
    let first_line = format!("{}{}", prefix, symbols);
    if first_line.len() <= max_width {
        return vec![first_line];
    }

    let words: Vec<&str> = symbols.split(' ').collect();
    let mut lines = Vec::new();
    let mut current = String::from(prefix);

    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = format!("{}{}", cont_indent, word);
        }
    }
    if !current.is_empty() && current != cont_indent {
        lines.push(current);
    }

    if lines.is_empty() {
        vec![first_line]
    } else {
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_source(source: &str) -> Option<String> {
        let result = crate::parse(source);
        format(&result)
    }

    #[test]
    fn test_simple_grammar() {
        let input = "grammar  Test\n\nlayout   WS\n\nRule\n  = \"hello\"\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(
            formatted,
            "grammar Test\n\nlayout WS\n\nRule\n  = \"hello\"\n"
        );
    }

    #[test]
    fn test_multiple_alternatives() {
        let input = "grammar T\n\nA\n  = \"x\"  #X\n  | \"y\"  #Y\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(
            formatted,
            "grammar T\n\nA\n  = \"x\" #X\n  | \"y\" #Y\n"
        );
    }

    #[test]
    fn test_priority_levels() {
        let input = "grammar T\n\nA\n  = \"x\"\n  > \"y\"\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar T\n\nA\n  = \"x\"\n  > \"y\"\n");
    }

    #[test]
    fn test_regex_rule() {
        let input = "grammar T\n\n@regex\nId = [a-zA-Z]+\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar T\n\n@regex\nId = [a-zA-Z]+\n");
    }

    #[test]
    fn test_label_alignment() {
        let input = "grammar T\n\nS\n  = \"a\" #Short\n  | \"longer\" \"alt\" #Long\n";
        let formatted = format_source(input).unwrap();
        let lines: Vec<_> = formatted.lines().collect();
        let short_line = lines.iter().find(|l| l.contains("#Short")).unwrap();
        let long_line = lines.iter().find(|l| l.contains("#Long")).unwrap();
        let short_col = short_line.find('#').unwrap();
        let long_col = long_line.find('#').unwrap();
        assert_eq!(short_col, long_col);
    }

    #[test]
    fn test_trailing_comment() {
        let input = "grammar T // a grammar\n\nRule\n  = \"a\"\n";
        let formatted = format_source(input).unwrap();
        assert!(formatted.starts_with("grammar T // a grammar\n"));
    }

    #[test]
    fn test_standalone_comment_between_rules() {
        let input = "grammar T\n\nA\n  = \"a\"\n\n// a comment\n\nB\n  = \"b\"\n";
        let formatted = format_source(input).unwrap();
        assert!(formatted.contains("// a comment"));
        let second = format_source(&formatted).unwrap();
        assert_eq!(formatted, second, "Comment formatting should be idempotent");
    }

    #[test]
    fn test_parse_failure_returns_none() {
        let result = crate::parse("not a valid {{{ grammar");
        assert!(format(&result).is_none());
    }

    #[test]
    fn test_wrap_line_short() {
        let lines = wrap_line("  = ", "A B C", "      ", 100);
        assert_eq!(lines, vec!["  = A B C"]);
    }

    #[test]
    fn test_wrap_line_long() {
        let lines = wrap_line("  = ", "A B C D E F", "      ", 12);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("  = "));
        assert!(lines[1].starts_with("      "));
    }

    #[test]
    fn test_idempotent() {
        let input = "grammar Iggy\n\nlayout Layout\n\nRule\n  = SyntaxRule #SyntaxRule\n  | RegexRule  #RegexRule\n";
        let first = format_source(input).unwrap();
        let second = format_source(&first).unwrap();
        assert_eq!(first, second, "Formatting should be idempotent");
    }
}
