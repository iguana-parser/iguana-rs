use std::{
    env, fs,
    hint::black_box,
    io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    arena::Arena,
    ids::NonterminalId,
    input::Input,
    parser::{GLLResult, Parser},
};

use super::{
    args::Args, benchmark::file_list, collect_files, start_nonterminal_id, start_nonterminal_name,
};

/// The environment variable that marks the parser process samply records.
const RECORDED: &str = "IGUANA_CPU_PROFILE_RECORDED";

/// Writes a CPU profile of the parser to `output` (the `--cpu-profile` flag).
/// The parser runs itself again under `samply record`, and the recorded run
/// parses the input:
/// - a single file (the positional argument) `--iters` times, 1 by default,
///   after reading the file once;
/// - every file in a directory (`--dir`) or in a file list (`--files-from`)
///   once.
///
/// A parse that succeeds builds its parse tree, also when the parse is
/// ambiguous. The profile includes reading and decoding the files. The
/// arguments and the files to parse are checked before recording, so an
/// invalid command does not leave an empty profile.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args, output: &Path) -> io::Result<()> {
    if args.iters.is_some() && args.file.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--iters applies to a single file only",
        ));
    }
    if args.iters == Some(0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--iters must be at least 1",
        ));
    }
    let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
    let batch = match args.file.as_ref() {
        Some(file) => {
            fs::metadata(file)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", file.display(), e)))?;
            Vec::new()
        }
        None => batch_files(args)?,
    };
    if env::var_os(RECORDED).is_none() {
        return record(output);
    }
    match args.file.as_ref() {
        Some(file) => parse_file::<P>(file, args.iters.unwrap_or(1), start_nonterminal_id),
        None => parse_files::<P>(&batch, start_nonterminal_id),
    }
}

/// Runs this process again, with the same arguments, under `samply record`.
fn record(output: &Path) -> io::Result<()> {
    let status = Command::new("samply")
        .args(["record", "--save-only", "-o"])
        .arg(output)
        .arg("--")
        .arg(env::current_exe()?)
        .args(env::args_os().skip(1))
        .env(RECORDED, "1")
        .status()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    e.kind(),
                    "samply not found on PATH. Install it with `cargo install --locked samply`.",
                )
            } else {
                e
            }
        })?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "recording the CPU profile failed ({status})"
        )));
    }
    eprintln!(
        "CPU profile written to {}. Open it with `samply load {}`.",
        output.display(),
        shell_word(&output.display().to_string()),
    );
    Ok(())
}

/// `text` as one word of a POSIX shell command. A text that has only letters,
/// digits, and `/._-` is unchanged. Any other text is put in single quotes,
/// with each single quote in it written as `'\''`.
fn shell_word(text: &str) -> String {
    let plain = !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c));
    if plain {
        text.to_string()
    } else {
        format!("'{}'", text.replace('\'', r"'\''"))
    }
}

/// Reads `file` once and parses it `iters` times.
fn parse_file<'i, 'arena, P: Parser<'i, 'arena>>(
    file: &Path,
    iters: u32,
    start_nonterminal_id: NonterminalId,
) -> io::Result<()> {
    let input = Input::try_from(file)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", file.display(), e)))?;
    let mut tree_arena = Arena::new();
    let mut parser_arena = Arena::new();
    for _ in 0..iters {
        parse::<P>(
            &input,
            start_nonterminal_id,
            &mut tree_arena,
            &mut parser_arena,
        );
    }
    Ok(())
}

/// Reads and parses each of `files` once. A file that cannot be read is
/// reported and skipped, and it is an error when no file can be read.
fn parse_files<'i, 'arena, P: Parser<'i, 'arena>>(
    files: &[PathBuf],
    start_nonterminal_id: NonterminalId,
) -> io::Result<()> {
    let mut tree_arena = Arena::new();
    let mut parser_arena = Arena::new();
    let mut parsed = 0;
    for path in files {
        match Input::try_from(path.as_path()) {
            Ok(input) => {
                parse::<P>(
                    &input,
                    start_nonterminal_id,
                    &mut tree_arena,
                    &mut parser_arena,
                );
                parsed += 1;
            }
            Err(e) => eprintln!("Skipped {}: {}", path.display(), e),
        }
    }
    if parsed == 0 {
        return Err(io::Error::other("none of the files could be read"));
    }
    Ok(())
}

/// The files of `--files-from`, in order, or of `--dir`, sorted. A list
/// without files is an error, since its profile would be empty.
fn batch_files(args: &Args) -> io::Result<Vec<PathBuf>> {
    let files = if let Some(list) = args.files_from.as_ref() {
        let text = fs::read_to_string(list)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", list.display(), e)))?;
        file_list(&text)
    } else if let Some(dir) = args.dir.as_ref() {
        let mut files = Vec::new();
        collect_files(dir, args.ext.as_deref(), &mut files)?;
        files.sort();
        files
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--cpu-profile requires an input file, --dir, or --files-from",
        ));
    };
    if files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no files to profile",
        ));
    }
    Ok(files)
}

/// Parses `input` and builds the parse tree when the parse succeeds. The
/// arenas are reset afterwards, so the next parse reuses their memory.
fn parse<'i, 'arena, P: Parser<'i, 'arena>>(
    input: &Input,
    start_nonterminal_id: NonterminalId,
    tree_arena: &mut Arena,
    parser_arena: &mut Arena,
) {
    let mut parser = P::ConcreteParser::<'_, '_>::new(input, parser_arena);
    if let GLLResult::Success(success) = parser.run(start_nonterminal_id) {
        black_box(parser.build_tree(success.sppf_node_id, tree_arena));
    }
    drop(parser);
    parser_arena.reset();
    tree_arena.reset();
}
