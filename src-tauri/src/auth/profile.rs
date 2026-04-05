#![allow(unused)]

use crate::auth::error::AuthError;

pub struct Profile {
    pub username: String,
    pub device_id: String,
}

impl Profile {
    pub fn new(username: String, password: String) -> Result<Self, AuthError> {
        let username = username.trim().to_string();
        let device_id = derive_device_id(username.clone(), password);
        Ok(Self {
            username,
            device_id,
        })
    }
}

fn derive_device_id(login: String, password: String) -> String {
    let device_id = get_device_id_or_fallback();
    let mut input = Vec::new();
    input.extend_from_slice(&(login.len() as u32).to_le_bytes());
    input.extend_from_slice(login.as_bytes());

    input.extend_from_slice(&(password.len() as u32).to_le_bytes());
    input.extend_from_slice(password.as_bytes());

    input.extend_from_slice(&(device_id.len() as u32).to_le_bytes());
    input.extend_from_slice(device_id.as_bytes());

    let hash = blake3::hash(&input);
    hash.to_hex().to_string()
}

fn fallback_device_id() -> String {
    let os = std::env::consts::OS;
    let hostname = whoami::hostname().unwrap_or("host".to_string());
    let username = whoami::username().unwrap_or("user".to_string());

    let mut hasher = blake3::Hasher::new();
    hasher.update(os.as_bytes());
    hasher.update(b"\0");
    hasher.update(hostname.as_bytes());
    hasher.update(b"\0");
    hasher.update(username.as_bytes());

    let hash = hasher.finalize();
    format!("fb:{}", hash.to_hex())
}

fn get_device_id_or_fallback() -> String {
    try_machine_id().unwrap_or_else(fallback_device_id)
}

fn try_machine_id() -> Option<String> {
    match machine_uid::get() {
        Ok(id) if !id.is_empty() => Some(id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_get_identity_string() {
        // todo!()
    }

    #[test]
    fn test_login() {
        let profile = Profile::new("test".to_string(), "test".to_string());
        assert!(profile.is_ok());
        let profile = profile.unwrap();
        assert_eq!(profile.username, "test".to_string());
        assert!(!profile.device_id.is_empty());
        assert_ne!(profile.device_id, "test".to_string());
    }

    #[test]
    fn test_derive_device_id() {
        let login = "test".to_string();
        let password = "test".to_string();
        let pass1 = derive_device_id(login.clone(), password.clone());
        let pass2 = derive_device_id(login, password);
        assert_eq!(pass1, pass2);
    }

    #[test]
    fn test_derive_device_id_is_different() {
        let pass1 = derive_device_id("test1".to_string(), "test1".to_string());
        let pass2 = derive_device_id("test1".to_string(), "test2".to_string());
        assert_ne!(pass1, pass2);
    }

    #[test]
    fn test_derive_device_id_not_empty() {
        let pass = derive_device_id("test".to_string(), "test".to_string());
        assert!(!pass.is_empty());
    }
}
