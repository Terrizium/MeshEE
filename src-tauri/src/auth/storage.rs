#![allow(unused)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::auth::error::AuthError;

pub async fn load(username: String, data_dir: &Path) -> Result<ProfileData, AuthError> {
    if username.is_empty() {
        return Err(AuthError::InvalidUsername);
    }
    let safe_username = sanitize_username(username);
    let file_path = data_dir.join(format!("{}.json", safe_username));

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => Ok(serde_json::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let default = ProfileData::default();
            let json = serde_json::to_string(&default)?;
            fs::write(&file_path, json).await?;
            Ok(default)
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileData {
    pub chats: Vec<Chat>,
    pub peer_id: String,
}
impl ProfileData {
    fn default() -> Self {
        ProfileData {
            chats: Vec::new(),
            peer_id: String::new(),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Chat {
    pub name: String,
    pub messages: Vec<Message>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Message {
    pub username: String,
    pub text: String,
    pub created_at: String,
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
