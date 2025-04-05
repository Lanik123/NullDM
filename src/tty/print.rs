use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    mem::MaybeUninit,
    os::fd::{AsRawFd, RawFd},
};

use nix::libc::{
        tcgetattr, tcsetattr, termios, ECHO, ECHONL, TCSANOW,
    };

struct HiddenInput {
    fd: RawFd,
    original: termios,
}

impl HiddenInput {
    fn new(fd: RawFd) -> io::Result<Self> {
        let mut term = tcgetattr_checked(fd)?;
        let original = term;

        term.c_lflag &= !ECHO;
        term.c_lflag |= ECHONL;

        tcset_checked(fd, &term)?;
        Ok(Self { fd, original })
    }
}

impl Drop for HiddenInput {
    fn drop(&mut self) {
        unsafe { tcsetattr(self.fd, TCSANOW, &self.original) };
    }
}

fn tcgetattr_checked(fd: RawFd) -> io::Result<termios> {
    let mut term = MaybeUninit::uninit();
    if unsafe { tcgetattr(fd, term.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { term.assume_init() })
}

fn tcset_checked(fd: RawFd, term: &termios) -> io::Result<()> {
    if unsafe { tcsetattr(fd, TCSANOW, term) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Reads from the TTY
fn read_line_raw(hidden: bool) -> io::Result<String> {
    let file = File::open("/dev/tty")?;
    let fd = file.as_raw_fd();
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    if hidden {
        let hidden_input = HiddenInput::new(fd).unwrap();
        reader.read_line(&mut line)?;
        std::mem::drop(hidden_input);
    } else {
        reader.read_line(&mut line)?;
    }

    // Clear control char
    line = line.trim_end_matches(&['\n', '\r'][..]).to_string();

    // CTRL + U
    if let Some(pos) = line.rfind('') {
        line = line[pos + 1..].to_string();
    }

    Ok(line)
}

/// Displays a message on the TTY
pub fn print_tty(prompt: impl ToString) -> std::io::Result<()> {
    let mut tty = OpenOptions::new().write(true).open("/dev/tty")?;
    tty.write_all(prompt.to_string().as_str().as_bytes()).and_then(|_| tty.flush())
}

/// Prompts on the TTY and then reads a username from TTY
pub fn prompt_username(prompt: impl ToString) -> io::Result<String> {
    print_tty(prompt.to_string().as_str()).and_then(|_| read_line_raw(false))
}

/// Prompts on the TTY and then reads a password from TTY
pub fn prompt_password(prompt: impl ToString) -> io::Result<String> {
    print_tty(prompt.to_string().as_str()).and_then(|_| read_line_raw(true))
}