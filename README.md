<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/logo-dark.svg">
    <img src="docs/logo.svg" alt="" width="140">
  </picture>
</p>

# Iguana

Iguana is a high-performance GLL parser generator. Grammars are written in [iggy](https://iguana-parser.org/docs/grammar-definition/), a declarative grammar definition language, and Iguana compiles them to fast Rust parsers.

- **General parsing.** Iguana accepts the full class of context-free grammars, with no restrictions: unbounded lookahead, left recursion (including indirect left recursion), and ambiguity.
- **All derivations, not one.** Iguana returns all derivations of an ambiguous input in the form of a shared parse forest. Ambiguity is a first-class concept, handled explicitly.
- **Lossless parse trees.** The parse tree represents the full input, including whitespace and comments.
- **Declarative disambiguation.** Priorities, associativity, and restrictions describe the intended shape of the parse tree. Iguana does not rely on heuristics or definition order for disambiguation.
- **Fast, self-contained parsers.** A generated parser is an ordinary Rust crate whose only Iguana dependency is the runtime crate, and it also compiles to WebAssembly to run in the browser.
- **Your parsers are yours.** The generator is GPL, but generated parsers and the runtime they link are permissively licensed (MIT or Apache 2.0): ship them under any license, including in closed-source products. See [LICENSE.md](LICENSE.md).

iguana-rs is a Rust re-implementation of the ideas from PhD research on generalized parsing and declarative disambiguation at [CWI](https://www.cwi.nl/en/research/software-analysis-and-transformation/). The [Java implementation](https://github.com/iguana-parser/iguana) is the version developed during that research. The publications are listed on the [research page](https://iguana-parser.org/docs/concepts/research/).

To use Iguana, see the documentation at [iguana-parser.org](https://iguana-parser.org): installation, a getting-started guide, the iggy grammar reference, and the concepts behind the parsing technology. This repository is the development home.

## Development

After cloning, run `./setup.sh` once (toolchain check, dev tools, git hooks). Then:

```bash
cargo build       # build all workspace crates
cargo xtask test  # run the test suite
```

[docs/architecture.md](docs/architecture.md) describes the repository layout: the crates, the web packages, the dependency graph, and the generation pipeline. [docs/development.md](docs/development.md) covers the xtask commands, the bootstrap workflow, testing, and the checklist after a generator change. Contributions require signing the CLA; see [CONTRIBUTING.md](CONTRIBUTING.md).
