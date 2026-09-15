//! Pure planning for the draft editor and final GitHub publication. Network
//! writes stay in the workflow so the approval boundaries remain visible.

use super::*;

pub(super) fn draft(
    releases: &Value,
    version: &str,
    commit: &str,
    notes: &str,
) -> io::Result<Value> {
    let parsed = parse_version(version)?;
    require(valid_commit(commit), "Invalid release commit")?;
    let tag = release_tag(version);
    let matches: Vec<_> = releases
        .as_array()
        .ok_or_else(|| io::Error::other("Expected a releases array"))?
        .iter()
        .filter(|release| release["tag_name"] == tag)
        .collect();
    require(matches.len() <= 1, "Multiple releases have this tag")?;
    if let Some(release) = matches.first() {
        validate(release, version, commit)?;
        // In particular, never regenerate notes when resuming a draft that
        // the maintainer has already edited (even if the notes are now empty).
        return Ok(json!({"method": "none", "id": release["id"]}));
    }
    require(
        !notes.trim().is_empty(),
        "Initial release notes cannot be empty",
    )?;
    Ok(json!({"method": "POST", "payload": {
        "tag_name": tag, "target_commitish": commit, "name": version,
        "body": notes, "draft": true, "prerelease": !parsed.pre.is_empty(),
    }}))
}

fn validate(release: &Value, version: &str, commit: &str) -> io::Result<()> {
    let parsed = parse_version(version)?;
    require(valid_commit(commit), "Invalid release commit")?;
    require(
        release["id"].as_u64().is_some_and(|id| id > 0),
        "Invalid release ID",
    )?;
    require(
        release["tag_name"] == release_tag(version),
        "Release tag was changed",
    )?;
    let draft = release["draft"]
        .as_bool()
        .ok_or_else(|| io::Error::other("Missing release draft status"))?;
    if draft {
        require(
            release["target_commitish"] == commit,
            "Draft targets a different commit. Restore the tested candidate before continuing",
        )?;
        require(
            release["name"] == version,
            "Draft title must match the release version",
        )?;
        require(
            release["prerelease"] == !parsed.pre.is_empty(),
            "Draft prerelease setting does not match the version",
        )?;
    }
    // For an already published release, the workflow verifies the actual tag
    // commit before allowing recovery. Preserve its published metadata.
    Ok(())
}

pub(super) fn review(release: &Value, version: &str, commit: &str) -> io::Result<()> {
    validate(release, version, commit)?;
    require(
        release["body"]
            .as_str()
            .is_some_and(|body| !body.trim().is_empty()),
        "Release notes cannot be empty. Open Releases, edit the draft with the pencil button, and click Save draft before approving publication",
    )
}

pub(super) fn publication(
    current: &Value,
    approved: &Value,
    version: &str,
    commit: &str,
) -> io::Result<Value> {
    review(current, version, commit)?;
    review(approved, version, commit)?;
    require(
        current["id"] == approved["id"],
        "The reviewed release was replaced",
    )?;
    if current["draft"] == false {
        return Ok(json!({"method": "none"}));
    }
    require(
        approved["draft"] == true,
        "The published release was changed back to a draft",
    )?;
    require(
        current["body"] == approved["body"],
        "Release notes changed during publication. Review them and retry the publish job",
    )?;
    // Freeze the text saved when this approval started. Do not take notes from
    // the PR, regenerate them, or reset the editor's other fields on retries.
    Ok(json!({"method": "PATCH", "id": current["id"], "payload": {
        "draft": false, "body": approved["body"],
        "make_latest": if parse_version(version)?.pre.is_empty() { "legacy" } else { "false" },
    }}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved() -> Value {
        json!({"id": 42, "tag_name": "v0.1.0-alpha.2", "target_commitish": "a".repeat(40),
            "name": "0.1.0-alpha.2", "draft": true, "prerelease": true,
            "body": "\nHand-edited **notes**.\n\n```sh\necho '$KEEP'\n```\n"})
    }

    #[test]
    fn creates_unpublished_draft_with_canonical_identity() {
        for version in ["0.1.0-alpha.2", "0.1.0"] {
            let value = draft(&json!([]), version, &"a".repeat(40), "Initial notes").unwrap();
            assert_eq!(value["method"], "POST");
            assert_eq!(value["payload"]["draft"], true);
            assert_eq!(value["payload"]["name"], version);
            assert_eq!(value["payload"]["tag_name"], format!("v{version}"));
            assert_eq!(value["payload"]["target_commitish"], "a".repeat(40));
            assert_eq!(value["payload"]["prerelease"], version.contains('-'));
            assert_eq!(value["payload"]["body"], "Initial notes");
        }
    }

    #[test]
    fn resuming_preserves_editor_contents_including_empty_work_in_progress() {
        for body in [saved()["body"].as_str().unwrap(), ""] {
            let mut release = saved();
            release["body"] = json!(body);
            assert_eq!(
                draft(
                    &json!([release]),
                    "0.1.0-alpha.2",
                    &"a".repeat(40),
                    "Regenerated notes"
                )
                .unwrap(),
                json!({"method": "none", "id": 42})
            );
            assert_eq!(
                review(&release, "0.1.0-alpha.2", &"a".repeat(40)).is_ok(),
                !body.is_empty()
            );
        }
    }

    #[test]
    fn publishes_exact_saved_markdown_and_rejects_edits_after_approval() {
        let approved = saved();
        let payload = publication(&saved(), &approved, "0.1.0-alpha.2", &"a".repeat(40)).unwrap();
        assert_eq!(
            payload["payload"],
            json!({"draft": false, "body": approved["body"], "make_latest": "false"})
        );
        for (field, value) in [
            ("id", json!(43)),
            ("body", json!("Changed while uploading")),
            ("body", json!(" ")),
        ] {
            let mut current = saved();
            current[field] = value;
            assert!(publication(&current, &approved, "0.1.0-alpha.2", &"a".repeat(40)).is_err());
        }
    }

    #[test]
    fn refuses_wrong_draft_commit_tag_title_or_prerelease_state() {
        for (field, value) in [
            ("target_commitish", json!("b".repeat(40))),
            ("target_commitish", json!("main")),
            ("tag_name", json!("v0.1.0-alpha.3")),
            ("name", json!("Wrong version")),
            ("prerelease", json!(false)),
            ("draft", Value::Null),
            ("id", json!(0)),
        ] {
            let mut release = saved();
            release[field] = value;
            assert!(
                review(&release, "0.1.0-alpha.2", &"a".repeat(40)).is_err(),
                "{field}"
            );
        }
        for releases in [
            json!({}),
            json!([saved(), saved()]),
            json!([{"tag_name": "v0.1.0-alpha.2"}]),
        ] {
            assert!(draft(&releases, "0.1.0-alpha.2", &"a".repeat(40), "Notes").is_err());
        }
    }

    #[test]
    fn completed_publications_are_never_overwritten() {
        let mut public = saved();
        public["draft"] = json!(false);
        public["name"] = json!("Edited published title");
        public["body"] = json!("Edited published notes");
        assert_eq!(
            draft(
                &json!([public]),
                "0.1.0-alpha.2",
                &"a".repeat(40),
                "New notes"
            )
            .unwrap(),
            json!({"method": "none", "id": 42})
        );
        assert_eq!(
            publication(&public, &saved(), "0.1.0-alpha.2", &"a".repeat(40)).unwrap(),
            json!({"method": "none"})
        );
    }
}
