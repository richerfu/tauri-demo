import { LazyPlugin, NativeAbility } from '@ohos-rs/ability';
import { AppControlPlugin } from '@ohos-rs/ability-plugin-app-control';
import { FilesPlugin } from '@ohos-rs/ability-plugin-files';
import { MenuPlugin } from '@ohos-rs/ability-plugin-menu';
import { PermissionPlugin } from '@ohos-rs/ability-plugin-permission';
import { ResourcePlugin } from '@ohos-rs/ability-plugin-resource';
import { StatusbarPlugin } from '@ohos-rs/ability-plugin-statusbar';
import { UrlPlugin } from '@ohos-rs/ability-plugin-url';
import { WebviewPlugin } from '@ohos-rs/ability-plugin-webview';
import { WindowPlugin } from '@ohos-rs/ability-plugin-window';
import { ClipboardPlugin } from '@ohos-rs/ability-plugin-clipboard';
import { SystemStatePlugin } from '@ohos-rs/ability-plugin-system-state';
import Want from '@ohos.app.ability.Want';
import { AbilityConstant } from '@kit.AbilityKit';
import window from '@ohos.window';
import { PluginManager, Plugin } from '@tauri/app';
import tauri_demo_lib from 'libtauri_demo_lib.so';
import { deviceInfo } from '@kit.BasicServicesKit';
import { hilog } from '@kit.PerformanceAnalysisKit';

const DEMO_DOMAIN = 0x0000;

interface TauriNativeModule {
  tauri_set_hap_module_name?: (name: string) => void;
  tauriSetHapModuleName?: (name: string) => void;
  tauri_init_plugins?: (manager: PluginManager) => string;
  tauriInitPlugins?: (manager: PluginManager) => string;
  tauri_handle_plugin_response?: (id: number, success: boolean, payload: string) => void;
  tauriHandlePluginResponse?: (id: number, success: boolean, payload: string) => void;
  tauri_send_channel_data?: (channelId: number, data: string) => void;
  tauriSendChannelData?: (channelId: number, data: string) => void;
  registerMainThreadArktsCallback?: (cb: (message: string) => string) => void;
  callMainThreadArktsCallback?: (message: string) => string;
  registerAsyncThreadArktsCallback?: (cb: (message: string) => void) => void;
  registerMainThreadDispatcher?: (cb: (message: string) => string) => void;
}

interface PluginConfig {
  name: string;
  identifier: string;
  className: string;
  config: string;
}

const nativeModule: TauriNativeModule = tauri_demo_lib;

const STATIC_PLUGINS: Map<string, Plugin> = new Map();

export default class EntryAbility extends NativeAbility {
  public moduleName: string = 'tauri_demo_lib';
  public defaultPage: boolean = true;
  // Stored for lifecycle notifications and handleNotificationAction dispatch.
  private pluginManager: PluginManager | null = null;
  // Process-level guard (multi-UIAbility, single-runtime invariant): the
  // PluginManager and the STATIC_PLUGINS JS singletons belong to the process —
  // a spawned UIAbility instance must not re-load them into a second manager,
  // which would rewire their transport to the spawned instance's context.
  private static pluginsInitialized: boolean = false;
  // ArkTS-side modular BridgePlugin factories matching Rust-registered bridge
  // plugins. Unused factories remain lazy and are never instantiated.
  public bridgePlugins = [
    new LazyPlugin(() => new AppControlPlugin()),
    new LazyPlugin(() => new FilesPlugin()),
    new LazyPlugin(() => new MenuPlugin()),
    new LazyPlugin(() => new PermissionPlugin()),
    new LazyPlugin(() => new ResourcePlugin()),
    new LazyPlugin(() => new StatusbarPlugin()),
    new LazyPlugin(() => new UrlPlugin()),
    new LazyPlugin(() => new WebviewPlugin()),
    new LazyPlugin(() => new WindowPlugin()),
    new LazyPlugin(() => new ClipboardPlugin()),
    new LazyPlugin(() => new SystemStatePlugin()),
  ];

  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    console.info('[Tauri] ===== EntryAbility onCreate =====');
    const setHapModuleName =
      nativeModule?.tauriSetHapModuleName ?? nativeModule?.tauri_set_hap_module_name;
    if (setHapModuleName != null) {
      const hapModuleName = this.context.currentHapModuleInfo?.name ?? '';
      if (hapModuleName.length > 0) {
        setHapModuleName(hapModuleName);
      }
    }
    super.onCreate(want, launchParam);
    this.registerArkTsCallbacks();
    void this.enqueueLifecycleOperation('tauri-init-plugins', async (): Promise<void> => {
      if (!EntryAbility.pluginsInitialized) {
        await this.initTauriPlugins();
        EntryAbility.pluginsInitialized = true;
      }
      this.handleNotificationAction(want);
    });
  }

  private registerArkTsCallbacks(): void {
    nativeModule.registerMainThreadArktsCallback?.((message: string): string => {
      const result = `ArkTS Function received: ${message}`;
      hilog.info(DEMO_DOMAIN, 'TauriDemo', '%{public}s', result);
      return result;
    });

    nativeModule.registerAsyncThreadArktsCallback?.((message: string): void => {
      hilog.info(DEMO_DOMAIN, 'TauriDemo', 'ArkTS TSFN received: %{public}s', message);
    });

    nativeModule.registerMainThreadDispatcher?.((message: string): string => {
      const result = nativeModule.callMainThreadArktsCallback?.(message) ?? '';
      hilog.info(DEMO_DOMAIN, 'TauriDemo', 'ArkTS Function dispatcher returned: %{public}s', result);
      return result;
    });
  }

  onNewWant(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    super.onNewWant(want, launchParam);
    this.handleNotificationAction(want);
  }

  onForeground(): void {
    console.info('[Tauri] ===== EntryAbility onForeground =====');
    super.onForeground();
    this.pluginManager?.notifyForeground();
  }

  onBackground(): void {
    console.info('[Tauri] ===== EntryAbility onBackground =====');
    super.onBackground();
    this.pluginManager?.notifyBackground();
  }

  onDestroy(): Promise<void> {
    console.info('[Tauri] ===== EntryAbility onDestroy =====');
    this.pluginManager?.notifyDestroy();
    return super.onDestroy();
  }

  /**
   * Dispatches notification action button clicks (from WantAgent) to the
   * NotificationPlugin instance. On cold start the webview may not be ready,
   * so emit failures are caught and logged.
   */
  private handleNotificationAction(want: Want): void {
    try {
      const params = want.parameters ?? {};
      const actionId = params['_tauriActionId'];
      const notificationIdStr = params['_tauriNotificationId'];
      const actionTypeId = params['_tauriActionTypeId'];
      if (actionId == null || notificationIdStr == null) {
        return;
      }
      const notificationId = parseInt(String(notificationIdStr), 10);
      if (isNaN(notificationId)) {
        return;
      }
      console.info('[Tauri] Notification action: id=' + notificationId + ', actionId=' + String(actionId));
      const plugin = this.pluginManager?.getPlugin('notification');
      if (plugin != null) {
        plugin.onNotificationAction(notificationId, String(actionId), String(actionTypeId ?? ''));
      }
    } catch (e) {
      console.warn('[Tauri] handleNotificationAction failed: ' + (e as Error).message);
    }
  }

  private async initTauriPlugins(): Promise<void> {
    console.info('[Tauri] ===== initTauriPlugins =====');

    const pluginManager: PluginManager = new PluginManager();
    console.info('[Tauri] PluginManager created');

    pluginManager.handlePluginResponse = (id: number, success: number, payload: string): void => {
      const handler = nativeModule?.tauri_handle_plugin_response ?? nativeModule?.tauriHandlePluginResponse;
      if (handler != null) {
        handler(id, success === 1, payload);
      }
    };

    pluginManager.setResponseHandler(pluginManager.handlePluginResponse);
    pluginManager.setContext(this.context);

    // Bridge ArkTS Plugin.emit(channelId, payload) to Rust NAPI tauri_send_channel_data.
    Plugin.setEmitHandler((channelId: number, data: string): void => {
      const emitFn = nativeModule?.tauri_send_channel_data ?? nativeModule?.tauriSendChannelData;
      if (emitFn != null) {
        emitFn(channelId, data);
      } else {
        console.error('[Tauri] tauri_send_channel_data not found in native module');
      }
    });

    this.pluginManager = pluginManager;

    console.info('[Tauri] Native module loaded (static import)');
    console.info('[Tauri] Module keys: ' + Object.keys(nativeModule).join(', '));

    try {
      if (nativeModule?.tauri_init_plugins != null || nativeModule?.tauriInitPlugins != null) {
        const initPlugins = nativeModule.tauri_init_plugins ?? nativeModule.tauriInitPlugins;
        if (initPlugins != null) {
          console.info('[Tauri] Calling initPlugins');
          const pluginsJson = initPlugins(pluginManager);
          const plugins: PluginConfig[] = JSON.parse(pluginsJson) as PluginConfig[];

          console.info('[Tauri] Loading ' + plugins.length + ' plugins');

          for (const plugin of plugins) {
            const staticPlugin: Plugin | undefined = STATIC_PLUGINS.get(plugin.name);
            if (staticPlugin != null) {
              pluginManager.load(plugin.name, staticPlugin, plugin.config);
              console.info('[Tauri] Plugin loaded: ' + plugin.name);
            } else {
              console.error('[Tauri] Plugin not in static registry: ' + plugin.name);
            }
          }
          console.info('[Tauri] All plugins loaded');
        }
      } else {
        console.warn('[Tauri] initPlugins not found in module');
      }
    } catch (e) {
      const err = e as Error;
      console.error('[Tauri] Error: ' + err.message);
      console.error('[Tauri] Stack: ' + err.stack);
    }
  }

  protected override async loadWindowStageContent(windowStage: window.WindowStage): Promise<void> {
    console.info('[Tauri] ===== loadWindowStageContent =====');
    const win = windowStage.getMainWindowSync();
    try {
      await win.setWindowLayoutFullScreen(true);
    } catch (e) {
      console.error('[Tauri] Failed to set full-screen layout: ' + JSON.stringify(e));
    }

    await super.loadWindowStageContent(windowStage);

    // Set window background AFTER super.loadWindowStageContent to avoid being
    // overwritten by loadContentByName, and ONLY when tauri.conf's first
    // `app.windows` background_color is explicitly configured.
    // NOTE: OHOS requires #AARRGGBB format (8-digit hex), not #RRGGBB.
    try {
      await win.setWindowSystemBarProperties({
        statusBarColor: '#00000000',
        statusBarContentColor: '#FF000000',
        navigationBarColor: '#00000000',
        navigationBarContentColor: '#FF000000'
      });
    } catch (e) {
      console.error('[Tauri] Failed to set system bar properties: ' + JSON.stringify(e));
    }
  }
}
