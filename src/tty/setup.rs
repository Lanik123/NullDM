use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    os::fd::RawFd,
};

use nix::{
    errno::Errno,
    fcntl::{self, OFlag},
    libc::{
        self,
    },
    sys::stat::Mode, unistd::close,
};

#[cfg(not(target_env = "musl"))]
type RequestType = libc::c_ulong;
#[cfg(target_env = "musl")]
type RequestType = libc::c_int;

const VT_ACTIVATE: RequestType = 0x5606;
const VT_WAITACTIVE: RequestType = 0x5607;
const KDGKBTYPE: RequestType = 0x4B33;
const KB_101: u8 = 0x02;
const KB_84: u8 = 0x01;

#[derive(Debug)]
pub enum VtChangeError {
    Activate,
    WaitActive,
    Close,
    OpenConsole,
    NotAConsole,
    GetFD,
}

impl Display for VtChangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for VtChangeError {}

fn is_console(fd: RawFd) -> bool {
    let mut arg = 0;
    if unsafe { libc::ioctl(fd, KDGKBTYPE, &mut arg) } > 0 {
        return false;
    }

    (arg == KB_101) || (arg == KB_84)
}

fn open_console(name: &str) -> Result<RawFd, VtChangeError> {
    for oflag in [OFlag::O_RDWR, OFlag::O_RDONLY, OFlag::O_WRONLY] {
        match fcntl::open(name, oflag, Mode::empty()) {
            Ok(fd) => {
                if !is_console(fd) {
                    let _ = close(fd);
                    return Err(VtChangeError::NotAConsole);
                }
                return Ok(fd);
            }
            Err(Errno::EACCES) => continue,
            _ => break,
        }
    }
    Err(VtChangeError::OpenConsole)
}

fn get_fd() -> Result<RawFd, VtChangeError> {
    for path in ["/dev/tty", "/dev/tty0", "/dev/vc/0", "/dev/console"] {
        if let Ok(fd) = open_console(path) {
            return Ok(fd);
        }
    }

    for fd in 0..3 {
        if is_console(fd) {
            return Ok(fd);
        }
    }

    Err(VtChangeError::GetFD)
}

pub unsafe fn setvt(ttynum: i32) -> Result<(), VtChangeError> {
    let fd = get_fd()?;

    unsafe {
        if libc::ioctl(fd, VT_ACTIVATE, ttynum) > 0 {
            return Err(VtChangeError::Activate);
        }
    
        if libc::ioctl(fd, VT_WAITACTIVE, ttynum) > 0 {
            return Err(VtChangeError::WaitActive);
        }
    }

    close(fd).map_err(|_| VtChangeError::Close)?;
    Ok(())
}