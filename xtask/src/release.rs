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

mod github;
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
    /// Describe the release changes and the Actions merge approval in the PR
    PrBody {
        version: String,
        java_commit: String,
        candidate_commit: String,
        run_url: String,
    },
    /// Generate initial Markdown notes for the separate draft release
    Notes {
        version: String,
        previous_tag: String,
        candidate_commit: String,
    },
    /// Create a draft release, or preserve an existing release and its edits
    GithubDraft {
        version: String,
        commit: String,
        notes: PathBuf,
        releases: PathBuf,
    },
    /// Validate and snapshot the saved release after publication approval
    ReviewRelease {
        version: String,
        commit: String,
        release: PathBuf,
    },
    /// Publish the approved draft without regenerating or replacing edited notes
    GithubRelease {
        version: String,
        commit: String,
        approved: PathBuf,
        release: PathBuf,
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
        Command::PrBody {
            version,
            java_commit,
            candidate_commit,
            run_url,
        } => {
            println!(
                "{}",
                pr_body(&version, &candidate_commit, &java_commit, &run_url)?
            );
            Ok(())
        }
        Command::Notes {
            version,
            previous_tag,
            candidate_commit,
        } => {
            println!(
                "{}",
                release_notes(root, &version, &previous_tag, &candidate_commit)?
            );
            Ok(())
        }
        Command::GithubDraft {
            version,
            commit,
            notes,
            releases,
        } => {
            println!(
                "{}",
                github::draft(
                    &read_json(&releases)?,
                    &version,
                    &commit,
                    &fs::read_to_string(notes)?
                )?
            );
            Ok(())
        }
        Command::ReviewRelease {
            version,
            commit,
            release,
        } => {
            let release = read_json(&release)?;
            github::review(&release, &version, &commit)?;
            println!("{release}");
            Ok(())
        }
        Command::GithubRelease {
            version,
            commit,
            approved,
            release,
        } => {
            println!(
                "{}",
                github::publication(
                    &read_json(&release)?,
                    &read_json(&approved)?,
                    &version,
                    &commit
                )?
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

fn read_json(path: &Path) -> io::Result<Value> {
    serde_json::from_str(&fs::read_to_string(path)?).map_err(io::Error::other)
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
    let tag = release_tag(version);
    if !output(root, "git", &["tag", "--list", &tag])?.is_empty() {
        require(
            candidate.is_some(),
            format!(
                "Tag {tag} already exists. Resume its release PR instead of preparing a new candidate"
            ),
        )?;
        let tagged = output(root, "git", &["rev-parse", &format!("{tag}^{{commit}}")])?;
        require(
            candidate == Some(tagged.as_str()),
            format!("Tag {tag} points to a different release candidate"),
        )?;
    }
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
        "tag": tag, "previous_tag": release_tag(&previous.to_string()),
        "base_commit": base,
    }))
}

fn pr_body(version: &str, candidate: &str, java: &str, run_url: &str) -> io::Result<String> {
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
    Ok(format!(
        "Release {version}.\n\n\
         This PR updates the five Iguana crate versions and their exact internal dependency pins to `{version}`. \
         It refreshes the root, grammar-test, LSP WebAssembly and Terrarium lockfiles. \
         The CLI and LSP README installation commands use the new version. \
         Generated parser sources should be unchanged.\n\n\
         CI runs the full test suite and validates the release packages. \
         It checks tool installation and parser generation from those packages. \
         CI also generates and builds the Java parser, then runs its golden and corpus tests.\n\n\
         **To merge:** review the diff and checks, then approve `release-merge` in the \
         [Actions run]({run_url}). Leave this PR as a draft. The workflow marks it ready \
         and fast-forwards `main` to the single `Release {version}` commit.\n\n\
         After merging, the workflow provides a separate draft release editor for the release notes. \
         Saving that draft and approving `release-publish` in Actions authorizes publication.\n\n\
         Candidate: `{candidate}`.\n\n\
         Java grammar: `iguana-parser/iguana-java-grammar@{java}`.\n"
    ))
}

fn release_notes(
    root: &Path,
    version: &str,
    previous_tag: &str,
    commit: &str,
) -> io::Result<String> {
    let parsed = parse_version(version)?;
    require(valid_commit(commit), "Invalid release commit")?;
    parse_version(previous_tag.strip_prefix('v').unwrap_or(""))?;
    // The version-bump commit itself is release bookkeeping, not a release note.
    let changes = output(
        root,
        "git",
        &[
            "log",
            "--reverse",
            "--format=- %s (%h)",
            &format!("{previous_tag}..{commit}^"),
        ],
    )?;
    Ok(format!(
        "## Changes\n\n{changes}\n\n## Installation\n\n```sh\n{}\n{}\n```\n",
        install_command("iguana", &parsed),
        install_command("iguana-lsp", &parsed)
    ))
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

fn require_install_command(text: &str, package: &str, version: &Version) -> io::Result<()> {
    let expected = install_command(package, version);
    require(
        text.lines().any(|line| line.trim() == expected),
        format!("Missing current installation command for {package}"),
    )
}

fn bump_readme(text: &str, package: &str, old: &Version, new: &Version) -> io::Result<String> {
    require_install_command(text, package, old)?;
    let expected = install_command(package, old);
    let replacement = install_command(package, new);
    let mut result = String::new();
    for line in text.split_inclusive('\n') {
        if line.trim() == expected {
            result.push_str(&line.replacen(&expected, &replacement, 1));
        } else {
            result.push_str(line);
        }
    }
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
    // Validate every input before writing anything. Fresh Cargo processes in
    // the workflow then update lockfiles and verify generated sources.
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
        require_install_command(&text, package, &parsed)?;
    }
    Ok(())
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
        .map(|s| s.trim_end().to_owned())
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
        format!("{package} {version} is yanked or has invalid metadata. Choose a new version"),
    )?;
    Ok(true)
}

fn check_provenance(vcs: &str, commit: &str) -> io::Result<()> {
    let data: Value = serde_json::from_str(vcs).map_err(io::Error::other)?;
    require(
        data["git"]["sha1"] == commit
            && (data["git"]["dirty"].is_null() || data["git"]["dirty"] == false),
        format!(
            "Existing crate was not published from clean commit {commit}. Choose a new version"
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
    // Check every crate before uploading any. A retry may skip only versions
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
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn existing_tag_blocks_new_candidates_and_mismatched_resumes() {
        let repo = Repository::new();
        repo.git(&["tag", "v0.1.0-alpha.2"]);
        let error = plan(&repo.0, "0.1.0-alpha.2", None).unwrap_err();
        assert!(error.to_string().contains("already exists"));
        let candidate = repo.commit_version("0.1.0-alpha.2");
        let error = plan(&repo.0, "0.1.0-alpha.2", Some(&candidate)).unwrap_err();
        assert!(error.to_string().contains("different release candidate"));
    }

    #[test]
    fn annotated_tag_at_the_candidate_allows_publication_recovery() {
        let repo = Repository::new();
        let candidate = repo.commit_version("0.1.0-alpha.2");
        repo.git(&["tag", "-a", "v0.1.0-alpha.2", "-m", "0.1.0-alpha.2"]);
        let resumed = plan(&repo.0, "0.1.0-alpha.2", Some(&candidate)).unwrap();
        assert_eq!(resumed["tag"], "v0.1.0-alpha.2");
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
    fn pr_describes_changes_and_approval_without_embedding_release_notes() {
        let body = pr_body(
            "0.1.0-alpha.2",
            &"a".repeat(40),
            &"b".repeat(40),
            "https://github.com/iguana-parser/iguana-rs/actions/runs/200",
        )
        .unwrap();
        assert!(body.contains("five Iguana crate versions"));
        assert!(body.contains("lockfiles"));
        assert!(body.contains("README installation commands"));
        assert!(body.contains("approve `release-merge`"));
        assert!(body.contains("Leave this PR as a draft"));
        assert!(body.contains("single `Release 0.1.0-alpha.2` commit"));
        assert!(body.contains("separate draft release editor"));
        assert!(!body.contains("release-notes:"));
        assert!(!body.contains("cargo install"));
    }

    #[test]
    fn notes_stop_before_release_bookkeeping_and_exclude_later_commits() {
        let repo = Repository::new();
        repo.git(&["commit", "--allow-empty", "-m", "Improve parsing"]);
        let candidate = repo.commit_version("0.1.0-alpha.2");
        repo.git(&["commit", "--allow-empty", "-m", "Later work"]);
        let notes = release_notes(&repo.0, "0.1.0-alpha.2", "v0.1.0-alpha.1", &candidate).unwrap();
        assert!(notes.contains("Improve parsing"));
        assert!(!notes.contains("Later work"));
        assert!(!notes.contains("- 0.1.0-alpha.2"));
        assert!(notes.contains("cargo install iguana --version 0.1.0-alpha.2"));
        assert!(notes.contains("cargo install iguana-lsp --version 0.1.0-alpha.2"));
        assert!(release_notes(&repo.0, "0.1.0-alpha.2", "bad-tag", &candidate).is_err());
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
