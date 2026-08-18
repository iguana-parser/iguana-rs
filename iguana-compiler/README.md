# iguana-compiler

The part of Iguana that translates iggy grammars into working parsers. It
contains every stage of that translation: reading and validating the grammar,
the analyses and transforms (precedence and associativity desugaring, LL(1)
classification, restriction handling), and the code generation that emits a
complete Rust parser crate.

The [`iguana`](https://github.com/iguana-parser/iguana-rs) CLI is its
interface, and the language server builds on it as well. Use the `iguana`
binary rather than depending on this crate: its API exists for that tooling
and may change freely between releases.

## License

Licensed under either the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option. Parsers
generated with Iguana may be used in open-source or proprietary projects.
