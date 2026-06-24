use crate::models::PlatformInfo;

pub(crate) fn platform_info() -> PlatformInfo {
    PlatformInfo {
        platform: platform_name(),
        runtime: runtime_name(),
        uses_native_ability: uses_native_ability(),
    }
}

#[cfg(target_env = "ohos")]
pub(crate) fn platform_name() -> &'static str {
    "ohos"
}

#[cfg(all(not(target_env = "ohos"), target_os = "android"))]
pub(crate) fn platform_name() -> &'static str {
    "android"
}

#[cfg(all(not(target_env = "ohos"), target_os = "ios"))]
pub(crate) fn platform_name() -> &'static str {
    "ios"
}

#[cfg(all(not(target_env = "ohos"), target_os = "macos"))]
pub(crate) fn platform_name() -> &'static str {
    "macos"
}

#[cfg(all(not(target_env = "ohos"), target_os = "windows"))]
pub(crate) fn platform_name() -> &'static str {
    "windows"
}

#[cfg(all(not(target_env = "ohos"), target_os = "linux"))]
pub(crate) fn platform_name() -> &'static str {
    "linux"
}

#[cfg(not(any(
    target_env = "ohos",
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux"
)))]
pub(crate) fn platform_name() -> &'static str {
    "unknown"
}

#[cfg(target_env = "ohos")]
fn runtime_name() -> &'static str {
    "openharmony-native-ability"
}

#[cfg(not(target_env = "ohos"))]
fn runtime_name() -> &'static str {
    "tauri"
}

#[cfg(target_env = "ohos")]
fn uses_native_ability() -> bool {
    true
}

#[cfg(not(target_env = "ohos"))]
fn uses_native_ability() -> bool {
    false
}
