# Iguana Tools

Developer tools for the [iguana](https://github.com/iguana-parser/iguana-rs) parser generator.

## Crates

| Crate | Description |
|-------|-------------|
| `lsp` | Language server for iggy grammars. Provides semantic highlighting for `.iggy` files. |
| `terrarium` | Grammar debugging tool (Tauri app). |

## LSP Server

### Building

```bash
cargo install --path lsp
```

This installs the `iguana-lsp` binary to `~/.cargo/bin/`.

### VS Code

Use the [iguana-vscode](https://github.com/iguana-parser/iguana-vscode) extension. It launches `iguana-lsp` automatically.

### Updating

After making changes to the `lsp` crate:

```bash
cargo install --path lsp
```

Then restart the LSP client in your editor.

## Terrarium

Grammar debugging tool built with Tauri.

```bash
cd terrarium
npm install
npm run tauri dev
```

## License

GPL-3.0 — see [LICENSE](LICENSE).
