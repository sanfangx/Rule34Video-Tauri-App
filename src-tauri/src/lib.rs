mod ext;

use ext::adblock::AdblockManager;
use ext::navigation;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
#[cfg(any(target_os = "android", target_os = "ios"))]
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
#[cfg(any(target_os = "android", target_os = "ios"))]
use url::Url;

pub fn build_init_script() -> String {
    navigation::build_init_script()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
fn open_child_window_cmd(app: AppHandle, url: String) -> Result<(), String> {
    println!("[child_windows] cmd url={}", url);
    crate::ext::child_windows::open_child_window(&app, &url).map_err(|e| e.to_string())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
fn child_post_message_cmd(
    app: tauri::AppHandle,
    data: String,
    origin: Option<String>,
) -> Result<(), String> {
    let origin_val = origin.unwrap_or_default();

    let payload = serde_json::json!({
        "data": data,
        "origin": origin_val,
    });

    println!("[child_windows] child_post_message_cmd origin={}", origin_val);
    println!("[child_windows] child_post_message_cmd data_len={}", data.len());

    app.emit_to("main", "rule34video:child-post-message", payload)
        .map_err(|e| e.to_string())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
fn show_native_context_menu_cmd(
    app: tauri::AppHandle,
    payload: crate::ext::context_menu::ContextMenuPayload,
) -> Result<(), String> {
    crate::ext::context_menu::show_native_context_menu(&app, payload).map_err(|e| e.to_string())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn run() {
    let builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_child_window_cmd,
            child_post_message_cmd,
            show_native_context_menu_cmd,
            crate::ext::adblock::check_url_blocked,
            crate::ext::adblock::get_page_cosmetic_filters,
        ])
        .plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            crate::ext::universal_deep_link::init_universal_deep_link(app.handle().clone())?;

            let adblock_manager = AdblockManager::new_with_bundled();
            app.manage(adblock_manager);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let manager = app_handle.state::<AdblockManager>();
                manager.load_filters(&app_handle).await;
            });

            let site_url: url::Url = "https://rule34video.com/"
                .parse()
                .expect("hardcoded URL is valid");

            let window_builder =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::External(site_url.clone()))
                    .title("Rule34Video")
                    .inner_size(1280.0, 800.0)
                    .min_inner_size(800.0, 600.0)
                    .resizable(true);
            let window_builder = window_builder.center();
            let window_builder = crate::ext::cloudfare::init_cloudfare(window_builder);

            let nav_app_handle = app.handle().clone();
            let _window = window_builder
                .initialization_script(build_init_script())
                .on_navigation(move |url| {
                    let state = nav_app_handle.state::<AdblockManager>();
                    !state.is_blocked(url.as_str(), "", "document")
                })
                .on_download(|_window, event| {
                    crate::ext::downloads::handle_download_event(_window.app_handle(), "main", event)
                })
                .build()?;

            crate::ext::webview_intercept::setup(&app.handle());
            let _ = crate::ext::context_menu::init_context_menu(&app.handle());
            let _ = crate::ext::downloads::init_downloads(&app.handle());
            let _ = crate::ext::global_shortcuts::init_global_shortcuts(&app.handle());
            let _ = crate::ext::webnotifications::init_webnotifications(&app.handle());
            crate::ext::tray::init_tray(&app.handle())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::mobile_entry_point]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            crate::ext::adblock::check_url_blocked,
            crate::ext::adblock::get_page_cosmetic_filters,
        ])
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            crate::ext::universal_deep_link::init_universal_deep_link(app.handle().clone())?;

            let adblock_manager = AdblockManager::new_with_bundled();
            app.manage(adblock_manager);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let manager = app_handle.state::<AdblockManager>();
                manager.load_filters(&app_handle).await;
            });

            let site_url: Url = "https://rule34video.com/"
                .parse()
                .expect("hardcoded URL is valid");

            let nav_app_handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(site_url))
                .title("Rule34Video")
                .initialization_script(crate::ext::adblock::adblock_script())
                .on_navigation(move |url| {
                    let state = nav_app_handle.state::<AdblockManager>();
                    !state.is_blocked(url.as_str(), "", "document")
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

