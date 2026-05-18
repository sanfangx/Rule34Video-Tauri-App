# 📐 Architecture

> How the Rule34Video Tauri App is structured under the hood.

---

## 🧱 Layered architecture

The app follows a modular, layered architecture with clear separation between the Tauri shell, the Rust backend, and the webview frontend.

```
┌─────────────────────────────────────────────┐
│                  WebView                     │
│  (rule34video.com + injected JS scripts)    │
├─────────────────────────────────────────────┤
│           Tauri Bridge (IPC)                │
│  check_url_blocked · get_page_cosmetic_     │
│  open_child_window · postMessage · etc.     │
├─────────────────────────────────────────────┤
│              Rust Backend                   │
│  ┌──────┐ ┌──────────┐ ┌───────────────┐   │
│  │Adblock│ │Navigation│ │Child Windows  │   │
│  ├──────┤ ├──────────┤ ├───────────────┤   │
│  │Tray  │ │Downloads │ │Context Menu   │   │
│  └──────┘ └──────────┘ └───────────────┘   │
├─────────────────────────────────────────────┤
│       Native Platform Layer                 │
│  WebView2 · WKWebView · GTK · Android WV   │
└─────────────────────────────────────────────┘
```

---

## 🪟 Desktop vs Mobile split

The app has two completely separate entry paths via `#[cfg]`:

### 🖥️ Desktop path (`cfg(not(any(android, ios)))`)
- Full feature set: tray icon, global shortcuts, child windows, context menu, notifications
- Native WebView2 resource interception on Windows (`webview_intercept.rs`)
- `build_init_script()` combines navigation + context menu + child bridge + adblock scripts

### 📱 Mobile path (`cfg(any(android, ios))`)
- Minimal feature set: adblock + deep linking only
- No tray, no shortcuts, no child windows, no context menu
- Only the adblock script is injected

---

## 🧩 Module architecture

### 📁 `src-tauri/src/ext/` — Feature modules

| Module | Platform | Purpose |
|---|---|---|
| `adblock.rs` | All | Adblock engine + JS injection + Tauri commands |
| `webview_intercept.rs` | Windows only | Native WebView2 COM request interception |
| `navigation.rs` | All | Link handling, init script composition |
| `child_windows.rs` | Desktop | In-app child webview windows |
| `context_menu.rs` | Desktop | Native right-click context menus |
| `downloads.rs` | All | Download interception + sanitization |
| `tray.rs` | Desktop | System tray icon + menu |
| `global_shortcuts.rs` | Desktop | Global keyboard shortcuts |
| `webnotifications.rs` | All | Notification permission handling |
| `cloudfare.rs` | All | User-agent for anti-bot bypass |
| `universal_deep_link.rs` | All | Deep link scheme (`rule34video://`) |

### 📁 `src-tauri/src/lib.rs` — Application wiring

The `run()` function (two versions, desktop and mobile):

1. Registers all Tauri commands via `generate_handler![]`
2. Installs all plugins (opener, notification, deep-link, shell, global-shortcut)
3. Creates the `AdblockManager` and stores it as managed state
4. Spawns an async task to download + merge filter lists
5. Builds the webview window with:
   - Initialization script (all injected JS)
   - `on_navigation` callback (adblock engine check)
   - `on_download` callback (download handler)
6. Initializes all feature modules (tray, shortcuts, deep links, etc.)

---

## 🔌 Tauri commands (IPC bridge)

| Command | Purpose | Called by |
|---|---|---|
| `check_url_blocked` | Check URL against adblock engine | `adblock_script()` JS |
| `get_page_cosmetic_filters` | Get cosmetic CSS/scriptlets | `adblock_script()` JS |
| `open_child_window_cmd` | Open internal link in child webview | Navigation JS (desktop) |
| `child_post_message_cmd` | Forward postMessage to main window | Child window bridge JS |
| `show_native_context_menu_cmd` | Show native context menu | Context menu JS (desktop) |

---

## 🧵 Thread model

- **Main thread**: Webview creation, plugin setup, tray, shortcuts
- **Async thread pool**: Tauri commands (`check_url_blocked`, `get_page_cosmetic_filters`), filter list downloads
- **WebView2 COM callback thread**: Resource request interception (Windows)
- All adblock engine access is guarded by `Mutex<Engine>` — safe for cross-thread use

---

## 🔒 Security model

| Concern | Mitigation |
|---|---|
| **Malicious ad URLs** | Adblock engine checks every external request at native level |
| **Popup spam** | `window.open` intercepted, external URLs go to system browser only |
| **Download path traversal** | Filenames sanitized, path separators stripped, length limited to 200 chars |
| **Deep link injection** | Only `rule34video://` and `rule34video.net` universal links are accepted |
| **Script injection** | Only our controlled init scripts are injected; website JS runs in normal webview sandbox |
| **Bot detection** | Realistic Chrome 120 User-Agent set to avoid Cloudflare challenges |
