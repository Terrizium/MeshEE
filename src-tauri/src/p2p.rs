#![allow(unused)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct MsgRequest {
    pub uuid: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgResponse {
    pub content: Vec<u8>,
}

impl MsgRequest {
    pub fn new(message: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            message: message.trim().to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_msg_request_new() {
        let mr = MsgRequest::new("test".to_string());
        assert_eq!(mr.message, "test".to_string());
        assert!(!mr.uuid.is_empty());
        assert!(!mr.created_at.is_empty());
    }
}
