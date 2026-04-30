use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbols {
    pub nonterminals: Vec<String>,
    pub terminals: Vec<String>,
    pub slots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParseResult {
    Success(ParseSuccess),
    Failure(ParseFailure),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseSuccess {
    pub parse_ms: u64,
    pub tree_construction_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseFailure {
    pub line: u32,
    pub column: u32,
    pub message: String,
}
