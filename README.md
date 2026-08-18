<h1 align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/logo-dark.svg">
    <img src="docs/logo.svg" alt="" width="72" align="middle">
  </picture>
  Iguana
</h1>

Iguana is a practical GLL parser generator. It reads a grammar written in [Iggy](https://iguana-parser.org/docs/grammar-definition/), a declarative grammar definition language, and generates a parser from that grammar. The generated code and parser runtime are written entirely in Rust. Generated parsers can be used as libraries or command-line tools and can also be compiled to WebAssembly.

Iguana's parsing model has three defining properties:

- **General parsing.** Iguana accepts any context-free grammar, including grammars with left recursion and ambiguity, and returns all derivations of an ambiguous input as a shared parse forest.
- **Lossless parse trees.** The parse tree includes whitespace and comments. Its leaves cover the whole input, so the source can be reconstructed from the tree alone.
- **Declarative disambiguation.** Disambiguation in Iguana is explicit. The grammar states precedence, associativity, and restrictions to remove the unintended derivations.

Iguana is based on PhD research in generalized parsing and declarative disambiguation conducted at [CWI](https://www.cwi.nl/en/research/software-analysis-and-transformation/). The [Java implementation](https://github.com/iguana-parser/iguana) was developed during that research. This repository reimplements the same ideas in Rust. The publications are listed on the [research page](https://iguana-parser.org/docs/research/).

The [Iguana documentation](https://iguana-parser.org) covers installing Iguana, writing Iggy grammars, using generated parsers, and understanding how Iguana parses.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a change. The [development guide](docs/development.md) covers setup, building, testing, bootstrapping, and generated-code changes. The [architecture guide](docs/architecture.md) describes the repository structure and generation pipeline.
