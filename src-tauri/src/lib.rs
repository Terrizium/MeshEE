use std::sync::Mutex;

use crate::auth::Profile;

mod auth;
mod p2p;

pub struct AppState {
    pub cur_profile: Mutex<Option<Profile>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cur_profile: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn login(name: &str, password: &str, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    match auth::login(name, password) {
        Ok(profile) => {
            //TODO:: Если переходить на tokio, то +async и заменить Mutex
            let mut lock = state.cur_profile.lock().unwrap();
            *lock = Some(profile);
            Ok(true)
        }
        Err(err) => Err(format!("Ошибка входа: {}", err.to_string())),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
