use log::error;
use nix::{sys::wait::waitpid, unistd::{execvp, fork, setsid, setuid, ForkResult, Uid, User}};
use std::ffi::CString;

pub fn start_shell(username: &str, shell: &str) {
    let user = match User::from_name(username) {
        Ok(Some(u)) => u,
        _ => {
            error!("User '{}' not found", username);
            return;
        }
    };

    match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            // Create new session
            if let Err(err) = setsid() {
                error!("setsid failed: {err}");
                std::process::exit(1);
            }

            // Change UID
            if let Err(err) = setuid(user.uid) {
                error!("setuid failed: {err}");
                std::process::exit(1);
            }

            let shell_c = CString::new(shell).unwrap();
            let _ = execvp(&shell_c, &[shell_c.clone()]);
        }

        Ok(ForkResult::Parent { child }) => {
            // Wait when process die
            let _ = waitpid(child, None);
        }

        Err(e) => {
            error!("Fork failed: {e}");
        }
    }
}