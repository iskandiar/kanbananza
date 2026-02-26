fn main() {
    // Bake .env vars into the binary at compile time.
    // cargo:rustc-env makes them accessible via env!("KEY") or option_env!("KEY").
    println!("cargo:rerun-if-changed=.env");
    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                println!("cargo:rustc-env={}={}", key.trim(), value.trim());
            }
        }
    }

    tauri_build::build()
}
