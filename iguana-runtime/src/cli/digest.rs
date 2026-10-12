use std::{
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

use crate::{
    arena::Arena,
    ids::NonterminalId,
    input::Input,
    parse_tree::{is_ambiguous, tree_digest},
    parser::{GLLResult, Parser},
};

use super::{args::Args, collect_files, start_nonterminal_id, start_nonterminal_name};

/// Runs `--digest`: parses every file under `--dir` and writes one line per
/// file, the status, a digest of its parse tree, and the path, separated by
/// tabs. The status is `ok`, `amb`, `error` or `ioerror`, and a file with no
/// tree has `-` for its digest. An output file that already exists is not
/// overwritten.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args) -> io::Result<()> {
    let dir = args.dir.as_deref().expect("--digest requires --dir");
    let out_path = args.digest.as_deref().expect("--digest is set");
    let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
    let mut files = Vec::new();
    collect_files(dir, args.ext.as_deref(), &mut files)?;
    files.sort();

    for file in &files {
        let Some(text) = file.to_str() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{} is not valid UTF-8 and cannot be recorded",
                    file.display()
                ),
            ));
        };
        if let Some(bad) = text.chars().find(|c| matches!(c, '\t' | '\n' | '\r')) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{} holds {:?}, which the digest output cannot record",
                    file.display(),
                    bad
                ),
            ));
        }
    }

    // Written as the run goes, so an interrupted run leaves what it has
    // written so far. `create_new` rather than `create` to ensure that a new
    // run writes to an empty file.
    let mut out = BufWriter::new(fs::File::create_new(out_path).map_err(|error| {
        io::Error::new(error.kind(), format!("{}: {error}", out_path.display()))
    })?);
    let mut tree_arena = Arena::new();
    let mut parser_arena = Arena::new();
    for path in &files {
        let (status, digest) = digest_file::<P>(
            path,
            start_nonterminal_id,
            &mut tree_arena,
            &mut parser_arena,
        );
        match digest {
            Some(digest) => writeln!(out, "{}\t{:016x}\t{}", status, digest, path.display())?,
            None => writeln!(out, "{}\t-\t{}", status, path.display())?,
        }
    }
    out.flush()
}

/// Parses one file and returns its status and the digest of its parse tree.
fn digest_file<'i, 'arena, P: Parser<'i, 'arena>>(
    path: &Path,
    start_nonterminal_id: NonterminalId,
    tree_arena: &mut Arena,
    parser_arena: &mut Arena,
) -> (&'static str, Option<u64>) {
    let Ok(text) = fs::read_to_string(path) else {
        return ("ioerror", None);
    };
    let input = Input::from(text);
    let mut parser = P::ConcreteParser::<'_, '_>::new(&input, parser_arena);
    let result = match parser.run(start_nonterminal_id) {
        GLLResult::Failure(_) => ("error", None),
        GLLResult::Success(success) => {
            let ambiguous = is_ambiguous(&parser, success.sppf_node_id);
            let tree = parser.build_tree(success.sppf_node_id, tree_arena);
            let status = if ambiguous { "amb" } else { "ok" };
            (status, Some(tree_digest(tree)))
        }
    };
    drop(parser);
    parser_arena.reset();
    tree_arena.reset();
    result
}
