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

    loop {
        let username = match tty::print::prompt_username("Login: ") {
            Ok(u) if !u.trim().is_empty() => u.trim().to_string(),
            _ => {
                error!("Empty username input");
                continue;
            }
        };

        let password = match tty::print::prompt_password("Password: ") {
            Ok(p) if !p.trim().is_empty() => p,
            _ => {
                error!("Empty password input");
                continue;
            }
        };

        let user = match User::from_name(&username) {
            Ok(Some(u)) => u,
            Ok(None) => {
                error!("Unknown user: {}", username);
                continue;
            }
            Err(_) => {
                error!("Unknown user: {}", username);
                continue;
            },
        };

        // User check
        if user.uid.as_raw() < config.min_uid {
            error!("User '{}' is below min_uid threshold", username);
            continue;
        }

        // Auth
        match auth::authenticate(&username, &password) {
            Ok(_) => {
                info!("Login successful for {}", username);
                session::start_shell(&username, &config.default_shell);
                break;
            }
            Err(e) => {
                error!("Login failed for {}: {:?}", username, e);
            }
        }
    }
}
