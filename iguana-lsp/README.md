# iguana-lsp

`iguana-lsp` provides diagnostics, semantic highlighting, formatting, document
symbols, folding, go-to-definition, and find-references navigation for
[Iggy](https://iguana-parser.org/docs/grammar-definition/) grammars. The server
communicates over standard input and output.

The [Iguana extension for VS Code](https://github.com/iguana-parser/iguana-rs/tree/main/editors/vscode)
launches `iguana-lsp` as a separate process.

## Installation

```bash
cargo install iguana-lsp --version 0.1.0-alpha.1
```

The VS Code extension first checks its `iguana.lsp.path` setting, then the
Cargo bin directory, and finally `PATH` when locating the server.

## Comment placement

Consider a standalone comment block between two rules:

```iggy
grammar Test

S = "a"
// explains S

T = "b"
```

The formatter produces:

```iggy
grammar Test

S
  = "a"
  // explains S

T
  = "b"
```

The comment has no blank line before it and one blank line after it, so the
formatter attaches it to `S` and indents it by two spaces. The formatter uses
these rules for each standalone block between grammar constructs:

- **Blank line after only.** The formatter attaches the block to the preceding
  construct.
- **Blank line before only.** The formatter attaches the block to the following
  construct, at the beginning of the line.
- **Blank lines before and after.** The formatter starts the block at the
  beginning of the line, separate from both constructs.
- **No blank line on either side.** The formatter cannot determine an
  attachment and returns the source unchanged.

When several blocks occur between two constructs, the formatter applies these
rules to each block independently. For example, with no outer blank lines and
one blank line between two blocks, the formatter attaches the first block to
the preceding construct and the second block to the following construct.

A comment on the same line as the end of a rule stays on that line. An attached
comment after a syntax rule or a multi-line lexical rule uses the rule body's
two-space indentation. An attached comment after a single-line lexical rule or
the grammar header starts at the beginning of the line. After the final rule,
the formatter attaches an adjacent block to that rule. A blank line before the
block separates it from the rule. The formatter returns the source unchanged
when it encounters any other comment position that it cannot preserve.

Consecutive comment lines form one block. When one or more blank lines separate
two comment blocks, the formatted output contains one blank line between the
blocks. A blank line before a standalone comment inside a rule also appears in
the output:

```iggy
grammar Test

S

  // explains the first alternative
  = "a"
```

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
