mod auth;
mod config;
mod logger;
mod session;
mod tty;

use log::{info, error};
use nix::unistd::getuid;
use session::SessionHandler;

fn main() {
    if getuid().as_raw() != 0 {
        eprintln!("Only start from root is allowed!");
        error!("Non-privileged run detected.");
        std::process::exit(1);
    }

    logger::setup_logger("/var/log/nulldm.log");
    let config = config::setup_config("/etc/nulldm/config.toml");

    unsafe { tty::setup::setvt(config.tty.into()) }.unwrap_or_else(|err| {
        error!("Failed to switch tty {}. Reason: {err}", config.tty);
    });

    match auth::handle_login(&config) {
        Ok((username, auth_session)) => {
            info!("Login successful for {}", username);
            let mut session = SessionHandler::new(&username, &config.default_shell);
            if let Err(e) = session.spawn() {
                error!("Failed to spawn session: {e}");
                std::process::exit(0);
            }

            let utmpx = auth::add_utmpx_entry(&username, config.tty, session.pid.unwrap().as_raw());
            // Wait when session die
            if let Some(status) = session.wait() {
                info!("Session exited with status: {:?}", status);
            }
            auth::drop_utmpx_entry(utmpx);
            drop(auth_session);
        }
        Err(e) => {
            error!("Login failed: {e}");
        }
    }

    std::process::exit(0);
}
