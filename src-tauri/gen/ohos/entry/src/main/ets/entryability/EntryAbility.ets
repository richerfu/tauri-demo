import { NativeAbility } from '@ohos-rs/ability'
import Want from '@ohos.app.ability.Want'
import { AbilityConstant } from '@kit.AbilityKit';
import window from '@ohos.window';
import { hilog } from '@kit.PerformanceAnalysisKit';
import rustNapi from 'libtauri_demo_lib.so';

const DOMAIN = 0x0000;

export default class EntryAbility extends NativeAbility {
  // change to dynamic library name
  public moduleName: string = "tauri_demo_lib"
  public defaultPage: boolean = true;

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    await super.onCreate(want, launchParam);
    this.registerArkTsCallbacks();
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    const window = windowStage.getMainWindowSync();
    await window.setWindowLayoutFullScreen(false);
    await super.onWindowStageCreate(windowStage);
  }

  private registerArkTsCallbacks(): void {
    rustNapi.registerMainThreadArktsCallback((message: string): string => {
      const result = `ArkTS Function received: ${message}`;
      hilog.info(DOMAIN, 'testTag', '%{public}s', result);
      return result;
    });

    rustNapi.registerAsyncThreadArktsCallback((message: string): void => {
      hilog.info(DOMAIN, 'testTag', 'ArkTS TSFN received: %{public}s', message);
    });

    rustNapi.registerMainThreadDispatcher((message: string): string => {
      const result = rustNapi.callMainThreadArktsCallback(message);
      hilog.info(DOMAIN, 'testTag', 'ArkTS Function dispatcher returned: %{public}s', result);
      return result;
    });
  }
}
