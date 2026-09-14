//! Local helpers for release.yml. GitHub operations and approval gates live in
//! the workflow; publishing is allowed only in that workflow's CI checkout.

use std::{
    collections::BTreeSet,
    env, fs, io,
    path::{Path, PathBuf},
    process::Command as Process,
};

use semver::Version;
use serde_json::{Value, json};
use toml_edit::{DocumentMut, Item};

mod smoke;

const PACKAGES: [&str; 5] = [
    "iguana-runtime",
    "iguana-iggy",
    "iguana-compiler",
    "iguana",
    "iguana-lsp",
];
const INTERNAL_KEYS: [&str; 3] = ["iguana-runtime", "iggy", "iguana-compiler"];
const LOCKFILES: [&str; 4] = [
    "Cargo.lock",
    "tests/Cargo.lock",
    "iguana-lsp/wasm/Cargo.lock",
    "terrarium/src-tauri/Cargo.lock",
];
const NOTES_START: &str = "<!-- release-notes:start -->";
const NOTES_END: &str = "<!-- release-notes:end -->";

#[derive(clap::Subcommand)]
pub enum Command {
    /// Plan a new candidate or resume an existing immutable candidate
    Plan {
        version: String,
        #[arg(long)]
        candidate: Option<String>,
    },
    /// Update the workspace manifest and current installation instructions
    Prepare { version: String },
    /// Check all publishable crates, exact internal pins, READMEs and lockfiles
    Check { version: String },
    /// Install the packaged tools offline and exercise a generated parser
    SmokePackages {
        version: String,
        commit: String,
        /// Directory containing the five .crate archives from cargo package
        packages: PathBuf,
        /// New temporary directory outside the release checkout
        directory: PathBuf,
    },
    /// Reject tracked or untracked changes in the release checkout
    Clean,
    /// Write a release PR body with editable release notes
    Notes {
        version: String,
        previous_tag: String,
        java_commit: String,
        candidate_commit: String,
        run_url: String,
        /// Preserve the editable notes from an existing PR
        #[arg(long)]
        body: Option<PathBuf>,
    },
    /// Extract the reviewed release notes from a saved PR body
    ExtractNotes { body: PathBuf },
    /// Plan a GitHub release create/update from the reviewed notes and API data
    GithubRelease {
        version: String,
        commit: String,
        notes: PathBuf,
        releases: PathBuf,
    },
    /// Publish missing crates; existing crates must come from the same commit
    Publish { version: String, commit: String },
}

pub fn run(command: Command, root: &Path) -> io::Result<()> {
    match command {
        Command::Plan { version, candidate } => {
            println!("{}", plan(root, &version, candidate.as_deref())?);
            Ok(())
        }
        Command::Prepare { version } => prepare(root, &version),
        Command::Check { version } => check(root, &version),
        Command::SmokePackages {
            version,
            commit,
            packages,
            directory,
        } => smoke::run(root, &version, &commit, &packages, &directory),
        Command::Clean => clean(root),
        Command::Notes {
            version,
            previous_tag,
            java_commit,
            candidate_commit,
            run_url,
            body,
        } => {
            let parsed = parse_version(&version)?;
            let notes = if let Some(body) = body {
                let body = fs::read_to_string(body)?;
                // Keep the entire marked region, including intentional whitespace.
                notes_region(&body)?.to_owned()
            } else {
                require(valid_commit(&candidate_commit), "Invalid candidate commit")?;
                parse_version(previous_tag.strip_prefix('v').unwrap_or(""))?;
                let changes = output(
                    root,
                    "git",
                    &[
                        "log",
                        "--reverse",
                        "--format=- %s (%h)",
                        &format!("{previous_tag}..{candidate_commit}"),
                    ],
                )?;
                format!(
                    "\n## Changes\n\n{changes}\n\n## Installation\n\n```sh\n{}\n{}\n```\n",
                    install_command("iguana", &parsed),
                    install_command("iguana-lsp", &parsed)
                )
            };
            println!(
                "{}",
                pr_body(&version, &candidate_commit, &java_commit, &run_url, &notes)?
            );
            Ok(())
        }
        Command::ExtractNotes { body } => {
            println!("{}", extract_notes(&fs::read_to_string(body)?)?);
            Ok(())
        }
        Command::GithubRelease {
            version,
            commit,
            notes,
            releases,
        } => {
            let releases =
                serde_json::from_str(&fs::read_to_string(releases)?).map_err(io::Error::other)?;
            println!(
                "{}",
                github_publication(&releases, &version, &commit, &fs::read_to_string(notes)?)?
            );
            Ok(())
        }
        Command::Publish { version, commit } => publish(root, &version, &commit),
    }
}

fn require(condition: bool, message: impl Into<String>) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message.into()))
    }
}

fn parse_version(value: &str) -> io::Result<Version> {
    let version = Version::parse(value).map_err(io::Error::other)?;
    require(
        version.build.is_empty(),
        "Release versions cannot contain build metadata",
    )?;
    Ok(version)
}

fn next_version(old: &str, new: &str) -> io::Result<Version> {
    let new = parse_version(new)?;
    require(
        new > parse_version(old)?,
        format!("New version must be greater than {old}"),
    )?;
    Ok(new)
}

fn release_tag(version: &str) -> String {
    format!("v{version}")
}

fn plan(root: &Path, version: &str, candidate: Option<&str>) -> io::Result<Value> {
    let requested = parse_version(version)?;
    let base = if let Some(commit) = candidate {
        require(valid_commit(commit), "Invalid candidate commit")?;
        let manifest = output(root, "git", &["show", &format!("{commit}:Cargo.toml")])?;
        require(
            document(&manifest)?["workspace"]["package"]["version"].as_str() == Some(version),
            "The selected release PR has a different version",
        )?;
        let ancestry = output(root, "git", &["rev-list", "--parents", "-n", "1", commit])?;
        let commits: Vec<_> = ancestry.split_whitespace().collect();
        require(
            commits.len() == 2,
            "A release candidate must have exactly one parent",
        )?;
        commits[1].to_owned()
    } else {
        next_version(&current_version(root)?, version)?;
        output(root, "git", &["rev-parse", "HEAD"])?
    };
    // An abandoned, merged candidate may not have a tag. Find the previous
    // actual release in this candidate's history, not from main's version.
    let tags = output(root, "git", &["tag", "--merged", &base, "--list", "v*"])?;
    let previous = tags
        .lines()
        .filter_map(|tag| parse_version(tag.strip_prefix('v')?).ok())
        .filter(|tag_version| tag_version < &requested)
        .max()
        .ok_or_else(|| {
            io::Error::other("No earlier v<version> release tag in candidate history")
        })?;
    Ok(json!({
        "tag": release_tag(version), "previous_tag": release_tag(&previous.to_string()),
        "base_commit": base,
    }))
}

fn pr_body(
    version: &str,
    candidate: &str,
    java: &str,
    run_url: &str,
    notes: &str,
) -> io::Result<String> {
    parse_version(version)?;
    require(
        valid_commit(candidate) && valid_commit(java),
        "Invalid candidate or Java commit",
    )?;
    require(
        run_url.starts_with("https://github.com/iguana-parser/iguana-rs/actions/runs/")
            && run_url
                .rsplit('/')
                .next()
                .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())),
        "Invalid release run URL",
    )?;
    let body = format!(
        "Prepare Iguana {version}.\n\n\
         Candidate: `{candidate}`.\n\n\
         Java grammar validation uses `iguana-parser/iguana-java-grammar@{java}`.\n\n\
         [Current release run]({run_url}).\n\n\
         Edit the notes between the markers before approving publication.\n\
         Preparation and validation run automatically. Review the diff and checks,\n\
         then approve merging this exact candidate in the Actions run.\n\
         Publication requires a separate approval after merging.\n\n\
         {NOTES_START}{notes}{NOTES_END}"
    );
    extract_notes(&body)?;
    Ok(body)
}

fn github_publication(
    releases: &Value,
    version: &str,
    commit: &str,
    notes: &str,
) -> io::Result<Value> {
    let parsed = parse_version(version)?;
    require(valid_commit(commit), "Invalid release commit")?;
    require(!notes.trim().is_empty(), "Release notes cannot be empty")?;
    let tag = release_tag(version);
    let matching: Vec<_> = releases
        .as_array()
        .ok_or_else(|| io::Error::other("Expected a releases array"))?
        .iter()
        .filter(|release| release["tag_name"] == tag)
        .collect();
    require(matching.len() <= 1, "Multiple releases have this tag")?;
    let (method, id) = if let Some(release) = matching.first() {
        let draft = release["draft"]
            .as_bool()
            .ok_or_else(|| io::Error::other("Missing release draft status"))?;
        if !draft {
            return Ok(json!({"method": "none"}));
        }
        let id = release["id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or_else(|| io::Error::other("Invalid draft release ID"))?;
        ("PATCH", Some(id))
    } else {
        ("POST", None)
    };
    Ok(json!({"method": method, "id": id, "payload": {
        "tag_name": tag, "target_commitish": commit, "name": version,
        "body": notes, "draft": false, "prerelease": !parsed.pre.is_empty(),
        "make_latest": if parsed.pre.is_empty() { "legacy" } else { "false" },
    }}))
}

fn document(text: &str) -> io::Result<DocumentMut> {
    text.parse().map_err(io::Error::other)
}

fn current_version(root: &Path) -> io::Result<String> {
    let doc = document(&fs::read_to_string(root.join("Cargo.toml"))?)?;
    doc["workspace"]["package"]["version"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| io::Error::other("Missing workspace version"))
}

fn replace_string(item: &mut Item, expected: &str, replacement: &str) -> io::Result<()> {
    require(
        item.as_str() == Some(expected),
        format!("Expected version {expected}, found {item}"),
    )?;
    let decor = item.as_value().unwrap().decor().clone();
    let mut value = toml_edit::Value::from(replacement);
    *value.decor_mut() = decor;
    *item = Item::Value(value);
    Ok(())
}

fn bump_manifest(text: &str, old: &str, new: &str) -> io::Result<String> {
    let mut doc = document(text)?;
    replace_string(&mut doc["workspace"]["package"]["version"], old, new)?;
    for key in INTERNAL_KEYS {
        replace_string(
            &mut doc["workspace"]["dependencies"][key]["version"],
            &format!("={old}"),
            &format!("={new}"),
        )?;
    }
    Ok(doc.to_string())
}

fn install_command(package: &str, version: &Version) -> String {
    if version.pre.is_empty() {
        format!("cargo install {package}")
    } else {
        format!("cargo install {package} --version {version}")
    }
}

fn bump_readme(text: &str, package: &str, old: &Version, new: &Version) -> io::Result<String> {
    let expected = install_command(package, old);
    let replacement = install_command(package, new);
    let mut found = false;
    let mut result = String::new();
    for line in text.split_inclusive('\n') {
        if line.trim() == expected {
            result.push_str(&line.replacen(&expected, &replacement, 1));
            found = true;
        } else {
            result.push_str(line);
        }
    }
    require(
        found,
        format!("Missing current installation command for {package}"),
    )?;
    Ok(result)
}

fn prepare(root: &Path, new: &str) -> io::Result<()> {
    clean(root)?;
    let old = current_version(root)?;
    let version = next_version(&old, new)?;
    let mut edits = vec![(
        "Cargo.toml".to_owned(),
        bump_manifest(&fs::read_to_string(root.join("Cargo.toml"))?, &old, new)?,
    )];
    for package in ["iguana", "iguana-lsp"] {
        let path = format!("{package}/README.md");
        let text = bump_readme(
            &fs::read_to_string(root.join(&path))?,
            package,
            &parse_version(&old)?,
            &version,
        )?;
        edits.push((path, text));
    }
    // Validate every input before writing anything. Lockfiles and generated
    // sources are then updated by fresh Cargo processes in the workflow.
    for (path, text) in edits {
        fs::write(root.join(path), text)?;
    }
    Ok(())
}

fn validate_metadata(metadata: &Value, version: &str) -> io::Result<()> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| io::Error::other("Missing packages"))?;
    let members = metadata["workspace_members"]
        .as_array()
        .ok_or_else(|| io::Error::other("Missing workspace members"))?;
    let mut names = BTreeSet::new();
    for package in packages
        .iter()
        .filter(|p| members.contains(&p["id"]) && p["publish"] != json!([]))
    {
        let name = package["name"]
            .as_str()
            .ok_or_else(|| io::Error::other("Missing package name"))?;
        names.insert(name);
        require(
            package["version"] == version,
            format!("{name} does not use {version}"),
        )?;
        for dependency in package["dependencies"]
            .as_array()
            .ok_or_else(|| io::Error::other("Missing dependencies"))?
        {
            if PACKAGES.contains(&dependency["name"].as_str().unwrap_or_default()) {
                require(
                    dependency["req"] == format!("={version}"),
                    format!("{name} has a mismatched internal dependency: {dependency}"),
                )?;
            }
        }
    }
    require(
        names == PACKAGES.into_iter().collect(),
        format!("Unexpected publishable packages: {names:?}"),
    )
}

fn check(root: &Path, version: &str) -> io::Result<()> {
    let parsed = parse_version(version)?;
    require(
        current_version(root)? == version,
        "Workspace version does not match release",
    )?;
    let metadata = output(
        root,
        "cargo",
        &["metadata", "--locked", "--no-deps", "--format-version", "1"],
    )?;
    validate_metadata(
        &serde_json::from_str(&metadata).map_err(io::Error::other)?,
        version,
    )?;
    for file in LOCKFILES {
        let doc = document(&fs::read_to_string(root.join(file))?)?;
        let packages = doc["package"]
            .as_array_of_tables()
            .ok_or_else(|| io::Error::other("Missing lockfile packages"))?;
        for package in packages {
            if !package.contains_key("source")
                && PACKAGES.contains(&package["name"].as_str().unwrap_or_default())
            {
                require(
                    package["version"].as_str() == Some(version),
                    format!("Stale Iguana version in {file}"),
                )?;
            }
        }
    }
    for package in ["iguana", "iguana-lsp"] {
        let text = fs::read_to_string(root.join(format!("{package}/README.md")))?;
        bump_readme(&text, package, &parsed, &parsed)?;
    }
    Ok(())
}

fn notes_region(body: &str) -> io::Result<&str> {
    require(
        body.matches(NOTES_START).count() == 1 && body.matches(NOTES_END).count() == 1,
        "PR body must contain exactly one pair of release-notes markers",
    )?;
    let after_start = body.split_once(NOTES_START).unwrap().1;
    let notes = after_start
        .split_once(NOTES_END)
        .ok_or_else(|| io::Error::other("Release notes markers are reversed"))?
        .0;
    require(!notes.trim().is_empty(), "Release notes cannot be empty")?;
    Ok(notes)
}

fn extract_notes(body: &str) -> io::Result<&str> {
    Ok(notes_region(body)?.trim())
}

fn clean(root: &Path) -> io::Result<()> {
    let status = output(
        root,
        "git",
        &["status", "--porcelain", "--untracked-files=normal"],
    )?;
    require(
        status.is_empty(),
        format!("Release checkout has uncommitted files:\n{status}"),
    )
}

fn output(root: &Path, program: &str, args: &[&str]) -> io::Result<String> {
    let result = Process::new(program)
        .args(args)
        .current_dir(root)
        .output()?;
    require(
        result.status.success(),
        format!(
            "{program} {args:?} failed:\n{}",
            String::from_utf8_lossy(&result.stderr)
        ),
    )?;
    String::from_utf8(result.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(io::Error::other)
}

fn execute(root: &Path, program: &str, args: &[&str]) -> io::Result<()> {
    require(
        Process::new(program)
            .args(args)
            .current_dir(root)
            .status()?
            .success(),
        format!("{program} {args:?} failed"),
    )
}

fn valid_commit(commit: &str) -> bool {
    commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit())
}

fn registry_status(status: &str, metadata: &str, package: &str, version: &str) -> io::Result<bool> {
    if status == "404" {
        return Ok(false);
    }
    require(
        status == "200",
        format!("Registry lookup for {package} failed: HTTP {status}"),
    )?;
    let data: Value = serde_json::from_str(metadata).map_err(io::Error::other)?;
    require(
        data["version"]["crate"] == package && data["version"]["num"] == version,
        format!("Registry returned unexpected version metadata for {package}"),
    )?;
    require(
        data["version"]["yanked"] == false,
        format!("{package} {version} is yanked or has invalid metadata; choose a new version"),
    )?;
    Ok(true)
}

fn check_provenance(vcs: &str, commit: &str) -> io::Result<()> {
    let data: Value = serde_json::from_str(vcs).map_err(io::Error::other)?;
    require(
        data["git"]["sha1"] == commit
            && (data["git"]["dirty"].is_null() || data["git"]["dirty"] == false),
        format!(
            "Existing crate was not published from clean commit {commit}; choose a new version"
        ),
    )
}

fn fetch(root: &Path, url: &str, destination: &Path) -> io::Result<String> {
    output(
        root,
        "curl",
        &[
            "--silent",
            "--show-error",
            "--location",
            "--retry",
            "3",
            "--connect-timeout",
            "20",
            "--max-time",
            "90",
            "--user-agent",
            "iguana-release-workflow (https://github.com/iguana-parser/iguana-rs)",
            "--write-out",
            "%{http_code}",
            "--output",
            destination.to_str().unwrap(),
            url,
        ],
    )
}

fn published(
    root: &Path,
    directory: &Path,
    package: &str,
    version: &str,
    commit: &str,
) -> io::Result<bool> {
    let metadata = directory.join(format!("{package}.json"));
    let status = fetch(
        root,
        &format!("https://crates.io/api/v1/crates/{package}/{version}"),
        &metadata,
    )?;
    if !registry_status(&status, &fs::read_to_string(metadata)?, package, version)? {
        return Ok(false);
    }
    let archive = directory.join(format!("{package}.crate"));
    let status = fetch(
        root,
        &format!("https://static.crates.io/crates/{package}/{package}-{version}.crate"),
        &archive,
    )?;
    require(
        status == "200",
        format!("Downloading {package} failed: HTTP {status}"),
    )?;
    let vcs = output(
        root,
        "tar",
        &[
            "-xOzf",
            archive.to_str().unwrap(),
            &format!("{package}-{version}/.cargo_vcs_info.json"),
        ],
    )?;
    check_provenance(&vcs, commit)?;
    Ok(true)
}

fn publish(root: &Path, version: &str, commit: &str) -> io::Result<()> {
    require(
        env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
            && env::var("GITHUB_EVENT_NAME").as_deref() == Ok("workflow_dispatch")
            && env::var("GITHUB_REPOSITORY").as_deref() == Ok("iguana-parser/iguana-rs")
            && env::var("GITHUB_REF").as_deref() == Ok("refs/heads/main"),
        "Publishing is allowed only in the manually started release workflow on main",
    )?;
    require(
        valid_commit(commit) && output(root, "git", &["rev-parse", "HEAD"])? == commit,
        "Checkout does not match the validated release commit",
    )?;
    check(root, version)?;
    clean(root)?;
    let directory = PathBuf::from(
        env::var_os("RUNNER_TEMP").ok_or_else(|| io::Error::other("Missing RUNNER_TEMP"))?,
    )
    .join("iguana-release-crates");
    fs::create_dir_all(&directory)?;
    // Check ALL crates before uploading ANY. A retry may skip only versions
    // whose archived VCS information proves they came from this same commit.
    let mut existing = Vec::new();
    for package in PACKAGES {
        if published(root, &directory, package, version, commit)? {
            existing.push(package);
        }
    }
    if existing.len() == PACKAGES.len() {
        return Ok(());
    }
    let mut args = vec!["publish", "--workspace", "--locked"];
    for package in existing {
        args.extend(["--exclude", package]);
    }
    execute(root, "cargo", &args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Repository(PathBuf);

    impl Repository {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = env::temp_dir().join(format!(
                "iguana-release-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let repo = Self(root);
            repo.git(&["init", "--initial-branch=main"]);
            repo.git(&["config", "user.name", "Release test"]);
            repo.git(&["config", "user.email", "release@example.invalid"]);
            repo.commit_version("0.1.0-alpha.1");
            repo.git(&["tag", "v0.1.0-alpha.1"]);
            repo
        }

        fn git(&self, args: &[&str]) -> String {
            output(&self.0, "git", args).unwrap()
        }

        fn commit_version(&self, version: &str) -> String {
            fs::write(
                self.0.join("Cargo.toml"),
                format!("[workspace.package]\nversion = '{version}'\n"),
            )
            .unwrap();
            self.git(&["add", "Cargo.toml"]);
            self.git(&["commit", "-m", version]);
            self.git(&["rev-parse", "HEAD"])
        }
    }

    impl Drop for Repository {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn untagged_merged_candidate_can_resume_after_main_advances_or_be_superseded() {
        let repo = Repository::new();
        let base = repo.git(&["rev-parse", "HEAD"]);
        let candidate = repo.commit_version("0.1.0-alpha.2");
        // A workflow fix on main must not become the release source commit.
        repo.git(&["commit", "--allow-empty", "-m", "Fix the release workflow"]);
        assert!(plan(&repo.0, "0.1.0-alpha.2", None).is_err());
        let resumed = plan(&repo.0, "0.1.0-alpha.2", Some(&candidate)).unwrap();
        assert_eq!(resumed["base_commit"], base);
        assert_eq!(resumed["previous_tag"], "v0.1.0-alpha.1");
        assert_eq!(resumed["tag"], "v0.1.0-alpha.2");

        let replacement = plan(&repo.0, "0.1.0-alpha.3", None).unwrap();
        assert_eq!(replacement["previous_tag"], "v0.1.0-alpha.1");
        assert_eq!(replacement["base_commit"], repo.git(&["rev-parse", "HEAD"]));
        assert!(plan(&repo.0, "0.1.0-alpha.3", Some(&candidate)).is_err());
        assert!(plan(&repo.0, "0.1.0-alpha.2", Some("HEAD")).is_err());
    }

    #[test]
    fn resumed_notes_use_candidate_history_even_after_later_releases() {
        let repo = Repository::new();
        let candidate = repo.commit_version("0.1.0-alpha.2");
        repo.git(&["tag", "v0.1.0-alpha.2"]);
        repo.commit_version("0.1.0-alpha.10");
        repo.git(&["tag", "v0.1.0-alpha.10"]);
        let resumed = plan(&repo.0, "0.1.0-alpha.2", Some(&candidate)).unwrap();
        assert_eq!(resumed["previous_tag"], "v0.1.0-alpha.1");
        let next = plan(&repo.0, "0.1.0-alpha.11", None).unwrap();
        assert_eq!(next["previous_tag"], "v0.1.0-alpha.10");
    }

    #[test]
    fn replacement_candidate_uses_new_main_without_changing_the_old_candidate() {
        let repo = Repository::new();
        let base = repo.git(&["rev-parse", "HEAD"]);
        repo.git(&["checkout", "-b", "release/0.1.0-alpha.2/100"]);
        let stale = repo.commit_version("0.1.0-alpha.2");
        repo.git(&["checkout", "main"]);
        repo.git(&["commit", "--allow-empty", "-m", "Advance main"]);
        let replacement = plan(&repo.0, "0.1.0-alpha.2", None).unwrap();
        assert_ne!(replacement["base_commit"], base);
        assert_eq!(replacement["base_commit"], repo.git(&["rev-parse", "HEAD"]));
        assert_eq!(repo.git(&["rev-parse", "release/0.1.0-alpha.2/100"]), stale);
    }

    #[test]
    fn refreshes_validation_metadata_without_changing_edited_notes() {
        let candidate = "a".repeat(40);
        let old_java = "b".repeat(40);
        let new_java = "c".repeat(40);
        let notes = "\n\nHand-edited **notes**.\n\n```sh\necho '$KEEP'\n```\n\n";
        let old_body = pr_body(
            "0.1.0-alpha.2",
            &candidate,
            &old_java,
            "https://github.com/iguana-parser/iguana-rs/actions/runs/100",
            notes,
        )
        .unwrap();
        let refreshed = pr_body(
            "0.1.0-alpha.2",
            &candidate,
            &new_java,
            "https://github.com/iguana-parser/iguana-rs/actions/runs/200",
            notes_region(&old_body).unwrap(),
        )
        .unwrap();
        assert_eq!(notes_region(&refreshed).unwrap(), notes);
        assert!(refreshed.contains(&candidate));
        assert!(refreshed.contains(&new_java));
        assert!(refreshed.contains("/runs/200"));
        assert!(!refreshed.contains(&old_java));
        assert!(!refreshed.contains("/runs/100"));
    }

    #[test]
    fn draft_and_new_releases_share_canonical_fields() {
        let commit = "a".repeat(40);
        for version in ["0.1.0-alpha.2", "0.1.0"] {
            let drafts = json!([{"id": 42, "tag_name": release_tag(version), "draft": true,
                "name": "Wrong title", "body": "Stale draft notes", "prerelease": false}]);
            let draft = github_publication(&drafts, version, &commit, "Reviewed notes").unwrap();
            let fresh = github_publication(&json!([]), version, &commit, "Reviewed notes").unwrap();
            assert_eq!(draft["method"], "PATCH");
            assert_eq!(draft["id"], 42);
            assert_eq!(fresh["method"], "POST");
            assert_eq!(draft["payload"], fresh["payload"]);
            assert_eq!(draft["payload"]["name"], version);
            assert_eq!(draft["payload"]["tag_name"], release_tag(version));
            assert_eq!(draft["payload"]["target_commitish"], commit);
            assert_eq!(draft["payload"]["body"], "Reviewed notes");
            assert_eq!(draft["payload"]["draft"], false);
            assert_eq!(draft["payload"]["prerelease"], version.contains('-'));
        }
    }

    #[test]
    fn already_public_releases_are_untouched_and_malformed_data_fails_closed() {
        let commit = "a".repeat(40);
        let public = json!([{"id": 42, "tag_name": "v0.1.0-alpha.2", "draft": false,
            "name": "Published title", "body": "Published notes", "prerelease": true}]);
        assert_eq!(
            github_publication(&public, "0.1.0-alpha.2", &commit, "New notes").unwrap(),
            json!({"method": "none"})
        );
        for data in [
            json!({}),
            json!([{"tag_name": "v0.1.0-alpha.2"}]),
            json!([{"tag_name": "v0.1.0-alpha.2", "draft": true, "id": 0}]),
            json!([public[0], public[0]]),
        ] {
            assert!(github_publication(&data, "0.1.0-alpha.2", &commit, "notes").is_err());
        }
    }

    #[test]
    fn versions_must_advance_with_semver_ordering() {
        assert!(next_version("0.1.0-alpha.2", "0.1.0-alpha.10").is_ok());
        assert!(next_version("0.1.0-alpha.10", "0.1.0").is_ok());
        for version in [
            "0.1.0-alpha.2",
            "0.1.0-alpha.1",
            "v0.1.0",
            "0.1.0+x",
            "0.1.0-alpha.01",
            "0.1.0\n",
            "$(id)",
        ] {
            assert!(next_version("0.1.0-alpha.2", version).is_err(), "{version}");
        }
    }

    #[test]
    fn manifest_bump_preserves_paths_features_and_comments() {
        let source = "[workspace.package]\nversion = '0.1.0-alpha.1' # keep\n[workspace.dependencies]\niguana-runtime = { path = 'runtime', version = '=0.1.0-alpha.1', features = ['cli'] }\niggy = { version = '=0.1.0-alpha.1', package = 'iguana-iggy' }\niguana-compiler = { version = '=0.1.0-alpha.1' }\nother = '0.1.0-alpha.1'\n";
        let bumped = bump_manifest(source, "0.1.0-alpha.1", "0.1.0-alpha.2").unwrap();
        assert!(bumped.contains("# keep"));
        assert!(bumped.contains("path = 'runtime'"));
        assert!(bumped.contains("features = ['cli']"));
        assert!(bumped.contains("other = '0.1.0-alpha.1'"));
        assert_eq!(bumped.matches("0.1.0-alpha.2").count(), 4);
        assert!(bump_manifest(&bumped, "0.1.0-alpha.1", "0.1.0-alpha.2").is_err());
        assert!(
            bump_manifest(
                &source.replace("'=0.1.0-alpha.1'", "'^0.1.0-alpha.1'"),
                "0.1.0-alpha.1",
                "0.1.0-alpha.2"
            )
            .is_err()
        );
    }

    #[test]
    fn readme_bump_changes_current_instructions_and_handles_stable_releases() {
        let old = parse_version("0.1.0-alpha.1").unwrap();
        let alpha = parse_version("0.1.0-alpha.2").unwrap();
        let stable = parse_version("0.1.0").unwrap();
        let source = "```sh\ncargo install iguana --version 0.1.0-alpha.1\n```\nMeasured with 0.1.0-alpha.1.\n";
        let bumped = bump_readme(source, "iguana", &old, &alpha).unwrap();
        assert!(bumped.contains("--version 0.1.0-alpha.2"));
        assert!(bumped.contains("Measured with 0.1.0-alpha.1."));
        let stable_text = bump_readme(&bumped, "iguana", &alpha, &stable).unwrap();
        assert!(stable_text.contains("cargo install iguana\n"));
        assert!(!stable_text.contains("--version"));
        assert!(bump_readme(source, "iguana-lsp", &old, &alpha).is_err());
    }

    fn metadata() -> Value {
        json!({
            "workspace_members": PACKAGES,
            "packages": PACKAGES.map(|name| json!({
                "id": name, "name": name, "version": "0.1.0-alpha.2", "publish": null,
                "dependencies": [{"name": "iguana-runtime", "req": "=0.1.0-alpha.2"}]
            }))
        })
    }

    #[test]
    fn rejects_partial_bumps_unpinned_dependencies_and_changed_package_set() {
        assert!(validate_metadata(&metadata(), "0.1.0-alpha.2").is_ok());
        let mut value = metadata();
        value["packages"][0]["version"] = json!("0.1.0-alpha.1");
        assert!(validate_metadata(&value, "0.1.0-alpha.2").is_err());
        let mut value = metadata();
        value["packages"][1]["dependencies"][0]["req"] = json!("^0.1.0-alpha.2");
        assert!(validate_metadata(&value, "0.1.0-alpha.2").is_err());
        let mut value = metadata();
        value["packages"][0]["publish"] = json!([]);
        assert!(validate_metadata(&value, "0.1.0-alpha.2").is_err());
    }

    #[test]
    fn release_notes_must_be_present_unambiguous_and_nonempty() {
        assert_eq!(
            extract_notes(&format!(
                "intro\n{NOTES_START}\nEdited notes\n{NOTES_END}\nfooter"
            ))
            .unwrap(),
            "Edited notes"
        );
        for body in [
            String::new(),
            format!("{NOTES_START} {NOTES_END}"),
            format!("{NOTES_END}notes{NOTES_START}"),
            format!("{NOTES_START}{NOTES_START}notes{NOTES_END}"),
        ] {
            assert!(extract_notes(&body).is_err());
        }
    }

    #[test]
    fn registry_errors_and_yanked_versions_do_not_count_as_available() {
        assert!(!registry_status("404", "not found", "iguana", "1.0.0").unwrap());
        for status in ["401", "403", "429", "500", "000"] {
            assert!(registry_status(status, "", "iguana", "1.0.0").is_err());
        }
        let valid = json!({"version": {"crate": "iguana", "num": "1.0.0", "yanked": false}});
        assert!(registry_status("200", &valid.to_string(), "iguana", "1.0.0").unwrap());
        let mut yanked = valid;
        yanked["version"]["yanked"] = json!(true);
        assert!(registry_status("200", &yanked.to_string(), "iguana", "1.0.0").is_err());
        assert!(registry_status("200", "{}", "iguana", "1.0.0").is_err());
    }

    #[test]
    fn retries_require_the_same_clean_archived_commit() {
        let commit = "a".repeat(40);
        for dirty in [false, true] {
            let vcs = json!({"git": {"sha1": commit, "dirty": dirty}}).to_string();
            assert_eq!(check_provenance(&vcs, &commit).is_ok(), !dirty);
            assert!(check_provenance(&vcs, &"b".repeat(40)).is_err());
        }
        assert!(check_provenance(&json!({"git": {"sha1": commit}}).to_string(), &commit).is_ok());
        assert!(check_provenance("{}", &commit).is_err());
        assert!(!valid_commit("HEAD"));
    }
}
