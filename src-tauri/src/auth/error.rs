#![allow(unused)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("password error")]
    PasswordError,

    #[error("login error")]
    LoginError,

    #[error("Ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("Ошибка сериализации/десериализации JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Некорректное имя пользователя")]
    InvalidUsername,
}
