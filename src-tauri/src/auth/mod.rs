#![allow(unused)]

use std::path::Path;

pub use crate::auth::profile::Profile;
pub use crate::auth::storage::ProfileData;

pub mod error;
mod profile;
mod storage;

pub fn login(username: &str, password: &str) -> Result<Profile, error::AuthError> {
    profile::Profile::new(username.to_string(), password.to_string())
}

pub async fn load(username: &str, data_dir: &Path) -> Result<ProfileData, error::AuthError> {
    storage::load(username.to_string(), data_dir).await
}

pub async fn save(profile_data: ProfileData, data_dir: &Path) -> Result<(), error::AuthError> {
    storage::save(profile_data, data_dir).await
}
