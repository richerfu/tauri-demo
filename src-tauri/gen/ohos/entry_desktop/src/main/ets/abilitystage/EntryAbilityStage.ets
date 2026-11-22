import AbilityStage from '@ohos.app.ability.AbilityStage';
import Want from '@ohos.app.ability.Want';

// D4 (openspec multi-uiability-windows, 2026-09-15 specified amendment):
// instance routing under launchType "specified". AMS consults onAcceptWant
// for every startAbility targeting this module's UIAbility and keys the
// mission by the returned string (official semantics, API 9+):
//  - want carries tauri_window_id → per-window unique key → AMS creates a
//    NEW UIAbility instance (multi-window spawn, D1/D2). Re-starting the
//    same id routes to the existing instance (onNewWant) instead of
//    spawning a duplicate that would fail stage registration.
//  - bare want (desktop icon / taskbar / tray panel / deep-link relaunch)
//    → fixed primary key → AMS reuses the live primary instance and pulls
//    it back to the top of the stack. This restores the singleton-era
//    foreground behavior that launchType "standard" lost: window-level
//    restore()/showWindow() cannot undo a context.hideAbility()-
//    backgrounded ability, and under "standard" the system's own
//    startAbility spawned a duplicate instead of foregrounding the primary.
export default class EntryAbilityStage extends AbilityStage {
  onAcceptWant(want: Want): string {
    const raw = want.parameters?.["tauri_window_id"];
    if (raw !== undefined && raw !== null && String(raw).length > 0) {
      return `tauri-window-${raw}`;
    }
    return "tauri-primary";
  }
}
