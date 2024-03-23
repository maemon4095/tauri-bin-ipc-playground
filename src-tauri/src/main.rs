// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod global_channel;

use global_channel::GlobalChannelSender;
use rand::{self, distributions::Alphanumeric, Rng, SeedableRng};
use tauri::State;

#[tauri::command]
async fn start(sender: State<'_, GlobalChannelSender>) -> Result<(), AnyError> {
    let mut rng = rand::rngs::SmallRng::from_entropy();
    loop {
        let mut buf = Vec::with_capacity(1024);
        for _ in 0..buf.capacity() {
            buf.push(rng.sample(Alphanumeric))
        }
        sender.send(buf).await?;
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(global_channel::init())
        .invoke_handler(tauri::generate_handler![start])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct AnyError(Box<dyn std::error::Error>);
impl<T: std::error::Error + 'static> From<T> for AnyError {
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}
impl serde::Serialize for AnyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self.0))
    }
}
