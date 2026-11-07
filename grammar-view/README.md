# grammar-view

`grammar-view` shows an Iggy grammar in a read-only CodeMirror editor on the
Iguana website. Once the language server loads, a click on a reference goes to
its rule, and hovering a reference shows a peek of the rule. Hovering a rule head
shows its reference count, and a click on the count lists the references. A Go to
rule button lists the rules.

The module mounts itself when it loads, so the host page needs one script tag:

```html
<script type="module" src="/grammar-view/grammar-view.js"></script>
```

The host page provides:

- **Mount points.** A `.grammar-view` element with `data-src` and `data-name`
  holds a fetched grammar. A `.gp-grammar` element in `.hero` holds a grammar
  rendered into the page.
- **Tabs.** A `.gp-tabbar` receives the Go to rule button. A `.gp-pane` delays the
  language server until the pane is first shown. Both are optional for a
  `.grammar-view` element.
- **Styles.** The `--gv-*` color properties and the rules for the view's classes
  are in the website's stylesheet.
- **Language server.** The module loads the WebAssembly wrapper built from
  `iguana-lsp/wasm` at `/playgrounds/iggy-lsp/iguana_lsp_wasm.js`.

CodeMirror is bundled, so the host does not need an import map. Install
dependencies at the repository root, then check and build the view:

```sh
cd grammar-view
npm run check
npm run build
```

The build output is the single ES module `dist/grammar-view.js`.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
