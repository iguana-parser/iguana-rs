# iguana-compiler

`iguana-compiler` implements the parser-generation pipeline used by Iguana. It
resolves and validates an Iggy grammar, computes analyses including first and
follow sets and LL(1) classification, applies the EBNF, restriction,
precedence, and layout transformations, and emits a Rust parser crate.

The [`iguana`](https://github.com/iguana-parser/iguana-rs/tree/main/iguana)
command is the supported interface to the generator, and `iguana-lsp` also
builds on this crate. The library API exists for Iguana's own tools and may
change between releases.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option. Generated code may be used under terms of your choice,
including in proprietary software, without an Iguana attribution or notice
requirement. See the repository's
[licensing terms](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSING.md)
for the authoritative details and dependency terms.
