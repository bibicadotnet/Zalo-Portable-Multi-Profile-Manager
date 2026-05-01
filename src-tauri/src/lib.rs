use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static WINDOW_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Profile {
    id: String,
    name: String,
    color: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfilesData {
    profiles: Vec<Profile>,
}

/// Get the portable base directory (next to the exe)
fn get_base_dir() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get executable path");
    let exe_dir = exe_path.parent().expect("Failed to get parent directory");
    exe_dir.to_path_buf()
}

/// Get path to profiles.json metadata file
fn get_profiles_file() -> PathBuf {
    get_base_dir().join("profiles.json")
}

/// Get the data directory for a specific profile
fn get_profile_data_dir(profile_id: &str) -> PathBuf {
    get_base_dir().join("ProfileData").join(profile_id)
}

/// Load profiles from disk
fn load_profiles() -> ProfilesData {
    let path = get_profiles_file();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).unwrap_or(ProfilesData {
            profiles: Vec::new(),
        })
    } else {
        ProfilesData {
            profiles: Vec::new(),
        }
    }
}

/// Save profiles to disk
fn save_profiles(data: &ProfilesData) -> Result<(), String> {
    let path = get_profiles_file();
    let content = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Generate a simple unique ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("profile_{}", timestamp)
}

/// Get current timestamp as ISO string
fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let secs = duration.as_secs();
    // Simple ISO-ish format
    format!("{}", secs)
}

// ─── Tauri Commands ─────────────────────────────────────────

#[tauri::command]
fn list_profiles() -> Result<Vec<Profile>, String> {
    let data = load_profiles();
    Ok(data.profiles)
}

#[tauri::command]
fn create_profile(name: String, color: String) -> Result<Profile, String> {
    let mut data = load_profiles();

    let profile = Profile {
        id: generate_id(),
        name,
        color,
        created_at: now_iso(),
    };

    // Create data directory for this profile
    let data_dir = get_profile_data_dir(&profile.id);
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    data.profiles.push(profile.clone());
    save_profiles(&data)?;

    Ok(profile)
}

#[tauri::command]
fn delete_profile(id: String) -> Result<(), String> {
    let mut data = load_profiles();
    data.profiles.retain(|p| p.id != id);
    save_profiles(&data)?;

    // Delete profile data directory
    let data_dir = get_profile_data_dir(&id);
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn rename_profile(id: String, new_name: String) -> Result<(), String> {
    let mut data = load_profiles();
    if let Some(profile) = data.profiles.iter_mut().find(|p| p.id == id) {
        profile.name = new_name;
    } else {
        return Err("Profile not found".to_string());
    }
    save_profiles(&data)?;
    Ok(())
}

#[tauri::command]
async fn open_profile(app_handle: tauri::AppHandle, id: String, name: String) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;

    let data_dir = get_profile_data_dir(&id);
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    
    // Make sure path is absolute and clean (helps WebView2)
    // Canonicalize on Windows adds \\?\ prefix which breaks WebView2 IndexedDB/LocalStorage!
    // Since get_base_dir() uses current_exe(), it is already absolute.
    let clean_data_dir_str = data_dir.to_string_lossy().replace("\\\\?\\", "");
    let clean_data_dir = std::path::PathBuf::from(clean_data_dir_str);
    println!("Opening profile '{}' with data dir: {:?}", name, clean_data_dir);

    // Generate a unique window label
    let counter = WINDOW_COUNTER.fetch_add(1, Ordering::SeqCst);
    let window_label = format!("zalo_{}", counter);

    let _win = WebviewWindowBuilder::new(
        &app_handle,
        &window_label,
        tauri::WebviewUrl::External("https://chat.zalo.me".parse().unwrap()),
    )
    .title(&format!("Zalo — {}", name))
    .inner_size(1000.0, 800.0)
    .min_inner_size(400.0, 300.0)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
    .data_directory(clean_data_dir)
    .on_navigation(|url| {
        // Allow Zalo and all standard web protocols
        let scheme = url.scheme();
        let _host = url.host_str().unwrap_or("");
        
        // Zalo often redirects to an auth domain or uses subdomains like zalo.me, zadn.vn
        scheme == "https" || scheme == "http" || scheme == "tauri" || scheme == "blob" || scheme == "data" || scheme == "ws" || scheme == "wss"
    })
    .build()
    .map_err(|e| {
        println!("Error building window: {}", e);
        e.to_string()
    })?;

    Ok(())
}

#[tauri::command]
fn update_profile_color(id: String, color: String) -> Result<(), String> {
    let mut data = load_profiles();
    if let Some(profile) = data.profiles.iter_mut().find(|p| p.id == id) {
        profile.color = color;
    } else {
        return Err("Profile not found".to_string());
    }
    save_profiles(&data)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            create_profile,
            delete_profile,
            rename_profile,
            open_profile,
            update_profile_color,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
