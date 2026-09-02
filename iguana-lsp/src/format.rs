// Iggy Grammar Formatter
//
// Formatting rules:
// - Two-space indentation for `=`, `|`, and `>`
// - `=` for the first priority level, `>` for subsequent ones
// - `|` between alternatives within the same priority level
// - Labels (`#Name`) are left-aligned to a column past the longest line of any
//   alternative in the rule (per nonterminal head, not global), so no
//   alternative body reaches into the label column
// - Maximum line width: 80 characters
// - Lines exceeding the limit wrap to the next line with 6-space continuation
//   indent (2 for rule + 4 extra); the label sits at the alignment column on
//   the last line
// - Widths are measured in characters, not bytes, so a multibyte character
//   (an accented letter in a string literal, say) counts as one column
// - Blank line between every rule
// - `@Regex` (optionally preceded by `@Layout` for a layout rule, or by
//   `@Identifier` for an identifier rule) on the line before the regex rule head
// - `@Layout` / `@NoLayout` / `@WithLayout(X)` annotation on the line before the syntax rule head
// - Regex rules with a single alternative are single-line
// - Regex rules with multiple alternatives use multi-line layout (one per line) (head = body postconditions)
// - Character classes have no internal spaces
// - Comments are emitted from Layout nodes during the tree walk
// - Final newline at end of file

use iggy::parse_tree::*;
use iguana_compiler::comments::is_same_line;
use iguana_runtime::input::{Input, Span};
use rustc_hash::FxHashSet;

const MAX_LINE_WIDTH: usize = 80;
const RULE_PREFIX: &str = "  = ";
const ALT_PREFIX: &str = "  | ";
const PRIO_PREFIX: &str = "  > ";
const CONT_INDENT: &str = "      "; // 6 spaces for continuation lines

/// Format an Iggy grammar from its parse tree.
///
/// The formatter returns the original source unchanged when it cannot preserve
/// a comment's position.
pub fn format(tree: &Start<&Grammar<'_>, &Layout<'_>>, input: &Input) -> String {
    let mut formatter = Formatter::new(tree, input);
    let formatted = formatter.format_grammar(tree);
    if formatter.all_comments_emitted() {
        formatted
    } else {
        input.text(Span::new(0, input.len()))
    }
}

struct Formatter<'a> {
    input: &'a Input,
    comments: Vec<Token>,
    emitted_comments: FxHashSet<Span>,
}

/// An alternative formatted into parts, before label alignment.
struct FormattedAlt {
    lines: Vec<String>,
    label: Option<String>,
    separator: Option<Span>,
}

impl<'a> Formatter<'a> {
    fn new(tree: &Start<&Grammar<'a>, &Layout<'a>>, input: &'a Input) -> Self {
        let mut comments = Vec::new();
        collect_comments(tree.as_parse_tree(), &mut comments);
        comments.sort_by_key(|comment| comment.span().left_extent);
        Self {
            input,
            comments,
            emitted_comments: FxHashSet::default(),
        }
    }

    fn all_comments_emitted(&self) -> bool {
        self.comments
            .iter()
            .all(|comment| self.emitted_comments.contains(&comment.span()))
    }

    /// Emits comments from a layout node. A comment on the line of `previous`
    /// is emitted after one space. Each standalone comment starts on its own
    /// line after `standalone_indent`. Returns whether the call emitted any
    /// comment.
    fn emit_comments(
        &mut self,
        out: &mut String,
        layout: &Layout,
        previous: Span,
        standalone_indent: &str,
    ) -> bool {
        let comments: Vec<_> = layout.line_comments().collect();
        self.emit_comment_tokens(out, comments, previous, standalone_indent)
    }

    fn emit_trailing_comments(
        &mut self,
        out: &mut String,
        layout: &Layout,
        previous: Span,
    ) -> bool {
        let comments: Vec<_> = layout.line_comments().collect();
        if comments.is_empty()
            || comments.iter().any(|comment| {
                comment.span().left_extent < previous.right_extent
                    || !is_same_line(
                        self.input,
                        comment.span().left_extent,
                        previous.right_extent,
                    )
            })
        {
            return false;
        }
        self.emit_comment_tokens(out, comments, previous, "")
    }

    fn emit_trailing_comment_between(
        &mut self,
        out: &mut String,
        left: u32,
        right: u32,
        previous: Span,
    ) -> bool {
        let Some(comment) = self.unemitted_comments_between(left, right).next() else {
            return false;
        };
        if !is_same_line(
            self.input,
            comment.span().left_extent,
            previous.right_extent,
        ) {
            return false;
        }
        self.emit_comment_tokens(out, vec![comment], previous, "")
    }

    fn emit_comments_between(
        &mut self,
        out: &mut String,
        left: u32,
        right: u32,
        previous: Span,
        standalone_indent: &str,
    ) -> bool {
        let comments = self.unemitted_comments_between(left, right).collect();
        self.emit_comment_tokens(out, comments, previous, standalone_indent)
    }

    fn unemitted_comments_between(
        &self,
        left: u32,
        right: u32,
    ) -> impl Iterator<Item = Token> + '_ {
        self.comments.iter().copied().filter(move |comment| {
            let start = comment.span().left_extent;
            start >= left && start < right && !self.emitted_comments.contains(&comment.span())
        })
    }

    /// Emits comments at a grammar-construct boundary.
    ///
    /// Each standalone comment block follows one of four placement rules:
    ///
    /// - **Blank line only after.** The formatter attaches the block to the
    ///   preceding construct.
    /// - **Blank line only before.** The formatter attaches the block to the
    ///   following construct.
    /// - **Blank lines on both sides.** The formatter separates the block from
    ///   both constructs.
    /// - **No blank line on either side.** The formatter cannot determine the
    ///   attachment. The block stays unemitted, and `format` returns the source
    ///   unchanged.
    ///
    /// A blank line separates consecutive blocks, so every block after the first
    /// has a blank line before it. When `following` is `None`, the end of the
    /// document acts as a blank line after the final block.
    fn emit_rule_boundary_comments(
        &mut self,
        out: &mut String,
        previous: Span,
        following: Option<Span>,
        previous_indent: &str,
    ) {
        let right = following.map_or_else(|| self.input.len(), |span| span.left_extent);
        self.emit_trailing_comment_between(out, previous.right_extent, right, previous);

        let comments: Vec<_> = self
            .unemitted_comments_between(previous.right_extent, right)
            .collect();
        if comments.is_empty() {
            if following.is_some() {
                out.push_str("\n\n");
            }
            return;
        }

        let blank_before =
            self.has_blank_line_between(previous.right_extent, comments[0].span().left_extent);
        let blank_after = following.is_none_or(|span| {
            self.has_blank_line_between(
                comments.last().unwrap().span().right_extent,
                span.left_extent,
            )
        });
        let block_boundaries: Vec<_> = comments
            .windows(2)
            .enumerate()
            .filter_map(|(index, pair)| {
                self.has_blank_line_between(pair[0].span().right_extent, pair[1].span().left_extent)
                    .then_some(index + 1)
            })
            .collect();

        if !blank_before && !blank_after && block_boundaries.is_empty() {
            out.push_str("\n\n");
            return;
        }

        let mut block_start = 0;
        let mut source_previous = previous;
        for block_end in block_boundaries
            .iter()
            .copied()
            .chain(std::iter::once(comments.len()))
        {
            let block_blank_before = block_start > 0 || blank_before;
            let block_blank_after = block_end < comments.len() || blank_after;
            let indent = if !block_blank_before && block_blank_after {
                previous_indent
            } else {
                ""
            };
            let block = comments[block_start..block_end].to_vec();
            self.emit_comment_tokens(out, block, source_previous, indent);
            source_previous = comments[block_end - 1].span();
            block_start = block_end;
        }

        if following.is_some() {
            if blank_after {
                out.push_str("\n\n");
            } else {
                out.push('\n');
            }
        }
    }

    fn has_blank_line_between(&self, left: u32, right: u32) -> bool {
        let left_line = self.input.line_column(left).0;
        let right_line = self.input.line_column(right).0;
        right_line.saturating_sub(left_line) > 1
    }

    fn emit_comment_tokens(
        &mut self,
        out: &mut String,
        comments: Vec<Token>,
        previous: Span,
        standalone_indent: &str,
    ) -> bool {
        let mut emitted_any = false;
        let mut source_previous = previous.right_extent;
        for comment in comments {
            if self.emitted_comments.contains(&comment.span()) {
                source_previous = comment.span().right_extent;
                continue;
            }
            if !out.is_empty()
                && !out.ends_with('\n')
                && is_same_line(self.input, comment.span().left_extent, source_previous)
            {
                out.push(' ');
                out.push_str(&self.input.text(comment.span()));
            } else {
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
                if !out.is_empty()
                    && !out.ends_with("\n\n")
                    && self.has_blank_line_between(source_previous, comment.span().left_extent)
                {
                    out.push('\n');
                }
                out.push_str(standalone_indent);
                out.push_str(&self.input.text(comment.span()));
            }
            self.emitted_comments.insert(comment.span());
            source_previous = comment.span().right_extent;
            emitted_any = true;
        }
        emitted_any
    }

    fn format_grammar(&mut self, tree: &Start<&Grammar<'a>, &Layout<'a>>) -> String {
        let mut out = String::new();
        let grammar = tree.node;

        let last_comment_before_grammar = tree.before.line_comments().last();
        self.emit_comments(&mut out, tree.before, Span::new(0, 0), "");
        if !out.is_empty() {
            out.push('\n');
            let grammar_start = first_syntax_token(grammar.as_parse_tree()).unwrap();
            if last_comment_before_grammar.is_some_and(|comment| {
                self.has_blank_line_between(comment.span().right_extent, grammar_start.left_extent)
            }) {
                out.push('\n');
            }
        }

        // grammar Name
        out.push_str("grammar ");
        out.push_str(&self.input.text(grammar.name().span()));
        self.emit_trailing_comments(&mut out, grammar.layout_3(), grammar.name().span());

        // Rules
        let rules: Vec<_> = grammar.rules().rules().collect();
        let mut previous_indent = "";
        for (index, rule) in rules.iter().enumerate() {
            let previous = if index == 0 {
                grammar.name().span()
            } else {
                last_syntax_token(rules[index - 1].as_parse_tree()).unwrap()
            };
            let current = first_syntax_token(rule.as_parse_tree()).unwrap();
            self.emit_rule_boundary_comments(&mut out, previous, Some(current), previous_indent);
            previous_indent = self.format_rule(&mut out, rule);
        }

        let previous = rules
            .last()
            .and_then(|rule| last_syntax_token(rule.as_parse_tree()))
            .unwrap_or_else(|| grammar.name().span());
        self.emit_rule_boundary_comments(&mut out, previous, None, previous_indent);
        out.push('\n');
        out
    }

    /// Returns the indentation of the rule body, which an attached comment shares.
    fn format_rule(&mut self, out: &mut String, rule: &Rule) -> &'static str {
        match rule {
            Rule::SyntaxRule { syntax_rule, .. } => {
                self.format_syntax_rule(out, syntax_rule);
                "  "
            }
            Rule::RegexRule { regex_rule, .. } => {
                if self.format_regex_rule(out, regex_rule) {
                    "  "
                } else {
                    ""
                }
            }
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

    fn format_syntax_rule(&mut self, out: &mut String, rule: &SyntaxRule) {
        if let Some(annotation) = rule.annotation().value() {
            self.format_annotation(out, annotation);
            self.emit_comments(out, rule.layout_1(), annotation.span(), "");
            out.push('\n');
        }

        out.push_str(&self.input.text(rule.head().span()));
        self.emit_comments(out, rule.layout_3(), rule.head().span(), "  ");

        // Pass 1: format alternatives, keep references to originals
        let priority_levels: Vec<_> = rule.priority_levels().priority_levels().collect();
        let priority_separators = priority_level_separators(rule.priority_levels());
        let mut formatted_alts: Vec<(FormattedAlt, &Alternative)> = Vec::new();

        for (pi, pl) in priority_levels.iter().enumerate() {
            let prefix_first = if pi == 0 { RULE_PREFIX } else { PRIO_PREFIX };
            let alternatives: Vec<_> = pl.alternatives().alternatives().collect();
            let alt_separators = alternative_separators(pl.alternatives());

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
                            chunks.push(self.symbol_to_string(sym));
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
                let separator = if ai > 0 {
                    Some(alt_separators[ai - 1])
                } else if pi > 0 {
                    Some(priority_separators[pi - 1])
                } else {
                    None
                };
                formatted_alts.push((
                    FormattedAlt {
                        lines,
                        label,
                        separator,
                    },
                    alt,
                ));
            }
        }

        // Compute label alignment column from the longest line of any
        // alternative, including the first lines of wrapped alternatives, so no
        // alternative body reaches into the label column.
        let has_any_label = formatted_alts.iter().any(|(fa, _)| fa.label.is_some());
        let label_column = if has_any_label {
            formatted_alts
                .iter()
                .flat_map(|(fa, _)| &fa.lines)
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0)
                + 1
        } else {
            0
        };

        // Pass 2: emit with alignment, visit layout nodes for comments
        let mut prev_span = rule.head().span();
        for (fa, alt) in &formatted_alts {
            if let Some(separator) = fa.separator {
                self.emit_comments_between(
                    out,
                    prev_span.right_extent,
                    separator.left_extent,
                    prev_span,
                    "  ",
                );
            }
            out.push('\n');
            for (i, line) in fa.lines.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                out.push_str(line);
            }
            if let Some(ref label) = fa.label {
                // The last line is one of the lines in label_column's max, so
                // label_column always clears it with at least one space.
                let last_len = fa.lines.last().unwrap().chars().count();
                for _ in 0..label_column - last_len {
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
            self.emit_trailing_comments(out, layout, prev_span);
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
            Annotation::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
        }
    }

    fn format_symbols<'i>(&self, out: &mut String, symbols: impl Iterator<Item = &'i Symbol<'i>>) {
        for (i, s) in symbols.enumerate() {
            if i > 0 {
                out.push(' ');
            }
            self.format_symbol(out, s);
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
            Symbol::Paren { seqs, .. } => {
                out.push('(');
                for (i, seq) in seqs.symbols().enumerate() {
                    if i > 0 {
                        out.push_str(" | ");
                    }
                    self.format_symbols(out, seq);
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
            Symbol::PostCondition {
                symbol, conditions, ..
            } => {
                self.format_symbol(out, symbol);
                // The grammar accepts the conditions in any order. The
                // formatter emits them in canonical order.
                let mut excludes = Vec::new();
                let mut excepts = Vec::new();
                let mut follow = Vec::new();
                let mut layout_aware_follow = Vec::new();
                for condition in conditions.post_conditions() {
                    match condition {
                        PostCondition::Exclude { identifier, .. } => excludes.push(identifier),
                        PostCondition::Except { identifier, .. } => excepts.push(identifier),
                        PostCondition::FollowRestriction { identifier, .. } => {
                            follow.push(identifier)
                        }
                        PostCondition::LayoutAwareFollowRestriction { identifier, .. } => {
                            layout_aware_follow.push(identifier)
                        }
                        PostCondition::Amb(_) => {
                            unreachable!("ambiguous trees are rejected before this point")
                        }
                    }
                }
                for id in excludes {
                    out.push('!');
                    out.push_str(&self.input.text(id.span()));
                }
                for id in excepts {
                    out.push_str(" \\ ");
                    out.push_str(&self.input.text(id.span()));
                }
                for id in follow {
                    out.push_str(" !>> ");
                    out.push_str(&self.input.text(id.span()));
                }
                for id in layout_aware_follow {
                    out.push_str(" !>>> ");
                    out.push_str(&self.input.text(id.span()));
                }
            }
            Symbol::PreCondition {
                conditions, symbol, ..
            } => {
                for condition in conditions.pre_conditions() {
                    out.push_str(&self.input.text(condition.identifier().span()));
                    out.push_str(" !<< ");
                }
                self.format_symbol(out, symbol);
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
        for condition in rule.regex_post_conditions().regex_post_conditions() {
            match condition {
                RegexPostCondition::Except { identifier, .. } => {
                    s.push_str(" \\ ");
                    s.push_str(&self.input.text(identifier.span()));
                }
                RegexPostCondition::FollowRestriction { identifier, .. } => {
                    s.push_str(" !>> ");
                    s.push_str(&self.input.text(identifier.span()));
                }
                RegexPostCondition::Amb(_) => {
                    unreachable!("ambiguous trees are rejected before this point")
                }
            }
        }
        s
    }

    /// Formats a lexical rule and reports whether its body spans multiple lines.
    fn format_regex_rule(&mut self, out: &mut String, rule: &RegexRule) -> bool {
        let mut annotations = Vec::new();
        if let Some(token) = rule.layout().value() {
            annotations.push(("@Layout", token.span()));
        }
        if let Some(token) = rule.id_annot().value() {
            annotations.push(("@Identifier", token.span()));
        }
        annotations.push(("@Regex", rule.lit_4().span()));
        for (index, (text, span)) in annotations.iter().enumerate() {
            out.push_str(text);
            let next = annotations
                .get(index + 1)
                .map_or_else(|| rule.identifier().span(), |(_, span)| *span);
            if self.emit_comments_between(out, span.right_extent, next.left_extent, *span, "") {
                out.push('\n');
            } else if index + 1 < annotations.len() {
                out.push(' ');
            } else {
                out.push('\n');
            }
        }

        let groups: Vec<Vec<_>> = rule
            .body()
            .regexes()
            .map(|group| group.collect::<Vec<_>>())
            .collect();

        let pre_prefix = rule
            .regex_pre_condition()
            .value()
            .map(|pre| format!("{} !<< ", self.input.text(pre.identifier().span())));
        let pre_str = pre_prefix.as_deref().unwrap_or("");
        let postcond = self.postconditions_to_string(rule);
        let name_span = rule.identifier().span();
        let name = self.input.text(name_span);
        let comments_after_name = self
            .unemitted_comments_between(name_span.right_extent, rule.lit_8().span().left_extent)
            .next()
            .is_some();

        if groups.len() > 1 {
            // Multi-alt: name on its own line, each alt on its own line(s)
            out.push_str(&name);
            self.emit_comments_between(
                out,
                name_span.right_extent,
                rule.lit_8().span().left_extent,
                name_span,
                "",
            );
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
            true
        } else {
            // Single-alt: try single line first
            let chunks: Vec<String> = groups[0].iter().map(|r| self.regex_to_string(r)).collect();
            let body = chunks.join(" ");
            let single = format!("{} = {}{}{}", name, pre_str, body, postcond);

            if single.chars().count() <= MAX_LINE_WIDTH && !comments_after_name {
                out.push_str(&single);
                false
            } else {
                // Too long: name on its own line, wrapped body
                out.push_str(&name);
                self.emit_comments_between(
                    out,
                    name_span.right_extent,
                    rule.lit_8().span().left_extent,
                    name_span,
                    "",
                );
                let prefix = format!("{}{}", RULE_PREFIX, pre_str);
                let mut lines = wrap_chunks(&prefix, &chunks, CONT_INDENT, MAX_LINE_WIDTH);
                if let Some(last) = lines.last_mut() {
                    last.push_str(&postcond);
                }
                for line in &lines {
                    out.push('\n');
                    out.push_str(line);
                }
                true
            }
        }
    }

    fn format_regexes<'i>(&self, out: &mut String, regexes: impl Iterator<Item = &'i Regex<'i>>) {
        for (i, r) in regexes.enumerate() {
            if i > 0 {
                out.push(' ');
            }
            self.format_regex(out, r);
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
            Regex::Paren { seqs, .. } => {
                out.push('(');
                for (i, seq) in seqs.regexes().enumerate() {
                    if i > 0 {
                        out.push_str(" | ");
                    }
                    self.format_regexes(out, seq);
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

fn collect_comments<'a>(tree: ParseTree<'a>, comments: &mut Vec<Token>) {
    match tree {
        ParseTree::Layout(layout) => comments.extend(layout.line_comments()),
        _ => {
            for child in tree.children() {
                collect_comments(child, comments);
            }
        }
    }
}

fn first_syntax_token(tree: ParseTree<'_>) -> Option<Span> {
    match tree {
        ParseTree::Layout(_) => None,
        ParseTree::Token(token) => Some(token.span()),
        _ => tree.children().into_iter().find_map(first_syntax_token),
    }
}

fn last_syntax_token(tree: ParseTree<'_>) -> Option<Span> {
    match tree {
        ParseTree::Layout(_) => None,
        ParseTree::Token(token) => Some(token.span()),
        _ => tree
            .children()
            .into_iter()
            .rev()
            .find_map(last_syntax_token),
    }
}

fn priority_level_separators(priority_levels: &Star1<'_>) -> Vec<Span> {
    let mut separators = Vec::new();
    match priority_levels {
        Star1::Alt0 { opt_2, .. } => match opt_2 {
            Opt2::Alt0 {
                priority_levels, ..
            } => collect_priority_level_separators(priority_levels, &mut separators),
            Opt2::Alt1 { .. } => {}
            Opt2::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
        },
        Star1::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
    }
    separators
}

fn collect_priority_level_separators(priority_levels: &Plus1<'_>, separators: &mut Vec<Span>) {
    match priority_levels {
        Plus1::Alt0 {
            priority_levels_0,
            lit_2,
            ..
        } => {
            collect_priority_level_separators(priority_levels_0, separators);
            separators.push(lit_2.span());
        }
        Plus1::Alt1 { .. } => {}
        Plus1::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
    }
}

fn alternative_separators(alternatives: &Star3<'_>) -> Vec<Span> {
    let mut separators = Vec::new();
    match alternatives {
        Star3::Alt0 { opt_8, .. } => match opt_8 {
            Opt8::Alt0 { alternatives, .. } => {
                collect_alternative_separators(alternatives, &mut separators);
            }
            Opt8::Alt1 { .. } => {}
            Opt8::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
        },
        Star3::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
    }
    separators
}

fn collect_alternative_separators(alternatives: &Plus5<'_>, separators: &mut Vec<Span>) {
    match alternatives {
        Plus5::Alt0 {
            alternatives_0,
            lit_2,
            ..
        } => {
            collect_alternative_separators(alternatives_0, separators);
            separators.push(lit_2.span());
        }
        Plus5::Alt1 { .. } => {}
        Plus5::Amb(_) => unreachable!("ambiguous trees are rejected before this point"),
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
    if single_line.chars().count() <= max_width {
        return vec![single_line];
    }

    let mut lines = Vec::new();
    let mut current = String::from(prefix);

    for (i, chunk) in chunks.iter().enumerate() {
        if i == 0 {
            current.push_str(chunk);
        } else if current.chars().count() + 1 + chunk.chars().count() <= max_width {
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
        let tree_arena = iguana_runtime::arena::Arena::new();
        match build(&input, &tree_arena) {
            BuildResult::Success { tree, .. } => Some(format(tree, &input)),
            BuildResult::Error(_) | BuildResult::Ambiguous => None,
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
    fn test_comments_before_alternatives_and_priority_levels() {
        let input = r#"grammar       Test

S
=    "a"
// explanation of the next alternative
|     "b"
// explanation of the next priority level
>   left   "c"
"#;
        let expected = r#"grammar Test

S
  = "a"
  // explanation of the next alternative
  | "b"
  // explanation of the next priority level
  > left "c"
"#;

        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, expected);
        for comment in [
            "// explanation of the next alternative",
            "// explanation of the next priority level",
        ] {
            assert_eq!(formatted.matches(comment).count(), 1);
        }
        assert_eq!(format_source(&formatted).unwrap(), expected);
    }

    #[test]
    fn test_regex_rule() {
        let input = "grammar T\n\n@Regex\nId = [a-zA-Z]+\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar T\n\n@Regex\nId = [a-zA-Z]+\n");
    }

    #[test]
    fn test_identifier_regex_rule() {
        let input = "grammar T\n\n@Identifier   @Regex\nId = [a-z]+\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, "grammar T\n\n@Identifier @Regex\nId = [a-z]+\n");
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
    fn test_layout_aware_follow_restriction() {
        let input = "grammar T\n\nS\n  = A  !>>>  B\n\n@Regex\nA = \"a\"\n\n@Regex\nB = \"b\"\n";
        let formatted = format_source(input).unwrap();
        assert_eq!(
            formatted,
            "grammar T\n\nS\n  = A !>>> B\n\n@Regex\nA = \"a\"\n\n@Regex\nB = \"b\"\n"
        );
    }

    #[test]
    fn test_postfix_conditions_normalized() {
        let input = r#"grammar T

S
  = A !>> B \ K !X A

A
  = "a" #X

B
  = "b"

@Regex
K = "k"
"#;
        let expected = r#"grammar T

S
  = A!X \ K !>> B A

A
  = "a" #X

B
  = "b"

@Regex
K = "k"
"#;
        assert_eq!(format_source(input).unwrap(), expected);
    }

    #[test]
    fn test_precede_restriction_list() {
        let input = r#"grammar T

S
  = X  !<<  Y  !<<  A

A
  = "a"

@Regex
X = "x"

@Regex
Y = "y"
"#;
        let expected = r#"grammar T

S
  = X !<< Y !<< A

A
  = "a"

@Regex
X = "x"

@Regex
Y = "y"
"#;
        assert_eq!(format_source(input).unwrap(), expected);
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
    fn test_label_alignment_with_wrapped_alternative() {
        // When an alternative is long enough to wrap, its first line must not
        // reach into the label column: every label aligns past every body line.
        let long_alt = "Sym ".repeat(30);
        let input = format!("grammar T\n\nS\n  = \"a\" #Short\n  | {long_alt}#Long\n");
        let formatted = format_source(&input).unwrap();
        let lines: Vec<_> = formatted.lines().collect();

        let label_cols: Vec<_> = lines
            .iter()
            .filter(|l| l.contains('#'))
            .map(|l| l.find('#').unwrap())
            .collect();
        assert!(
            label_cols.windows(2).all(|w| w[0] == w[1]),
            "labels are not aligned to one column: {label_cols:?}"
        );

        let label_col = label_cols[0];
        for line in &lines {
            // The body ends where the padding before the label starts; an
            // unlabeled line's body is the whole trimmed line.
            let body_end = line.find('#').unwrap_or_else(|| line.trim_end().len());
            assert!(
                body_end <= label_col,
                "alternative body reaches into the label column: {line:?}"
            );
        }
    }

    /// Assert every label in the formatted grammar starts at the same column.
    fn assert_labels_aligned(input: &str) {
        let formatted = format_source(input).unwrap();
        let cols: Vec<_> = formatted
            .lines()
            .filter(|l| l.contains('#'))
            .map(|l| l[..l.find('#').unwrap()].chars().count())
            .collect();
        assert!(cols.len() >= 2, "expected at least two labels: {cols:?}");
        assert!(
            cols.windows(2).all(|w| w[0] == w[1]),
            "labels are not aligned: {cols:?}"
        );
    }

    #[test]
    fn test_label_alignment_not_byte_based() {
        // A non-ASCII string literal spans more bytes than characters, so a
        // byte-based width count would misalign these labels.
        assert_labels_aligned("grammar T\n\nS\n  = \"café\" #A\n  | \"ab\" #B\n");
    }

    #[test]
    fn test_trailing_comment() {
        let input = "grammar T // a grammar\n\nRule\n  = \"a\"\n";
        let formatted = format_source(input).unwrap();
        assert!(formatted.starts_with("grammar T // a grammar\n"));
    }

    #[test]
    fn test_standalone_comments_between_rules_retain_attachment() {
        let cases = [
            (
                "grammar T\n\nA=\"a\" // trailing\n// explains A\n\nB=\"b\"\n",
                "grammar T\n\nA\n  = \"a\" // trailing\n  // explains A\n\nB\n  = \"b\"\n",
            ),
            (
                "grammar T\n\nA=\"a\"\n\n// explains B\nB=\"b\"\n",
                "grammar T\n\nA\n  = \"a\"\n\n// explains B\nB\n  = \"b\"\n",
            ),
            (
                "grammar T\n\nA=\"a\"\n\n// separate context\n\nB=\"b\"\n",
                "grammar T\n\nA\n  = \"a\"\n\n// separate context\n\nB\n  = \"b\"\n",
            ),
            (
                "grammar T\n\nA=\"a\"\n// explains A\n\n// explains B\nB=\"b\"\n",
                "grammar T\n\nA\n  = \"a\"\n  // explains A\n\n// explains B\nB\n  = \"b\"\n",
            ),
            (
                "grammar T\n\nA=\"a\"\n// explains A\n\n// separate context\n\n// explains B\nB=\"b\"\n",
                "grammar T\n\nA\n  = \"a\"\n  // explains A\n\n// separate context\n\n// explains B\nB\n  = \"b\"\n",
            ),
        ];

        for (input, expected) in cases {
            let formatted = format_source(input).unwrap();
            assert_eq!(formatted, expected);
            assert_eq!(format_source(&formatted).unwrap(), expected);
        }
    }

    #[test]
    fn test_trailing_comments_before_next_rule() {
        let cases = [
            (
                "grammar T\n\n@Regex\nX = \"x\" // t\n@Regex\nY = \"y\"\n",
                "grammar T\n\n@Regex\nX = \"x\" // t\n\n@Regex\nY = \"y\"\n",
            ),
            (
                "grammar T\n\nS = \"a\" #L // t\nT = \"b\"\n",
                "grammar T\n\nS\n  = \"a\" #L // t\n\nT\n  = \"b\"\n",
            ),
            (
                "grammar T\n\nS = \"a\" // t\n\n// c\nT = \"b\"\n",
                "grammar T\n\nS\n  = \"a\" // t\n\n// c\nT\n  = \"b\"\n",
            ),
        ];

        for (input, expected) in cases {
            let formatted = format_source(input).unwrap();
            assert_eq!(formatted, expected);
            assert_eq!(format_source(&formatted).unwrap(), expected);
        }
    }

    #[test]
    fn test_blank_lines_between_comment_blocks() {
        let cases = [
            (
                "grammar T\n\nA=\"a\"\n\n// first block\n\n\n// second block\nB=\"b\"\n",
                "grammar T\n\nA\n  = \"a\"\n\n// first block\n\n// second block\nB\n  = \"b\"\n",
            ),
            (
                "grammar T\n\nA=\"a\"\n// first block\n\n\n// second block\n\nB=\"b\"\n",
                "grammar T\n\nA\n  = \"a\"\n  // first block\n\n// second block\n\nB\n  = \"b\"\n",
            ),
        ];

        for (input, expected) in cases {
            let formatted = format_source(input).unwrap();
            assert_eq!(formatted, expected);
            assert_eq!(format_source(&formatted).unwrap(), expected);
        }
    }

    #[test]
    fn test_comments_around_syntax_and_regex_rules() {
        let input = r#"grammar Test

Rule
// explanation of the first alternative
=   "a"

@Regex
First="first"

// explanation of the following lexical rule
@Regex
Second   = "second"
"#;
        let expected = r#"grammar Test

Rule
  // explanation of the first alternative
  = "a"

@Regex
First = "first"

// explanation of the following lexical rule
@Regex
Second = "second"
"#;

        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, expected);
        for comment in [
            "// explanation of the first alternative",
            "// explanation of the following lexical rule",
        ] {
            assert_eq!(formatted.matches(comment).count(), 1);
        }
        assert_eq!(format_source(&formatted).unwrap(), expected);
    }

    #[test]
    fn test_unsupported_comment_positions_leave_source_unchanged() {
        for input in [
            "grammar Test\n\nS=(\"a\" // inside group\n| \"b\")\n",
            "grammar Test\n\nS = \"a\"\n| // after the alternative marker\n\"b\"\n",
            "grammar Test\n\nA=\"a\"\n// ambiguous attachment\nB=\"b\"\n",
        ] {
            assert_eq!(format_source(input).unwrap(), input);
        }
    }

    #[test]
    fn test_comments_inside_regex_rule_layouts() {
        let input = r#"grammar Test

@Regex // describes the lexical rule
Token // separates the head from equals
= "token"
"#;
        let expected = r#"grammar Test

@Regex // describes the lexical rule
Token // separates the head from equals
  = "token"
"#;

        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, expected);
        for comment in [
            "// describes the lexical rule",
            "// separates the head from equals",
        ] {
            assert_eq!(formatted.matches(comment).count(), 1);
        }
        assert_eq!(format_source(&formatted).unwrap(), expected);
    }

    #[test]
    fn test_comments_before_and_after_the_grammar() {
        let input = "// before grammar\ngrammar Test\n\nS = \"a\"\n// after final rule\n";
        let expected = "// before grammar\ngrammar Test\n\nS\n  = \"a\"\n  // after final rule\n";

        let formatted = format_source(input).unwrap();
        assert_eq!(formatted, expected);
        for comment in ["// before grammar", "// after final rule"] {
            assert_eq!(formatted.matches(comment).count(), 1);
        }
        assert_eq!(format_source(&formatted).unwrap(), expected);
    }

    #[test]
    fn test_comments_after_final_rule_use_rule_indentation() {
        let cases = [
            (
                "grammar Test\n\nS = \"a\"\n\n// separate context\n",
                "grammar Test\n\nS\n  = \"a\"\n\n// separate context\n",
            ),
            (
                "grammar Test\n\n@Regex\nX=\"x\"\n// explains X\n",
                "grammar Test\n\n@Regex\nX = \"x\"\n// explains X\n",
            ),
            (
                "grammar Test\n\n@Regex\nX=\"x\"|\"y\"\n// explains X\n",
                "grammar Test\n\n@Regex\nX\n  = \"x\"\n  | \"y\"\n  // explains X\n",
            ),
        ];

        for (input, expected) in cases {
            let formatted = format_source(input).unwrap();
            assert_eq!(formatted, expected);
            assert_eq!(format_source(&formatted).unwrap(), expected);
        }
    }

    #[test]
    fn test_comments_after_lexical_rules_follow_body_indentation() {
        let cases = [
            (
                "grammar Test\n\n@Regex\nX=\"x\"\n// explains X\n\n@Regex\nY=\"y\"\n",
                "grammar Test\n\n@Regex\nX = \"x\"\n// explains X\n\n@Regex\nY = \"y\"\n",
            ),
            (
                "grammar Test\n\n@Regex\nX=\"x\"|\"y\"\n// explains X\n\n@Regex\nY=\"y\"\n",
                "grammar Test\n\n@Regex\nX\n  = \"x\"\n  | \"y\"\n  // explains X\n\n@Regex\nY = \"y\"\n",
            ),
        ];

        for (input, expected) in cases {
            let formatted = format_source(input).unwrap();
            assert_eq!(formatted, expected);
            assert_eq!(format_source(&formatted).unwrap(), expected);
        }
    }

    #[test]
    fn test_comments_around_grammar_and_first_rule_preserve_blank_lines() {
        let cases = [
            (
                "// before grammar\n\ngrammar Test\n\nS = \"a\"\n",
                "// before grammar\n\ngrammar Test\n\nS\n  = \"a\"\n",
            ),
            (
                "grammar Test\n\n// separate context\n\nS = \"a\"\n",
                "grammar Test\n\n// separate context\n\nS\n  = \"a\"\n",
            ),
            (
                "grammar Test // trailing\n// explains grammar\n\nS = \"a\"\n",
                "grammar Test // trailing\n// explains grammar\n\nS\n  = \"a\"\n",
            ),
        ];

        for (input, expected) in cases {
            let formatted = format_source(input).unwrap();
            assert_eq!(formatted, expected);
            assert_eq!(format_source(&formatted).unwrap(), expected);
        }
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
