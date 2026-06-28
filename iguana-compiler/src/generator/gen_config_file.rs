use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::generator::GenConfig;

/// The persisted generation config: a `gen.toml` next to the grammar that holds
/// the generation knobs so `iguana generate` and the test infrastructure do not
/// need to re-pass them on every run. Every field is optional; an absent field
/// falls through to the next layer, which is the explicit CLI flag or the
/// built-in default.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenConfigFile {
    pub ll1: Option<bool>,
    pub match_memo: Option<bool>,
    #[serde(rename = "unsafe")]
    pub unsafe_mode: Option<bool>,
    pub bin_name: Option<String>,
    pub runtime_path: Option<PathBuf>,
}

impl GenConfigFile {
    /// Load `gen.toml` from the grammar file's directory. A missing file yields
    /// an empty config (every field `None`); a malformed file is an error naming
    /// the path. A relative `runtime_path` is resolved against that directory so
    /// the file stays portable when committed beside the grammar.
    pub fn load(grammar_path: &Path) -> io::Result<Self> {
        let dir = grammar_path.parent().unwrap_or(Path::new("."));
        let path = dir.join("gen.toml");
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e),
        };
        let mut file: GenConfigFile = toml::from_str(&text)
            .map_err(|e| io::Error::other(format!("{}: {e}", path.display())))?;
        if let Some(runtime_path) = &file.runtime_path {
            if runtime_path.is_relative() {
                file.runtime_path = Some(dir.join(runtime_path));
            }
        }
        Ok(file)
    }
}

impl GenConfig {
    /// Layer the file's parser knobs onto this config: each field the file
    /// specifies overwrites the matching field; an absent field leaves it
    /// unchanged. Only the parser-generation knobs map onto `GenConfig`; the
    /// scaffold knobs `bin_name` and `runtime_path` are not stored here and are
    /// applied by the caller.
    pub fn apply_file(&mut self, file: &GenConfigFile) {
        if let Some(ll1) = file.ll1 {
            self.ll1_optimization = ll1;
        }
        if let Some(match_memo) = file.match_memo {
            self.match_memo = match_memo;
        }
        if let Some(unsafe_mode) = file.unsafe_mode {
            self.unsafe_mode = unsafe_mode;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_parses_to_all_none() {
        let file: GenConfigFile = toml::from_str("").unwrap();
        assert_eq!(file.ll1, None);
        assert_eq!(file.match_memo, None);
        assert_eq!(file.unsafe_mode, None);
        assert_eq!(file.bin_name, None);
        assert_eq!(file.runtime_path, None);
    }

    #[test]
    fn unsafe_key_maps_to_unsafe_mode() {
        let file: GenConfigFile = toml::from_str("unsafe = true\n").unwrap();
        assert_eq!(file.unsafe_mode, Some(true));
    }

    #[test]
    fn unknown_key_is_rejected() {
        assert!(toml::from_str::<GenConfigFile>("optimize = true\n").is_err());
    }

    #[test]
    fn apply_file_overwrites_only_specified_fields() {
        let mut config = GenConfig::default();
        let file = GenConfigFile {
            ll1: Some(false),
            unsafe_mode: Some(true),
            ..GenConfigFile::default()
        };
        config.apply_file(&file);
        assert!(!config.ll1_optimization);
        assert!(config.unsafe_mode);
        // match_memo is unspecified, so it keeps the built-in default.
        assert!(config.match_memo);
    }
}
