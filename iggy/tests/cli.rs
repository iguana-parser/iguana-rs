use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_iggy"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(args)
        .output()
        .unwrap()
}

const PARSING_MODES: &[&[&str]] = &[
    &["iggy.iggy"],
    &["--dir", "."],
    &["--repl"],
    &["--benchmark", "iggy.iggy"],
    &["--benchmark", "--dir", "."],
    &["--check-sexpr", "iggy.iggy"],
    &["--regenerate-sexpr", "iggy.iggy"],
];

#[test]
fn missing_start_is_a_readable_usage_error() {
    for args in PARSING_MODES {
        let output = run(args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "Error: --start is required for parsing\n",
            "{args:?}"
        );
    }
}

#[test]
fn unknown_start_points_to_list_nonterminals() {
    let listing = run(&["--list-nonterminals"]);
    assert!(listing.status.success());
    assert!(listing.stderr.is_empty());
    let names = String::from_utf8(listing.stdout).unwrap();
    assert!(names.starts_with("Grammar\nRule\n"));
    assert!(!names.contains("StartGrammar"));

    // The error points to --list-nonterminals for both a typo and a generated wrapper name.
    for name in ["Gramar", "StartGrammar"] {
        for mode in PARSING_MODES {
            let mut args = mode.to_vec();
            args.extend(["--start", name]);
            let output = run(&args);
            assert_eq!(output.status.code(), Some(1), "{args:?}");
            assert!(output.stdout.is_empty(), "{args:?}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                format!(
                    "Error: Unknown start nonterminal: '{name}'. Use --list-nonterminals to see the valid names.\n"
                ),
                "{args:?}"
            );
        }
    }
}
