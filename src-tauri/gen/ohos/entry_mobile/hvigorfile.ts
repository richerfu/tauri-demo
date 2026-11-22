import { hapTasks } from '@ohos/hvigor-ohos-plugin';
import { hvigor, HvigorPlugin, HvigorNode } from '@ohos/hvigor';
import { execFileSync } from 'child_process';
import { resolve } from 'path';

export default {
  system: hapTasks,  /* Built-in plugin of Hvigor. It cannot be modified. */
  plugins: [tauriPlugin()]         /* Custom plugin to extend the functionality of Hvigor. */
}

function tauriPlugin(): HvigorPlugin {
  return {
    pluginId: 'tauri',
    apply(node: HvigorNode) {
      const buildRustCode = () => {
        // When the Tauri CLI drives the build directly (non `--open` paths:
        // `ohos build`, `ohos dev` run, `ohos build --app`), it has already
        // compiled the Rust `.so` itself via `Target::build`.
        if (process.env.TAURI_OHOS_SKIP_DEVECO_SCRIPT) return;
        // Bake this entry module's form so the `.so` compiled via
        // `dev-eco-studio-script` gets the right `cfg(mobile)`/`cfg(desktop)`.
        process.env.OHOS_DEVICE_TYPE = "mobile";
        const properties = hvigor.getParameter().getProperties();
        const target = properties.target || "aarch64";
        execFileSync(`/Volumes/PSSD/code/southorange/tauri/target/debug/cargo-tauri`,
          ["tauri", "ohos", "dev-eco-studio-script", "--target", target.toString()], {
            cwd: resolve(__dirname, "../../../"),
            stdio: "inherit",
          });
      };

      const cmakeTask = node.getTaskByName('default@ConfigureCmake');
      if (cmakeTask) {
        cmakeTask.afterRun(buildRustCode);
      } else {
        node.getTaskByName('default@PreBuild')?.afterRun(buildRustCode);
      }
    }
  };
}
