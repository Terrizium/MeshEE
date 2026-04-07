use serde_json::json;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::{path::BaseDirectory, Manager};
use tokio::time::{interval, Duration};

use crate::auth::Profile;
use crate::auth::ProfileData;

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

#[tauri::command]
async fn start_periodic_messages(app_handle: tauri::AppHandle) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(3));
        let mut counter = 0;
        loop {
            interval.tick().await;
            counter += 1;
            let message = format!("Автоматическое сообщение #{}", counter);
            let _ = app_handle.emit(
                "new-message",
                &json!({
                    "id": counter,
                    "msg": message
                }),
            );
        }
    });
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

#[tauri::command]
async fn load(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ProfileData, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let username = state
        .cur_profile
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .username
        .clone();
    match auth::load(&username, &data_dir).await {
        Ok(data) => Ok(data),
        Err(err) => Err(format!("Failed to load data: {}", err)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            login,
            start_periodic_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
