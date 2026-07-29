# Iguana

Iguana is a high-performance GLL parser generator. It supports the full class of context-free grammars without limitation (left recursion, ambiguity, and more) and returns all derivations of an ambiguous input in the form of a shared parse forest. Disambiguation is declarative: priorities, associativity, and restrictions describe the intended shape of the parse tree, and Iguana does not rely on heuristics or definition order. Generated parsers are fast, self-contained Rust crates that also compile to WebAssembly to run in the browser.

To use Iguana, see the documentation at [iguana-parser.org](https://iguana-parser.org): installation, a getting-started guide, the iggy grammar reference, and the concepts behind the parsing technology. This repository is the development home.

## Development

After cloning, run `./setup.sh` once (toolchain check, dev tools, git hooks). Then:

```bash
cargo build       # build all workspace crates
cargo xtask test  # run the test suite
```

[docs/architecture.md](docs/architecture.md) describes the repository layout: the crates, the web packages, the dependency graph, and the generation pipeline. [docs/development.md](docs/development.md) covers the xtask commands, the bootstrap workflow, testing, and the checklist after a generator change. Contributions require signing the CLA; see [CONTRIBUTING.md](CONTRIBUTING.md).