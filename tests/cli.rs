//! End-to-end tests: run the real binary against real rsync on temp dirs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Env {
    root: PathBuf,
    config: PathBuf,
}

impl Env {
    fn new(test: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(test);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let config = root.join("profiles.toml");
        Env { root, config }
    }

    /// Creates a directory under the test root with the given files
    fn dir(&self, name: &str, files: &[(&str, &str)]) -> PathBuf {
        let d = self.root.join(name);
        fs::create_dir_all(&d).unwrap();
        for (f, content) in files {
            fs::write(d.join(f), content).unwrap();
        }
        d
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ez-sync"))
            .arg("--config")
            .arg(&self.config)
            .args(args)
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
    }

    /// Adds a profile; `local` and `remote` are used verbatim
    fn add(&self, name: &str, local: &str, remote: &str) {
        let out = self.run(&["add", name, local, remote]);
        assert!(out.status.success(), "add failed: {}", stderr(&out));
    }
}

fn slash(p: &Path) -> String {
    format!("{}/", p.display())
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn read(p: impl AsRef<Path>) -> String {
    fs::read_to_string(p).unwrap()
}

#[test]
fn push_copies_files_and_exits_zero() {
    let e = Env::new("push_ok");
    let local = e.dir("local", &[("a.txt", "hello")]);
    let remote = e.dir("remote", &[]);
    e.add("p", &slash(&local), &slash(&remote));

    let out = e.run(&["push", "p"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(read(remote.join("a.txt")), "hello");
}

#[test]
fn pull_copies_files_back() {
    let e = Env::new("pull_ok");
    let local = e.dir("local", &[]);
    let remote = e.dir("remote", &[("b.txt", "from remote")]);
    e.add("p", &slash(&local), &slash(&remote));

    let out = e.run(&["pull", "p"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(read(local.join("b.txt")), "from remote");
}

#[test]
fn sync_deletes_files_missing_from_source() {
    let e = Env::new("deletes_extras");
    let local = e.dir("local", &[("keep.txt", "k")]);
    let remote = e.dir("remote", &[("keep.txt", "old"), ("stale.txt", "x")]);
    e.add("p", &slash(&local), &slash(&remote));

    assert!(e.run(&["push", "p"]).status.success());
    assert_eq!(read(remote.join("keep.txt")), "k");
    assert!(!remote.join("stale.txt").exists());
}

#[test]
fn missing_source_fails_with_nonzero_exit_and_stderr() {
    let e = Env::new("missing_source");
    let remote = e.dir("remote", &[("untouched.txt", "u")]);
    let missing = e.root.join("does-not-exist");
    e.add("p", &slash(&missing), &slash(&remote));

    let out = e.run(&["push", "p"]);
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("[p]"), "{err}");
    assert!(err.contains("rsync failed (exit 23)"), "{err}");
    assert!(remote.join("untouched.txt").exists());
}

#[test]
fn unreachable_remote_fails_on_push_and_pull() {
    let e = Env::new("unreachable");
    let local = e.dir("local", &[("a.txt", "a")]);
    e.add("p", &slash(&local), "nobody@ez-sync-test.invalid:/x/");

    for cmd in ["push", "pull"] {
        let out = e.run(&[cmd, "p"]);
        assert_eq!(out.status.code(), Some(1), "{cmd}: {}", stderr(&out));
        assert!(stderr(&out).contains("rsync failed"), "{cmd}");
    }
}

#[test]
fn all_syncs_good_profiles_and_reports_only_failures() {
    let e = Env::new("all_mixed");
    let good_l = e.dir("good_l", &[("g.txt", "good")]);
    let good_r = e.dir("good_r", &[]);
    let bad_r = e.dir("bad_r", &[]);
    e.add("good", &slash(&good_l), &slash(&good_r));
    e.add("bad", &slash(&e.root.join("nope")), &slash(&bad_r));

    let out = e.run(&["push", "all"]);
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("[bad]"), "{err}");
    assert!(!err.contains("[good] rsync failed"), "{err}");
    assert_eq!(read(good_r.join("g.txt")), "good");
}

#[test]
fn parent_profile_syncs_every_child() {
    let e = Env::new("parent_children");
    let a_l = e.dir("a_l", &[("a.txt", "a")]);
    let b_l = e.dir("b_l", &[("b.txt", "b")]);
    let a_r = e.dir("a_r", &[]);
    let b_r = e.dir("b_r", &[]);
    e.add("p.a", &slash(&a_l), &slash(&a_r));
    e.add("p.b", &slash(&b_l), &slash(&b_r));

    let out = e.run(&["push", "p"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(read(a_r.join("a.txt")), "a");
    assert_eq!(read(b_r.join("b.txt")), "b");
}

#[test]
fn warns_when_source_lacks_trailing_slash() {
    let e = Env::new("warn_no_slash");
    let local = e.dir("local", &[("a.txt", "a")]);
    let remote = e.dir("remote", &[]);
    e.add("p", &local.display().to_string(), &slash(&remote));

    let out = e.run(&["push", "p"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stderr(&out).contains("no trailing slash"));
    // the behaviour the warning describes: directory nested, not mirrored
    assert!(remote.join("local/a.txt").exists());
}

#[test]
fn no_warning_with_trailing_slash() {
    let e = Env::new("no_warn_slash");
    let local = e.dir("local", &[("a.txt", "a")]);
    let remote = e.dir("remote", &[]);
    e.add("p", &slash(&local), &slash(&remote));

    let out = e.run(&["push", "p"]);
    assert!(out.status.success());
    assert!(!stderr(&out).contains("trailing slash"));
}

#[test]
fn warning_checks_the_source_side_per_direction() {
    let e = Env::new("warn_direction");
    let local = e.dir("local", &[]);
    let remote = e.dir("remote", &[("r.txt", "r")]);
    // slash on local only: push is fine, pull has a slash-less source
    e.add("p", &slash(&local), &remote.display().to_string());

    assert!(!stderr(&e.run(&["push", "p"])).contains("trailing slash"));
    assert!(stderr(&e.run(&["pull", "p"])).contains("trailing slash"));
}

#[test]
fn add_list_remove_roundtrip() {
    let e = Env::new("crud");
    e.add("p.a", "/l/a/", "/r/a/");

    let list = stdout(&e.run(&["list"]));
    assert!(list.contains("p.a") && list.contains("/l/a/"), "{list}");

    let out = e.run(&["remove", "p.a"]);
    assert!(out.status.success());
    assert!(!stdout(&e.run(&["list"])).contains("p.a"));
}

#[test]
fn reserved_name_all_is_rejected() {
    let e = Env::new("reserved_all");
    assert!(!e.run(&["add", "all", "/l/", "/r/"]).status.success());
    assert!(!e.run(&["remove", "all"]).status.success());
}

#[test]
fn unknown_profile_fails() {
    let e = Env::new("unknown_profile");
    let out = e.run(&["push", "ghost"]);
    assert!(!out.status.success());
}
