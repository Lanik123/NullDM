mod auth;
mod config;
mod logger;
mod session;
mod tty;

use log::{info, error};
use nix::unistd::User;

fn main() {
    logger::setup_logger("/var/log/nulldm.log");
    let config = config::setup_config("/etc/nulldm/config.toml");

    unsafe { tty::setup::setvt(config.tty.into()) }.unwrap_or_else(|err| {
        error!("Failed to switch tty {}. Reason: {err}", config.tty);
    });

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

        // User check
        if user.uid.as_raw() < config.min_uid {
            error!("[Attempt {attempt}] User '{}' is below min_uid threshold", username);
            continue;
        }

        // Auth
        match auth::authenticate(&username, &password) {
            Ok(session) => {
                info!("Login successful for {}", username);
                session::start_shell(&username, &config.default_shell);
                drop(session);
                break;
            }
            Err(e) => {
                error!("[Attempt {attempt}] Login failed for {}: {e:?}", username);
            }
        }

        if attempt == config.max_attempts {
            error!("Maximum number of attempts reached. Exiting.");
        }
    }
}
