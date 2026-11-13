use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "persona_device_id";

#[cfg(target_arch = "wasm32")]
pub fn device_id() -> String {
    let storage = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten();
    if let Some(storage) = storage.as_ref() {
        if let Ok(Some(value)) = storage.get_item(STORAGE_KEY) {
            let value = value.trim().to_owned();
            if !value.is_empty() {
                return value;
            }
        }
    }
    let id = Uuid::new_v4().to_string();
    if let Some(storage) = storage {
        let _ = storage.set_item(STORAGE_KEY, &id);
    }
    id
}

#[cfg(not(target_arch = "wasm32"))]
fn device_path() -> Option<std::path::PathBuf> {
    let base = if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .ok()
            .map(|home| std::path::PathBuf::from(home).join("Library/Application Support"))
    } else if cfg!(target_os = "windows") {
        std::env::var("APPDATA").ok().map(std::path::PathBuf::from)
    } else {
        return None;
    };
    Some(base?.join("PersonaSanus").join("device_id"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn device_id() -> String {
    if let Some(path) = device_path() {
        if let Ok(value) = std::fs::read_to_string(&path) {
            let value = value.trim().to_owned();
            if !value.is_empty() {
                return value;
            }
        }
        let id = Uuid::new_v4().to_string();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &id);
        return id;
    }
    Uuid::new_v4().to_string()
}

#[cfg(target_arch = "wasm32")]
pub fn platform() -> &'static str {
    "web"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
pub fn platform() -> &'static str {
    "macos"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub fn platform() -> &'static str {
    "windows"
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "macos"),
    not(target_os = "windows")
))]
pub fn platform() -> &'static str {
    "web"
}

pub fn name() -> &'static str {
    match platform() {
        "macos" => "macOS",
        "windows" => "Windows",
        _ => "Web",
    }
}
