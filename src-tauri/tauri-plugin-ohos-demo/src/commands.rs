use tauri::{command, AppHandle, Runtime};

use crate::{models::PlatformInfo, OhosDemoExt};

#[command]
pub(crate) async fn platform_info<R: Runtime>(app: AppHandle<R>) -> PlatformInfo {
    app.ohos_demo().platform_info()
}

#[command]
pub(crate) async fn echo<R: Runtime>(app: AppHandle<R>, message: String) -> String {
    app.ohos_demo().echo(message)
}
