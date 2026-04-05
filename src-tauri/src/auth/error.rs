#![allow(unused)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("password error")]
    PasswordError,

    #[error("login error")]
    LoginError,
}
