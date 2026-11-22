#[cfg(target_env = "ohos")]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(target_env = "ohos")]
use napi_derive_ohos::napi;
#[cfg(target_env = "ohos")]
use napi_ohos::{
    bindgen_prelude::{Env, FunctionRef, Result as NapiResult, Status},
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};

use serde::Serialize;
use tauri::Manager;
use tauri_plugin_ohos_demo::OhosDemoExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(target_env = "ohos")]
type MainThreadArkTsCallback = FunctionRef<(String,), String>;
#[cfg(target_env = "ohos")]
type AsyncArkTsCallback = ThreadsafeFunction<String, (), String, Status, false>;
#[cfg(target_env = "ohos")]
type MainThreadDispatcher = ThreadsafeFunction<String, String, String, Status, false>;

#[cfg(target_env = "ohos")]
static MAIN_THREAD_CALLBACK: OnceLock<Mutex<Option<MainThreadArkTsCallback>>> = OnceLock::new();
#[cfg(target_env = "ohos")]
static ASYNC_THREAD_CALLBACK: OnceLock<Mutex<Option<Arc<AsyncArkTsCallback>>>> = OnceLock::new();
#[cfg(target_env = "ohos")]
static MAIN_THREAD_DISPATCHER: OnceLock<Mutex<Option<Arc<MainThreadDispatcher>>>> = OnceLock::new();

#[cfg(target_env = "ohos")]
fn main_thread_callback() -> &'static Mutex<Option<MainThreadArkTsCallback>> {
    MAIN_THREAD_CALLBACK.get_or_init(|| Mutex::new(None))
}

#[cfg(target_env = "ohos")]
fn async_thread_callback() -> &'static Mutex<Option<Arc<AsyncArkTsCallback>>> {
    ASYNC_THREAD_CALLBACK.get_or_init(|| Mutex::new(None))
}

#[cfg(target_env = "ohos")]
fn main_thread_dispatcher() -> &'static Mutex<Option<Arc<MainThreadDispatcher>>> {
    MAIN_THREAD_DISPATCHER.get_or_init(|| Mutex::new(None))
}

#[cfg(target_env = "ohos")]
#[napi]
pub fn register_main_thread_arkts_callback(callback: MainThreadArkTsCallback) {
    *main_thread_callback()
        .lock()
        .expect("main thread ArkTS callback lock poisoned") = Some(callback);
}

#[cfg(target_env = "ohos")]
#[napi]
pub fn call_main_thread_arkts_callback(env: Env, message: String) -> NapiResult<String> {
    call_main_thread_arkts_callback_with_env(env, message)
}

#[cfg(target_env = "ohos")]
fn call_main_thread_arkts_callback_with_env(env: Env, message: String) -> NapiResult<String> {
    let callback_guard = main_thread_callback()
        .lock()
        .expect("main thread ArkTS callback lock poisoned");
    let callback = callback_guard.as_ref().ok_or_else(|| {
        napi_ohos::Error::new(Status::InvalidArg, "main thread callback is not registered")
    })?;

    callback.borrow_back(&env)?.call((message,))
}

#[cfg(target_env = "ohos")]
#[napi]
pub fn register_async_thread_arkts_callback(callback: Arc<AsyncArkTsCallback>) {
    *async_thread_callback()
        .lock()
        .expect("async thread ArkTS callback lock poisoned") = Some(callback);
}

#[cfg(target_env = "ohos")]
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
        let status = callback.call(message, ThreadsafeFunctionCallMode::NonBlocking);
        debug_assert_eq!(status, Status::Ok, "async ArkTS tsfn call failed");
    });

    Ok(())
}

#[cfg(target_env = "ohos")]
#[napi]
pub fn register_main_thread_dispatcher(callback: Arc<MainThreadDispatcher>) {
    *main_thread_dispatcher()
        .lock()
        .expect("main thread dispatcher lock poisoned") = Some(callback);
}

#[tauri::command]
fn call_arkts_function_from_frontend(message: String) -> Result<String, String> {
    #[cfg(target_env = "ohos")]
    {
        let dispatcher = main_thread_dispatcher()
            .lock()
            .expect("main thread dispatcher lock poisoned")
            .as_ref()
            .cloned()
            .ok_or_else(|| "main thread dispatcher is not registered".to_string())?;
        let display_message = message.clone();
        let status = dispatcher.call_with_return_value(
            message,
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                match result {
                    Ok(value) => log::info!("ArkTS Function callback returned: {value}"),
                    Err(error) => log::error!("ArkTS Function callback failed: {error}"),
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
    #[cfg(not(target_env = "ohos"))]
    {
        Ok(format!("unsupported on non-ohos: {message}"))
    }
}

#[tauri::command]
fn call_arkts_tsfn_from_frontend(message: String) -> Result<String, String> {
    #[cfg(target_env = "ohos")]
    {
        call_async_thread_arkts_callback(message.clone())
            .map(|_| format!("queued TSFN callback: {message}"))
            .map_err(|error| error.to_string())
    }
    #[cfg(not(target_env = "ohos"))]
    {
        Ok(format!("unsupported on non-ohos: {message}"))
    }
}

#[derive(Debug, Clone, Serialize)]
struct RegressionCheck {
    name: String,
    ok: bool,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct RegressionReport {
    passed: usize,
    failed: usize,
    checks: Vec<RegressionCheck>,
}

fn push_check(checks: &mut Vec<RegressionCheck>, name: &str, ok: bool, detail: impl Into<String>) {
    let detail = detail.into();
    log::info!(
        "[TAURI_REGRESSION] {} => {} ({})",
        name,
        if ok { "PASS" } else { "FAIL" },
        detail
    );
    checks.push(RegressionCheck {
        name: name.to_string(),
        ok,
        detail,
    });
}

#[tauri::command]
async fn run_ohos_regression_checks(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> RegressionReport {
    let mut checks = Vec::new();

    // 1. Custom plugin (tauri-plugin-ohos-demo)
    let demo_info = app.ohos_demo().platform_info();
    let demo_echo = app.ohos_demo().echo("regression".to_string());
    push_check(
        &mut checks,
        "plugin-ohos-demo",
        demo_echo.contains("regression") && (!cfg!(target_env = "ohos") || demo_info.uses_native_ability),
        format!(
            "platform={}, runtime={}, native={}, echo={}",
            demo_info.platform, demo_info.runtime, demo_info.uses_native_ability, demo_echo
        ),
    );

    // 2. PathResolver (app_data_dir, app_cache_dir, resource_dir)
    let data_dir = app.path().app_data_dir();
    let cache_dir = app.path().app_cache_dir();
    let resource_dir = app.path().resource_dir();
    push_check(
        &mut checks,
        "path-resolver",
        data_dir.is_ok() && cache_dir.is_ok() && resource_dir.is_ok(),
        format!(
            "data={:?}, cache={:?}, resource={:?}",
            data_dir, cache_dir, resource_dir
        ),
    );

    // 3. tao Window & Monitor APIs (inner_size, outer_position, scale_factor, monitors, cached title)
    let inner_size = window.inner_size();
    let outer_pos = window.outer_position();
    let scale = window.scale_factor();
    let monitors = window.available_monitors();
    let current_mon = window.current_monitor();
    let set_title_res = window.set_title("tauri-demo verified");
    let title_readback = window.title();
    let theme = window.theme();
    let title_ok = set_title_res.is_ok()
        && title_readback
            .as_deref()
            .map(|t| t == "tauri-demo verified")
            .unwrap_or(false);
    push_check(
        &mut checks,
        "tao-window-and-monitors",
        inner_size.is_ok() && scale.is_ok() && monitors.as_ref().map(|m| !m.is_empty()).unwrap_or(false) && title_ok,
        format!(
            "size={:?}, pos={:?}, scale={:?}, monitors={}, current={:?}, title={:?}, theme={:?}",
            inner_size,
            outer_pos,
            scale,
            monitors.as_ref().map(|m| m.len()).unwrap_or(0),
            current_mon.as_ref().ok().and_then(|o| o.as_ref().map(|m| m.size())),
            title_readback,
            theme
        ),
    );

    // 4. wry WebView APIs (url, eval)
    let url = window.url();
    let eval_res = window.eval("window.__wryEvalProbe = 'ok';");
    push_check(
        &mut checks,
        "wry-url-and-eval",
        url.is_ok() && eval_res.is_ok(),
        format!("url={:?}, eval={:?}", url, eval_res),
    );

    // 5. muda Menu + MenuBridgePlugin
    let menu_res = (|| -> tauri::Result<()> {
        let item = tauri::menu::MenuItem::with_id(&app, "probe_item", "Probe Item", true, None::<&str>)?;
        let sub = tauri::menu::Submenu::with_items(&app, "File", true, &[&item])?;
        let menu = tauri::menu::Menu::with_items(&app, &[&sub])?;
        let _ = window.set_menu(menu)?;
        Ok(())
    })();
    push_check(
        &mut checks,
        "muda-menubar",
        menu_res.is_ok(),
        format!("{:?}", menu_res),
    );

    // 6. Direct WindowClient Vibrancy (no window-vibrancy fork)
    let vib_apply = window.set_effects(tauri::window::EffectsBuilder::new().effect(tauri::window::Effect::Blur).radius(12.0).build());
    let vib_clear = window.set_effects(None);
    push_check(
        &mut checks,
        "tauri-direct-vibrancy",
        vib_apply.is_ok() && vib_clear.is_ok(),
        format!("apply={:?}, clear={:?}", vib_apply, vib_clear),
    );

    // 7. tauri-plugin-os
    let os_platform = tauri_plugin_os::platform();
    let os_family = tauri_plugin_os::family();
    let os_version = tauri_plugin_os::version();
    let os_locale = tauri_plugin_os::locale();
    push_check(
        &mut checks,
        "plugin-os",
        !os_platform.is_empty() && !os_family.is_empty(),
        format!(
            "platform={}, family={}, version={}, locale={:?}",
            os_platform, os_family, os_version, os_locale
        ),
    );

    // 8. tauri-plugin-clipboard-manager (write_text + write_html with alt_text)
    {
        use tauri_plugin_clipboard_manager::ClipboardExt;
        #[cfg(target_env = "ohos")]
        let (w_text, w_html, r_text) = {
            let w_text = app.clipboard().write_text("tauri-ohos-probe").await;
            let w_html = app
                .clipboard()
                .write_html("<b>tauri-ohos-probe</b>", Some("tauri-ohos-probe"))
                .await;
            let r_text = app.clipboard().read_text().await;
            (w_text, w_html, r_text)
        };
        #[cfg(not(target_env = "ohos"))]
        let (w_text, w_html, r_text) = {
            let w_text = app.clipboard().write_text("tauri-ohos-probe");
            let w_html = app
                .clipboard()
                .write_html("<b>tauri-ohos-probe</b>", Some("tauri-ohos-probe"));
            let r_text = app.clipboard().read_text();
            (w_text, w_html, r_text)
        };
        push_check(
            &mut checks,
            "plugin-clipboard-manager",
            w_text.is_ok() && w_html.is_ok(),
            format!(
                "write_text={:?}, write_html={:?}, read_text={:?}",
                w_text, w_html, r_text
            ),
        );
    }

    // 9. Multi-window (tao SubWindow + wry WebView) create & close
    let sub_res = (|| -> tauri::Result<String> {
        let sub = tauri::WebviewWindowBuilder::new(
            &app,
            "sub-regression",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("Sub Window Probe")
        .inner_size(320.0, 240.0)
        .build()?;
        let label = sub.label().to_string();
        sub.close()?;
        Ok(label)
    })();
    push_check(
        &mut checks,
        "multi-window-subwindow",
        sub_res.is_ok(),
        format!("{:?}", sub_res),
    );

    // 10. ArkTS callbacks
    let arkts_fn = call_arkts_function_from_frontend("regression-fn".to_string());
    let arkts_tsfn = call_arkts_tsfn_from_frontend("regression-tsfn".to_string());
    push_check(
        &mut checks,
        "arkts-callbacks",
        arkts_fn.is_ok() && arkts_tsfn.is_ok(),
        format!("fn={:?}, tsfn={:?}", arkts_fn, arkts_tsfn),
    );

    let passed = checks.iter().filter(|c| c.ok).count();
    let failed = checks.len() - passed;
    log::info!(
        "[TAURI_REGRESSION_SUMMARY] passed={} failed={} total={}",
        passed,
        failed,
        checks.len()
    );
    RegressionReport {
        passed,
        failed,
        checks,
    }
}

#[tauri::command]
async fn trigger_dialog_message_probe(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .message("OpenHarmony plugin-window prompt probe")
        .title("Tauri Dialog Probe")
        .show(move |ok| {
            log::info!("[TAURI_DIALOG_PROBE] callback ok={ok}");
            let _ = tx.send(ok);
        });
    let res = tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_secs(30))
    })
    .await
    .map_err(|e| format!("dialog join error: {e}"))?
    .map_err(|e| format!("dialog timeout: {e}"))?;
    Ok(format!("dialog_result={res}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[cfg_attr(target_env = "ohos", tauri::ohos::entry_point)]
pub fn run() {
    let mut log_builder = tauri_plugin_log::Builder::new().level(log::LevelFilter::Info);
    #[cfg(target_env = "ohos")]
    {
        log_builder = log_builder.target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Hilog {
                tag: "TauriDemo".to_string(),
            },
        ));
    }

    tauri::Builder::default()
        .plugin(log_builder.build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_ohos_demo::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            call_arkts_function_from_frontend,
            call_arkts_tsfn_from_frontend,
            run_ohos_regression_checks,
            trigger_dialog_message_probe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
