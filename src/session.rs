
use nix::unistd::{execvp, setuid, User};
use std::ffi::CString;

pub fn start_shell(username: &str, shell: &str) {
    if let Ok(Some(user)) = User::from_name(username) {
        setuid(user.uid).expect("Failed to drop privileges");

        let shell_c = CString::new(shell).unwrap();
        execvp(&shell_c, &[shell_c.clone()]).expect("Failed to exec shell");
    }
}