#![expect(
    clippy::unwrap_used,
    reason = "test helpers; a panic is a test failure"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct Project {
    root: PathBuf,
    directory: PathBuf,
}

impl Project {
    fn new(subdirectory: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "roostr-git-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let directory = root.join(subdirectory);
        fs::create_dir_all(&directory).unwrap();
        let project = Self { root, directory };
        project.git(&["init", "-q"]);
        project.git(&["config", "user.name", "Roostr test"]);
        project.git(&["config", "user.email", "test@example.invalid"]);
        project.write(".roostr/project.yml", "name: Garden\n");
        project.write(
            ".roostr/entrypoints.tsv",
            "# covers: cli\ncli\tgrow\tsrc/grow.rs\n",
        );
        project.write(
            ".roostr/processes/grow.yml",
            "name: Grow\ndescription: Grow a garden.\nfiles:\n  - src/grow.rs\n",
        );
        project.write("src/grow.rs", "initial\n");
        project
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.directory.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn git_command(&self) -> Command {
        let mut command = Command::new("git");
        command
            .current_dir(&self.root)
            .args(["-c", "core.fsmonitor=false", "-c", "commit.gpgsign=false"])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null");
        command
    }

    fn git(&self, args: &[&str]) {
        successful(self.git_command().args(args).output().unwrap());
    }

    fn commit(&self, subject: &str, date: &str) {
        self.git(&["add", "--all"]);
        successful(
            self.git_command()
                .args(["commit", "-q", "-m", subject])
                .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
                .env("GIT_COMMITTER_DATE", date)
                .output()
                .unwrap(),
        );
    }

    fn roostr(&self, cwd: &Path, args: &[&str], timezone: &str) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_roostr"))
            .current_dir(cwd)
            .args(args)
            .env("ROOSTR_TODAY", "2026-10-10")
            .env("TZ", timezone)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        successful(output)
    }

    fn changes(&self, timezone: &str) -> String {
        self.roostr(
            &self.directory,
            &["changes", "--since", "2026-10-03"],
            timezone,
        )
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        #[expect(
            clippy::let_underscore_must_use,
            reason = "best-effort temp dir cleanup in Drop"
        )]
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn successful(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn baseline_with_equal_commit_dates_uses_the_later_commit_in_history() {
    let project = Project::new("");
    project.commit("Initial garden", "2026-09-01T00:00:00Z");
    project.write(
        ".roostr/entrypoints.tsv",
        "# covers: cli\ncli\tgrow\tsrc/grow.rs\ncli\tprune\tsrc/grow.rs\n",
    );
    project.commit("Add pruning", "2026-09-01T00:00:00Z");
    let output = project.changes("UTC");
    assert!(
        output.contains("entrypoints: 0 added, 0 removed"),
        "{output}"
    );
    assert!(!output.contains("ENTRYPOINTS"), "{output}");
}

#[test]
fn nested_projects_keep_literal_paths_and_git_reads_do_not_run_helpers_or_update_the_index() {
    let project = Project::new("garden space");
    let filename = "src/庭 [x] code.rs";
    project.write(filename, "initial\n");
    project.write(
        ".roostr/processes/grow.yml",
        &format!("name: Grow\ndescription: Grow a garden.\nfiles:\n  - '{filename}'\n"),
    );
    project.commit("Initial garden", "2026-09-01T00:00:00Z");
    project.write(filename, "water\n");
    project.commit("Water the garden", "2026-10-05T00:00:00Z");
    let helper = project.root.join("unexpected-helper");
    let marker = project.root.join("helper-ran");
    fs::write(
        &helper,
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
    project.git(&["config", "core.fsmonitor", helper.to_str().unwrap()]);
    project.git(&["config", "diff.external", helper.to_str().unwrap()]);
    let index = fs::read(project.root.join(".git/index")).unwrap();
    let output = project.roostr(&project.directory.join("src"), &["changes"], "UTC");
    assert!(output.contains("Water the garden"), "{output}");
    assert!(
        output.contains("entrypoints: 0 added, 0 removed   processes: 1 changed"),
        "{output}"
    );
    let overview = project.roostr(&project.directory, &[], "UTC");
    assert!(
        overview.contains(&format!("{filename} changed since")),
        "{overview}"
    );
    assert!(!marker.exists());
    assert_eq!(index, fs::read(project.root.join(".git/index")).unwrap());
}

#[test]
fn periods_use_commit_dates_even_when_dates_run_backwards_in_history() {
    let project = Project::new("");
    project.commit("Initial garden", "2026-09-01T00:00:00Z");
    project.write("src/grow.rs", "water\n");
    project.commit("Water the garden", "2026-10-04T00:00:00Z");
    project.write("src/grow.rs", "prune\n");
    project.commit("An older dated descendant", "2026-10-02T00:00:00Z");
    let output = project.changes("UTC");
    assert!(output.contains("Water the garden"), "{output}");
    assert!(!output.contains("An older dated descendant"), "{output}");
}

#[test]
fn midnight_in_the_local_time_zone_is_inclusive() {
    let project = Project::new("");
    project.commit("Initial garden", "2026-09-01T00:00:00Z");
    project.write("src/grow.rs", "before\n");
    project.commit("Before local midnight", "2026-10-03T06:59:59Z");
    project.write("src/grow.rs", "at\n");
    project.commit("At local midnight", "2026-10-03T07:00:00Z");
    let output = project.changes("America/Los_Angeles");
    assert!(output.contains("At local midnight"), "{output}");
    assert!(!output.contains("Before local midnight"), "{output}");
}

impl Project {
    // `roostr check` with its exit code; findings make it exit 1, so success is not asserted.
    fn check(&self, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_roostr"))
            .current_dir(&self.directory)
            .arg("check")
            .args(args)
            .env("ROOSTR_TODAY", "2026-10-10")
            .env("TZ", "UTC")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        (
            output.status.code().unwrap(),
            String::from_utf8(output.stdout).unwrap(),
        )
    }

    // A main branch with the garden committed, and a feature branch started from it.
    fn branched(&self) {
        self.commit("Initial garden", "2026-09-01T00:00:00Z");
        self.git(&["branch", "-M", "main"]);
        self.git(&["checkout", "-q", "-b", "feature"]);
    }
}

#[test]
fn merging_main_into_a_branch_does_not_make_mains_changes_the_branchs_work() {
    let project = Project::new("");
    project.branched();
    project.git(&["checkout", "-q", "main"]);
    project.write("src/grow.rs", "changed on main\n");
    project.commit("Grow faster", "2026-09-02T00:00:00Z");
    project.git(&["checkout", "-q", "feature"]);
    project.write("src/other.rs", "branch work\n");
    project.commit("Branch work", "2026-09-03T00:00:00Z");
    project.git(&["merge", "-q", "--no-edit", "main"]);
    // The record is behind src/grow.rs, but main made that change, not the branch.
    let (code, output) = project.check(&[]);
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("    grow\n      src/grow.rs\n"), "{output}");
    assert_eq!(
        project.check(&["--base", "main"]),
        (0, "No problems found\n".to_owned())
    );
}

#[test]
fn a_merge_that_resolves_a_conflict_in_a_listed_file_is_the_branchs_work() {
    let project = Project::new("");
    project.branched();
    project.write("src/grow.rs", "branch version\n");
    project.commit("Grow on the branch", "2026-09-02T00:00:00Z");
    project.git(&["checkout", "-q", "main"]);
    project.write("src/grow.rs", "main version\n");
    project.commit("Grow on main", "2026-09-03T00:00:00Z");
    project.git(&["checkout", "-q", "feature"]);
    let merge = project
        .git_command()
        .args(["merge", "-q", "--no-edit", "main"])
        .output()
        .unwrap();
    assert!(!merge.status.success(), "the merge should conflict");
    project.write("src/grow.rs", "both versions\n");
    project.commit("Merge main", "2026-09-04T00:00:00Z");
    // Only the merge, which differs from both parents, touched src/grow.rs after main.
    let (code, output) = project.check(&["--base", "main~0"]);
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("    grow\n      src/grow.rs\n"), "{output}");
}

#[test]
fn a_record_updated_on_a_branch_has_not_seen_a_change_main_made_meanwhile() {
    let project = Project::new("");
    project.branched();
    project.write(
        ".roostr/processes/grow.yml",
        "name: Grow\ndescription: Grow a garden faster.\nfiles:\n  - src/grow.rs\n",
    );
    project.commit("Describe faster growing", "2026-09-02T00:00:00Z");
    project.git(&["checkout", "-q", "main"]);
    project.write("src/grow.rs", "changed on main\n");
    project.commit("Grow faster", "2026-09-03T00:00:00Z");
    project.git(&["merge", "-q", "--no-edit", "feature"]);
    // The merge brings the record in unchanged from the branch, so it does not count as
    // an update made after main's change.
    let (code, output) = project.check(&[]);
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("    grow\n      src/grow.rs\n"), "{output}");
    let overview = project.roostr(&project.directory, &[], "UTC");
    assert!(
        overview.contains("src/grow.rs changed since this record was updated"),
        "{overview}"
    );
}

#[test]
fn check_with_nothing_changed_runs_no_scan_and_never_writes_the_index() {
    let project = Project::new("app");
    let marker = project.root.join("scan-ran");
    project.write(
        ".roostr/scan",
        &format!(
            "#!/bin/sh\ntouch '{}'\nprintf '# covers: cli\\ncli\\tgrow\\tsrc/grow.rs\\n'\n",
            marker.display()
        ),
    );
    fs::set_permissions(
        project.directory.join(".roostr/scan"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    project.commit("Initial garden", "2026-09-01T00:00:00Z");
    // A change outside the project folder is not the project's.
    fs::write(project.root.join("README.md"), "elsewhere\n").unwrap();
    // Make the index's stat data stale so a refreshing command would rewrite it.
    project.write("src/grow.rs", "initial\n");
    let index = fs::read(project.root.join(".git/index")).unwrap();
    assert_eq!(
        project.check(&["--base", "HEAD"]),
        (0, "No problems found\n".to_owned())
    );
    assert!(!marker.exists());
    project.write("src/grow.rs", "watered\n");
    let (code, output) = project.check(&["--base", "HEAD"]);
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("    grow\n      src/grow.rs\n"), "{output}");
    assert!(marker.exists());
    assert_eq!(index, fs::read(project.root.join(".git/index")).unwrap());
}
