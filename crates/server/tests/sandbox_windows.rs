//! What a session on Windows can reach, asked by running a probe inside a
//! Conversation's own AppContainer.
//!
//! The third arm of `tests/sandbox.rs` and `tests/sandbox_macos.rs`, settled the
//! same way both of those are: **nothing here reads what the rendering wrote.**
//! The access-control entries *are* what is being tested, and a test that read
//! one back would be asserting itself — it would go on passing while Windows
//! changed what an entry meant, or while an inherited allow quietly outranked a
//! deny somewhere above it. What settles whether the rest of the machine is
//! still reachable is a program inside the container attempting each access and
//! saying what the operating system said.
//!
//! **The vocabulary is the Mac's** — `write`, `read`, `refused`, `absent` — and
//! for the Mac's reason (ADR-0012). A path a session may not reach on Linux is
//! `absent`, because the mount namespace it is in never had one; the same path
//! here is `refused`, really there and every open of it denied. So a probe that
//! finds `absent` on this platform has found something odd, and one that finds
//! `refused` has found the boundary working. The one place `absent` is right is
//! a name nobody ever made, and this suite asks for one on purpose: the two look
//! identical to anything coarser, and only one of them is a boundary.
//!
//! **Two of the vocabulary's kinds name no path here, so there is nothing to
//! attempt.** `ProcessTable` is a directory on the platform that keeps one in
//! the filesystem and nothing on this one, and `Devices` is the machine's own —
//! neither comes to an access-control entry at all, which is a claim about what
//! a description comes to rather than about what a container can open, and is
//! held where the entries are worked out (see the server's `sandbox::granting`).
//! Every kind that does name a path is attempted below.
//!
//! **What the probe says, it says on one short line.** A console is a grid and
//! what comes off a pseudoconsole is a drawing of one, so a path printed on it
//! is a path with a line break in the middle — which is why
//! `tests/sessions_windows.rs` reads its evidence off files. Nothing here is on
//! a console at all: [`off_a_console`] hands the probe three pipes, so what it
//! prints arrives as it was written. The lines are `name=word` all the same,
//! because a word is the whole of what a classification is and the paths are
//! this file's already.
//!
//! **What this costs the `windows-2025` job.** Nothing here waits on anything:
//! [`off_a_console`] blocks until the probe has exited, so there is no deadline
//! to run out of and no `PATIENCE` to raise — unlike
//! `tests/sessions_windows.rs`, whose every assertion is a poll. What the suite
//! costs the job is therefore wall time and nothing else, and each test's is one
//! AppContainer profile created and deleted, the entries of one description
//! written and taken off, and one Windows PowerShell started inside. Measured on
//! the job: *(unmeasured — the first `windows-2025` run of this branch is what
//! fills this in; see the pull request the stage finishes with)*.
//!
//! Windows only, which is where "everywhere" stops for this one: what it starts
//! is an AppContainer, and the two Unixes have no such call to make.
#![cfg(windows)]

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use verkstead_server::attachments::Attachments;
use verkstead_server::build_cache::BuildCache;
use verkstead_server::handoffs::Handoffs;
use verkstead_server::platform::Platform;
use verkstead_server::sandbox::{Bind, Executable, Homes, Reachable, Sandbox, off_a_console};
use verkstead_server::settings::Settings;
use verkstead_server::skills::Skills;
use verkstead_server::store;

/// Where the server this Conversation belongs to would be listening, which is
/// what a session inside is told to put its Question Sets to.
///
/// Nothing listens on it and nothing here asks: what a Windows session really
/// asks through is the named pipe, and `crates/cli/tests/sandbox_windows.rs` is
/// where that is proved end to end.
const LISTENING: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8422);

/// The shell every Windows machine has, which is what the probe is written in.
///
/// Named without a path and without an extension, because resolving it is part
/// of what is being asked: a session's `PATH` is the description's, and a probe
/// that named an image outright would have proved the boundary without ever
/// proving that a session can find a program at all.
const POWERSHELL: &str = "powershell";

/// And how it is started: the script is a file rather than a command line, for
/// the reason `tests/sessions_windows.rs` starts its stand-in that way —
/// `-File` hands what follows to the script rather than appending it, and
/// nothing between here and there re-reads the quoting.
///
/// `-ExecutionPolicy Bypass` because the script is one this test wrote a moment
/// ago, which is not a thing a machine's signing policy has anything useful to
/// say about.
const HOW: [&str; 4] = [
    "-NoProfile",
    "-NonInteractive",
    "-ExecutionPolicy",
    "Bypass",
];

/// What the probe is called inside the Worktree it is written into.
///
/// The Worktree because it is the one directory every description grants and
/// every session starts in — a probe written anywhere else would need a reach
/// of its own described before it could ask what the description reaches.
const PROBE: &str = "verkstead-boundary-probe.ps1";

/// What each fixture leaves in a directory for the probe to read, so that a
/// `read` is a file really opened rather than an empty directory listed.
const MARKER: &str = "what-is-in-here.txt";

/// And what is in it, which nothing asserts: what is being asked is whether the
/// bytes could be reached at all.
const SAID: &str = "the description named this directory\n";

/// The account's own skills, which a description names in order to say a session
/// finds nothing at them.
const THEIR_SKILL: &str = "# what the account would have been grilled by\n";

/// And what stands in for the server's own image: a file that is a file, which
/// is the whole of what [`Executable::at`] asks of one.
///
/// Never run — what a session does with the binary it asks with is
/// `crates/cli/tests/sandbox_windows.rs`'s to prove, that being the crate with
/// a real one to hand. What it is here is a path the description grants
/// read-only, and the one file in a directory it grants nothing else of.
const SAYS_WHICH_BUILD: &str = "the server's own image would be here\n";

/// The probe itself: what a classification is, in this platform's own words.
///
/// **Attempted rather than asked of the metadata.** A read-only grant and a
/// directory the account simply has no write permission on are the same answer
/// to anything that only looks; only one of them is the surface being
/// described. So a directory is classified by creating a file in it, and then —
/// where that was refused — by listing it; and a file by opening it for writing,
/// and then by reading it.
///
/// **And absence is told from refusal by the exception's own name**, which is
/// the one thing this platform makes easy and the one thing the whole suite
/// turns on. `UnauthorizedAccessException` is the boundary — what .NET makes of
/// the `ERROR_ACCESS_DENIED` a refused open comes back with — and a
/// `FileNotFoundException` or a `DirectoryNotFoundException` is a name nobody
/// ever made.
///
/// **Anything else is reported as itself, with the machine's own number after
/// it.** A word this file does not know about is a failure worth reading rather
/// than one worth guessing at, and the `HResult` behind it is the Win32 error
/// the platform really gave — which is what tells a boundary this suite has
/// misread from an open that failed for some reason of its own. `.NET` on a
/// Unix, for one, reports a denied open as a plain `IOException`; nothing here
/// runs there, and a reader who ends up with one should be told which it was.
///
/// The innermost exception is the one asked, because a .NET method called from
/// PowerShell arrives wrapped.
const CLASSIFYING: &str = r#"
$ErrorActionPreference = 'Stop'

function Innermost($caught) {
    $why = $caught.Exception
    while ($why.InnerException) { $why = $why.InnerException }
    return $why
}

function Why($caught) { return (Innermost $caught).GetType().Name }

function Unexpected($caught) {
    $why = Innermost $caught
    return ($why.GetType().Name + '/' + ('0x{0:x8}' -f $why.HResult))
}

function Report($name, $said) { [Console]::Out.WriteLine($name + '=' + $said) }

function Directory($path) {
    $probe = Join-Path $path '.verkstead-probe'

    try {
        $handle = [System.IO.File]::Open($probe, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write)
        $handle.Close()
        [System.IO.File]::Delete($probe)
        return 'write'
    } catch {
        $why = Why $_
        if ($why -eq 'DirectoryNotFoundException') { return 'absent' }
        if ($why -ne 'UnauthorizedAccessException') { return (Unexpected $_) }
    }

    try {
        [System.IO.Directory]::GetFileSystemEntries($path) | Out-Null
        return 'read'
    } catch {
        $why = Why $_
        if ($why -eq 'DirectoryNotFoundException') { return 'absent' }
        if ($why -eq 'UnauthorizedAccessException') { return 'refused' }
        return (Unexpected $_)
    }
}

function File($path) {
    try {
        $handle = [System.IO.File]::Open($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Write)
        $handle.Close()
        return 'write'
    } catch {
        $why = Why $_
        if ($why -eq 'FileNotFoundException') { return 'absent' }
        if ($why -eq 'DirectoryNotFoundException') { return 'absent' }
        if ($why -ne 'UnauthorizedAccessException') { return (Unexpected $_) }
    }

    try {
        [System.IO.File]::ReadAllBytes($path) | Out-Null
        return 'read'
    } catch {
        $why = Why $_
        if ($why -eq 'FileNotFoundException') { return 'absent' }
        if ($why -eq 'DirectoryNotFoundException') { return 'absent' }
        if ($why -eq 'UnauthorizedAccessException') { return 'refused' }
        return (Unexpected $_)
    }
}
"#;

/// One thing the probe is asked about: what to call it, where it is, and which
/// of the two shapes it has.
///
/// The shape is the caller's rather than something the probe works out, and
/// deliberately: asking the filesystem what is at a path is exactly the question
/// a refused path will not answer, so a probe that decided for itself would
/// classify every refusal as whichever shape it guessed.
struct Asked {
    name: &'static str,
    path: PathBuf,
    directory: bool,
}

/// A directory the probe is to classify.
fn directory(name: &'static str, path: impl Into<PathBuf>) -> Asked {
    Asked {
        name,
        path: path.into(),
        directory: true,
    }
}

/// And a file.
fn file(name: &'static str, path: impl Into<PathBuf>) -> Asked {
    Asked {
        name,
        path: path.into(),
        directory: false,
    }
}

/// A Conversation part-way through its first grilling, with one of everything a
/// description can name laid out on a real filesystem.
///
/// Everything is real, for the reason the other two suites' fixtures are: what
/// the description says is read off these paths, and a fixture that hand-built
/// them would prove the probe works rather than that the boundary does.
struct Grilling {
    /// Kept alive for as long as the fixture is: the directories go when these
    /// drop, and a Worktree that vanished mid-probe would fail obscurely.
    _watched: tempfile::TempDir,
    state: tempfile::TempDir,
    home: tempfile::TempDir,

    /// Where the Repo is, and the checkout beside it that no session has any
    /// business seeing.
    repo: PathBuf,
    sibling: PathBuf,

    /// Where the Profile's account is on the host, which is the far end of the
    /// junction a session finds it through.
    account: PathBuf,

    conversation: store::Conversation,
    profile: store::Profile,

    skills: Skills,
    verkstead: Executable,
    handoffs: Handoffs,
    attachments: Attachments,
    settings: Settings,

    /// The two directories Sandbox Configuration was told to add, which is how
    /// an `Own` at each reach gets into a description without being one of the
    /// things a session always has.
    writable: PathBuf,
    readable: PathBuf,
}

impl Grilling {
    /// The sandbox this Conversation's session runs in.
    fn sandbox(&self) -> Sandbox {
        Sandbox::for_conversation(
            &self.conversation,
            &self.profile,
            &self.homes(),
            &Reachable::at(LISTENING),
            &self.skills,
            &self.verkstead,
            &self.handoffs,
            &self.attachments,
            &self.settings.secrets(),
            &self.settings.config(),
            // Nothing is compiled inside a probe, so there is no cache for one
            // to compile into: the shared build cache is a directory granted
            // read-write like any other, and the two below are what say what
            // that comes to.
            &BuildCache::none(),
            vec![
                Bind::writable(self.writable.clone()),
                Bind::readable(self.readable.clone()),
            ],
        )
        .expect("a grilling Conversation has a worktree to build a sandbox around")
    }

    /// The homes this server hands out: the fixture's own directory where the
    /// human's profile is, and its state directory to make a session's own
    /// under.
    ///
    /// The two are not the same thing, and half of what this suite asks turns on
    /// that: the first is the human's own, which a session is refused, and a
    /// session's `USERPROFILE` is a directory of Verkstead's made fresh under
    /// the second.
    fn homes(&self) -> Homes {
        Homes::on(
            Platform::HERE,
            self.home.path().to_owned(),
            self.state.path(),
        )
    }

    fn worktree(&self) -> &Path {
        self.conversation
            .worktree
            .as_deref()
            .expect("a grilling Conversation has a worktree")
    }

    /// The Repo's own git directory, which is what a commit inside writes to.
    fn git_dir(&self) -> PathBuf {
        self.repo.join(".git")
    }

    /// The profile a session is given, which is what it reads `USERPROFILE` as.
    fn profile_dir(&self) -> PathBuf {
        self.state
            .path()
            .join("homes")
            .join(self.conversation.id.to_string())
    }

    /// This Conversation's handoff directory as a session finds it: under the
    /// profile, there being no mount on this platform to put it anywhere else.
    fn handoffs_inside(&self) -> PathBuf {
        self.profile_dir().join("verkstead")
    }

    /// And where the files the human attached to it are, which on this platform
    /// is also the path a session reads them at.
    fn attachments_dir(&self) -> PathBuf {
        self.state
            .path()
            .join("attachments")
            .join(self.conversation.id.to_string())
    }

    /// The human's own Documents, which nothing in a description ever names.
    fn documents(&self) -> PathBuf {
        self.home.path().join("Documents")
    }

    /// Run the probe inside this Conversation's container and hand back the
    /// `name=word` lines it printed.
    ///
    /// **The container is made here rather than by this file**, which is the
    /// point: [`Sandbox::command`] is what a session start calls, so what is
    /// probed is the boundary a session really gets rather than one this test
    /// assembled beside it. The `Closing` is held for as long as the probe runs
    /// and dropped after it, which is what takes the entries off the human's own
    /// directories and deletes the profile again.
    ///
    /// **A probe that did not run says everything it can about why.** The only
    /// machines that enforce this boundary are the `windows-2025` runner and
    /// whoever is reading this on a Windows box, so what a failure here can be
    /// diagnosed from is what this prints: how the process ended, what it said
    /// on either stream, and which program the description resolved.
    fn probe(&self, asked: &[Asked]) -> BTreeMap<String, String> {
        let script = self.worktree().join(PROBE);
        std::fs::write(&script, classifying(asked)).expect("the Worktree to be writable out here");

        let sandbox = self.sandbox();
        let mut argv: Vec<String> = vec![POWERSHELL.to_owned()];
        argv.extend(HOW.iter().map(|word| (*word).to_owned()));
        argv.push("-File".to_owned());
        argv.push(script.display().to_string());

        let (rendering, closing) = sandbox
            .command(&argv)
            .expect("this machine to make the AppContainer a session runs inside");

        let output = off_a_console(&rendering, b"").expect("the probe to start inside it");

        closing.close();

        let printed = String::from_utf8_lossy(&output.stdout).into_owned();

        assert!(
            output.status.success(),
            "the probe did not run inside the container.\n\
             how it ended: {}\n\
             what it printed: {printed:?}\n\
             what it complained: {:?}\n\
             the program the description resolved: {:?}\n\
             the container it was started inside: {:?}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            rendering.program(),
            rendering.container(),
        );

        printed
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(name, said)| (name.trim().to_owned(), said.trim().to_owned()))
            .collect()
    }
}

/// The whole script the probe runs: how a classification is made, and then one
/// line per thing this test wants classified.
fn classifying(asked: &[Asked]) -> String {
    let mut script = CLASSIFYING.to_owned();

    for one in asked {
        script.push_str(&format!(
            "Report '{}' ({} '{}')\r\n",
            one.name,
            if one.directory { "Directory" } else { "File" },
            one.path.display().to_string().replace('\'', "''"),
        ));
    }

    script
}

/// Stand a Conversation up, with one of everything a description names really
/// on the disk.
async fn grilling() -> Grilling {
    let watched = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    // The human's own profile, with something in it that is nobody else's: this
    // is what stands where `C:\Users\<them>` does, and "their Documents are
    // refused" is only a claim if there is something in there to refuse.
    std::fs::create_dir_all(home.path().join("Documents")).unwrap();
    std::fs::write(
        home.path().join("Documents").join(MARKER),
        "the human's own\n",
    )
    .unwrap();

    let repo = repository(watched.path().join("verkstead"));
    let sibling = repository(watched.path().join("something-else"));

    let pool = store::open_database(&state.path().join("verkstead.db"))
        .await
        .unwrap();

    let repo_row = store::register_repo(&pool, &repo, "verkstead", "main")
        .await
        .unwrap()
        .expect("the Repo registers");

    // The account a session runs under: the pair of files a Claude Profile is,
    // with skills of the account's own inside the directory half. Really there,
    // so that a probe finding them refused has found the rule covering them
    // rather than an empty name.
    let account = watched.path().join("account");
    let claude_dir = account.join(".claude");
    let config_file = account.join(".claude.json");
    std::fs::create_dir_all(claude_dir.join("skills")).unwrap();
    std::fs::write(claude_dir.join(MARKER), SAID).unwrap();
    std::fs::write(claude_dir.join("skills").join("theirs.md"), THEIR_SKILL).unwrap();
    std::fs::write(&config_file, "{}\n").unwrap();

    let profile = store::create_profile(
        &pool,
        &store::ProfileFacts {
            name: "work".to_owned(),
            account: store::Account::Claude {
                claude_dir,
                config_file,
            },
            models: vec!["claude-opus-5".to_owned()],
        },
    )
    .await
    .unwrap()
    .expect("the Profile saves");

    let id = store::start_conversation(&pool, repo_row.id, "rate-limiting")
        .await
        .unwrap()
        .expect("the Conversation starts");

    store::set_grilling_pairing(&pool, id, profile.id, profile.model())
        .await
        .unwrap();

    // The worktree git itself made, where the server puts one.
    let worktree = state.path().join("worktrees/verkstead-rate-limiting");
    std::fs::create_dir_all(worktree.parent().unwrap()).unwrap();
    let commit = git(&repo, &["rev-parse", "HEAD"]).trim().to_owned();
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-b",
            "rate-limiting",
            &worktree.to_string_lossy(),
            &commit,
        ],
    );

    store::start_grilling(&pool, id, &commit, &worktree, &[])
        .await
        .unwrap();

    let conversation = store::load_conversation(&pool, id)
        .await
        .unwrap()
        .expect("the Conversation is there");

    let skills =
        Skills::installed(Platform::HERE, state.path()).expect("this binary carries skills");
    let handoffs = Handoffs::under(state.path());
    let attachments = Attachments::under(state.path());
    let settings = Settings::in_data_dir(state.path());

    // A file the human attached, which is what makes the attachments directory
    // one the description names at all.
    let attached = state.path().join("attachments").join(id.to_string());
    std::fs::create_dir_all(&attached).unwrap();
    std::fs::write(attached.join(MARKER), SAID).unwrap();

    // And the handoff directory, made the way a sandbox makes one — asked for
    // here so that the marker below has somewhere to land.
    let handoff_dir = handoffs
        .directory(id)
        .expect("the handoff directory to be one this machine can make");
    std::fs::write(handoff_dir.join(MARKER), SAID).unwrap();

    // The server's own image, in a directory holding one other file: what the
    // description grants is the image, and the file beside it is how this suite
    // asks whether anything else in that directory came with it.
    let image = state.path().join("image");
    std::fs::create_dir_all(&image).unwrap();
    std::fs::write(image.join("verkstead.exe"), SAYS_WHICH_BUILD).unwrap();
    std::fs::write(
        image.join("beside-the-image.txt"),
        "the host put this here\n",
    )
    .unwrap();

    let verkstead = Executable::at(Platform::HERE, image.join("verkstead.exe"), state.path())
        .expect("the image was just written");

    // And the two Sandbox Configuration was told to add, one at each reach.
    let writable = watched.path().join("a-configured-bind");
    let readable = watched.path().join("a-configured-read-only-bind");

    for bind in [&writable, &readable] {
        std::fs::create_dir_all(bind).unwrap();
        std::fs::write(bind.join(MARKER), SAID).unwrap();
    }

    Grilling {
        _watched: watched,
        state,
        home,
        repo,
        sibling,
        account,
        conversation,
        profile,
        skills,
        verkstead,
        handoffs,
        attachments,
        settings,
        writable,
        readable,
    }
}

/// A git repository at `path`, with one commit on `main`.
fn repository(path: PathBuf) -> PathBuf {
    std::fs::create_dir_all(&path).unwrap();
    git(&path, &["init", "--initial-branch", "main"]);
    git(&path, &["config", "user.email", "local@verkstead.invalid"]);
    git(&path, &["config", "user.name", "Whatever The Repo Says"]);
    std::fs::write(path.join(MARKER), SAID).unwrap();
    git(&path, &["add", MARKER]);
    git(&path, &["commit", "-m", "first"]);

    path
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("git should be on the PATH for these tests");

    assert!(
        output.status.success(),
        "git {args:?} failed in {}",
        dir.display()
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// What the probe said about `name`, or a failure naming everything it did say.
fn said<'a>(classified: &'a BTreeMap<String, String>, name: &str) -> &'a str {
    classified
        .get(name)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("the probe said nothing about {name}. It said: {classified:?}"))
}

/// Every access kind a description can name, classified by attempting it — and
/// each of them what the description said it would be.
///
/// **One container and one probe**, which is not a shortcut: these are one
/// description, and a suite that stood a container up per path would be asking
/// one question a dozen times over on the runner this stage has to fit inside.
/// What is asserted is a path at a time all the same, so a failure names the one
/// that moved.
///
/// The kinds, in the order a session's own description says them: the profile it
/// is given and the two halves and the temporary directory inside it, which are
/// made rather than reached; its Worktree and the git directory behind it; the
/// account joined in by junction and the file half by hard link; the handoff
/// directory; the skills, the attached files and the image it asks with,
/// read-only; and the two Sandbox Configuration added, one at each reach.
#[tokio::test]
async fn every_access_kind_is_classified_as_the_description_said() {
    let fixture = grilling().await;
    let profile = fixture.profile_dir();

    let classified = fixture.probe(&[
        directory("profile", &profile),
        directory("roaming", profile.join("AppData").join("Roaming")),
        directory("local", profile.join("AppData").join("Local")),
        directory("temp", profile.join("AppData").join("Local").join("Temp")),
        directory("worktree", fixture.worktree()),
        directory("git", fixture.git_dir()),
        directory("account", profile.join(".claude")),
        file("config", profile.join(".claude.json")),
        directory("handoffs", fixture.handoffs_inside()),
        directory("bind", &fixture.writable),
        directory("skills", fixture.skills.path()),
        directory("attachments", fixture.attachments_dir()),
        directory("readable-bind", &fixture.readable),
        file("verkstead", fixture.verkstead.path()),
    ]);

    for (name, what) in [
        ("profile", "the profile a session is given, made fresh"),
        ("roaming", "the roaming half of it"),
        ("local", "the local half"),
        ("temp", "what a session throws away, inside that half"),
        ("worktree", "the Conversation's own checkout"),
        ("git", "the Repo's git directory behind it"),
        ("account", "the Profile's account, through the junction"),
        ("config", "the file half of it, through the hard link"),
        (
            "handoffs",
            "the Conversation's own directory outside the worktree",
        ),
        ("bind", "what Sandbox Configuration asked for read-write"),
    ] {
        assert_eq!(
            said(&classified, name),
            "write",
            "{what} is read-write in the description, and the probe said: {classified:?}",
        );
    }

    for (name, what) in [
        ("skills", "the skills a session is grilled by"),
        ("attachments", "the files the human attached"),
        (
            "readable-bind",
            "what Sandbox Configuration asked for read-only",
        ),
        ("verkstead", "the image a session asks with"),
    ] {
        assert_eq!(
            said(&classified, name),
            "read",
            "{what} is read-only in the description — reachable and not \
             writable, which is two claims and this is both of them. The probe \
             said: {classified:?}",
        );
    }

    // And from the host, which is the other half of what a `write` on the
    // account means: what a session's profile holds is a *name* for the account
    // rather than a copy of it, so what it writes through that name is written
    // where the Profile said and is still there when the session has gone.
    assert!(
        std::fs::symlink_metadata(profile.join(".claude"))
            .expect("the account is joined into the profile")
            .is_symlink(),
        "what a session finds its account at is the junction the rendering made, \
         a reparse point reading as a link — a copy would have taken the write \
         with it when the profile was emptied",
    );
    assert!(
        fixture.account.join(".claude").join(MARKER).is_file(),
        "and the account itself is untouched, where the Profile said it is",
    );
}

/// And what the description does not name is refused rather than absent, with
/// a name nobody ever made told apart from both.
///
/// The whole point of there being a boundary at all, and the one thing a coarser
/// test would pass without: the human's own Documents, which no description
/// mentions; another checkout on the same machine, which is somebody else's
/// work; Verkstead's own record of every Conversation, which is nobody's
/// business inside; the account's own skills, which a description mentions in
/// order to say a session finds nothing there; and the file the host left beside
/// the image, which says that what is granted on that `PATH` entry is the one
/// file rather than the directory holding it.
///
/// **`absent` is asked for on purpose.** A path that was never made and a path
/// that is there and wholly denied are the same answer to anything that only
/// looks, and a suite that could not tell them apart would pass just as happily
/// against a description that named nothing at all.
#[tokio::test]
async fn what_no_description_names_is_refused_and_a_name_nobody_made_is_absent() {
    let fixture = grilling().await;
    let profile = fixture.profile_dir();

    let classified = fixture.probe(&[
        directory("their-skills", profile.join(".claude").join("skills")),
        directory("documents", fixture.documents()),
        directory("sibling", &fixture.sibling),
        file("verksteads-own", fixture.state.path().join("verkstead.db")),
        file(
            "beside-the-image",
            fixture
                .verkstead
                .path()
                .parent()
                .expect("the image is in a directory")
                .join("beside-the-image.txt"),
        ),
        file("never-a-file", fixture.worktree().join("nobody-wrote-this")),
        directory(
            "never-a-directory",
            fixture.worktree().join("nobody-made-this"),
        ),
    ]);

    for (name, what) in [
        (
            "their-skills",
            "the account's own skills, which the description names as nothing \
             at all",
        ),
        (
            "documents",
            "the human's own Documents, which no description names",
        ),
        (
            "sibling",
            "another checkout on the machine, which is somebody else's work",
        ),
        (
            "verksteads-own",
            "Verkstead's own record of every Conversation",
        ),
        (
            "beside-the-image",
            "what the host left beside the image, the description having \
             granted the one file rather than the directory",
        ),
    ] {
        assert_eq!(
            said(&classified, name),
            "refused",
            "{what} is refused from inside — refused rather than absent, the \
             machine being there and denied. The probe said: {classified:?}",
        );
    }

    for name in ["never-a-file", "never-a-directory"] {
        assert_eq!(
            said(&classified, name),
            "absent",
            "a name nobody ever made, inside a directory the description grants, \
             is absent rather than refused — which is what says the refusals \
             above are about a boundary rather than about a machine with \
             nothing on it. The probe said: \
             {classified:?}",
        );
    }

    // And from the host, which is the other half of the same claim: what was
    // refused is still there and still says what it said. A boundary that
    // worked by taking something away would be no boundary.
    assert_eq!(
        std::fs::read_to_string(
            fixture
                .account
                .join(".claude")
                .join("skills")
                .join("theirs.md")
        )
        .expect("the account's own skills are the account's"),
        THEIR_SKILL,
    );
    assert!(
        fixture.documents().join(MARKER).is_file(),
        "and so are the human's own Documents",
    );
}
