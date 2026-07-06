use std::sync::{Arc, Mutex, OnceLock};

use napi_derive_ohos::napi;
use napi_ohos::{
    bindgen_prelude::{Env, FunctionRef, Result as NapiResult, Status},
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

type MainThreadArkTsCallback = FunctionRef<(String,), String>;
type AsyncArkTsCallback = ThreadsafeFunction<(String,), ()>;
type MainThreadDispatcher = ThreadsafeFunction<(String,), String>;

static MAIN_THREAD_CALLBACK: OnceLock<Mutex<Option<MainThreadArkTsCallback>>> = OnceLock::new();
static ASYNC_THREAD_CALLBACK: OnceLock<Mutex<Option<Arc<AsyncArkTsCallback>>>> = OnceLock::new();
static MAIN_THREAD_DISPATCHER: OnceLock<Mutex<Option<Arc<MainThreadDispatcher>>>> = OnceLock::new();

fn main_thread_callback() -> &'static Mutex<Option<MainThreadArkTsCallback>> {
    MAIN_THREAD_CALLBACK.get_or_init(|| Mutex::new(None))
}

fn async_thread_callback() -> &'static Mutex<Option<Arc<AsyncArkTsCallback>>> {
    ASYNC_THREAD_CALLBACK.get_or_init(|| Mutex::new(None))
}

fn main_thread_dispatcher() -> &'static Mutex<Option<Arc<MainThreadDispatcher>>> {
    MAIN_THREAD_DISPATCHER.get_or_init(|| Mutex::new(None))
}

#[napi]
pub fn register_main_thread_arkts_callback(callback: MainThreadArkTsCallback) {
    *main_thread_callback()
        .lock()
        .expect("main thread ArkTS callback lock poisoned") = Some(callback);
}

#[napi]
pub fn call_main_thread_arkts_callback(env: Env, message: String) -> NapiResult<String> {
    call_main_thread_arkts_callback_with_env(env, message)
}

fn call_main_thread_arkts_callback_with_env(env: Env, message: String) -> NapiResult<String> {
    let callback_guard = main_thread_callback()
        .lock()
        .expect("main thread ArkTS callback lock poisoned");
    let callback = callback_guard.as_ref().ok_or_else(|| {
        napi_ohos::Error::new(Status::InvalidArg, "main thread callback is not registered")
    })?;

    callback.borrow_back(&env)?.call((message,))
}

#[napi]
pub fn register_async_thread_arkts_callback(callback: Arc<AsyncArkTsCallback>) {
    *async_thread_callback()
        .lock()
        .expect("async thread ArkTS callback lock poisoned") = Some(callback);
}

#[napi]
pub fn call_async_thread_arkts_callback(message: String) -> NapiResult<()> {
    let callback = async_thread_callback()
        .lock()
        .expect("async thread ArkTS callback lock poisoned")
        .as_ref()
        .cloned()
        .ok_or_else(|| {
            napi_ohos::Error::new(
                Status::InvalidArg,
                "async thread callback is not registered",
            )
        })?;

    std::thread::spawn(move || {
        let status = callback.call(Ok((message,)), ThreadsafeFunctionCallMode::NonBlocking);
        debug_assert_eq!(status, Status::Ok, "async ArkTS tsfn call failed");
    });

    Ok(())
}

#[napi]
pub fn register_main_thread_dispatcher(callback: Arc<MainThreadDispatcher>) {
    *main_thread_dispatcher()
        .lock()
        .expect("main thread dispatcher lock poisoned") = Some(callback);
}

#[tauri::command]
fn call_arkts_function_from_frontend(message: String) -> Result<String, String> {
    let dispatcher = main_thread_dispatcher()
        .lock()
        .expect("main thread dispatcher lock poisoned")
        .as_ref()
        .cloned()
        .ok_or_else(|| "main thread dispatcher is not registered".to_string())?;
    let display_message = message.clone();
    let status = dispatcher.call_with_return_value(
        Ok((message,)),
        ThreadsafeFunctionCallMode::NonBlocking,
        move |result, _env| {
            match result {
                Ok(value) => println!("ArkTS Function callback returned: {value}"),
                Err(error) => eprintln!("ArkTS Function callback failed: {error}"),
            }
            Ok(())
        },
    );

    if status != Status::Ok {
        return Err(format!(
            "failed to queue main thread dispatcher: {status:?}"
        ));
    }

    Ok(format!("queued ArkTS Function callback: {display_message}"))
}

#[tauri::command]
fn call_arkts_tsfn_from_frontend(message: String) -> Result<String, String> {
    call_async_thread_arkts_callback(message.clone())
        .map(|_| format!("queued TSFN callback: {message}"))
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_ohos_demo::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            call_arkts_function_from_frontend,
            call_arkts_tsfn_from_frontend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
