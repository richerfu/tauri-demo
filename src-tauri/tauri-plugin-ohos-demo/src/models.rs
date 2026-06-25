use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub platform: &'static str,
    pub runtime: &'static str,
    pub uses_native_ability: bool,
}
