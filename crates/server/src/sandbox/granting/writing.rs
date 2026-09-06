//! The entries, really written: what [`super::entries`] worked out, on the
//! machine's own access-control lists.
//!
//! **A grant is one entry that inherits.** `SetEntriesInAclW` merges it into
//! whatever the directory's list already says and `SetNamedSecurityInfoW`
//! writes it back, with the inheritance flags that make everything under the
//! directory reachable too — which is what the probe found works, and is why a
//! Worktree is one entry rather than one per file in it.
//!
//! **A grant already there is left alone**, which is not an optimisation to
//! shrug at: writing an inheriting entry on a directory makes Windows walk
//! every path beneath it, so a session started in a repository with a large
//! `target/` would pay for the whole tree at every launch. The second session
//! of a Conversation finds its own entries and writes nothing.
//!
//! **A refusal is the one thing the probe could not settle from outside**, and
//! what it found is why this is not simply a deny entry. A deny written under a
//! granted tree did not refuse a file *below* it: on that file both entries are
//! inherited — the allow from the account above, the deny from the directory
//! between — and a check reads a list in the order it is in rather than by how
//! near the entry was written. The allow was propagated first and so is first.
//!
//! So the refusal is a **protected** list on that one directory: inheritance is
//! cut there, the entries the directory had inherited are kept as its own so
//! that the human's own reach is exactly what it was, everything of the
//! container's is dropped, and a deny of the container goes at the front. Cut
//! that way, no grant above can reach into the tree at all — and the deny says
//! in words what the missing grant would only imply, so an entry re-granted
//! above by some other hand still refuses.

use std::io;
use std::path::Path;
use std::ptr;
use std::sync::{LazyLock, Mutex, MutexGuard};

use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Authorization::{
    EXPLICIT_ACCESS_W, GetNamedSecurityInfoW, NO_MULTIPLE_TRUSTEE, REVOKE_ACCESS, SE_FILE_OBJECT,
    SET_ACCESS, SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_SID, TRUSTEE_IS_UNKNOWN,
    TRUSTEE_W,
};
use windows_sys::Win32::Security::{
    ACE_HEADER, ACL, ACL_REVISION, ACL_SIZE_INFORMATION, AclSizeInformation, AddAccessDeniedAceEx,
    AddAce, DACL_SECURITY_INFORMATION, GetAce, GetAclInformation, InitializeAcl,
    OBJECT_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    SUB_CONTAINERS_AND_OBJECTS_INHERIT, UNPROTECTED_DACL_SECURITY_INFORMATION,
};
use windows_sys::Win32::Storage::FileSystem::{
    DELETE, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
};

use super::super::container::Sid;
use super::super::starting::wide;
use super::super::surface::Reach;
use super::{Entry, Wanted};

/// One description's entries at a time.
///
/// **Because writing an entry is a read of a list and a write of it back**, and
/// two Conversations' sessions starting together name paths in common — the
/// shared build cache, Verkstead's own directory, a configured bind. Written at
/// once, the second read would be of the list before the first wrote it and one
/// of the two grants would be lost: a session behind a boundary narrower than
/// its description, which is exactly as wrong as one behind a wider.
///
/// Held for a whole description rather than for one entry, so that what a
/// session is given arrives whole. It is a handful of calls per session start
/// and nothing waits on it but another session start.
static ONE_AT_A_TIME: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// That lock taken, with a poisoned one taken all the same: what it guards is
/// the machine's own lists rather than anything held here, so a thread that
/// panicked between a read and a write left nothing behind for the next one to
/// find in a bad state.
fn one_at_a_time() -> MutexGuard<'static, ()> {
    ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner())
}

/// What an entry may name where a whole list is being built by hand: append.
///
/// Win32's own `MAXDWORD`, which the bindings do not carry.
const AT_THE_END: u32 = u32::MAX;

/// The two ACE types whose SID is where this reads one.
///
/// An allow and a deny are the same shape — a header, a mask, and the SID after
/// them — which is what makes [`whose`] one function rather than two. Anything
/// else in a list is copied as it stands and never asked about: an object ACE
/// carries two more fields before its SID and is a thing directories in Active
/// Directory have rather than files on a disk.
const ALLOWED: u8 = 0;
const DENIED: u8 = 1;

/// How far past an ACE's start its SID begins, on both of the types above: four
/// bytes of header and four of mask.
const THE_SID: usize = 8;

/// And the one flag of a header this reads: the bit that says an entry is the
/// parent's rather than this path's own.
///
/// Written here rather than taken from Win32's own `INHERITED_ACE`, which is
/// the same bit in the flag *word* a call is handed: a header keeps its flags
/// in one byte, and a constant that had to be narrowed at every use would be a
/// conversion standing in front of a bit test.
const INHERITED: u8 = 0x10;

/// Everything `entries` says, written for the container `sid` names.
///
/// **A path that is not there gets no entry**, which is the same answer
/// [`super::super::on_the_machine`] gives a system directory this machine has
/// not got: a rule about a path that does not exist is a rule about nothing. It
/// is the ordinary case for one thing in every description — the file half of a
/// Claude account, which a Profile that has never been logged in with has not
/// written yet — and a session that logs in and writes one has made a file of
/// its own that its own identity can read.
///
/// **Anything else refuses the session.** A grant that will not be written is a
/// session that would start behind a boundary nobody described (ADR-0014, Q18),
/// so what comes back says which path and what the machine said about it, and
/// the caller starts nothing.
pub(crate) fn write(entries: &[Entry], sid: &str) -> io::Result<()> {
    let sid = Sid::of(sid)?;
    let _one = one_at_a_time();

    for entry in entries {
        if !entry.path.exists() {
            continue;
        }

        let written = match entry.wanted {
            Wanted::Granted(reach) => grant(&sid, &entry.path, reach),
            Wanted::Refused => refuse(&sid, &entry.path),
        };

        written.map_err(|error| {
            io::Error::other(format!(
                "{} could not be made {} the AppContainer: {error}",
                entry.path.display(),
                match entry.wanted {
                    Wanted::Granted(Reach::ReadOnly) => "readable by",
                    Wanted::Granted(Reach::ReadWrite) => "writable by",
                    Wanted::Refused => "unreachable from",
                },
            ))
        })?;
    }

    Ok(())
}

/// And every one of them taken back off the directories it was written on.
///
/// **Nothing here refuses anything.** This is what a container's ending does
/// with what it wrote — see [`super::super::container::Container`] — and a
/// container has already ended by the time anything could be refused. An entry
/// that will not come off is named in the log and the rest go.
///
/// **What a refused path is left as is what `icacls` leaves one.** Cutting its
/// inheritance kept the entries it had been given from above as its own — that
/// being the only way to cut it without taking the human's own reach away with
/// it — and putting the inheritance back does not know which of the entries
/// there now were those copies. So they stay beside the ones that come back
/// from above: the same reach, said twice, on the one directory a description
/// ever refuses.
pub(crate) fn strip(entries: &[Entry], sid: &str) {
    let Ok(sid) = Sid::of(sid) else {
        return;
    };

    let _one = one_at_a_time();

    for entry in entries {
        if !entry.path.exists() {
            continue;
        }

        // A refusal cut the inheritance on that directory as well as writing a
        // deny, so taking it back is putting the inheritance back — see this
        // module's own documentation, and [`refuse`].
        let taken = match entry.wanted {
            Wanted::Granted(_) => revoked(&sid, &entry.path, DACL_SECURITY_INFORMATION),
            Wanted::Refused => revoked(&sid, &entry.path, UNPROTECTED_DACL_SECURITY_INFORMATION),
        };

        if let Err(error) = taken {
            tracing::warn!(
                error = ?error,
                path = %entry.path.display(),
                "an access-control entry written for a session's AppContainer could not be \
                 taken off again, so it is left on a directory naming an identity that has \
                 gone"
            );
        }
    }
}

/// `path` reachable at `reach`, and everything under it with it.
fn grant(sid: &Sid, path: &Path, reach: Reach) -> io::Result<()> {
    let rights = match reach {
        // Read and traverse, which is what reading a directory of tools is.
        Reach::ReadOnly => FILE_GENERIC_READ | FILE_GENERIC_EXECUTE,

        // And `DELETE` beside the write, which is not a widening to wave
        // through: `FILE_GENERIC_WRITE` is writing a file's *contents*, and a
        // session that could not delete or rename one could not run `git
        // checkout` in its own Worktree. What is deliberately not in it is
        // `WRITE_DAC` and `WRITE_OWNER` — a session that could rewrite an
        // access-control list could grant itself the rest of the machine.
        Reach::ReadWrite => FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE,
    };

    if already(sid, path, rights)? {
        return Ok(());
    }

    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: rights,
        grfAccessMode: SET_ACCESS,
        grfInheritance: SUB_CONTAINERS_AND_OBJECTS_INHERIT,
        Trustee: trustee(sid),
    };

    merged(path, &access, DACL_SECURITY_INFORMATION)
}

/// And `path` refused, whatever a grant above it says — see this module's own
/// documentation, which is where the whole of why this is not one entry is.
fn refuse(sid: &Sid, path: &Path) -> io::Result<()> {
    let held = Held::of(path)?;

    let Some(existing) = held.list() else {
        // A list that is not there at all is a path everybody reaches, which is
        // a thing an account directory is not and a thing this cannot cut
        // inheritance on without taking that reach away from the human too. So
        // the deny goes on alone, unprotected, and says so.
        tracing::warn!(
            path = %path.display(),
            "the path a session is to find nothing at has no access-control list of its own, \
             so it was refused by an entry rather than by cutting what it inherits"
        );

        return written(path, &denial(sid, &[])?, DACL_SECURITY_INFORMATION);
    };

    let kept: Vec<Vec<u8>> = existing
        .iter()
        .filter(|ace| whose(ace).is_none_or(|theirs| !sid.is(theirs)))
        .map(|ace| {
            // Kept as the directory's own rather than as something it inherits,
            // because after this it inherits nothing: the flag is what a list
            // that has been cut off from its parent stops saying, and a copy
            // that went on claiming to be inherited would be one the next
            // propagation from above took away again.
            let mut kept = ace.clone();
            kept[1] &= !INHERITED;
            kept
        })
        .collect();

    written(
        path,
        &denial(sid, &kept)?,
        DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
    )
}

/// A list with the container denied at the front of it and `kept` behind, in
/// the order they were in.
///
/// The front is the whole of it. A check reads a list in order and stops at the
/// first entry that answers, so a deny anywhere behind an allow is a deny that
/// is never reached — which is exactly what the probe found and what this is
/// the answer to.
fn denial(sid: &Sid, kept: &[Vec<u8>]) -> io::Result<Vec<u32>> {
    let size = size_of::<ACL>() + THE_SID + sid.length() + kept.iter().map(Vec::len).sum::<usize>();

    // Words rather than bytes, which is not a spelling to tidy: a list has to
    // begin on a four-byte boundary and a `Vec<u8>` promises nothing about
    // where its bytes start. Rounding the length up only ever leaves slack at
    // the end, which is what the size handed to the call below is for.
    let mut list = vec![0u32; size.div_ceil(size_of::<u32>())];
    let acl = list.as_mut_ptr().cast::<ACL>();

    if unsafe { InitializeAcl(acl, u32::try_from(size).unwrap_or(u32::MAX), ACL_REVISION) } == 0 {
        return Err(io::Error::last_os_error());
    }

    // Everything, rather than the read and the write alone: what is being said
    // is that this path is not the container's business, and a right left out
    // would be a right a grant above could still hand it.
    if unsafe {
        AddAccessDeniedAceEx(
            acl,
            ACL_REVISION,
            SUB_CONTAINERS_AND_OBJECTS_INHERIT,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE,
            sid.as_psid(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    for ace in kept {
        if unsafe {
            AddAce(
                acl,
                ACL_REVISION,
                AT_THE_END,
                ace.as_ptr().cast(),
                u32::try_from(ace.len()).unwrap_or(0),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(list)
}

/// Whether `path` already says what a grant of `rights` would say.
///
/// Its own entry rather than one it inherits, at exactly the rights and exactly
/// the inheritance this would write: an entry that says something narrower is
/// one that has to be written over, and one it inherits from above is one a
/// path of its own has to be written for — a grant is what makes a *directory*
/// reachable, and inheriting one from somewhere else says nothing about that.
fn already(sid: &Sid, path: &Path, rights: u32) -> io::Result<bool> {
    let held = Held::of(path)?;

    let Some(existing) = held.list() else {
        return Ok(false);
    };

    Ok(existing.iter().any(|ace| {
        ace.len() > THE_SID
            && ace[0] == ALLOWED
            && u32::from(ace[1]) == SUB_CONTAINERS_AND_OBJECTS_INHERIT
            && mask(ace) == rights
            && whose(ace).is_some_and(|theirs| sid.is(theirs))
    }))
}

/// Everything this wrote for `sid` on `path`, taken off — and `what` said as
/// well, which is how a refusal's cut inheritance is put back.
fn revoked(sid: &Sid, path: &Path, what: OBJECT_SECURITY_INFORMATION) -> io::Result<()> {
    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: 0,
        grfAccessMode: REVOKE_ACCESS,
        grfInheritance: 0,
        Trustee: trustee(sid),
    };

    merged(path, &access, what | DACL_SECURITY_INFORMATION)
}

/// One entry merged into whatever `path`'s list already says, and the result
/// written back.
fn merged(
    path: &Path,
    access: &EXPLICIT_ACCESS_W,
    what: OBJECT_SECURITY_INFORMATION,
) -> io::Result<()> {
    let held = Held::of(path)?;
    let mut wanted: *mut ACL = ptr::null_mut();

    let made = unsafe { SetEntriesInAclW(1, access, held.dacl, &mut wanted) };

    if made != 0 {
        return Err(refused(made));
    }

    let list = Made(wanted);
    let name = wide(path.as_os_str());

    let set = unsafe {
        SetNamedSecurityInfoW(
            name.as_ptr().cast_mut(),
            SE_FILE_OBJECT,
            what,
            ptr::null_mut(),
            ptr::null_mut(),
            list.0,
            ptr::null(),
        )
    };

    if set != 0 {
        return Err(refused(set));
    }

    Ok(())
}

/// And a list built here written onto `path`.
fn written(path: &Path, list: &[u32], what: OBJECT_SECURITY_INFORMATION) -> io::Result<()> {
    let name = wide(path.as_os_str());

    let set = unsafe {
        SetNamedSecurityInfoW(
            name.as_ptr().cast_mut(),
            SE_FILE_OBJECT,
            what,
            ptr::null_mut(),
            ptr::null_mut(),
            list.as_ptr().cast(),
            ptr::null(),
        )
    };

    if set != 0 {
        return Err(refused(set));
    }

    Ok(())
}

/// What a call that would not do it said, as an error a caller can read.
///
/// The four calls in this file that touch an access-control list hand back a
/// Win32 status rather than setting the last error, which is the one thing
/// about them that is not like the rest of Win32 — so this is where that is
/// turned back into the sentence the machine has for the number.
fn refused(status: u32) -> io::Error {
    io::Error::from_raw_os_error(status as i32)
}

/// Who the trustee is, said the one way everything here says it: a SID rather
/// than a name, because an AppContainer's identity has no name to be looked up
/// under.
fn trustee(sid: &Sid) -> TRUSTEE_W {
    TRUSTEE_W {
        pMultipleTrustee: ptr::null_mut(),
        MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_UNKNOWN,
        ptstrName: sid.as_psid().cast(),
    }
}

/// What one entry of a list allows or denies.
fn mask(ace: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&ace[4..THE_SID]);

    u32::from_le_bytes(bytes)
}

/// And whose it is, where it is one of the two shapes whose SID is where this
/// reads one — see [`ALLOWED`] and [`DENIED`].
fn whose(ace: &[u8]) -> Option<&[u8]> {
    (matches!(ace[0], ALLOWED | DENIED) && ace.len() > THE_SID).then(|| &ace[THE_SID..])
}

/// A path's access-control list, read and held for as long as it is being read.
///
/// One block of memory holds the descriptor and the list inside it, so the list
/// is only a list while this is alive — which is the whole reason it is a value
/// rather than two calls in a row.
struct Held {
    descriptor: PSECURITY_DESCRIPTOR,
    dacl: *mut ACL,
}

impl Held {
    fn of(path: &Path) -> io::Result<Held> {
        let mut dacl: *mut ACL = ptr::null_mut();
        let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
        let name = wide(path.as_os_str());

        let read = unsafe {
            GetNamedSecurityInfoW(
                name.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut dacl,
                ptr::null_mut(),
                &mut descriptor,
            )
        };

        if read != 0 {
            return Err(refused(read));
        }

        Ok(Held { descriptor, dacl })
    }

    /// Every entry of it, each copied out as the bytes it is — or nothing at
    /// all where the path has no list, which is a path everybody reaches.
    ///
    /// Copied rather than pointed at, because what is done with them is to
    /// build another list: an entry read out of the block below and written
    /// into a list being built here would be a read of memory this is about to
    /// hand back.
    fn list(&self) -> Option<Vec<Vec<u8>>> {
        if self.dacl.is_null() {
            return None;
        }

        let mut sizes = ACL_SIZE_INFORMATION::default();

        if unsafe {
            GetAclInformation(
                self.dacl,
                ptr::from_mut(&mut sizes).cast(),
                u32::try_from(size_of::<ACL_SIZE_INFORMATION>()).unwrap_or(0),
                AclSizeInformation,
            )
        } == 0
        {
            return None;
        }

        let mut aces = Vec::new();

        for index in 0..sizes.AceCount {
            let mut ace: *mut core::ffi::c_void = ptr::null_mut();

            if unsafe { GetAce(self.dacl, index, &mut ace) } == 0 || ace.is_null() {
                continue;
            }

            // Safety: what `GetAce` handed back begins with a header, and the
            // size in that header is how much of the list belongs to it.
            let size = usize::from(unsafe { ace.cast::<ACE_HEADER>().read_unaligned() }.AceSize);

            aces.push(unsafe { std::slice::from_raw_parts(ace.cast::<u8>(), size) }.to_vec());
        }

        Some(aces)
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        if !self.descriptor.is_null() {
            unsafe { LocalFree(self.descriptor) };
        }
    }
}

/// And a list Windows itself built, given back the way one is.
struct Made(*mut ACL);

impl Drop for Made {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { LocalFree(self.0.cast()) };
        }
    }
}

/// This whole module is one platform's, so these are that platform's too: they
/// run on the `windows-2025` job and nowhere else.
#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::sandbox::container::Container;
    use crate::sandbox::rendering::Rendering;
    use crate::sandbox::starting::off_a_console;

    /// A SID that is a SID and names nobody, for the entries that are about
    /// what gets written rather than about who reaches what.
    ///
    /// An unresolvable identity is a perfectly ordinary thing to find in an
    /// access-control list — it is what one that has been deleted leaves — so
    /// the calls this makes are the calls a real grant makes.
    const NOBODY: &str = "S-1-15-2-1-1-1-1-1-1-1-1-1-1";

    /// What a probe inside a container has to be handed to run at all — the
    /// same list `tests/container_windows.rs` gives one, and for the same
    /// reason: a rendering is the whole of what a process gets.
    const NEEDED: [&str; 8] = [
        "ComSpec",
        "PATH",
        "PATHEXT",
        "SystemDrive",
        "SystemRoot",
        "TEMP",
        "TMP",
        "LOCALAPPDATA",
    ];

    /// An entry's SID is read where an entry keeps one, which is the offset
    /// every list built or read here turns on.
    #[test]
    fn an_entry_is_read_as_a_header_a_mask_and_a_sid() {
        let mut ace = vec![ALLOWED, 3, 20, 0];
        ace.extend_from_slice(&0x001f_01ffu32.to_le_bytes());
        ace.extend_from_slice(&[1, 2, 3, 4]);

        assert_eq!(mask(&ace), 0x001f_01ff);
        assert_eq!(whose(&ace), Some(&[1u8, 2, 3, 4][..]));

        // And an entry of a shape whose SID is somewhere else is one nothing
        // here claims to know — see [`ALLOWED`].
        assert_eq!(whose(&[5, 3, 8, 0, 0, 0, 0, 0, 9]), None);
    }

    /// An identity this machine cannot read is the whole description refused,
    /// before a single entry is written.
    ///
    /// Which is what a session gets: [`crate::sandbox::Sandbox::command`] hands
    /// this refusal up and the caller starts nothing (ADR-0014, Q18).
    #[test]
    fn an_identity_that_will_not_resolve_refuses_and_says_which() {
        let refused = write(&[], "this is not a SID")
            .expect_err("an identity that is not a SID should not be written for");

        assert!(
            refused.to_string().contains("this is not a SID"),
            "the refusal should say what would not resolve, and it said: {refused}"
        );
    }

    /// And a path that is not there is a rule about nothing rather than a
    /// refusal — see [`write`], which is where the reason is.
    #[test]
    fn a_path_that_is_not_there_gets_no_entry_and_refuses_nobody() {
        let held = tempfile::tempdir().expect("a directory to describe");

        let entries = vec![
            Entry {
                path: held.path().join("an-account-never-logged-in-with.json"),
                wanted: Wanted::Granted(Reach::ReadWrite),
            },
            Entry {
                path: held.path().join("skills-this-account-has-not-got"),
                wanted: Wanted::Refused,
            },
        ];

        write(&entries, NOBODY).expect("a description naming paths that are not there");
    }

    /// The whole of what this module is for, asked of the machine by attempting
    /// it from inside a real container: a granted directory is read, a refused
    /// one under it is not, and a path no entry ever named is not either.
    ///
    /// **The refusal is the half the probe could not settle**, and it is the
    /// half this is really about — see this module's own documentation. The
    /// refused directory is deliberately *inside* the granted one, because that
    /// is the shape a description asks for: the account is granted and the
    /// skills inside it are not, and a deny that only worked outside a granted
    /// tree would answer a question nobody asked.
    ///
    /// **And the refusal is told from a path that was never there**, because
    /// the two look the same to anything coarser and only one of them is a
    /// boundary.
    #[test]
    fn a_granted_path_is_read_from_inside_and_a_refused_one_is_not() {
        let held = tempfile::tempdir().expect("a directory to lay a description out in");

        let granted = held.path().join("worktree");
        let refused = granted.join("their-skills");
        let unnamed = held.path().join("documents");

        for (directory, file, said) in [
            (&granted, "readable.txt", "the description named this"),
            (&refused, "theirs.txt", "the account's own"),
            (&unnamed, "private.txt", "nobody named this"),
        ] {
            std::fs::create_dir_all(directory).unwrap();
            std::fs::write(directory.join(file), said).unwrap();
        }

        // Named the way a session's is, off a Data Directory of this test's own
        // — so the name is this run's alone and a profile some earlier run left
        // under it is replaced rather than in the way. See
        // [`Container::for_conversation`].
        let container = Container::for_conversation(held.path(), 1)
            .expect("this machine to make an AppContainer");

        let entries = vec![
            Entry {
                path: granted.clone(),
                wanted: Wanted::Granted(Reach::ReadWrite),
            },
            Entry {
                path: refused.clone(),
                wanted: Wanted::Refused,
            },
        ];

        write(&entries, container.sid()).expect("the entries this description comes to");

        // Held by the container from here, so that everything written above
        // comes off the human's own temporary directory when this test ends —
        // see `Container`'s `Drop`.
        container.wrote(entries);

        let said = attempted(
            container.sid(),
            &[
                ("granted", &granted.join("readable.txt")),
                ("refused", &refused.join("theirs.txt")),
                ("unnamed", &unnamed.join("private.txt")),
            ],
        );

        assert!(
            said.contains("granted=read"),
            "a granted directory is read from inside, and the probe said: {said:?}"
        );
        assert!(
            said.contains("refused=UnauthorizedAccessException"),
            "a refused directory inside a granted one is refused rather than \
             absent, and the probe said: {said:?}"
        );
        assert!(
            said.contains("unnamed=UnauthorizedAccessException"),
            "and so is a path no entry ever named, and the probe said: {said:?}"
        );
    }

    /// Read each of `paths` from inside the container `sid` names, and hand
    /// back the `name=word` lines it printed.
    ///
    /// The word is the name of what stopped the read rather than one of this
    /// test's own, for the reason `tests/sessions_windows.rs` reads one:
    /// `UnauthorizedAccessException` is a boundary and `FileNotFoundException`
    /// is a path that was never there, and a test that could not tell them
    /// apart would pass against a description that named nothing at all.
    fn attempted(sid: &str, paths: &[(&str, &PathBuf)]) -> String {
        let asked = paths
            .iter()
            .map(|(name, path)| {
                format!(
                    "@('{name}','{}')",
                    path.display().to_string().replace('\'', "''")
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let mut probe = Rendering::running("powershell.exe");

        for name in NEEDED {
            if let Some(value) = std::env::var_os(name) {
                probe.set(name, value);
            }
        }

        probe
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(format!(
                "foreach ($asked in @({asked})) {{ \
                   try {{ \
                     [System.IO.File]::ReadAllText($asked[1]) | Out-Null; \
                     Write-Output ($asked[0] + '=read') \
                   }} catch {{ \
                     $why = $_.Exception; \
                     while ($why.InnerException) {{ $why = $why.InnerException }}; \
                     Write-Output ($asked[0] + '=' + $why.GetType().Name) \
                   }} \
                 }}"
            ))
            .inside(sid);

        let printed = off_a_console(&probe, b"").expect("a probe inside a container");

        format!(
            "{}{}",
            String::from_utf8_lossy(&printed.stdout),
            String::from_utf8_lossy(&printed.stderr),
        )
    }
}
