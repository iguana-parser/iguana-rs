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

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
