use std::{fs, path::PathBuf, process::Command};

use iguana_compiler::{
    alternative, bind, call, cond, eq,
    generator::{GenConfig, generate_sources},
    grammar::{def::Grammar, first_follow::FirstFollowSets, symbols::Expr},
    grammar_def, id, lit, priority_level, ret, syntax_rule, tuple,
};

// The parser assigns tuple components independently:
//
//   Pair = "a" return (1, 2)
//   Triple = "a" return (-8192, 8191, 31)
//   Value(n) = return n
//   S = (x, y)=Pair x=Value(7) [x == 7] [y == 2]
//
// The cases below also rebind both components, repeat a name in one pattern,
// and shadow a parameter. In (x, x)=Pair, the second component shadows the first.
// Replay calls Pair from two alternatives at the same
// input position. One edge receives a normal return and the other reuses the
// saved result. Both must bind the components in their own caller environment.
#[test]
fn generated_bindings_preserve_independent_values_and_shadowing() {
    let grammar: Grammar = grammar_def!("Bindings", syntax: [
        syntax_rule!("Pair" => priority_level!(alternative!(
            lit!("a"), ret!(expr tuple!(1, 2))))),
        syntax_rule!("Triple" => priority_level!(alternative!(
            lit!("a"), ret!(expr tuple!(-8192, 8191, 31))))),
        syntax_rule!("Value"("n": I32) => priority_level!(alternative!(
            ret!(expr Expr::Ref("n".into()))))),
        syntax_rule!("S" => priority_level!(
            alternative!(lit!("ordinary"),
                bind!("x", call!("Value", 1)), bind!("x", call!("Value", 7)),
                cond!(eq!("x", 7))),
            alternative!(lit!("component"),
                bind!(("x", "y"), id!("Pair")), bind!("x", call!("Value", 7)),
                cond!(eq!("x", 7)), cond!(eq!("y", 2))),
            alternative!(lit!("both"),
                bind!(("x", "y"), id!("Pair")),
                bind!("x", call!("Value", 7)), bind!("y", call!("Value", 8)),
                cond!(eq!("x", 7)), cond!(eq!("y", 8))),
            alternative!(lit!("repeat"),
                bind!(("x", "x"), id!("Pair")), cond!(eq!("x", 2))),
            alternative!(lit!("tuple"), bind!("x", call!("Value", 7)),
                bind!(("x", "y"), id!("Pair")), cond!(eq!("x", 1)), cond!(eq!("y", 2))),
            alternative!(lit!("triple"), bind!(("x", "y", "z"), id!("Triple")),
                cond!(eq!("x", -8192)), bind!("x", call!("Value", 7)),
                cond!(eq!("x", 7)), cond!(eq!("y", 8191)), cond!(eq!("z", 31))),
            alternative!(lit!("argument"), bind!("x", call!("Value", 7)),
                bind!("x", call!("Value", "x")), cond!(eq!("x", 7)))
        )),
        syntax_rule!("Parameter"("x": I32) => priority_level!(alternative!(
            bind!("x", call!("Value", 7)), cond!(eq!("x", 7))))),
        syntax_rule!("Replay" => priority_level!(
            alternative!(bind!("keep", call!("Value", 10)),
                bind!(("x", "y"), id!("Pair")), lit!("b"),
                cond!(eq!("x", 1)), cond!(eq!("y", 2)), cond!(eq!("keep", 10))),
            alternative!(bind!("keep", call!("Value", 20)),
                bind!(("u", "v"), id!("Pair")), lit!("c"),
                cond!(eq!("u", 1)), cond!(eq!("v", 2)), cond!(eq!("keep", 20)))
        ))
    ])
    .try_into()
    .unwrap();

    let ff = FirstFollowSets::new(&grammar);
    assert!(!ff.is_ll1(grammar.nonterminal("Pair").unwrap()));
    assert!(!ff.is_ll1(grammar.nonterminal("S").unwrap()));

    let dir = std::env::temp_dir().join(format!("iguana-binding-values-{}", std::process::id()));
    fs::create_dir_all(dir.join("tests")).unwrap();
    let runtime = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../iguana-runtime");
    fs::write(
        dir.join("Cargo.toml"),
        format!(
            r#"
[package]
name = "binding_values"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
iguana-runtime = {{ path = "{}" }}
rustc-hash = "2.1.1"
serde_json = "1.0"
[features]
debug-trace = ["iguana-runtime/debug-trace"]
instrument = ["iguana-runtime/instrument"]
"#,
            runtime.display()
        ),
    )
    .unwrap();
    fs::write(
        dir.join("tests/values.rs"),
        r#"
use binding_values::{grammar_data::*, parser::BindingsParser};
use iguana_runtime::{arena::Arena, input::Input, parser::{GLLResult, Parser}};

#[test]
fn values_survive_rebinding_and_shared_returns() {
    for (start, text) in [
        (START_S, "ordinary"), (START_S, "componenta"),
        (START_S, "botha"), (START_S, "repeata"),
        (START_S, "tuplea"), (START_S, "triplea"),
        (START_S, "argument"), (START_PARAMETER, ""),
        (START_REPLAY, "ab"), (START_REPLAY, "ac"),
    ] {
        let input = Input::from(text);
        let arena = Arena::new();
        let mut parser = BindingsParser::new(&input, start, &arena);
        assert!(matches!(parser.run(), GLLResult::Success(_)), "{text}");
    }
}
"#,
    )
    .unwrap();

    // A separate target directory avoids taking the outer Cargo invocation's
    // lock. The configurations share their dependency build. Safe mode uses the
    // default LL(1) optimization; unsafe mode also checks generation without it.
    let target = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("binding-values-target");
    for (unsafe_mode, ll1_optimization) in [(false, true), (true, false)] {
        generate_sources(
            &grammar,
            &dir,
            GenConfig {
                unsafe_mode,
                ll1_optimization,
                cli: false,
                ..GenConfig::default()
            },
        )
        .unwrap();
        let output = Command::new(env!("CARGO"))
            .args(["test", "--offline", "--quiet", "--test", "values"])
            .arg("--manifest-path")
            .arg(dir.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", &target)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "unsafe={unsafe_mode}, ll1={ll1_optimization}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(dir).unwrap();
}
