# Iguana for VS Code

Language support for iggy grammars, the grammar definition language of the
[Iguana](https://iguana-parser.org) parser generator. The extension registers
the `.iggy` file type and launches [`iguana-lsp`](../../iguana-lsp), which
provides diagnostics, semantic highlighting, formatting, and symbols.

The language server is a separate binary. Install it first:

```bash
cargo install iguana-lsp --version 0.1.0-alpha
```

The extension looks for `iguana-lsp` on PATH. The `iguana.lsp.path` setting
points it at a binary elsewhere.

## License

Licensed under either the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
