//! Exercise the archives before their versions exist on crates.io. Cargo's
//! directory source supplies the candidate packages and locked dependencies;
//! manifests, sources and packaged lockfiles remain unchanged.

use super::*;
use std::collections::BTreeMap;

pub(super) fn run(
    root: &Path,
    version: &str,
    commit: &str,
    packages: &Path,
    directory: &Path,
) -> io::Result<()> {
    parse_version(version)?;
    let root = root.canonicalize()?;
    require(
        valid_commit(commit) && output(&root, "git", &["rev-parse", "HEAD"])? == commit,
        "Package smoke test must use the candidate commit",
    )?;
    check(&root, version)?;
    clean(&root)?;
    let packages = packages.canonicalize()?;
    // Refuse to reuse caches or write inside the candidate checkout.
    let parent = directory
        .parent()
        .ok_or_else(|| io::Error::other("Missing temporary parent directory"))?
        .canonicalize()?;
    require(
        !parent.starts_with(&root),
        "Smoke directory must be outside the checkout",
    )?;
    let directory = parent.join(
        directory
            .file_name()
            .ok_or_else(|| io::Error::other("Invalid smoke directory"))?,
    );
    fs::create_dir(&directory)?;
    let vendor = directory.join("vendor");
    // Downloads are allowed here only. The actual installations and parser
    // build use a fresh Cargo home, fresh target directory and offline mode.
    let config = output(
        &root,
        "cargo",
        &[
            "vendor",
            "--locked",
            "--versioned-dirs",
            vendor.to_str().unwrap(),
        ],
    )?;
    for package in PACKAGES {
        let name = format!("{package}-{version}");
        let archive = packages.join(format!("{name}.crate"));
        require(
            archive.is_file(),
            format!("Missing package: {}", archive.display()),
        )?;
        let vcs = output(
            &directory,
            "tar",
            &[
                "-xOzf",
                archive.to_str().unwrap(),
                &format!("{name}/.cargo_vcs_info.json"),
            ],
        )?;
        check_provenance(&vcs, commit)?;
        let contents = output(&directory, "tar", &["-tzf", archive.to_str().unwrap()])?;
        require(
            contents.lines().all(|entry| {
                entry.starts_with(&format!("{name}/"))
                    && Path::new(entry)
                        .components()
                        .all(|part| matches!(part, std::path::Component::Normal(_)))
            }),
            format!("Unexpected paths in {name}.crate"),
        )?;
        require(
            !vendor.join(&name).exists(),
            "Candidate already exists in vendored dependencies",
        )?;
        execute(
            &directory,
            "tar",
            &[
                "-xzf",
                archive.to_str().unwrap(),
                "-C",
                vendor.to_str().unwrap(),
            ],
        )?;
        let unpacked = vendor.join(&name);
        validate_manifest(
            &fs::read_to_string(unpacked.join("Cargo.toml"))?,
            package,
            version,
        )?;
        let mut files = BTreeMap::new();
        checksums(&unpacked, &unpacked, &mut files)?;
        fs::write(
            unpacked.join(".cargo-checksum.json"),
            json!({"package": sha256(&archive)?, "files": files}).to_string(),
        )?;
    }
    let home = directory.join("cargo-home");
    fs::create_dir(&home)?;
    fs::write(home.join("config.toml"), config)?;
    let sandbox = Sandbox { directory, home };
    let tools = sandbox.directory.join("tools");
    for package in ["iguana", "iguana-lsp"] {
        sandbox.execute(
            &sandbox.directory,
            "cargo",
            &[
                "install",
                package,
                "--version",
                &format!("={version}"),
                "--locked",
                "--offline",
                "--root",
                tools.to_str().unwrap(),
            ],
        )?;
    }
    let iguana = tools.join("bin/iguana");
    let iguana = iguana.to_str().unwrap();
    require(
        sandbox.output(&sandbox.directory, iguana, &["--version"])? == format!("iguana {version}"),
        "Installed compiler has the wrong version",
    )?;
    sandbox.execute(&sandbox.directory, iguana, &["new", "release_smoke"])?;
    let project = sandbox.directory.join("release_smoke");
    sandbox.execute(&project, iguana, &["generate"])?;
    let metadata = sandbox.output(
        &project,
        "cargo",
        &["metadata", "--offline", "--format-version", "1"],
    )?;
    validate_runtime(
        &serde_json::from_str(&metadata).map_err(io::Error::other)?,
        version,
        &vendor,
    )?;
    fs::write(project.join("hello.txt"), "hello")?;
    fs::write(project.join("hello.sexpr"), "(S \"hello\")\n")?;
    sandbox.execute(&project, "cargo", &["build", "--locked", "--offline"])?;
    let parser = sandbox.directory.join("target/debug/release_smoke");
    sandbox.execute(
        &project,
        parser.to_str().unwrap(),
        &["hello.txt", "--start", "S", "--check-sexpr"],
    )?;
    clean(&root)?;
    println!("Packaged installation and generated parser passed for {version} at {commit}.");
    Ok(())
}

fn validate_manifest(text: &str, package: &str, version: &str) -> io::Result<()> {
    let manifest = document(text)?;
    require(
        manifest["package"]["name"].as_str() == Some(package)
            && manifest["package"]["version"].as_str() == Some(version),
        format!("Unexpected package identity in {package} archive"),
    )?;
    fn dependencies(table: &dyn toml_edit::TableLike, version: &str) -> io::Result<()> {
        for kind in ["dependencies", "build-dependencies", "dev-dependencies"] {
            if let Some(deps) = table.get(kind).and_then(Item::as_table_like) {
                for (key, dependency) in deps.iter() {
                    let name = dependency
                        .get("package")
                        .and_then(Item::as_str)
                        .unwrap_or(key);
                    require(
                        dependency.get("path").is_none() && dependency.get("git").is_none(),
                        "Packaged dependencies must use the registry",
                    )?;
                    if PACKAGES.contains(&name) {
                        let req = dependency
                            .as_str()
                            .or_else(|| dependency.get("version").and_then(Item::as_str));
                        require(
                            req == Some(format!("={version}").as_str()),
                            format!("Unpinned packaged dependency: {name}"),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
    dependencies(manifest.as_table(), version)?;
    if let Some(targets) = manifest.get("target").and_then(Item::as_table_like) {
        for (_, target) in targets.iter() {
            dependencies(
                target
                    .as_table_like()
                    .ok_or_else(|| io::Error::other("Invalid target dependencies"))?,
                version,
            )?;
        }
    }
    Ok(())
}

fn sha256(path: &Path) -> io::Result<String> {
    let text = output(
        path.parent().unwrap(),
        "shasum",
        &["-a", "256", "--", path.to_str().unwrap()],
    )?;
    let hash = text.split_whitespace().next().unwrap_or_default();
    require(
        hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid SHA-256 output",
    )?;
    Ok(hash.to_owned())
}

fn checksums(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<String, String>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            checksums(root, &path, files)?;
        } else {
            require(
                entry.file_type()?.is_file(),
                "Unexpected link in packaged files",
            )?;
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            require(
                name != ".cargo-checksum.json",
                "Archive already contains directory-source checksums",
            )?;
            files.insert(name, sha256(&path)?);
        }
    }
    Ok(())
}

fn validate_runtime(metadata: &Value, version: &str, vendor: &Path) -> io::Result<()> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| io::Error::other("Missing packages"))?;
    let runtime: Vec<_> = packages
        .iter()
        .filter(|p| p["name"] == "iguana-runtime")
        .collect();
    require(
        runtime.len() == 1,
        "Expected exactly one generated parser runtime",
    )?;
    let expected = vendor.join(format!("iguana-runtime-{version}/Cargo.toml"));
    require(
        runtime[0]["version"] == version
            && runtime[0]["source"]
                .as_str()
                .is_some_and(|s| s.starts_with("registry+"))
            && runtime[0]["manifest_path"].as_str() == expected.to_str(),
        "Generated parser must use the packaged candidate runtime",
    )?;
    require(
        packages
            .iter()
            .filter(|p| p["name"] == "release_smoke")
            .any(|p| {
                p["dependencies"].as_array().is_some_and(|deps| {
                    deps.iter().any(|d| {
                        d["name"] == "iguana-runtime"
                            && d["req"] == format!("={version}")
                            && d["path"].is_null()
                    })
                })
            }),
        "Generated manifest must pin the exact registry runtime",
    )
}

struct Sandbox {
    directory: PathBuf,
    home: PathBuf,
}

impl Sandbox {
    fn command(&self, cwd: &Path, program: &str, args: &[&str]) -> Process {
        let mut command = Process::new(program);
        command
            .args(args)
            .current_dir(cwd)
            .env("CARGO_HOME", &self.home)
            .env("CARGO_TARGET_DIR", self.directory.join("target"))
            .env("CARGO_NET_OFFLINE", "true")
            .env_remove("CARGO_REGISTRY_TOKEN")
            .env_remove("CARGO_REGISTRIES_CRATES_IO_TOKEN");
        command
    }

    fn execute(&self, cwd: &Path, program: &str, args: &[&str]) -> io::Result<()> {
        require(
            self.command(cwd, program, args).status()?.success(),
            format!("Package smoke command failed: {program} {args:?}"),
        )
    }

    fn output(&self, cwd: &Path, program: &str, args: &[&str]) -> io::Result<String> {
        let result = self.command(cwd, program, args).output()?;
        require(
            result.status.success(),
            format!(
                "Package smoke command failed: {program} {args:?}\n{}",
                String::from_utf8_lossy(&result.stderr)
            ),
        )?;
        String::from_utf8(result.stdout)
            .map(|s| s.trim().to_owned())
            .map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_manifests_reject_wrong_identity_paths_and_loose_internal_pins() {
        let valid = "[package]\nname = 'iguana'\nversion = '0.1.0-alpha.2'\n[target.'cfg(unix)'.dependencies.iggy]\npackage = 'iguana-iggy'\nversion = '=0.1.0-alpha.2'\n";
        assert!(validate_manifest(valid, "iguana", "0.1.0-alpha.2").is_ok());
        for invalid in [
            valid.replace("name = 'iguana'", "name = 'other'"),
            valid.replace("version = '0.1.0-alpha.2'", "version = '0.1.0-alpha.1'"),
            valid.replace("'=0.1.0-alpha.2'", "'^0.1.0-alpha.2'"),
            format!("{valid}path = '../iggy'\n"),
        ] {
            assert!(validate_manifest(&invalid, "iguana", "0.1.0-alpha.2").is_err());
        }
    }

    #[test]
    fn generated_runtime_must_come_from_the_candidate_archive() {
        let vendor = Path::new("/tmp/vendor");
        let valid = json!({"packages": [
            {"name": "iguana-runtime", "version": "0.1.0-alpha.2", "source": "registry+https://github.com/rust-lang/crates.io-index", "manifest_path": "/tmp/vendor/iguana-runtime-0.1.0-alpha.2/Cargo.toml"},
            {"name": "release_smoke", "dependencies": [{"name": "iguana-runtime", "req": "=0.1.0-alpha.2"}]}
        ]});
        assert!(validate_runtime(&valid, "0.1.0-alpha.2", vendor).is_ok());
        for (field, value) in [
            ("version", json!("0.1.0-alpha.1")),
            ("source", Value::Null),
            (
                "manifest_path",
                json!("/checkout/iguana-runtime/Cargo.toml"),
            ),
        ] {
            let mut invalid = valid.clone();
            invalid["packages"][0][field] = value;
            assert!(validate_runtime(&invalid, "0.1.0-alpha.2", vendor).is_err());
        }
        let mut invalid = valid;
        invalid["packages"][1]["dependencies"][0]["req"] = json!("^0.1.0-alpha.2");
        assert!(validate_runtime(&invalid, "0.1.0-alpha.2", vendor).is_err());
    }
}
