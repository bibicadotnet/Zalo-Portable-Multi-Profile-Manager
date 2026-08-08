use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;


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

fn prepare_preferences_file(profile_data_dir: &std::path::Path) {
    let default_dir = profile_data_dir.join("EBWebView").join("Default");
    let preferences_path = default_dir.join("Preferences");
    
    if let Err(_) = fs::create_dir_all(&default_dir) {
        return;
    }
    
    let mut preferences_json: serde_json::Value = if preferences_path.exists() {
        let content = fs::read_to_string(&preferences_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    
    // Set notification permission to 1 (Allow) for chat.zalo.me
    if let Some(profile) = preferences_json.as_object_mut() {
        let profile_obj = profile.entry("profile").or_insert_with(|| serde_json::json!({}));
        if let Some(prof) = profile_obj.as_object_mut() {
            let content_settings = prof.entry("content_settings").or_insert_with(|| serde_json::json!({}));
            if let Some(cs) = content_settings.as_object_mut() {
                let exceptions = cs.entry("exceptions").or_insert_with(|| serde_json::json!({}));
                if let Some(ex) = exceptions.as_object_mut() {
                    let notifications = ex.entry("notifications").or_insert_with(|| serde_json::json!({}));
                    if let Some(notif) = notifications.as_object_mut() {
                        notif.insert("https://chat.zalo.me:443,*".to_string(), serde_json::json!({
                            "setting": 1
                        }));
                    }
                }
            }
        }
    }
    
    if let Ok(content) = serde_json::to_string(&preferences_json) {
        let _ = fs::write(&preferences_path, content);
    }
}

#[tauri::command]
async fn open_profile(app_handle: tauri::AppHandle, id: String, name: String) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;

    let window_label = format!("zalo_{}", id);

    // If window already exists, just show and focus it
    if let Some(win) = app_handle.get_webview_window(&window_label) {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let data_dir = get_profile_data_dir(&id);
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    
    // Auto-grant notifications in preferences file before starting the webview
    prepare_preferences_file(&data_dir);
    
    // Make sure path is absolute and clean (helps WebView2)
    // Canonicalize on Windows adds \\?\ prefix which breaks WebView2 IndexedDB/LocalStorage!
    // Since get_base_dir() uses current_exe(), it is already absolute.
    let clean_data_dir_str = data_dir.to_string_lossy().replace("\\\\?\\", "");
    let clean_data_dir = std::path::PathBuf::from(clean_data_dir_str);
    println!("Opening profile '{}' with data dir: {:?}", name, clean_data_dir);

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

    // Create a dedicated system tray icon for this specific profile
    let tray_id = format!("tray_{}", id);
    if app_handle.tray_by_id(&tray_id).is_none() {
        use tauri::tray::{TrayIconBuilder, TrayIconEvent};
        use tauri::menu::{Menu, MenuItem};

        let quit = MenuItem::with_id(&app_handle, format!("quit_{}", id), "Thoát", true, None::<&str>).map_err(|e| e.to_string())?;
        let menu = Menu::with_items(&app_handle, &[&quit]).map_err(|e| e.to_string())?;

        let window_label_clone = window_label.clone();
        let tray_id_clone = tray_id.clone();
        let id_clone = id.clone();

        let mut tray_builder = TrayIconBuilder::with_id(&tray_id)
            .menu(&menu)
            .on_menu_event(move |app, event| {
                let quit_id = format!("quit_{}", id_clone);
                if event.id.as_ref() == &quit_id {
                    if let Some(win) = app.get_webview_window(&window_label_clone) {
                        let _ = win.destroy();
                    }
                    let _ = app.remove_tray_by_id(&tray_id_clone);
                }
            })
            .on_tray_icon_event(move |tray, event| {
                if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                    let app = tray.app_handle();
                    if let Some(win) = app.get_webview_window(&window_label) {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            });

        if let Some(icon) = app_handle.default_window_icon() {
            tray_builder = tray_builder.icon(icon.clone());
        }

        let _tray = tray_builder.build(&app_handle).map_err(|e| e.to_string())?;
    }

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
        .setup(|_app| {
            // Main manager window does not have a system tray icon on startup
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if window.label().starts_with("zalo_") {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    let label = window.label();
                    if label.starts_with("zalo_") {
                        let id = label.trim_start_matches("zalo_");
                        let tray_id = format!("tray_{}", id);
                        let _ = window.app_handle().remove_tray_by_id(&tray_id);
                    }
                }
                _ => {}
            }
        })
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
