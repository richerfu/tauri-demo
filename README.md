# Tauri prototype for OpenHarmony/HarmonyNext

![Logo](./fixture/demo.png)

## Setup

1. Install tauri-cli and ohrs from git.
```bash
cargo install tauri-cli --git https://github.com/tauri-apps/tauri --branch feat/open-harmony

cargo install ohrs
```

2. Clone the repo

```
git clone https://github.com/richerfu/tauri-demo.git
```

3. Install the dependencies.

```bash
pnpm install

cd src-tauri && cargo fetch
```

4. Configure the OHOS SDK. `OHOS_HOME` must point to the SDK directory that
   contains `ets`, `js`, `native`, and `toolchains`. Make sure `ohpm` and
   `hvigorw` are available in `PATH`.

```bash
export OHOS_HOME=/path/to/ohos-sdk/darwin
export PATH=/path/to/command-line-tools/bin:$PATH
```

## Build and run

Build with tauri-cli. Run the command from the repo root so the overridden
`beforeBuildCommand` can always build the frontend from the correct directory.

```bash
ROOT="$(pwd)"

cd src-tauri \
OHOS_HOME="${OHOS_HOME:-/path/to/ohos-sdk/darwin}" \
cargo tauri ohos build \
  --config "{\"build\":{\"beforeBuildCommand\":\"pnpm --dir '${ROOT}' build\"}}"
```

If you get the following error, ignore it.

![error](./fixture/image.png)

If the Rust build succeeds but the Tauri CLI cannot assemble the HAP, assemble
the generated OHOS project directly:

```bash
cd src-tauri/gen/ohos
ohpm install
hvigorw clean --no-daemon
hvigorw assembleHap --no-daemon --stacktrace
```

The unsigned HAP is generated at:

```text
src-tauri/gen/ohos/entry/build/default/outputs/default/entry-default-unsigned.hap
```

Open `src-tauri/gen/ohos` within DevEco Studio to run or configure signing. The
default command above creates an unsigned HAP because this demo does not include
`signingConfigs`.

## Note

1. `libentry.so` is a template library and you can ignore it.
2. `RustAbility` will forward lifecycle automatically.

## Tauri plugin OHOS example

This repo includes a standard Tauri plugin project at `src-tauri/tauri-plugin-ohos-demo`.

The plugin has two layers:

- Rust/native layer: owns the Tauri plugin, commands, state, and OHOS `cfg`
  branches.
- TypeScript guest layer: owns the frontend API wrapper used by the WebView.

For this command-only demo no extra ArkTS file is needed. Add ArkTS or N-API
code only when the plugin must call OHOS APIs that are not available from the
Rust side.

Rust side:

```rust
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("ohos-demo")
        .invoke_handler(tauri::generate_handler![
            commands::platform_info,
            commands::echo,
        ])
        .setup(|app, _api| {
            app.manage(OhosDemo);
            Ok(())
        })
        .build()
}
```

The app registers it in `src-tauri/src/lib.rs`:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_ohos_demo::init());
```

TS side, from the plugin's `guest-js/index.ts`:

```ts
import { echo, getPlatformInfo } from "../src-tauri/tauri-plugin-ohos-demo/guest-js";

await getPlatformInfo();
await echo("hello from Vue");
```

Packaging checklist:

1. Keep the Rust dependency in `src-tauri/Cargo.toml`:

   ```toml
   tauri-plugin-ohos-demo = { path = "tauri-plugin-ohos-demo" }
   ```

2. Register the plugin in `src-tauri/src/lib.rs`:

   ```rust
   tauri::Builder::default()
       .plugin(tauri_plugin_opener::init())
       .plugin(tauri_plugin_ohos_demo::init());
   ```

3. Keep the permission in `src-tauri/capabilities/default.json`:

   ```json
   "ohos-demo:default"
   ```

4. After packaging, verify the OHOS app library exists inside the HAP:

   ```bash
   unzip -l src-tauri/gen/ohos/entry/build/default/outputs/default/entry-default-unsigned.hap \
     | rg "libtauri_demo_lib.so|module.json|ets/modules.abc"
   ```

5. Runtime verification: launch the OHOS app and use the "Load plugin info" and
   "Plugin echo" buttons. Those call `plugin:ohos-demo|platform_info` and
   `plugin:ohos-demo|echo` through the TypeScript guest API.

The plugin was shaped after `tauri plugin new`: Rust/native code lives under `src/`, and the frontend API lives under `guest-js/`. OHOS-specific Rust branches use `#[cfg(target_env = "ohos")]`. For this command-only plugin no extra ArkTS code is required: the OHOS entry ability loads the app Rust cdylib through `NativeAbility.moduleName`, and the WebView calls the plugin through Tauri IPC. If a plugin needs a real OHOS native API, keep the TS API stable and put the platform implementation behind the OHOS cfg or an OHOS N-API/ArkTS layer.
