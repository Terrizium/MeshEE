pub use crate::auth::profile::Profile;

pub mod error;
mod profile;

pub fn login(username: &str, password: &str) -> Result<Profile, error::AuthError> {
    profile::Profile::new(username.to_string(), password.to_string())
}
