use pam::{Authenticator, PasswordConv};

pub fn authenticate<'a>(username: &str, password: &str) -> pam::PamResult<Authenticator<'a, PasswordConv>> {
    let mut auth = Authenticator::with_password("login")?;
    auth.get_handler().set_credentials(username, password);
    auth.authenticate()?;
    auth.open_session()?;
    Ok(auth)
}