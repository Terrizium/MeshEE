#![allow(unused)]

use std::path::Path;

use chrono::Local;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::auth::error::AuthError;

pub async fn save(profile_data: ProfileData, data_dir: &Path) -> Result<(), AuthError> {
    if profile_data.username.is_empty() {
        return Err(AuthError::InvalidUsername);
    }
    let file_path = data_dir.join(format!("{}.json", profile_data.username.clone()));
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let json = serde_json::to_string(&profile_data)?;
    fs::write(&file_path, json).await?;
    Ok(())
}

pub async fn load(username: String, data_dir: &Path) -> Result<ProfileData, AuthError> {
    if username.is_empty() {
        return Err(AuthError::InvalidUsername);
    }
    let safe_username = sanitize_username(username);
    let file_path = data_dir.join(format!("{}.json", safe_username.clone()));

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => Ok(serde_json::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut default = ProfileData::default();
            default.username = safe_username;
            let json = serde_json::to_string(&default)?;
            fs::write(&file_path, json).await?;
            Ok(default)
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileData {
    pub username: String,
    pub peer_id: String,
    pub chats: Vec<Chat>,
}
impl ProfileData {
    fn default() -> Self {
        ProfileData {
            username: String::new(),
            peer_id: String::new(),
            chats: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Chat {
    pub name: String,
    pub messages: Vec<Message>,
}

impl Chat {
    fn default() -> Self {
        Self {
            name: String::new(),
            messages: Vec::new(),
        }
    }
    fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Message {
    pub username: String,
    pub text: String,
    pub created_at: String,
    pub received_at: String,
}

impl Message {
    fn new(username: String, text: String) -> Self {
        Self {
            username,
            text,
            created_at: Local::now().to_rfc3339(),
            received_at: String::new(),
        }
    }
    fn received(&mut self) {
        self.received_at = Local::now().to_rfc3339();
    }
}

fn sanitize_username(username: String) -> String {
    username
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_save_and_load_equal() {
        let mut profile_data = ProfileData::default();
        profile_data.username = "test".to_string();
        profile_data.peer_id = "test".to_string();
        let dir = tempdir().unwrap();
        save(profile_data.clone(), dir.path())
            .await
            .expect("Not save");
        let load_profile_data = load("test".to_string(), dir.path())
            .await
            .expect("Not load");
        assert_eq!(profile_data.username, load_profile_data.username);
        assert_eq!(profile_data.peer_id, load_profile_data.peer_id);
    }

    #[tokio::test]
    async fn test_save() {
        let mut profile_data = ProfileData::default();
        profile_data.username = "test".to_string();
        let dir = tempdir().unwrap();
        save(profile_data, dir.path()).await.expect("Not save");
    }

    #[tokio::test]
    async fn test_load_new_username() {
        let dir = tempdir().unwrap();
        let profile_data = load("test".to_string(), dir.path())
            .await
            .expect("Not load");
    }
    #[tokio::test]
    async fn test_init_storage() {
        let dir = tempdir().unwrap();
        let _ = load("test".to_string(), dir.path())
            .await
            .expect("Not load");
    }
}
