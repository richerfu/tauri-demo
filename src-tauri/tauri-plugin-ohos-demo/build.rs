const COMMANDS: &[&str] = &["platform_info", "echo"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
