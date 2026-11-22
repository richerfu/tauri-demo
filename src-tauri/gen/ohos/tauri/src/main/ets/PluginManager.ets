import { Plugin, Invoke, CommandHandler } from './Plugin';
import { common } from '@kit.AbilityKit';

interface PluginInstance {
  name: string;
  instance: Plugin;
  // Serialized JSON config from tauri.conf.json's `plugins` section for this
  // plugin — the ArkTS counterpart of Android's PluginHandle.config.
  config: string;
}

interface ErrorResponse {
  message: string;
  // Machine-readable error code (e.g. biometric's 'userCancel'). Optional —
  // matches the Rust-side ErrorResponse's optional `code` field.
  code?: string;
}

type ResponseHandler = (id: number, success: number, payload: string) => void;

let globalPlugins: Map<string, PluginInstance> = new Map();
let globalHandlePluginResponse: ResponseHandler | null = null;
let globalContext: common.UIAbilityContext | null = null;

class InvokeImpl implements Invoke {
  private args: string;
  private id: number;

  constructor(args: string, id: number) {
    this.args = args;
    this.id = id;
  }

  parseArgs(): string {
    return this.args;
  }

  resolve(result: string): void {
    handleResponseGlobal(this.id, true, result);
  }

  reject(error: string, code?: string): void {
    const errorResponse: ErrorResponse = { message: error };
    if (code != null) {
      errorResponse.code = code;
    }
    handleResponseGlobal(this.id, false, JSON.stringify(errorResponse));
  }
}

function handleResponseGlobal(id: number, success: boolean, payload: string): void {
  console.info('[PluginManager] Response: id=' + id + ', success=' + success);

  if (globalHandlePluginResponse !== null) {
    globalHandlePluginResponse(id, success ? 1 : 0, payload);
  } else {
    console.error('[PluginManager] handlePluginResponse callback not set');
  }
}

export class PluginManager {
  public handlePluginResponse: ResponseHandler | null = null;

  setContext(context: common.UIAbilityContext): void {
    globalContext = context;
  }

  load(pluginName: string, plugin: Plugin, config: string): void {
    plugin.context = globalContext;
    // Hand the config to the plugin too — Plugin.getConfig<T>() reads it
    // without needing a back-reference to the manager (avoiding an ArkTS
    // import cycle).
    plugin.config = config;
    globalPlugins.set(pluginName, {
      name: pluginName,
      instance: plugin,
      config: config,
    });
    console.info('[PluginManager] Loaded: ' + pluginName);
  }

  setResponseHandler(handler: ResponseHandler): void {
    this.handlePluginResponse = handler;
    globalHandlePluginResponse = handler;
  }

  /**
   * Returns the plugin instance registered under `name`, or undefined if not found.
   * Used by EntryAbility to dispatch notification action clicks to the
   * NotificationPlugin instance without needing a direct reference.
   */
  getPlugin(name: string): Plugin | undefined {
    const entry = globalPlugins.get(name);
    return entry != null ? entry.instance : undefined;
  }

  /**
   * Called by EntryAbility.onForeground(). Iterates all loaded plugins and
   * invokes onForeground() on each. Plugins use this to settle pending
   * operations (e.g. permission request promises) that may have been
   * interrupted when the ability went to background (system permission dialog).
   */
  notifyForeground(): void {
    console.info('[PluginManager] notifyForeground — ' + globalPlugins.size + ' plugins');
    globalPlugins.forEach((entry: PluginInstance): void => {
      try {
        entry.instance.onForeground();
      } catch (e) {
        console.warn('[PluginManager] onForeground failed for ' + entry.name + ': ' + (e as Error).message);
      }
    });
  }

  /**
   * Called by EntryAbility.onBackground(). Iterates all loaded plugins and
   * invokes onBackground() on each so plugins can pause foreground-only work.
   */
  notifyBackground(): void {
    console.info('[PluginManager] notifyBackground — ' + globalPlugins.size + ' plugins');
    globalPlugins.forEach((entry: PluginInstance): void => {
      try {
        entry.instance.onBackground();
      } catch (e) {
        console.warn('[PluginManager] onBackground failed for ' + entry.name + ': ' + (e as Error).message);
      }
    });
  }

  /**
   * Called by EntryAbility.onDestroy(). Iterates all loaded plugins and
   * invokes onDestroy() on each so plugins can clean up watchers and handles.
   */
  notifyDestroy(): void {
    console.info('[PluginManager] notifyDestroy — ' + globalPlugins.size + ' plugins');
    globalPlugins.forEach((entry: PluginInstance): void => {
      try {
        entry.instance.onDestroy();
      } catch (e) {
        console.warn('[PluginManager] onDestroy failed for ' + entry.name + ': ' + (e as Error).message);
      }
    });
  }

  runCommand(id: number, pluginName: string, command: string, payload: string): void {
    console.info('[PluginManager] runCommand: id=' + id + ', plugin=' + pluginName + ', cmd=' + command);

    const pluginInstance = globalPlugins.get(pluginName);
    if (!pluginInstance) {
      console.error('[PluginManager] Plugin not found: ' + pluginName);
      const errorResponse: ErrorResponse = { message: 'Plugin not found: ' + pluginName };
      handleResponseGlobal(id, false, JSON.stringify(errorResponse));
      return;
    }

    const commands = pluginInstance.instance.getCommands();
    const handler = commands.get(command);

    if (handler) {
      const invoke = new InvokeImpl(payload, id);
      // The handler is user/plugin code — if it throws, the exception would
      // propagate back into the Rust TSFN callback and this call would never
      // resolve or reject, leaving the frontend promise pending forever.
      // Mirrors the Android PluginManager: catch and surface via reject.
      try {
        handler(invoke);
      } catch (e) {
        console.error('[PluginManager] Command handler threw: ' + e);
        invoke.reject(String(e));
      }
    } else {
      console.error('[PluginManager] Command not found: ' + command);
      const errorResponse: ErrorResponse = { message: 'Command not found: ' + command };
      handleResponseGlobal(id, false, JSON.stringify(errorResponse));
    }
  }

  handleResponse(id: number, success: boolean, payload: string): void {
    handleResponseGlobal(id, success, payload);
  }
}

export default PluginManager;