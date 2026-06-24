import { invoke } from "@tauri-apps/api/core";

export interface OhosDemoPlatformInfo {
  platform: string;
  runtime: string;
  usesNativeAbility: boolean;
}

export async function getPlatformInfo(): Promise<OhosDemoPlatformInfo> {
  return invoke("plugin:ohos-demo|platform_info");
}

export async function echo(message: string): Promise<string> {
  return invoke("plugin:ohos-demo|echo", { message });
}
