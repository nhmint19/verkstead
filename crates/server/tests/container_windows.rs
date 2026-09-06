//! The AppContainer a Windows session runs inside, asked by attempting it: a
//! profile really made, a process really started with its identity, and the
//! machine's own answer about the token that process got.
//!
//! **Nothing here is about what a session may reach.** A container with no
//! access-control entries written for it reaches nothing at all, which is why
//! the probes below run out of the system directory and print rather than read:
//! what is being proved is the mechanism — the profile, the capability
//! attribute beside the pseudoconsole, the two ways of starting something
//! inside one — and the description that makes a real session work is the task
//! after this one's.
//!
//! **The containment is asked of the operating system rather than assumed.** A
//! `CreateProcessW` that took the attribute list and started the process
//! outside a container anyway would look exactly like one that worked, so what
//! says a process is inside is [`container::around`], which reads the SID off
//! the token Windows gave it.
//!
//! Everything made here is taken back: a [`Container`] deletes its profile as
//! it is dropped, and the tests that prove that is so are the ones that make a
//! profile twice under one name.
#![cfg(windows)]

use std::time::{Duration, Instant};

use verkstead_server::sandbox::container::{self, Container};
use verkstead_server::sandbox::{Rendering, off_a_console};
use verkstead_server::terminal::Terminal;

/// How long to wait for something a probe says.
///
/// Generously long, for the reason `terminal_windows.rs` is: what is being
/// waited on is a process starting, and a first `powershell.exe` inside a fresh
/// container on a cold runner is not quick.
const PATIENCE: Duration = Duration::from_secs(60);

/// The shell every Windows machine has that can print and then wait without
/// being typed at.
///
/// Windows PowerShell rather than `cmd`, because what the probe on a console
/// has to do is stay alive while its token is asked about, and every way `cmd`
/// waits is a way of reading the console — see
/// [`a_session_on_a_console_runs_inside_its_container`].
const POWERSHELL: &str = "powershell.exe";

/// And the one that sorts what it is given, which is how the start with nothing
/// watching it is asked both of its directions at once: what went in at the
/// standard input, and what came back out.
const SORT: &str = "sort.exe";

/// What the environment of a probe is: the names nothing on Windows runs
/// without.
///
/// Said rather than inherited, because a rendering is the whole of what a
/// process is handed — see [`Rendering`] — and a shell with no `SystemRoot` is
/// one that will not start. The test's own values, because what is being asked
/// here is the container rather than what a session may reach.
const NEEDED: [&str; 10] = [
    "ComSpec",
    "PATH",
    "PATHEXT",
    "SystemDrive",
    "SystemRoot",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
];

/// A session on a Verkstead pseudoconsole, inside a container: what it printed
/// reaches the Screen, its token carries the profile's SID, and the Job still
/// takes it away.
///
/// The three things this stage turns on, in one process because they are one
/// process: a container that could not print would be no session, a process
/// that printed but was outside its container would be no boundary, and a
/// session the Job had lost hold of would be one nothing could end.
#[tokio::test]
async fn a_session_on_a_console_runs_inside_its_container() {
    let held = tempfile::tempdir().expect("a directory to keep a Data Directory in");
    let container =
        Container::for_conversation(held.path(), 1).expect("this machine to make an AppContainer");

    let mut terminal = Terminal::open().expect("this machine has pseudoconsoles");

    let mut probe = probing(POWERSHELL);
    probe
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg("Write-Output 'inside'; Start-Sleep 600")
        .inside(container.sid());

    let child = terminal
        .spawn(&probe)
        .expect("a shell to run inside the container");

    let session = child.id().expect("a started session has a process id");

    let said = until(&terminal, |said| said.contains("inside")).await;

    assert_eq!(
        container::around(session).expect("the machine to say what a process is inside"),
        Some(container.sid().to_owned()),
        "the session's token should carry the profile's SID, and it said: {said:?}"
    );

    drop(child);

    assert!(
        gone(session).await,
        "a dropped child should have taken the session with it, and it said: {said:?}"
    );
}

/// And what one printed with nothing watching it, read back — with what it was
/// given at its standard input going the other way.
///
/// The start everything that is not a session stands on: the boundary suite,
/// the `verkstead ask` a session makes, the Compile Server. A `Command` cannot
/// make this process at all — see [`off_a_console`] — so what proves it is a
/// program inside a container reading what it was handed and printing what it
/// made of it.
#[test]
fn what_a_rendering_printed_off_a_console_is_read_back() {
    let held = tempfile::tempdir().expect("a directory to keep a Data Directory in");
    let container =
        Container::for_conversation(held.path(), 2).expect("this machine to make an AppContainer");

    let mut probe = probing(SORT);
    probe.inside(container.sid());

    let output = off_a_console(&probe, b"beta\r\nalpha\r\n").expect("a program inside a container");

    let printed = String::from_utf8_lossy(&output.stdout).into_owned();

    assert!(
        output.status.success(),
        "the probe should have exited cleanly, got {:?} having complained: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let (alpha, beta) = (printed.find("alpha"), printed.find("beta"));

    assert!(
        alpha.is_some() && alpha < beta,
        "what was put in should have come back sorted, and it said: {printed:?}"
    );
}

/// A profile Verkstead did not make is one it will not use, and a profile it
/// made goes when the thing holding it does.
///
/// Both halves of the same fact, asked with one name: while the first container
/// is held the name is taken and making it again says so, and once that
/// container has gone the name is free — which nothing but the profile really
/// being deleted would leave it.
#[test]
fn a_profile_that_is_taken_is_an_error_and_a_container_that_goes_frees_it() {
    let held = tempfile::tempdir().expect("a directory to keep a Data Directory in");

    let container =
        Container::for_conversation(held.path(), 3).expect("this machine to make an AppContainer");
    let name = container.name().to_owned();

    let refused = Container::named(&name)
        .expect_err("a profile that is already there should not be made a second time");

    assert!(
        refused.to_string().contains(&name),
        "the refusal should say which profile could not be made, and it said: {refused}"
    );

    drop(container);

    let again =
        Container::named(&name).expect("the profile to have gone with the container that made it");

    assert_eq!(again.name(), name);
}

/// And a container that cannot be made sense of refuses the process, rather
/// than starting it outside one.
///
/// Both ways of starting a rendering are asked, because both would otherwise
/// have somewhere to fall back to: what a session is refused is the whole of
/// ADR-0014's Q18, and a boundary that quietly is not there is worse than one
/// that was never promised.
#[tokio::test]
async fn a_container_that_will_not_resolve_refuses_the_process() {
    let mut probe = probing(POWERSHELL);
    probe
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg("exit 0")
        .inside("this is not a SID");

    let mut terminal = Terminal::open().expect("this machine has pseudoconsoles");

    // Matched rather than unwrapped for its error, because what a spawn hands
    // back on the way it does not go is a running session: there is nothing
    // here to print it with, and a test that started one would have to end it.
    let refused = match terminal.spawn(&probe) {
        Ok(_) => panic!("a session whose container will not resolve should be refused"),
        Err(refused) => refused,
    };

    assert!(
        refused.to_string().contains("this is not a SID"),
        "the refusal should say what could not be resolved, and it said: {refused}"
    );

    let refused =
        off_a_console(&probe, b"").expect_err("and the same rendering started off a console");

    assert!(
        refused.to_string().contains("this is not a SID"),
        "which should say the same thing, and it said: {refused}"
    );
}

/// What every probe here has in common: `program`, and the environment it takes
/// to run at all.
fn probing(program: &str) -> Rendering {
    let mut probe = Rendering::running(program);

    for name in NEEDED {
        if let Some(value) = std::env::var_os(name) {
            probe.set(name, value);
        }
    }

    probe
}

/// Read the terminal until what has arrived on it satisfies `enough`, and hand
/// back the whole of it — or give up, saying what did arrive.
async fn until(terminal: &Terminal, enough: impl Fn(&str) -> bool) -> String {
    let deadline = Instant::now() + PATIENCE;
    let mut said = String::new();
    let mut buffer = [0u8; 4096];

    while !enough(&said) {
        let read = tokio::time::timeout(
            deadline.saturating_duration_since(Instant::now()),
            terminal.read(&mut buffer),
        )
        .await
        .unwrap_or_else(|_| panic!("the session never said it. It said: {said:?}"))
        .expect("the terminal to be readable");

        assert!(read > 0, "the session ended having said: {said:?}");

        said.push_str(&String::from_utf8_lossy(&buffer[..read]));
    }

    said
}

/// Whether process `running` is no longer on this machine, waited for.
///
/// Waited for rather than asked once, and asked of Windows itself: a Job kills
/// what is in it, and a kill is something the machine gets round to rather than
/// something that has already happened when the handle closes.
async fn gone(running: u32) -> bool {
    let deadline = Instant::now() + PATIENCE;

    while Instant::now() < deadline {
        let listed = std::process::Command::new("tasklist.exe")
            .arg("/fi")
            .arg(format!("PID eq {running}"))
            .output()
            .expect("tasklist is part of Windows");

        // Which is what `tasklist` says of a filter that matched nothing — and
        // the id itself is what it prints when it matched.
        if !String::from_utf8_lossy(&listed.stdout).contains(&running.to_string()) {
            return true;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    false
}
