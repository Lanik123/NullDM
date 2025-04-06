use log::error;
use nix::{sys::wait::{waitpid, WaitStatus}, unistd::{execvp, fork, getgrouplist, setgid, setgroups, setuid, ForkResult, Pid, User}};
use std::ffi::CString;

pub struct SessionHandler {
    pub username: String,
    pub shell: String,
    pub pid: Option<Pid>,
}

impl SessionHandler {
    pub fn new(username: &str, shell: &str) -> Self {
        Self {
            username: username.to_string(),
            shell: shell.to_string(),
            pid: None,
        }
    }

    pub fn spawn(&mut self) -> Result<(), String> {
        let user = match User::from_name(&self.username) {
            Ok(Some(u)) => u,
            _ => {
                return Err(format!("User '{}' not found", self.username));
            }
        };
    
        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                // Change groups
                if let Err(err) = setgroups(&getgrouplist(CString::new(user.name).unwrap().as_c_str(), user.gid).unwrap()) {
                    error!("setgroups failed: {err}");
                    std::process::exit(1);
                }

                // Change GID
                if let Err(err) = setgid(user.gid) {
                    error!("setgid failed: {err}");
                    std::process::exit(1);
                }
    
                // Change UID
                if let Err(err) = setuid(user.uid) {
                    error!("setuid failed: {err}");
                    std::process::exit(1);
                }
    
                let shell_c = CString::new(self.shell.clone()).unwrap();
                let _ = execvp(&shell_c, &[shell_c.clone()]);
                Ok(())
            }

            Ok(ForkResult::Parent { child }) => {
                // Wait when process die
                self.pid = Some(child);
                Ok(())
            }
    
            Err(e) => Err(format!("fork failed: {}", e)),
        }
    }

    pub fn wait(&self) -> Option<WaitStatus> {
        self.pid.and_then(|pid| waitpid(pid, None).ok())
    }
}