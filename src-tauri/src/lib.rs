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

// ─── Tray Helper ──────────────────────────────────────────
fn rebuild_tray_menu(app_handle: &tauri::AppHandle) -> Result<(), String> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    let profiles_data = load_profiles();
    
    let menu = Menu::new(app_handle).map_err(|e| e.to_string())?;

    let toggle = MenuItem::with_id(app_handle, "toggle", "Quản lý Profile", true, None::<&str>).map_err(|e| e.to_string())?;
    menu.append(&toggle).map_err(|e| e.to_string())?;
    
    let sep1 = PredefinedMenuItem::separator(app_handle).map_err(|e| e.to_string())?;
    menu.append(&sep1).map_err(|e| e.to_string())?;

    for p in &profiles_data.profiles {
        let item = MenuItem::with_id(
            app_handle,
            format!("open_profile:{}", p.id),
            &p.name,
            true,
            None::<&str>,
        ).map_err(|e| e.to_string())?;
        menu.append(&item).map_err(|e| e.to_string())?;
    }

    let sep2 = PredefinedMenuItem::separator(app_handle).map_err(|e| e.to_string())?;
    menu.append(&sep2).map_err(|e| e.to_string())?;

    let quit = MenuItem::with_id(app_handle, "quit", "Thoát hoàn toàn", true, None::<&str>).map_err(|e| e.to_string())?;
    menu.append(&quit).map_err(|e| e.to_string())?;

    if let Some(tray) = app_handle.tray_by_id("main") {
        let _ = tray.set_menu(Some(menu));
    }

    Ok(())
}

// ─── Tauri Commands ─────────────────────────────────────────

#[tauri::command]
fn list_profiles() -> Result<Vec<Profile>, String> {
    let data = load_profiles();
    Ok(data.profiles)
}

#[tauri::command]
fn create_profile(app_handle: tauri::AppHandle, name: String, color: String) -> Result<Profile, String> {
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

    let _ = rebuild_tray_menu(&app_handle);

    Ok(profile)
}

#[tauri::command]
fn delete_profile(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut data = load_profiles();
    data.profiles.retain(|p| p.id != id);
    save_profiles(&data)?;

    // Delete profile data directory
    let data_dir = get_profile_data_dir(&id);
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }

    let _ = rebuild_tray_menu(&app_handle);

    Ok(())
}

#[tauri::command]
fn rename_profile(app_handle: tauri::AppHandle, id: String, new_name: String) -> Result<(), String> {
    let mut data = load_profiles();
    if let Some(profile) = data.profiles.iter_mut().find(|p| p.id == id) {
        profile.name = new_name;
    } else {
        return Err("Profile not found".to_string());
    }
    save_profiles(&data)?;

    let _ = rebuild_tray_menu(&app_handle);

    Ok(())
}

#[tauri::command]
fn show_system_notification(title: String, body: String) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let script = format!(
        r#"[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); $toastXml = [xml]$template.GetXml(); $toastXml.GetElementsByTagName("text")[0].AppendChild($toastXml.CreateTextNode("{}")) > $null; $toastXml.GetElementsByTagName("text")[1].AppendChild($toastXml.CreateTextNode("{}")) > $null; $xml = New-Object Windows.Data.Xml.Dom.XmlDocument; $xml.LoadXml($toastXml.OuterXml); $toast = New-Object Windows.UI.Notifications.ToastNotification $xml; [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("Zalo Portable").Show($toast)"#,
        title.replace("\"", "`\"").replace("'", "`'"),
        body.replace("\"", "`\"").replace("'", "`'")
    );

    let _ = std::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .spawn();
}

#[tauri::command]
fn update_window_attention(window: tauri::Window, has_unread: bool) {
    if has_unread {
        let _ = window.request_user_attention(Some(tauri::UserAttentionType::Informational));
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
    .initialization_script(r#"
        (function() {
            // Hook Notification API
            window.Notification = class CustomNotification {
                static permission = 'granted';
                static requestPermission(callback) {
                    if (callback) callback('granted');
                    return Promise.resolve('granted');
                }
                constructor(title, options) {
                    this.title = title;
                    this.options = options;
                    
                    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                        window.__TAURI__.core.invoke('show_system_notification', {
                            title: title,
                            body: options && options.body ? options.body : ''
                        }).catch(console.error);
                    }
                }
                close() {}
                addEventListener() {}
                removeEventListener() {}
                dispatchEvent() {}
            };

            // Watch document title for unread message indicator
            let lastUnread = false;
            function checkTitle() {
                const title = document.title;
                const hasUnread = /^\(\d+\)/.test(title) || /^\(\*\)/.test(title);
                if (hasUnread !== lastUnread) {
                    lastUnread = hasUnread;
                    if (hasUnread && window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                        window.__TAURI__.core.invoke('update_window_attention', {
                            hasUnread: true
                        }).catch(console.error);
                    }
                }
            }

            // Monitor title changes via MutationObserver
            const observer = new MutationObserver(() => {
                checkTitle();
            });
            observer.observe(document.head, {
                childList: true,
                subtree: true,
                characterData: true
            });
            setInterval(checkTitle, 2000);
        })();
    "#)
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
fn update_profile_color(app_handle: tauri::AppHandle, id: String, color: String) -> Result<(), String> {
    let mut data = load_profiles();
    if let Some(profile) = data.profiles.iter_mut().find(|p| p.id == id) {
        profile.color = color;
    } else {
        return Err("Profile not found".to_string());
    }
    save_profiles(&data)?;

    let _ = rebuild_tray_menu(&app_handle);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::tray::{TrayIconBuilder, TrayIconEvent};

            let mut tray_builder = TrayIconBuilder::with_id("main")
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    match id {
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ if id.starts_with("open_profile:") => {
                            let profile_id = id.trim_start_matches("open_profile:");
                            let data = load_profiles();
                            if let Some(p) = data.profiles.iter().find(|p| p.id == profile_id) {
                                let app_clone = app.clone();
                                let p_id = p.id.clone();
                                let p_name = p.name.clone();
                                tauri::async_runtime::spawn(async move {
                                    let _ = open_profile(app_clone, p_id, p_name).await;
                                });
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        for window in app.webview_windows().values() {
                            if window.label().starts_with("zalo_") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;

            // Rebuild menu to populate profiles
            let _ = rebuild_tray_menu(app.app_handle());

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label().starts_with("zalo_") {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            create_profile,
            delete_profile,
            rename_profile,
            open_profile,
            update_profile_color,
            show_system_notification,
            update_window_attention,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
