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
// - `@Regex` (optionally preceded by `@Layout` for a layout rule) on the line before the regex rule head
// - `@Start` / `@Layout` / `@NoLayout` / `@WithLayout(X)` annotation on the line before the syntax rule head
// - Regex rules with a single alternative are single-line
// - Regex rules with multiple alternatives use multi-line layout (one per line) (head = body postconditions)
// - Character classes have no internal spaces
// - Comments are emitted from Layout nodes during the tree walk
// - Final newline at end of file

use crate::layout::is_same_line;
use iggy::parse_tree::*;
use iguana_runtime::input::{Input, Span};

const MAX_LINE_WIDTH: usize = 100;
const RULE_PREFIX: &str = "  = ";
const ALT_PREFIX: &str = "  | ";
const PRIO_PREFIX: &str = "  > ";
const CONT_INDENT: &str = "      "; // 6 spaces for continuation lines

/// Format an iggy grammar from its parse tree.
pub fn format(tree: &Start<&Grammar<'_>, &Layout<'_>>, input: &Input) -> String {
    let f = Formatter::new(input);
    f.format_grammar(&tree.node)
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

    /// Emit comments from a layout node. Trailing comments (same line as
    /// the previous token) get a space prefix. Standalone comments get
    /// their own line.
    fn emit_comments(&self, out: &mut String, layout: &Layout, previous: Span) {
        for comment in layout.line_comments() {
            if is_same_line(
                self.input,
                comment.span().left_extent,
                previous.right_extent,
            ) {
                out.push(' ');
                out.push_str(&self.input.text(comment.span()));
            } else {
                out.push('\n');
                out.push_str(&self.input.text(comment.span()));
            }
        }
    }

    fn format_grammar(&self, grammar: &Grammar) -> String {
        let mut out = String::new();

        // grammar Name
        out.push_str("grammar ");
        out.push_str(&self.input.text(grammar.name().span()));
        self.emit_comments(&mut out, grammar.layout_3(), grammar.name().span());

        // Rules
        for rule in grammar.rules().rules() {
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
            Rule::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
        }
    }

    fn symbol_to_string(&self, symbol: &Symbol) -> String {
        let mut s = String::new();
        self.format_symbol(&mut s, symbol);
        s
    }

    fn regex_to_string(&self, regex: &Regex) -> String {
        let mut s = String::new();
        self.format_regex(&mut s, regex);
        s
    }

    fn format_syntax_rule(&self, out: &mut String, rule: &SyntaxRule) {
        if let Some(annotation) = rule.annotation().value() {
            self.format_annotation(out, annotation);
            out.push('\n');
        }

        out.push_str(&self.input.text(rule.head().span()));

        // Pass 1: format alternatives, keep references to originals
        let priority_levels: Vec<_> = rule.priority_levels().priority_levels().collect();
        let mut formatted_alts: Vec<(FormattedAlt, &Alternative)> = Vec::new();

        for (pi, pl) in priority_levels.iter().enumerate() {
            let prefix_first = if pi == 0 { RULE_PREFIX } else { PRIO_PREFIX };
            let alternatives: Vec<_> = pl.alternatives().alternatives().collect();

            let assoc_str = pl.associativity().value().map(|a| match a {
                Associativity::Alt0 { .. } => "left",
                Associativity::Alt1 { .. } => "right",
                Associativity::Alt2 { .. } => "none",
                Associativity::Amb(_) => {
                    unreachable!("ambiguous trees are rejected before this point")
                }
            });

            for (ai, alt) in alternatives.iter().enumerate() {
                let prefix = if ai == 0 { prefix_first } else { ALT_PREFIX };
                let mut chunks: Vec<String> = Vec::new();
                if ai == 0 {
                    if let Some(assoc) = assoc_str {
                        chunks.push(assoc.to_string());
                    }
                }
                let label = match alt {
                    Alternative::Symbols { symbols, label, .. } => {
                        for sym in symbols.symbols() {
                            chunks.push(self.symbol_to_string(&sym));
                        }
                        label.value()
                    }
                    Alternative::Empty { label, .. } => {
                        chunks.push("()".to_string());
                        label.value()
                    }
                    Alternative::Amb(_) => {
                        unreachable!("ambiguous trees are rejected before this point")
                    }
                }
                .map(|t| self.input.text(t.span()));
                let lines = wrap_chunks(prefix, &chunks, CONT_INDENT, MAX_LINE_WIDTH);
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
        let mut prev_span = rule.head().span();
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
                prev_span = match alt {
                    Alternative::Symbols { label, .. } | Alternative::Empty { label, .. } => {
                        label.value().unwrap().span()
                    }
                    Alternative::Amb(_) => {
                        unreachable!("ambiguous trees are rejected before this point")
                    }
                };
            } else {
                match alt {
                    Alternative::Symbols { symbols, .. } => {
                        if let Some(last_sym) = symbols.symbols().last() {
                            prev_span = last_sym.span();
                        }
                    }
                    Alternative::Empty { lit_2, .. } => prev_span = lit_2.span(),
                    Alternative::Amb(_) => {
                        unreachable!("ambiguous trees are rejected before this point")
                    }
                }
            }
            let layout = match alt {
                Alternative::Symbols { layout, .. } => *layout,
                Alternative::Empty { layout_3, .. } => *layout_3,
                Alternative::Amb(_) => {
                    unreachable!("ambiguous trees are rejected before this point")
                }
            };
            self.emit_comments(out, layout, prev_span);
        }
    }

    fn format_annotation(&self, out: &mut String, annotation: &Annotation) {
        match annotation {
            Annotation::NoLayout { .. } => out.push_str("@NoLayout"),
            Annotation::Layout { .. } => out.push_str("@Layout"),
            Annotation::WithLayout { identifier, .. } => {
                out.push_str("@WithLayout(");
                out.push_str(&self.input.text(identifier.span()));
                out.push(')');
            }
            Annotation::Start { .. } => out.push_str("@Start"),
            Annotation::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
        }
    }

    fn format_symbol(&self, out: &mut String, symbol: &Symbol) {
        match symbol {
            Symbol::Identifier { identifier, .. } => {
                out.push_str(&self.input.text(identifier.span()));
            }
            Symbol::Lit { string, .. } => {
                out.push_str(&self.input.text(string.span()));
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
            Symbol::Except {
                symbol, excepts, ..
            } => {
                self.format_symbol(out, symbol);
                for id in excepts.identifiers() {
                    out.push_str(" \\ ");
                    out.push_str(&self.input.text(id.span()));
                }
            }
            Symbol::FollowRestriction {
                symbol,
                restrictions,
                ..
            } => {
                self.format_symbol(out, symbol);
                for id in restrictions.identifiers() {
                    out.push_str(" !>> ");
                    out.push_str(&self.input.text(id.span()));
                }
            }
            Symbol::PrecedeRestriction {
                identifier, symbol, ..
            } => {
                out.push_str(&self.input.text(identifier.span()));
                out.push_str(" !<< ");
                self.format_symbol(out, symbol);
            }
            Symbol::Exclude { symbol, labels, .. } => {
                self.format_symbol(out, symbol);
                for id in labels.identifiers() {
                    out.push('!');
                    out.push_str(&self.input.text(id.span()));
                }
            }
            Symbol::Labeled { label, symbol, .. } => {
                out.push_str(&self.input.text(label.span()));
                out.push(':');
                self.format_symbol(out, symbol);
            }
            Symbol::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
        }
    }

    fn postconditions_to_string(&self, rule: &RegexRule) -> String {
        let mut s = String::new();
        for pc in rule.post_conditions().post_conditions() {
            match pc {
                PostCondition::Except { identifier, .. } => {
                    s.push_str(" \\ ");
                    s.push_str(&self.input.text(identifier.span()));
                }
                PostCondition::FollowRestriction { identifier, .. } => {
                    s.push_str(" !>> ");
                    s.push_str(&self.input.text(identifier.span()));
                }
                PostCondition::Amb(_) => {
                    unreachable!("ambiguous trees are rejected before this point")
                }
            }
        }
        s
    }

    fn format_regex_rule(&self, out: &mut String, rule: &RegexRule) {
        if rule.layout().value().is_some() {
            out.push_str("@Layout @Regex\n");
        } else {
            out.push_str("@Regex\n");
        }

        let groups: Vec<Vec<_>> = rule
            .body()
            .regexes()
            .map(|group| group.collect::<Vec<_>>())
            .collect();

        let pre_prefix = rule
            .pre_condition()
            .value()
            .map(|pre| format!("{} !<< ", self.input.text(pre.identifier().span())));
        let pre_str = pre_prefix.as_deref().unwrap_or("");
        let postcond = self.postconditions_to_string(rule);
        let name = self.input.text(rule.identifier().span());

        if groups.len() > 1 {
            // Multi-alt: name on its own line, each alt on its own line(s)
            out.push_str(&name);
            for (i, group) in groups.iter().enumerate() {
                let chunks: Vec<String> = group.iter().map(|r| self.regex_to_string(r)).collect();
                let prefix = if i == 0 {
                    format!("{}{}", RULE_PREFIX, pre_str)
                } else {
                    ALT_PREFIX.to_string()
                };
                let lines = wrap_chunks(&prefix, &chunks, CONT_INDENT, MAX_LINE_WIDTH);
                for (j, line) in lines.iter().enumerate() {
                    out.push('\n');
                    out.push_str(line);
                    if i == groups.len() - 1 && j == lines.len() - 1 {
                        out.push_str(&postcond);
                    }
                }
            }
        } else {
            // Single-alt: try single line first
            let chunks: Vec<String> = groups[0].iter().map(|r| self.regex_to_string(r)).collect();
            let body = chunks.join(" ");
            let single = format!("{} = {}{}{}", name, pre_str, body, postcond);

            if single.len() <= MAX_LINE_WIDTH {
                out.push_str(&single);
            } else {
                // Too long: name on its own line, wrapped body
                out.push_str(&name);
                let prefix = format!("{}{}", RULE_PREFIX, pre_str);
                let mut lines = wrap_chunks(&prefix, &chunks, CONT_INDENT, MAX_LINE_WIDTH);
                if let Some(last) = lines.last_mut() {
                    last.push_str(&postcond);
                }
                for line in &lines {
                    out.push('\n');
                    out.push_str(line);
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
                let regs: Vec<_> = regexes.regexes().collect();
                for (i, r) in regs.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    self.format_regex(out, r);
                }
                out.push(')');
            }
            Regex::CharClass { char_class, .. } => {
                self.format_char_class(out, char_class);
            }
            Regex::Char { char, .. } => {
                out.push_str(&self.input.text(char.span()));
            }
            Regex::String { string, .. } => {
                out.push_str(&self.input.text(string.span()));
            }
            Regex::Identifier { identifier, .. } => {
                out.push_str(&self.input.text(identifier.span()));
            }
            Regex::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
        }
    }

    fn format_char_class(&self, out: &mut String, cc: &CharClass) {
        if cc.neg().value().is_some() {
            out.push('!');
        }
        out.push('[');
        for re in cc.range_elements().range_elements() {
            match re {
                RangeElement::Alt0 { range, .. } => {
                    out.push_str(&self.input.text(range.start().span()));
                    out.push('-');
                    out.push_str(&self.input.text(range.end().span()));
                }
                RangeElement::Alt1 { range_char, .. } => {
                    out.push_str(&self.input.text(range_char.span()));
                }
                RangeElement::Amb(_) => {
                    unreachable!("ambiguous trees are rejected before this point")
                }
            }
        }
        out.push(']');
    }
}

/// Wrap a sequence of atomic chunks across lines, breaking only between chunks.
fn wrap_chunks(
    prefix: &str,
    chunks: &[String],
    cont_indent: &str,
    max_width: usize,
) -> Vec<String> {
    let single_line = format!("{}{}", prefix, chunks.join(" "));
    if single_line.len() <= max_width {
        return vec![single_line];
    }

    let mut lines = Vec::new();
    let mut current = String::from(prefix);

    for (i, chunk) in chunks.iter().enumerate() {
        if i == 0 {
            current.push_str(chunk);
        } else if current.len() + 1 + chunk.len() <= max_width {
            current.push(' ');
            current.push_str(chunk);
        } else {
            lines.push(current);
            current = format!("{}{}", cont_indent, chunk);
        }
    }
    if !current.is_empty() && current != cont_indent {
        lines.push(current);
    }

    if lines.is_empty() {
        vec![single_line]
    } else {
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildResult, build};

    fn format_source(source: &str) -> Option<String> {
        let input = Input::from(source);
        let ctx = iguana_runtime::parse_tree::ParseContext::new();
        match build(&input, &ctx) {
            BuildResult::Success { ref tree, .. } => Some(format(tree, &input)),
            BuildResult::Error { .. } | BuildResult::Ambiguous => None,
        }
    }

    #[test]
    fn test_simple_grammar() {
        let input = "grammar  Test\n\nRule\n  = \"hello\"\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar Test\n\nRule\n  = \"hello\"\n");
    }

    #[test]
    fn test_multiple_alternatives() {
        let input = "grammar T\n\nA\n  = \"x\"  #X\n  | \"y\"  #Y\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar T\n\nA\n  = \"x\" #X\n  | \"y\" #Y\n");
    }

    #[test]
    fn test_priority_levels() {
        let input = "grammar T\n\nA\n  = \"x\"\n  > \"y\"\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar T\n\nA\n  = \"x\"\n  > \"y\"\n");
    }

    #[test]
    fn test_regex_rule() {
        let input = "grammar T\n\n@Regex\nId = [a-zA-Z]+\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar T\n\n@Regex\nId = [a-zA-Z]+\n");
    }

    #[test]
    fn test_regex_rule_multi_alt() {
        let input = "grammar T\n\n@Regex\nInt = Dec | Hex | Oct\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(
            formatted,
            "grammar T\n\n@Regex\nInt\n  = Dec\n  | Hex\n  | Oct\n"
        );
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
        assert!(format_source("not a valid {{{ grammar").is_none());
    }

    #[test]
    fn test_wrap_chunks_short() {
        let chunks: Vec<String> = vec!["A", "B", "C"].into_iter().map(String::from).collect();
        let lines = wrap_chunks("  = ", &chunks, "      ", 100);
        assert_eq!(lines, vec!["  = A B C"]);
    }

    #[test]
    fn test_wrap_chunks_long() {
        let chunks: Vec<String> = vec!["A", "B", "C", "D", "E", "F"]
            .into_iter()
            .map(String::from)
            .collect();
        let lines = wrap_chunks("  = ", &chunks, "      ", 12);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("  = "));
        assert!(lines[1].starts_with("      "));
    }

    #[test]
    fn test_wrap_chunks_preserves_atomic_symbol() {
        // A symbol like { A "," }+ should never be split
        let chunks: Vec<String> = vec!["AAAA", "{ B \",\" }+", "CCCC"]
            .into_iter()
            .map(String::from)
            .collect();
        let lines = wrap_chunks("  = ", &chunks, "      ", 20);
        // { B "," }+ must appear intact on one line
        let joined = lines.join("\n");
        assert!(
            joined.contains("{ B \",\" }+"),
            "separator list symbol was split: {lines:?}"
        );
    }

    #[test]
    fn test_regex_rule_long_single_alt() {
        let input = "grammar T\n\n\
            @Regex\n\
            VeryLongRuleNameHere =[a-zA-Z] [a-zA-Z] [a-zA-Z] [a-zA-Z] [a-zA-Z] [a-zA-Z] [a-zA-Z] [a-zA-Z] [a-zA-Z]+\n";
        let formatted = format_source(input).unwrap();
        // Should wrap: name on its own line, body wrapped
        assert!(formatted.contains("VeryLongRuleNameHere\n  = "));
        // Idempotent
        let second = format_source(&formatted).unwrap();
        assert_eq!(formatted, second);
    }

    #[test]
    fn test_idempotent() {
        let input = "grammar Iggy\n\n@Layout\nLayout = WS*\n\nRule\n  = SyntaxRule #SyntaxRule\n  | RegexRule  #RegexRule\n";
        let first = format_source(input).unwrap();
        let second = format_source(&first).unwrap();
        assert_eq!(first, second, "Formatting should be idempotent");
    }
}
