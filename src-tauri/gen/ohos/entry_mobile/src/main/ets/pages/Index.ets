// Placeholder page (Eulogizethesun/tauri#227): hvigor's main_pages.json
// schema enforces at least one registered page per entry module
// ("src" minItems:1), but this app never routes here — the ability loads
// the Tauri NativeAbility webview directly. Kept content-free so it adds
// nothing but the required registration to the HAP.
@Entry
@Component
struct Index {
  build() {
    Column() {
    }
    .width('100%')
    .height('100%')
  }
}
