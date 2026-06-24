use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

mod commands;
mod models;
mod platform;

/// Access to the ohos-demo APIs.
pub struct OhosDemo;

impl OhosDemo {
    pub fn platform_info(&self) -> PlatformInfo {
        platform::platform_info()
    }

    pub fn echo(&self, message: String) -> String {
        format!("[{}] {}", platform::platform_name(), message)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the ohos-demo APIs.
pub trait OhosDemoExt<R: Runtime> {
    fn ohos_demo(&self) -> &OhosDemo;
}

impl<R: Runtime, T: Manager<R>> OhosDemoExt<R> for T {
    fn ohos_demo(&self) -> &OhosDemo {
        self.state::<OhosDemo>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ohos-demo")
        .invoke_handler(tauri::generate_handler![
            commands::platform_info,
            commands::echo
        ])
        .setup(|app, _api| {
            app.manage(OhosDemo);
            Ok(())
        })
        .build()
}
