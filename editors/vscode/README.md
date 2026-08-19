# Iguana for VS Code

The Iguana extension adds language support for
[Iggy](https://iguana-parser.org/docs/grammar-definition/) grammars. It
registers the `.iggy` file type and provides diagnostics, semantic
highlighting, formatting, document symbols, folding, go-to-definition, and
find-references navigation.

## Local installation

From the repository root, install the current language server and extension:

```sh
cargo xtask install-vscode-extension
```

This builds `iguana-lsp` in release mode, installs it into `$CARGO_HOME/bin`,
packages the extension, and installs the resulting VSIX with the `code`
command. Reload VS Code when it finishes. Node.js, npm, and the `code` command
must be on `PATH`; the extension tooling requires Node.js 22.12 or newer.

By default, the extension checks the Cargo bin directory and then `PATH` for
`iguana-lsp`. The `iguana.lsp.path` setting can point to a binary elsewhere.

## Development

Install the extension's dependencies and compile it from this directory:

```sh
npm install
npm run compile
```

`compile` type-checks the extension and bundles it into `dist/extension.js`.
Run `npm run watch` to rebuild the bundle after source changes, or run
`npm run package:vsix` to create an installable VSIX. To test a changed bundle
in your normal VS Code installation, rerun
`cargo xtask install-vscode-extension` from the repository root and reload VS
Code.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
