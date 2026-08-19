# iggy

`iggy` is the parser for the
[Iggy grammar language](https://iguana-parser.org/docs/grammar-definition/).
Iguana generates the parser from `iggy.iggy`. The generator and language
server use the parser to read grammars. Most users should use the `iguana`
command rather than depend on this implementation crate directly.

The package publishes on crates.io as `iguana-iggy`, since the name `iggy`
belongs to an unrelated project. The library target keeps the name `iggy`.

## License

Licensed under either the
[MIT License](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-MIT)
or the
[Apache License, Version 2.0](https://github.com/iguana-parser/iguana-rs/blob/main/LICENSE-APACHE),
at your option.
