use pam::Authenticator;

pub fn authenticate(username: &str, password: &str) -> pam::PamResult<()> {
    let mut auth = Authenticator::with_password("login")?;
    auth.get_handler().set_credentials(username, password);
    auth.authenticate()?;
    auth.open_session()?;
    Ok(())
}