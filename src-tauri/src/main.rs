// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use async_channel::{bounded, Receiver, Sender};
use rand::{self, distributions::Alphanumeric, Rng, SeedableRng};
use tauri::{http, AppHandle, Manager, State};

#[tauri::command]
async fn start(sender: State<'_, BinIpcSender>) -> Result<(), AnyError> {
    let mut rng = rand::rngs::SmallRng::from_entropy();
    loop {
        let mut buf = Vec::with_capacity(1024);
        for _ in 0..buf.capacity() {
            buf.push(rng.sample(Alphanumeric))
        }
        sender.send(buf).await?;
    }
}

fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("bin-ipc")
        .setup(|app| {
            let (sender, if_receiver) = bounded(32);
            let (if_sender, receiver) = bounded(32);
            app.manage(BinIpcState { sender, receiver });
            app.manage(BinIpcSender {
                sender: if_sender,
                event_name: "bin-ipc:ready",
                app_handle: app.app_handle(),
            });
            app.manage(BinIpcReceiver {
                receiver: if_receiver,
            });

            Ok(())
        })
        .register_uri_scheme_protocol(
            "bin-ipc",
            |app_handle: &AppHandle<R>, req: &http::Request| {
                if req.method() != http::method::Method::POST {
                    return Err(InvalidMethodError.into());
                }

                let uri: http::Uri = req.uri().parse().unwrap();
                let state = app_handle.state::<BinIpcState>();

                if uri.path() == "/pop" {
                    let response = http::ResponseBuilder::new()
                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
                    match state.receiver.try_recv() {
                        Ok(body) => {
                            return response.status(http::status::StatusCode::OK).body(body);
                        }
                        Err(e) => match e {
                            async_channel::TryRecvError::Empty => {
                                return response
                                    .status(http::status::StatusCode::CONTINUE)
                                    .body(Vec::new())
                            }
                            async_channel::TryRecvError::Closed => {
                                return response
                                    .status(http::status::StatusCode::NO_CONTENT)
                                    .body(Vec::new())
                            }
                        },
                    };
                }
                if uri.path() == "/push" {
                    let body = req.body();
                    tauri::async_runtime::block_on(async {
                        state.sender.send(body.clone()).await
                    })?;
                    return http::ResponseBuilder::new()
                        .status(http::status::StatusCode::ACCEPTED)
                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(Vec::new());
                }

                Err(UnknownBinIpcRequestMethod.into())
            },
        )
        .build()
}

fn main() {
    tauri::Builder::default()
        .plugin(init())
        .invoke_handler(tauri::generate_handler![start])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct BinIpcReceiver {
    receiver: Receiver<Vec<u8>>,
}
impl BinIpcReceiver {
    pub async fn recv(&self) -> Result<Vec<u8>, async_channel::RecvError> {
        self.receiver.recv().await
    }

    pub async fn try_recv(&self) -> Result<Vec<u8>, async_channel::TryRecvError> {
        self.receiver.try_recv()
    }
}

struct BinIpcSender<R: tauri::Runtime = tauri::Wry> {
    sender: Sender<Vec<u8>>,
    event_name: &'static str,
    app_handle: AppHandle<R>,
}
impl<R: tauri::Runtime> BinIpcSender<R> {
    pub async fn send(&self, message: Vec<u8>) -> Result<(), async_channel::SendError<Vec<u8>>> {
        self.sender.send(message).await?;
        self.app_handle.emit_all(self.event_name, ()).unwrap();
        Ok(())
    }

    pub fn try_send(&self, message: Vec<u8>) -> Result<(), async_channel::TrySendError<Vec<u8>>> {
        self.sender.try_send(message)?;
        self.app_handle.emit_all(self.event_name, ()).unwrap();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }

    pub fn len(&self) -> usize {
        self.sender.len()
    }
}

struct BinIpcState {
    sender: Sender<Vec<u8>>,
    receiver: Receiver<Vec<u8>>,
}

#[derive(Debug)]
struct InvalidMethodError;
impl std::fmt::Display for InvalidMethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "binary ipc request method must be POST")
    }
}
impl std::error::Error for InvalidMethodError {}

#[derive(Debug)]
struct MissingRequestMethodError;

impl std::fmt::Display for MissingRequestMethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "missing binary ipc request method")
    }
}
impl std::error::Error for MissingRequestMethodError {}

#[derive(Debug)]
struct UnknownBinIpcRequestMethod;
impl std::fmt::Display for UnknownBinIpcRequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unexpected bin ipc method")
    }
}
impl std::error::Error for UnknownBinIpcRequestMethod {}

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
