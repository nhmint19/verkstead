//! What a session may reach, as an access-control entry on each real directory
//! the description names.
//!
//! The Windows half of a rendering, and the half that is the boundary. There is
//! nothing to mount and nothing to write a policy at: an AppContainer reaches
//! what its identity has been granted and nothing else, so a
//! [`Surface`](super::surface::Surface) becomes a list of entries written on the
//! human's own directories for the profile's SID — see
//! [`super::container`], which is where that identity comes from.
//!
//! **The list is worked out here and written next door.** What a description
//! comes to is a fact about the description rather than about Win32, so
//! [`entries`] is built and read on every platform and
//! [`writing`](self::writing) — which is compiled where there is an
//! access-control list to write — is the only part that touches the machine.
//! Which is what lets a test on a Linux box ask what a Windows session would be
//! granted, the way [`super::bwrap`]'s tests read flags nothing runs.
//!
//! **The vocabulary, as the probe found this platform answers it** (ADR-0014,
//! *What the probe answered*):
//!
//! - **`Own` and `Elsewhere`** are a grant at the Surface's reach on the real
//!   path — the *host* side of an `Elsewhere`, because a junction is followed
//!   and the grant is checked on its target, so what is granted is the account
//!   itself rather than the name a session finds it under.
//! - **`Empty` and `Temporary`** are a grant read-write: they are the session's
//!   own profile and the directory it throws things away in, which nobody else
//!   has any business in and which a session cannot do without.
//! - **`Nothing`** is refused — see [`Wanted::Refused`], and
//!   [`writing::refuse`] for the mechanism, which is the one part of this the
//!   probe could not settle from outside.
//! - **`ProcessTable` and `Devices`** are nothing at all here. There is no
//!   process table in the filesystem on this platform, and the devices a
//!   program opens by name are the machine's own.
//!
//! **And one thing that is in no description**: each `PATH` entry under the
//! human's own profile, read-only. Program Files, the system directory and
//! Windows PowerShell are all readable by a container with no entry at all —
//! the probe ran `node`, `git` and PowerShell out of them — but a per-user tool
//! install is not, and an agent installed by npm is exactly that. So the
//! directories a session is told to look for a program in are granted where
//! they are the human's own, and nothing else of that profile is.
//!
//! **Nothing above a granted path is granted.** The probe reached a directory
//! deep inside the human's profile with no entry anywhere above it, so a
//! rendering never grants a profile on the way to a Worktree — which is the
//! whole reason a session can be given its Worktree without being given the
//! account that holds it.

#[cfg(windows)]
pub(crate) mod writing;

use std::path::{Path, PathBuf};

use super::surface::{Access, Reach, Surface};

/// What one path is to be, once the entries are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wanted {
    /// Reachable that far, and everything under it with it: a grant inherits
    /// down a tree, which is what makes one entry the answer for a Worktree
    /// rather than one per file in it.
    Granted(Reach),

    /// And refused, whatever a grant above it says — the account's own skills,
    /// which a session is to find nothing at.
    Refused,
}

/// One path and what it is to be.
///
/// Read by name rather than through accessors, because the one thing that reads
/// one is [`writing`], which is inside this module: an entry is a pair, and a
/// pair with two functions in front of it would be two functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The real path on the host the entry goes on.
    path: PathBuf,

    /// And what it says.
    wanted: Wanted,
}

/// Everything `surface` says, as the entries that make it true — in the order
/// the description said it.
///
/// **The order is the description's**, for the reason [`Surface`] keeps one: a
/// path said twice is the second one, and what covers the account's own skills
/// is said after the account it is inside. Written in that order, the refusal
/// lands over a grant rather than under it.
///
/// `profile` is the human's own — where the account running the server keeps
/// its things — and is what the `PATH` rule above is measured against. `None`
/// where the machine will not say, which costs a session the per-user tools on
/// its `PATH` and nothing else.
pub(crate) fn entries(surface: &Surface, profile: Option<&Path>) -> Vec<Entry> {
    let mut entries = Vec::new();

    // First, because these are the machine's floor rather than this session's:
    // a description that goes on to grant one of them at a wider reach is a
    // description whose word is the later one.
    if let Some(profile) = profile {
        for directory in super::open::looked_in(surface) {
            if beneath(directory, profile) {
                entries.push(granted(directory, Reach::ReadOnly));
            }
        }
    }

    for access in surface.reaches() {
        match access {
            Access::Own { path, reach } => entries.push(granted(path, *reach)),

            // The host's side of it, which is where the grant belongs: a
            // junction is followed and the grant is checked on its target, so
            // an entry on the name a session finds the account under would be
            // an entry on the junction rather than on the account.
            Access::Elsewhere { host, reach, .. } => entries.push(granted(host, *reach)),

            // The session's own profile and what it throws away, both of them
            // made by the rendering a moment before this — see
            // [`super::open::command`], which is what puts the directory there
            // for the entry to go on.
            Access::Empty(path) | Access::Temporary(path) => {
                entries.push(granted(path, Reach::ReadWrite));
            }

            Access::Nothing { inside, .. } => entries.push(Entry {
                path: inside.clone(),
                wanted: Wanted::Refused,
            }),

            // Neither of which is a path on this platform — see this module's
            // own documentation.
            Access::ProcessTable | Access::Devices => {}
        }
    }

    entries
}

/// One path granted that far.
fn granted(path: impl Into<PathBuf>, reach: Reach) -> Entry {
    Entry {
        path: path.into(),
        wanted: Wanted::Granted(reach),
    }
}

/// Whether `path` is somewhere under `directory`, as this platform reads two
/// paths.
///
/// Case-folded, because a Windows filesystem is: a `PATH` written
/// `C:\Users\Ada\AppData` and a profile read back as `C:\Users\ada` are one
/// directory, and a comparison that said otherwise would leave the tools a
/// session needs ungranted on half the machines there are.
///
/// **Strictly under**, which is the whole rule rather than a nicety: a `PATH`
/// entry that *is* the profile would otherwise grant the human's whole account
/// read-only, and the account is the one thing this platform's boundary is
/// about.
fn beneath(path: &Path, directory: &Path) -> bool {
    let (path, directory) = (folded(path), folded(directory));

    path.len() > directory.len()
        && path.starts_with(&directory)
        && matches!(path.as_bytes().get(directory.len()), Some(b'\\' | b'/'))
}

/// A path as the comparison above reads one: lower case, and with a trailing
/// separator dropped so that a `PATH` entry written with one and a profile
/// read back without are still one directory and one under it.
fn folded(path: &Path) -> String {
    path.to_string_lossy()
        .to_lowercase()
        .trim_end_matches(['\\', '/'])
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;

    /// A description with one of everything in it, in the order a session's own
    /// is built in.
    fn described(account: &Path, home: &Path) -> Surface {
        let mut surface = Surface::starting_in(PathBuf::from(r"C:\repo"));

        surface.made(Access::Empty(home.to_owned()));
        surface.made(Access::Temporary(home.join("Temp")));
        surface.made(Access::ProcessTable);
        surface.made(Access::Devices);
        surface
            .own(r"C:\repo", Reach::ReadWrite)
            .own(r"C:\repo\.git", Reach::ReadWrite)
            .own(r"C:\ProgramData\verkstead\skills", Reach::ReadOnly)
            .elsewhere(account, home.join(".claude"), Reach::ReadWrite)
            .nothing(home.join(".claude").join("skills"), PathBuf::from("unused"));

        surface
    }

    /// Every part of the vocabulary, as the entry it comes to.
    #[test]
    fn each_kind_of_access_is_the_entry_it_says() {
        let (account, home) = (Path::new(r"C:\Users\ada\.claude"), Path::new(r"D:\homes\7"));

        let entries = entries(&described(account, home), None);

        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.path.clone(), entry.wanted))
                .collect::<Vec<_>>(),
            vec![
                (home.to_owned(), Wanted::Granted(Reach::ReadWrite)),
                (home.join("Temp"), Wanted::Granted(Reach::ReadWrite)),
                (PathBuf::from(r"C:\repo"), Wanted::Granted(Reach::ReadWrite)),
                (
                    PathBuf::from(r"C:\repo\.git"),
                    Wanted::Granted(Reach::ReadWrite)
                ),
                (
                    PathBuf::from(r"C:\ProgramData\verkstead\skills"),
                    Wanted::Granted(Reach::ReadOnly)
                ),
                // The account itself rather than the name it is found under,
                // and the refusal after it, which is the order the description
                // said them in.
                (account.to_owned(), Wanted::Granted(Reach::ReadWrite)),
                (home.join(".claude").join("skills"), Wanted::Refused),
            ],
            "the process table and the devices are nothing at all here, and \
             everything else is one entry",
        );
    }

    /// And the `PATH` rule, which is in no description: what is under the
    /// human's own profile is granted read-only and what is not is left alone.
    #[test]
    fn only_the_path_entries_under_the_humans_profile_are_granted() {
        let (account, home) = (Path::new(r"C:\Users\ada\.claude"), Path::new(r"D:\homes\7"));
        let mut surface = described(account, home);

        surface.set(
            "Path",
            OsString::from(concat!(
                r"D:\verkstead\bin;",
                r"C:\Users\Ada\AppData\Roaming\npm;",
                r"C:\Program Files\Git\cmd;",
                r"C:\Users\ada",
            )),
        );

        let granted: Vec<_> = entries(&surface, Some(Path::new(r"C:\Users\ada")))
            .into_iter()
            .filter(|entry| entry.wanted == Wanted::Granted(Reach::ReadOnly))
            .map(|entry| entry.path.clone())
            .collect();

        assert!(
            granted.contains(&PathBuf::from(r"C:\Users\Ada\AppData\Roaming\npm")),
            "a per-user tool install is what this rule is for, whatever case \
             the machine wrote it in: {granted:?}"
        );
        assert!(
            !granted.contains(&PathBuf::from(r"C:\Program Files\Git\cmd")),
            "and Program Files needs no entry at all: {granted:?}"
        );
        assert!(
            !granted.contains(&PathBuf::from(r"D:\verkstead\bin")),
            "and Verkstead's own directory is granted by the description \
             rather than by this: {granted:?}"
        );
        assert!(
            !granted.contains(&PathBuf::from(r"C:\Users\ada")),
            "and a `PATH` entry that is the profile itself would be the whole \
             account read-only, which is the one thing this boundary is \
             about: {granted:?}"
        );
    }
}
