const COMMANDS: &[&str] = &["enroll", "open", "forget", "confirm"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS).android_path("android").build();
}
