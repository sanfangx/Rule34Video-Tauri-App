pub mod adblock;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod child_windows;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod cloudfare;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod context_menu;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod downloads;
pub mod navigation;
pub mod universal_deep_link;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod webnotifications;
pub mod webview_intercept;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod global_shortcuts;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod tray;
