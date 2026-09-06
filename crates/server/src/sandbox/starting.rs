//! What a [`Rendering`] comes to on Windows: the Win32 half of starting one,
//! shared by the two places that start anything.
//!
//! **Because the standard library cannot start either of them.** A session runs
//! on a pseudoconsole, which is an attribute on a `CreateProcessW` that
//! `std::process::Command` has no way to carry — see [`crate::terminal`] — and
//! from this stage on a Windows process also runs inside an AppContainer, which
//! is a second attribute on the same list. So the words a rendering becomes —
//! the command line, the environment block — and the list they are carried on
//! are here, where both callers can reach them, rather than copied into each.
//!
//! **The list is one block sized for what it holds**, which is why widening it
//! is what this module is for rather than adding a second one:
//! `InitializeProcThreadAttributeList` is asked for a count before anything
//! goes on it, and a console and a container are two attributes on one list.
//!
//! And beside them, the other way of starting a rendering: [`off_a_console`],
//! for everything that reads what a process printed rather than watching it —
//! the boundary suite, the ask test, the Compile Server. A rendering that names
//! a container cannot go through `Command` at all, so it goes through here.

use std::collections::BTreeMap;
use std::ffi::{OsStr, c_void};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::os::windows::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::ptr;

use windows_sys::Win32::Foundation::{
    CloseHandle, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, LocalFree,
    SetHandleInformation, WAIT_FAILED,
};
use windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW;
use windows_sys::Win32::Security::{PSID, SECURITY_ATTRIBUTES, SECURITY_CAPABILITIES};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CREATE_UNICODE_ENVIRONMENT, CreateProcessW, DeleteProcThreadAttributeList,
    EXTENDED_STARTUPINFO_PRESENT, GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, PROCESS_INFORMATION,
    STARTF_USESTDHANDLES, STARTUPINFOEXW, UpdateProcThreadAttribute, WaitForSingleObject,
};

use super::rendering::Rendering;

/// `rendering` run to its end with nothing watching it, and everything it
/// printed read back.
///
/// The other half of the seam from [`crate::terminal::Terminal::spawn`]: a
/// session is started on a console and read as it draws, and this is for
/// everything that wants what a process *said* — a probe inside the boundary
/// suite, the `verkstead ask` a session makes, the Compile Server. What comes
/// back is what `Command::output` hands back, because that is what a caller
/// here already knows how to read.
///
/// `typed` is put in at its standard input and the input is then closed, which
/// is what says *that is the whole of it* to a program reading one. Nothing at
/// all is the ordinary case; a Set on the way to `verkstead ask` is the case
/// that is not.
///
/// **A rendering that names no container is an ordinary `Command`**, which is
/// what the two Unix platforms and an unsandboxed Windows process are. One that
/// names a container is the whole reason this exists: the security
/// capabilities go on an attribute list, and a list is precisely what the
/// standard library will not carry.
///
/// It waits, so a caller that has to answer the process it started — the ask
/// test, whose Set has to be answered before the process will exit — runs this
/// on a thread of its own.
pub fn off_a_console(rendering: &Rendering, typed: &[u8]) -> io::Result<Output> {
    match rendering.container() {
        Some(container) => inside(rendering, container, typed),
        None => ordinarily(rendering, typed),
    }
}

/// A rendering with no container, started as anything else is started.
fn ordinarily(rendering: &Rendering, typed: &[u8]) -> io::Result<Output> {
    let mut running = Command::try_from(rendering)?
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut typing) = running.stdin.take() {
        typing.write_all(typed)?;
    }

    running.wait_with_output()
}

/// And one inside `container`, which is the same four things said to
/// `CreateProcessW` by hand: the command line, the environment block, where it
/// starts, and the attribute list carrying the container's identity.
///
/// Each of the three standard handles is a pipe of its own, with the child's
/// end inheritable and this process's end not — see [`piped`]. Both of the ends
/// this holds are read on threads of their own and the input is written on a
/// third, so that a process printing more than a pipe holds is not one waiting
/// on a reader that is waiting on it.
fn inside(rendering: &Rendering, container: &str, typed: &[u8]) -> io::Result<Output> {
    let capabilities = Capabilities::of(container)?;

    let mut attributes = Attributes::of(1)?;
    attributes.carrying(
        PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
        capabilities.attribute(),
        size_of::<SECURITY_CAPABILITIES>(),
    )?;

    let (given, mut typing) = piped(Reads::TheChild)?;
    let (printing, printed) = piped(Reads::ThisProcess)?;
    let (complaining, complained) = piped(Reads::ThisProcess)?;

    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = u32::try_from(size_of::<STARTUPINFOEXW>()).unwrap_or(u32::MAX);
    startup.lpAttributeList = attributes.list();
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = given.0;
    startup.StartupInfo.hStdOutput = printing.0;
    startup.StartupInfo.hStdError = complaining.0;

    let mut line = command_line(rendering);
    let environment = environment(rendering);
    let chdir = rendering.chdir().map(|chdir| wide(chdir.as_os_str()));

    let mut information: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    // Handles inherited, which is what puts the three pipes at the other end
    // where a program looks for its standard handles. Nothing else of this
    // process's goes with them: what a pipe of Verkstead's own is created as is
    // uninheritable — see [`crate::terminal`] — and the three that are
    // inheritable are these.
    let started = unsafe {
        CreateProcessW(
            ptr::null(),
            line.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            environment.as_ptr().cast::<c_void>(),
            chdir.as_ref().map_or(ptr::null(), |chdir| chdir.as_ptr()),
            &raw const startup.StartupInfo,
            &mut information,
        )
    };

    if started == 0 {
        return Err(io::Error::last_os_error());
    }

    let process = Handle(information.hProcess);
    drop(Handle(information.hThread));

    // The child's ends of all three, let go of here: a pipe whose write end
    // this process is still holding is one the read below would wait on for
    // ever.
    drop(given);
    drop(printing);
    drop(complaining);

    let typed = typed.to_vec();

    // Whatever it would not take is nothing to report: a program that read none
    // of its input, or exited before reading the rest of it, has answered the
    // caller in what it printed rather than in a broken pipe here.
    let putting = std::thread::spawn(move || {
        let _ = typing.write_all(&typed);
    });

    let saying = std::thread::spawn(move || drained(printed));
    let complaining = std::thread::spawn(move || drained(complained));

    let said = saying
        .join()
        .map_err(|_| lost("reading what it printed"))??;
    let complained = complaining
        .join()
        .map_err(|_| lost("reading what it complained about"))??;
    putting
        .join()
        .map_err(|_| lost("writing what it was given"))?;

    if unsafe { WaitForSingleObject(process.0, INFINITE) } == WAIT_FAILED {
        return Err(io::Error::last_os_error());
    }

    let mut code = 0u32;

    if unsafe { GetExitCodeProcess(process.0, &mut code) } == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(Output {
        status: ExitStatus::from_raw(code),
        stdout: said,
        stderr: complained,
    })
}

/// Everything there is to read at one end of a pipe, read until there is no
/// more of it.
fn drained(mut end: File) -> io::Result<Vec<u8>> {
    let mut read = Vec::new();

    end.read_to_end(&mut read)?;

    Ok(read)
}

/// What a thread that went away without answering is reported as.
fn lost(what: &str) -> io::Error {
    io::Error::other(format!(
        "the thread {what} of a process started off a console went away"
    ))
}

/// Which end of a pipe the child gets.
enum Reads {
    /// Its standard input: the child reads, and this process writes.
    TheChild,

    /// Its output or its complaints: the child writes, and this process reads.
    ThisProcess,
}

/// One pipe, as the child's end and this process's own.
///
/// The child's end is inheritable and this one's is not, which is the whole of
/// what makes a read here end: a copy of the write end left inheritable would
/// go to the next process this server starts, and a pipe with a writer is a
/// pipe with more to come.
fn piped(reads: Reads) -> io::Result<(Handle, File)> {
    let mut reading: HANDLE = ptr::null_mut();
    let mut writing: HANDLE = ptr::null_mut();

    let attributes = SECURITY_ATTRIBUTES {
        nLength: u32::try_from(size_of::<SECURITY_ATTRIBUTES>()).unwrap_or(u32::MAX),
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };

    if unsafe { CreatePipe(&mut reading, &mut writing, &attributes, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let (theirs, ours) = match reads {
        Reads::TheChild => (reading, writing),
        Reads::ThisProcess => (writing, reading),
    };

    if unsafe { SetHandleInformation(ours, HANDLE_FLAG_INHERIT, 0) } == 0 {
        let failed = io::Error::last_os_error();

        drop(Handle(theirs));
        drop(Handle(ours));

        return Err(failed);
    }

    // Safety: the handle is this process's own, made a moment ago and held by
    // nothing else, so the file is the one owner of it from here.
    Ok((Handle(theirs), unsafe {
        File::from_raw_handle(ours as RawHandle)
    }))
}

/// The container a process is started inside, as `CreateProcessW` is told one.
///
/// Held rather than made where it is used, because what goes on an attribute
/// list is a *pointer*: the structure and the SID it points at both have to
/// outlive the call that reads them, which is the `CreateProcessW` several
/// lines further down.
pub(crate) struct Capabilities {
    /// The container's SID, whose block this owns — see [`Sid`]. Read through
    /// the pointer inside [`Capabilities::capabilities`] rather than from here.
    #[allow(dead_code)]
    sid: Sid,

    /// And what the attribute is: that SID, and no capabilities beside it.
    ///
    /// **The capability list is empty, and that is the decision.** What a
    /// container may reach of the network is granted by the profile it was
    /// created with — the internet client and nothing else, see
    /// [`super::container`] — and a capability said again here would be one
    /// this call could widen the identity with.
    capabilities: SECURITY_CAPABILITIES,
}

impl Capabilities {
    /// The container named by `sid`, as the SID a person reads.
    ///
    /// A SID that will not resolve is an error rather than a process started
    /// outside a container: what names a container is a value that crossed the
    /// seam from the sandbox — see [`Rendering::container`] — and a rendering
    /// asking for a boundary is not one to start without it (ADR-0014).
    pub(crate) fn of(sid: &str) -> io::Result<Capabilities> {
        let held = Sid::of(sid)?;

        let capabilities = SECURITY_CAPABILITIES {
            AppContainerSid: held.0,
            Capabilities: ptr::null_mut(),
            CapabilityCount: 0,
            Reserved: 0,
        };

        Ok(Capabilities {
            sid: held,
            capabilities,
        })
    }

    /// What goes on the attribute list, and how much of it there is to read.
    pub(crate) fn attribute(&self) -> *const c_void {
        ptr::from_ref(&self.capabilities).cast::<c_void>()
    }
}

/// One SID, read out of the spelling a person writes and freed when it is let
/// go of.
struct Sid(PSID);

impl Sid {
    fn of(sid: &str) -> io::Result<Sid> {
        let mut read: PSID = ptr::null_mut();

        if unsafe { ConvertStringSidToSidW(wide(OsStr::new(sid)).as_ptr(), &mut read) } == 0 {
            return Err(io::Error::other(format!(
                "the container {sid} is not a SID this machine can read: {}",
                io::Error::last_os_error()
            )));
        }

        Ok(Sid(read))
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // Allocated by `ConvertStringSidToSidW`, which says a local
            // allocation and this is how one is given back.
            unsafe { LocalFree(self.0.cast()) };
        }
    }
}

/// The attribute list a process is started with, sized for as many attributes
/// as it is going to be given.
///
/// A list is a block of memory Windows lays out itself, so this is a buffer of
/// pointer-sized words — the alignment a list wants — with the list written
/// into it, and it is deleted when it is dropped.
pub(crate) struct Attributes(Vec<usize>);

impl Attributes {
    /// A list with room for `count` attributes and nothing on it yet.
    pub(crate) fn of(count: usize) -> io::Result<Attributes> {
        let count = u32::try_from(count).unwrap_or(u32::MAX);
        let mut wanted = 0usize;

        // The first call always fails: what it is for is the size, which is
        // what it writes on its way out.
        unsafe { InitializeProcThreadAttributeList(ptr::null_mut(), count, 0, &mut wanted) };

        if wanted == 0 {
            return Err(io::Error::last_os_error());
        }

        // The buffer before the list rather than after it: an [`Attributes`]
        // deletes the list as it is dropped, and there is no list to delete
        // until the call below has written one.
        let mut buffer = vec![0usize; wanted.div_ceil(size_of::<usize>())];

        let made = unsafe {
            InitializeProcThreadAttributeList(
                buffer.as_mut_ptr().cast::<c_void>(),
                count,
                0,
                &mut wanted,
            )
        };

        if made == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Attributes(buffer))
    }

    /// One more attribute on it: `value` is a pointer the list keeps rather
    /// than a value it copies, so whatever is at the other end of it has to
    /// outlive the `CreateProcessW` this list is passed to.
    pub(crate) fn carrying(
        &mut self,
        attribute: usize,
        value: *const c_void,
        size: usize,
    ) -> io::Result<()> {
        let list = self.list();

        let carried = unsafe {
            UpdateProcThreadAttribute(
                list,
                0,
                attribute,
                value,
                size,
                ptr::null_mut(),
                ptr::null(),
            )
        };

        if carried == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    /// The list itself, as everything that takes one wants it.
    pub(crate) fn list(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.0.as_mut_ptr().cast::<c_void>()
    }
}

impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe { DeleteProcThreadAttributeList(self.list()) };
    }
}

/// One handle of the process's own, closed when it is let go of.
///
/// A handle is a pointer as far as the bindings are concerned and therefore
/// neither `Send` nor `Sync` by itself, and it is both in fact: it is a number
/// the kernel looks up in a table this whole process shares, and nothing about
/// which thread holds it means anything.
pub(crate) struct Handle(pub(crate) HANDLE);

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// `rendering` as the command line `CreateProcessW` takes, program first.
///
/// Windows has no argument vector to hand over: a process is given one string
/// and takes it apart again, so this is the taking-apart run backwards — see
/// [`quoted`].
pub(crate) fn command_line(rendering: &Rendering) -> Vec<u16> {
    let mut line = Vec::new();

    quoted(rendering.program(), &mut line);

    for argument in rendering.argv() {
        line.push(u16::from(b' '));
        quoted(argument, &mut line);
    }

    line.push(0);

    line
}

/// One word of a command line, written so that `CommandLineToArgvW` reads back
/// the word that went in.
///
/// Which is the rule everything on Windows that takes a command line apart
/// follows: a run of backslashes means itself unless a quote comes next, and
/// then it means half of itself and the quote is the word's rather than the
/// quoting's. So a run before a quote is doubled and the quote escaped, and a
/// run at the end of a quoted word is doubled because the closing quote comes
/// next.
fn quoted(word: &OsStr, line: &mut Vec<u16>) {
    const SPACE: u16 = b' ' as u16;
    const TAB: u16 = b'\t' as u16;
    const QUOTE: u16 = b'"' as u16;
    const BACKSLASH: u16 = b'\\' as u16;

    let word: Vec<u16> = word.encode_wide().collect();

    // A word with nothing in it to misread is written as it is — which is most
    // of them, and is what makes a command line readable in a log.
    if !word.is_empty() && !word.iter().any(|unit| matches!(*unit, SPACE | TAB | QUOTE)) {
        line.extend_from_slice(&word);

        return;
    }

    line.push(QUOTE);

    let mut backslashes = 0usize;

    for unit in word {
        match unit {
            BACKSLASH => backslashes += 1,
            QUOTE => {
                line.extend(std::iter::repeat_n(BACKSLASH, backslashes + 1));
                backslashes = 0;
            }
            _ => backslashes = 0,
        }

        line.push(unit);
    }

    line.extend(std::iter::repeat_n(BACKSLASH, backslashes));
    line.push(QUOTE);
}

/// And `rendering`'s environment as the block `CreateProcessW` takes: every
/// name and value in one run of text, sorted, and the whole ended by a second
/// nothing.
///
/// Sorted and case-folded because that is what Windows asks of a block, and
/// because an environment where `Path` and `PATH` are two variables is one no
/// program on this platform expects: the last of a name is the one that stands,
/// which is what setting a variable twice means everywhere else in this
/// codebase.
pub(crate) fn environment(rendering: &Rendering) -> Vec<u16> {
    let mut named: BTreeMap<Vec<u16>, (Vec<u16>, Vec<u16>)> = BTreeMap::new();

    for (key, value) in rendering.env() {
        let name: Vec<u16> = key.encode_wide().collect();
        let folded = name
            .iter()
            .map(|unit| match u8::try_from(*unit) {
                Ok(byte) => u16::from(byte.to_ascii_uppercase()),
                Err(_) => *unit,
            })
            .collect();

        named.insert(folded, (name, value.encode_wide().collect()));
    }

    let mut block = Vec::new();

    for (name, value) in named.into_values() {
        block.extend_from_slice(&name);
        block.push(u16::from(b'='));
        block.extend_from_slice(&value);
        block.push(0);
    }

    // An environment with nothing in it is still a block, and a block is a run
    // of strings ended by an empty one.
    if block.is_empty() {
        block.push(0);
    }

    block.push(0);

    block
}

/// A string as every one of these calls wants one: what it says, and then
/// nothing.
pub(crate) fn wide(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}
