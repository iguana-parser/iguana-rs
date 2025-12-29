# Iguana-rs

A GLL parser generator written in Rust.

## Development

### Bootstrapping

After making changes to the parser generator (e.g., `src/generator/parser_gen.rs`), you need to regenerate the iggy parser. Run:

```bash
./bootstrap.sh
```

This script regenerates the parser in the `iggy` directory.
