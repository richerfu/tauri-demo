# tauri-plugin-ohos-demo

This is a standard Tauri v2 plugin project adapted for the OHOS prototype in this repo.

- `src/lib.rs` registers the Rust/native plugin.
- `src/commands.rs` exposes Tauri commands.
- `src/models.rs` owns serializable command payloads.
- `src/platform.rs` owns platform-specific Rust cfg branches, including OHOS.
- `guest-js/index.ts` is the TypeScript API consumed by the WebView.

The important OHOS point is that the generated OHOS app loads the Rust cdylib through `NativeAbility.moduleName` in `gen/ohos/entry/src/main/ets/entryability/EntryAbility.ets`. A command-only plugin can therefore live in the same Rust library as the app and be called from the WebView with the normal Tauri plugin IPC route:

```ts
await invoke("plugin:ohos-demo|platform_info");
```

The TypeScript API wraps that IPC route:

```ts
import { getPlatformInfo, echo } from "./guest-js";

await getPlatformInfo();
await echo("hello from OHOS");
```

For platform-specific native work, keep the public Tauri command shape stable and move the implementation behind Rust cfgs:

```rust
#[cfg(target_env = "ohos")]
fn platform_name() -> &'static str {
    "ohos"
}
```

If a plugin needs ArkTS or N-API code instead of pure Rust commands, add that OHOS native layer beside the generated OHOS project and keep the TS API in `guest-js/index.ts` unchanged.
