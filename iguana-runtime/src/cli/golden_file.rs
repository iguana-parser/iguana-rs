use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::{
    arena::Arena,
    grammar::Grammar,
    input::Input,
    parse_tree::{DisplayOptions, to_sexpr_with},
    parser::{GLLResult, Parser},
};

use super::{
    Color, args::Args, collect_files, print_status, start_nonterminal_id, start_nonterminal_name,
};

/// Runs `--check-sexpr` or `--regenerate-sexpr` over the input file or the
/// `--dir` files, and exits with failure when a check did not pass.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args) -> io::Result<()> {
    let mode = if args.regenerate_sexpr {
        GoldenMode::Regenerate
    } else {
        GoldenMode::Check
    };
    let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
    if args.write_parse_tree.is_some()
        || args.write_sppf.is_some()
        || args.write_gss.is_some()
        || args.write_gss_nodes.is_some()
        || args.write_result.is_some()
        || args.write_stats.is_some()
        || args.trace.is_some()
    {
        eprintln!(
            "Warning: output flags (--write-parse-tree, --write-sppf, --write-gss, --write-gss-nodes, --write-result, --write-stats, --trace) are ignored in golden-file mode."
        );
    }
    let mut inputs = Vec::new();
    if let Some(dir) = args.dir.as_ref() {
        let ext = args.ext.as_deref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--ext is required with --dir for golden testing (so .sexpr golden files are not parsed as inputs)",
            )
        })?;
        collect_files(dir, Some(ext), &mut inputs)?;
        inputs.sort();
    } else if let Some(file) = args.file.as_ref() {
        inputs.push(file.clone());
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "golden-file testing requires an input file or --dir",
        ));
    }
    let display_options = DisplayOptions {
        show_layout: args.show_layout,
        show_empty: args.show_empty,
        show_wrappers: args.show_wrappers,
    };
    let passed = run_golden(
        mode,
        inputs,
        args.dir.as_deref(),
        args.quiet,
        args.full_diff,
        |path| {
            let input = Input::try_from(path)?;
            let tree_arena = Arena::new();
            let parser_arena = Arena::new();
            let mut parser = P::ConcreteParser::<'_, '_>::new(&input, &parser_arena);
            let content = match parser.run(start_nonterminal_id) {
                GLLResult::Success(success) => {
                    let tree = parser.build_tree(success.sppf_node_id, &tree_arena);
                    to_sexpr_with(tree, P::Grammar::LAYOUT_NAME, display_options)
                }
                GLLResult::Failure(error) => {
                    format!("{}\n", parser.to_parse_error(&error).render(&input))
                }
            };
            Ok(content)
        },
    )?;
    if !passed {
        std::process::exit(1);
    }
    Ok(())
}

/// Whether the golden-file harness checks parser output against existing
/// golden files or rewrites them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum GoldenMode {
    Check,
    Regenerate,
}

/// Runs golden-file testing over `inputs`. For each input file `X.<ext>`, the
/// golden file is `X.sexpr` in the same directory. `golden_content` produces the text
/// to compare or write for one input: the parse-tree s-expression on success,
/// or the caret-annotated `Parse error at line N, column M: <message>` render
/// on failure. It only
/// fails (`Err`) when the input itself is unreadable.
///
/// In `Check` mode each file is `OK`, `DIFF`, `MISS`, or `ERR`, and the run
/// passes only if every file is `OK`. In `Regenerate` mode each file is
/// `WRITE`, `unchanged`, or `ERR`, and the run passes unless an `ERR` occurred.
/// Returns whether the run passed so the caller can set the process exit code.
///
/// `quiet` suppresses the per-file status lines and the summary but never the
/// diff on a `DIFF` (whose header names the file). `full_diff` disables the
/// 200-line diff truncation. Paths are shown relative to `root` (the `--dir`
/// directory) when given, so the output reads cleanly; `None` shows them as
/// given (single-file mode).
fn run_golden(
    mode: GoldenMode,
    inputs: Vec<PathBuf>,
    root: Option<&Path>,
    quiet: bool,
    full_diff: bool,
    mut golden_content: impl FnMut(&Path) -> io::Result<String>,
) -> io::Result<bool> {
    let color = Color::for_stdout();
    let start = Instant::now();

    if inputs.is_empty() {
        eprintln!("Warning: no input files to check.");
    }

    let mut ok = 0usize;
    let mut diff = 0usize;
    let mut miss = 0usize;
    let mut written = 0usize;
    let mut unchanged = 0usize;
    let mut errs = 0usize;

    for input in &inputs {
        let golden_path = input.with_extension("sexpr");
        let shown_input = rel(input, root).display().to_string();
        let shown_golden = rel(&golden_path, root).display().to_string();
        let actual = match golden_content(input) {
            Ok(actual) => actual,
            Err(e) => {
                errs += 1;
                if !quiet {
                    print_status(&color, "ERR", false, &format!("{shown_input}: {e}"));
                }
                continue;
            }
        };

        match mode {
            GoldenMode::Check => match read_golden(&golden_path) {
                Ok(Some(golden)) if golden == actual => {
                    ok += 1;
                    if !quiet {
                        print_status(&color, "OK", true, &shown_input);
                    }
                }
                Ok(Some(golden)) => {
                    diff += 1;
                    if !quiet {
                        print_status(&color, "DIFF", false, &shown_input);
                    }
                    let body = unified_diff(&golden, &actual, &shown_golden, full_diff);
                    print_indented(&body);
                }
                Ok(None) => {
                    miss += 1;
                    if !quiet {
                        print_status(&color, "MISS", false, &shown_input);
                        println!("  (no golden file; regenerate with --regenerate-sexpr)");
                    }
                }
                Err(e) => {
                    errs += 1;
                    if !quiet {
                        print_status(&color, "ERR", false, &format!("{shown_golden}: {e}"));
                    }
                }
            },
            GoldenMode::Regenerate => {
                // An unchanged golden file gets no per-file line; only writes and
                // errors are worth reporting (the count lands in the summary).
                let same = matches!(read_golden(&golden_path), Ok(Some(ref g)) if *g == actual);
                if same {
                    unchanged += 1;
                } else if let Err(e) = fs::write(&golden_path, &actual) {
                    errs += 1;
                    if !quiet {
                        print_status(&color, "ERR", false, &format!("{shown_golden}: {e}"));
                    }
                } else {
                    written += 1;
                    if !quiet {
                        print_status(&color, "WRITE", true, &shown_input);
                    }
                }
            }
        }
    }

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    if !quiet {
        println!();
        match mode {
            GoldenMode::Check => println!(
                "Checked {} files: {} OK, {} DIFF, {} MISS, {} errors",
                inputs.len(),
                ok,
                diff,
                miss,
                errs
            ),
            GoldenMode::Regenerate => println!(
                "Regenerated {} files: {} written, {} unchanged, {} errors",
                inputs.len(),
                written,
                unchanged,
                errs
            ),
        }
        println!("Elapsed {:.0} ms", elapsed_ms);
    }

    let passed = match mode {
        GoldenMode::Check => diff == 0 && miss == 0 && errs == 0,
        GoldenMode::Regenerate => errs == 0,
    };
    Ok(passed)
}

/// Strips `root` from `path` for display, falling back to the full path when
/// `path` is not under `root` or `root` is `None` (single-file mode).
fn rel<'a>(path: &'a Path, root: Option<&Path>) -> &'a Path {
    root.and_then(|r| path.strip_prefix(r).ok()).unwrap_or(path)
}

/// Reads a golden file, normalizing CRLF to LF so golden files written on Windows
/// still compare equal. Returns `Ok(None)` when the golden file does not exist (a
/// `MISS`), and `Err` only on a real I/O failure.
fn read_golden(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text.replace("\r\n", "\n"))),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn print_indented(body: &str) {
    for line in body.lines() {
        println!("  {}", line);
    }
}

/// One line of a unified diff: a shared line, a line only in the golden file,
/// or a line only in the actual output.
enum Edit<'a> {
    Eq(&'a str),
    Del(&'a str),
    Ins(&'a str),
}

/// Renders a unified diff of `golden` against `actual` with three lines of
/// context and `--- name (golden)` / `+++ name (actual)` headers. Lines are
/// split on `\n`, so a trailing-newline difference shows as a changed final
/// line. Unless `full_diff`, the body is truncated past 200 lines with a
/// `... (N more lines)` marker.
fn unified_diff(golden: &str, actual: &str, name: &str, full_diff: bool) -> String {
    let a: Vec<&str> = golden.split('\n').collect();
    let b: Vec<&str> = actual.split('\n').collect();
    let edits = diff_lines(&a, &b);

    let mut out = String::new();
    out.push_str(&format!("--- {name} (golden)\n"));
    out.push_str(&format!("+++ {name} (actual)\n"));
    out.push_str(&render_hunks(&edits));

    if !full_diff {
        // Keep the two headers plus 200 body lines before truncating. `out` is
        // newline-terminated, so the line count is its number of newlines.
        const LIMIT: usize = 202;
        let line_count = out.matches('\n').count();
        if line_count > LIMIT {
            let kept: String = out.split_inclusive('\n').take(LIMIT).collect();
            let remaining = line_count - LIMIT;
            return format!("{kept}... ({remaining} more lines)\n");
        }
    }
    out
}

/// Longest-common-subsequence line diff. Golden files are small, so the
/// quadratic table is fine; an oversized input falls back to deleting the whole
/// golden file and inserting the whole actual output rather than allocating a
/// huge table.
fn diff_lines<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Edit<'a>> {
    let (n, m) = (a.len(), b.len());
    if n.saturating_mul(m) > 4_000_000 {
        let mut edits = Vec::with_capacity(n + m);
        edits.extend(a.iter().map(|l| Edit::Del(l)));
        edits.extend(b.iter().map(|l| Edit::Ins(l)));
        return edits;
    }

    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut edits = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            edits.push(Edit::Eq(a[i]));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            edits.push(Edit::Del(a[i]));
            i += 1;
        } else {
            edits.push(Edit::Ins(b[j]));
            j += 1;
        }
    }
    edits.extend(a[i..].iter().map(|l| Edit::Del(l)));
    edits.extend(b[j..].iter().map(|l| Edit::Ins(l)));
    edits
}

/// Groups an edit script into unified-diff hunks: each changed run is padded
/// with up to three lines of context, and two runs separated by no more than
/// six shared lines merge into one hunk.
fn render_hunks(edits: &[Edit]) -> String {
    const CONTEXT: usize = 3;

    // Count of lines consumed in each side before edit `i`; the hunk header
    // adds 1 to turn these into 1-based line numbers.
    let mut old_pos = vec![0usize; edits.len() + 1];
    let mut new_pos = vec![0usize; edits.len() + 1];
    for (i, e) in edits.iter().enumerate() {
        let (d_old, d_new) = match e {
            Edit::Eq(_) => (1, 1),
            Edit::Del(_) => (1, 0),
            Edit::Ins(_) => (0, 1),
        };
        old_pos[i + 1] = old_pos[i] + d_old;
        new_pos[i + 1] = new_pos[i] + d_new;
    }

    let changed: Vec<usize> = edits
        .iter()
        .enumerate()
        .filter(|(_, e)| !matches!(e, Edit::Eq(_)))
        .map(|(i, _)| i)
        .collect();
    if changed.is_empty() {
        return String::new();
    }

    // Merge adjacent changes whose gap of shared lines is within twice the
    // context, then expand each group by the context on both ends.
    let mut out = String::new();
    let mut group_start = changed[0];
    let mut group_end = changed[0];
    let mut groups = Vec::new();
    for &c in &changed[1..] {
        if c - group_end - 1 <= 2 * CONTEXT {
            group_end = c;
        } else {
            groups.push((group_start, group_end));
            group_start = c;
            group_end = c;
        }
    }
    groups.push((group_start, group_end));

    for (gs, ge) in groups {
        let start = gs.saturating_sub(CONTEXT);
        let end = (ge + 1 + CONTEXT).min(edits.len());
        let old_len = old_pos[end] - old_pos[start];
        let new_len = new_pos[end] - new_pos[start];
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_pos[start] + 1,
            old_len,
            new_pos[start] + 1,
            new_len
        ));
        for e in &edits[start..end] {
            match e {
                Edit::Eq(line) => out.push_str(&format!(" {line}\n")),
                Edit::Del(line) => out.push_str(&format!("-{line}\n")),
                Edit::Ins(line) => out.push_str(&format!("+{line}\n")),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_diff_one_changed_line() {
        let diff = unified_diff("a\nb\nc\n", "a\nB\nc\n", "g", false);
        assert_eq!(
            diff,
            "--- g (golden)\n\
             +++ g (actual)\n\
             @@ -1,4 +1,4 @@\n\
             \x20a\n\
             -b\n\
             +B\n\
             \x20c\n\
             \x20\n"
        );
    }

    #[test]
    fn unified_diff_flags_trailing_newline() {
        // The golden file ends with a newline, the actual output does not: the
        // golden file's trailing blank line is deleted, which flags the difference.
        let diff = unified_diff("(S)\n", "(S)", "g", false);
        assert_eq!(
            diff,
            "--- g (golden)\n\
             +++ g (actual)\n\
             @@ -1,2 +1,1 @@\n\
             \x20(S)\n\
             -\n"
        );
    }

    #[test]
    fn unified_diff_truncates_past_limit() {
        let golden: String = (0..300).map(|i| format!("g{i}\n")).collect();
        let actual: String = (0..300).map(|i| format!("a{i}\n")).collect();
        let diff = unified_diff(&golden, &actual, "g", false);
        assert!(diff.contains("more lines)"));
        assert!(diff.lines().count() <= 203);

        let full = unified_diff(&golden, &actual, "g", true);
        assert!(!full.contains("more lines)"));
    }
}
