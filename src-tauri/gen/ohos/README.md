# OpenHarmony permissions

The `entry_mobile` / `entry_desktop` `module.json5` files declare the
permissions the scaffold needs out of the box:

- both forms: `ohos.permission.INTERNET`,
  `ohos.permission.SET_WINDOW_TRANSPARENT`,
  `ohos.permission.PREPARE_APP_TERMINATE`,
  `ohos.permission.PRIVACY_WINDOW`,
  `ohos.permission.PUBLISH_AGENT_REMINDER`
- desktop form only: `ohos.permission.WINDOW_TOPMOST`,
  `ohos.permission.LOCK_WINDOW_CURSOR`, `ohos.permission.PRINT`

## READ_PASTEBOARD is deliberately not declared

`ohos.permission.READ_PASTEBOARD` (clipboard reading) is a restricted
permission: a fresh scaffold's default signing Profile carries no ACL for
it, and declaring it breaks install. If your app reads the clipboard:

1. Add it to the `requestPermissions` of **every** entry form that needs
   it, with a reason and used scene:

   ```json5
   {
     "name": "ohos.permission.READ_PASTEBOARD",
     "reason": "$string:reason_read_pasteboard",
     "usedScene": { "abilities": ["EntryAbility"], "when": "inuse" }
   }
   ```

2. Add the `reason_read_pasteboard` string to that form's
   `src/main/resources/base/element/string.json` (the scaffold does not
   ship it).

3. Include the permission in your signing Profile's `allowed-acls` and
   re-sign.

On non-PC/2in1 devices only special scenarios (bank-card or password
copy, document editing, system IME, ...) pass the AGC
restricted-permission review; PC/2in1 apps can request it freely. If
your clipboard use fits, prefer the `PasteButton` secure component —
it needs no permission at all.

## PUBLISH_AGENT_REMINDER

A normal `system_grant` permission, declared unconditionally. Note that
the agent-powered-reminder **capability** itself requires the AGC
open-capability application (covering phones, tablets and PC/2in1);
after approval you must regenerate the Profile and re-sign — declaring
the permission does not bypass that flow.

## Scope: the authoritative OpenHarmony scaffold

`tauri ohos init` generates OpenHarmony projects exclusively from this
template. `cargo-mobile2` — the library `tauri-cli` builds on for OpenHarmony
tooling — also ships its own DevEco Studio platform template
(`templates/platforms/dev-eco-studio` in the cargo-mobile2 repository), used
only by its standalone `cargo mobile init` path; `tauri-cli` does not scaffold
from it. That template is a minimal scaffold that intentionally defers to this
one for the appearance baseline (`EntryAbility.ets.hbs`) and the permission
baseline above, and is expected to lag behind (e.g. its `EntryAbility` still
extends `RustAbility`, which the `@ohos-rs/ability` bridge package renamed to
`NativeAbility` in March 2026).

## The placeholder `pages/Index`

`main_pages.json` must register at least one page — hvigor's schema for the
file enforces `src` `minItems: 1`, and the file itself is required for entry
modules (both verified against the real build pipeline). The app never routes
to `pages/Index` (the ability loads the Tauri NativeAbility webview
directly), so the page is kept as a content-free placeholder: registered to
satisfy the constraint, empty of any demo scaffolding. Feel free to replace
it with your own pages.
