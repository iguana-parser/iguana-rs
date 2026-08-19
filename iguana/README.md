# iguana

Iguana is a practical GLL parser generator for Rust. It accepts any
context-free grammar, including grammars with left recursion and ambiguity,
and generates a Rust parser crate from an Iggy grammar. Disambiguation is
explicit: the grammar states precedence, associativity, and restrictions to
remove unintended derivations.

The `iguana` crate provides the command-line interface for creating grammar
projects, generating parser crates, building WebAssembly parser bundles, and
serving those bundles locally. The
[`iguana-compiler`](https://github.com/iguana-parser/iguana-rs/tree/main/iguana-compiler)
crate provides the grammar IR, validation, analysis, transformations, and Rust
code generation.

## Installation

```sh
cargo install iguana --version 0.1.0-alpha
```

Create a project:

```sh
iguana new calculator
cd calculator
```

Define the grammar in `calculator.iggy`, then generate and build the parser
crate:

```sh
iguana generate
cargo build --release
```

## Commands

- `iguana new <path>` creates a directory with a starter Iggy grammar.
- `iguana generate` reads an Iggy grammar and generates a complete Cargo
  project containing the parser crate. With `--wasm`, it packages the generated
  parser as a WebAssembly bundle for the browser.
- `iguana try` serves the generated WebAssembly bundle at
  `http://127.0.0.1:8000/` by default.

The [Iguana documentation](https://iguana-parser.org) covers installation,
writing Iggy grammars, using generated parsers, and the complete command-line
interface.

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
