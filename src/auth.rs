use log::{error, info};
use nix::{libc::{pid_t, utmpx}, unistd::User};
use pam::{Authenticator, PasswordConv};

use crate::{config::Config, tty};

pub fn handle_login<'a>(config: &Config) -> Result<(String, Authenticator<'a, PasswordConv>), String> {
    for attempt in 1..=config.max_attempts {
        let username = match tty::print::prompt_username("Login: ") {
            Ok(u) if !u.trim().is_empty() => u.trim().to_string(),
            _ => {
                error!("[Attempt {attempt}] Empty username input");
                continue;
            }
        };

        let password = match tty::print::prompt_password("Password: ") {
            Ok(p) if !p.trim().is_empty() => p,
            _ => {
                error!("[Attempt {attempt}] Empty password input");
                continue;
            }
        };

        let user = match User::from_name(&username) {
            Ok(Some(u)) => u,
            _ => {
                error!("[Attempt {attempt}] Unknown user: {username}");
                continue;
            }
        };

        if user.uid.as_raw() < config.min_uid {
            error!("[Attempt {attempt}] User '{}' is below min_uid threshold", username);
            continue;
        }

        match authenticate(&username, &password) {
            Ok(session) => return Ok((username, session)),
            Err(e) => {
                error!("[Attempt {attempt}] Login failed for {}: {e:?}", username);
            }
        }

        if attempt == config.max_attempts {
            error!("Maximum number of attempts reached. Exiting.");
        }
    }

    Err("Too many failed login attempts.".into())
}

fn authenticate<'a>(username: &str, password: &str) -> pam::PamResult<Authenticator<'a, PasswordConv>> {
    let mut auth = Authenticator::with_password("login")?;
    auth.get_handler().set_credentials(username, password);
    info!("Got handler");

    auth.authenticate()?;
    info!("Validated account");

    auth.open_session()?;
    info!("Opened session");

    Ok(auth)
}

/// Taked from lemurs DM
pub fn add_utmpx_entry(username: &str, tty: u8, pid: pid_t) -> utmpx {
    log::info!("Adding UTMPX record");

    // Check the MAN page for utmp for more information
    // `man utmp`
    //
    // https://man7.org/linux/man-pages/man0/utmpx.h.0p.html
    // https://github.com/fairyglade/ly/blob/master/src/login.c
    let entry = {
        // SAFETY: None of the fields in libc::utmpx have a drop implementation.
        let mut s: nix::libc::utmpx = unsafe { std::mem::zeroed() };

        // ut_line    --- Device name of tty - "/dev/"
        // ut_id      --- Terminal name suffix
        // ut_user    --- Username
        // ut_host    --- Hostname for remote login, or kernel version for run-level messages
        // ut_exit    --- Exit status of a process marked as DEAD_PROCESS; not used by Linux init(1)
        // ut_session --- Session ID (getsid(2)) used for windowing
        // ut_tv {    --- Time entry was made
        //     tv_sec     --- Seconds
        //     tv_usec    --- Microseconds
        // }
        // ut_addr_v6 --- Internet address of remote

        s.ut_type = nix::libc::USER_PROCESS;
        s.ut_pid = pid;

        for (i, b) in username.as_bytes().iter().take(32).enumerate() {
            s.ut_user[i] = *b as nix::libc::c_char;
        }

        if tty > 12 {
            error!("Invalid TTY");
            std::process::exit(1);
        }
        let tty_c_char = (b'0' + tty) as nix::libc::c_char;

        s.ut_line[0] = b't' as nix::libc::c_char;
        s.ut_line[1] = b't' as nix::libc::c_char;
        s.ut_line[2] = b'y' as nix::libc::c_char;
        s.ut_line[3] = tty_c_char;

        s.ut_id[0] = tty_c_char;

        use std::time::SystemTime;

        let epoch_duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| {
                error!("Invalid System Time");
                std::process::exit(1);
            })
            .as_micros();

        s.ut_tv.tv_sec = (epoch_duration / 1_000_000).try_into().unwrap_or_else(|_| {
            error!("Invalid System Time (TV_SEC Overflow)");
            std::process::exit(1);
        });
        s.ut_tv.tv_usec = (epoch_duration % 1_000_000).try_into().unwrap_or_else(|_| {
            error!("Invalid System Time (TV_USEC Overflow)");
            std::process::exit(1);
        });

        s
    };

    unsafe {
        nix::libc::setutxent();
        nix::libc::pututxline(&entry as *const nix::libc::utmpx);
    };

    info!("Added UTMPX record");

    entry
}

/// Taked from lemurs DM
pub fn drop_utmpx_entry(mut entry: utmpx) {
    info!("Removing UTMPX record");

    entry.ut_type = nix::libc::DEAD_PROCESS;

    entry.ut_line = <[nix::libc::c_char; 32]>::default();
    entry.ut_user = <[nix::libc::c_char; 32]>::default();

    entry.ut_tv.tv_usec = 0;
    entry.ut_tv.tv_sec = 0;

    unsafe {
        nix::libc::setutxent();
        nix::libc::pututxline(&entry as *const nix::libc::utmpx);
        nix::libc::endutxent();
    }
}