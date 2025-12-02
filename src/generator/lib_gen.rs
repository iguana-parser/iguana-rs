pub fn generate() -> String {
    r#"
pub mod parser;
pub mod parse_tree;
pub mod scanner;
    "#
    .trim()
    .to_owned()
}
