<h1 align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/logo-dark.svg">
    <img src="docs/logo.svg" alt="" width="72">
  </picture>
  Iguana
</h1>

Iguana is a practical GLL parser generator. It reads a grammar written in [Iggy](https://iguana-parser.org/docs/grammar-definition/), a declarative grammar definition language, and produces a Rust parser crate.

- **General parsing.** Iguana accepts any context-free grammar, including grammars with left recursion and ambiguity, and returns all derivations of an ambiguous input as a shared parse forest.
- **Lossless parse trees.** The parse tree includes whitespace and comments. Its leaves cover the whole input, so the source can be reconstructed from the tree alone.
- **Declarative disambiguation.** Disambiguation in Iguana is explicit. The grammar states precedence, associativity, and restrictions to remove the unintended derivations.
- **Rust parsers.** The generated code and parser runtime are written entirely in Rust. Generated parsers can be used as libraries or command-line tools and can also be compiled to WebAssembly.

Iguana is based on PhD research in generalized parsing and declarative disambiguation conducted at [CWI](https://www.cwi.nl/en/research/software-analysis-and-transformation/). The [Java implementation](https://github.com/iguana-parser/iguana) was developed during that research. This repository reimplements the same ideas in Rust. The publications are listed on the [research page](https://iguana-parser.org/docs/research/).

To use Iguana, see the documentation at [iguana-parser.org](https://iguana-parser.org): installation, a getting-started guide, the Iggy grammar reference, and the concepts behind the parsing technology. This repository is the development home.

## Development

After cloning, run `./setup.sh` once (toolchain check, dev tools, git hooks). Then:

```bash
cargo build       # build all workspace crates
cargo xtask test  # run the test suite
```

[docs/architecture.md](docs/architecture.md) describes the repository layout: the crates, the web packages, the dependency graph, and the generation pipeline. [docs/development.md](docs/development.md) covers the xtask commands, the bootstrap workflow, testing, and the checklist after a generator change. See [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.
