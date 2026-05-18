# 🏠 Rule34Video Tauri App

> **Version:** v1.0.4 · **Updated:** 2026-05-18

A privacy-forward native wrapper for [rule34video.com](https://rule34video.com) — built with **Tauri v2** and **Rust** — that turns a standard web browsing experience into a secure, ad-blocking, desktop-class application.

---

## 🎯 What is this?

This app wraps the rule34video.com website in a native Tauri webview, adding:

- 🛡️ **Ad blocking** at both the native OS and JavaScript level
- 🔗 **Controlled navigation** — popups and new tabs open inside the app or your system browser safely
- 📥 **Secure downloads** — filename sanitization and safe save paths
- 🖥️ **System tray integration** — minimize to tray, quick restore with a global keyboard shortcut
- 📱 **Cross-platform** — works on Windows, macOS, Linux, Android, and iOS

All of this without a separate browser, without extensions, and without the bloat.

---

## ✨ Quick features

| Feature | Description |
|---|---|
| 🔒 **Native ad blocking** | WebView2 COM-level request interception blocks ads before they load (Windows) |
| 🚀 **Lightweight JS injection** | Minimal overhead — no more freezing from heavy DOM observers |
| 📋 **AdGuard filter lists** | Downloads and merges AdGuard Base + Tracking Protection lists on first launch |
| 🗂️ **Bundled fallback rules** | ~900 pre-packaged filter rules so blocking works immediately on first run |
| 🔗 **Smart navigation** | Internal links open in the app; external links go to your system browser |
| 🪟 **Child window management** | `window.open()` and `target="_blank"` open as managed child webviews |
| 📥 **Safe downloads** | Filename sanitization and collision handling with native notifications |
| 🖥️ **System tray** | Background operation with quick restore (desktop-only) |
| ⌨️ **Global shortcut** | `Ctrl+Shift+O` to show the window from anywhere (desktop-only) |
| 📱 **Deep linking** | `rule34video://` scheme and universal links for seamless app entry |
| ☁️ **Anti-bot** | Realistic Chrome 120 User-Agent to avoid Cloudflare challenges |

---

## 🏗️ Tech stack

| Layer | Technology |
|---|---|
| **Desktop shell** | Tauri v2 (`tauri-runtime-wry`) |
| **Webview** | WebView2 (Windows), WKWebView (macOS/iOS), WebViewGTK (Linux), Android WebView |
| **Adblock engine** | `adblock-rust` v0.12 (Brave's engine) |
| **Filter sources** | AdGuard Base + Tracking Protection, bundled custom rules |
| **Native COM** | `webview2-com` + `windows` crates for WebView2 resource interception |
| **HTTP client** | `reqwest` for filter list downloads |
| **Build system** | `cargo tauri` with platform-specific configs |

---

## 📂 Project layout

```
rule34video-tauri-app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Binary entrypoint
│   │   ├── lib.rs                # App setup, command registration
│   │   └── ext/
│   │       ├── mod.rs            # Module declarations
│   │       ├── adblock.rs        # Adblock engine + JS injection
│   │       ├── adblock_bundled.txt  # ~900 bundled filter rules
│   │       ├── webview_intercept.rs  # Native WebView2 interception (Win)
│   │       ├── navigation.rs     # Link handling + init script builder
│   │       ├── child_windows.rs  # Child webview windows
│   │       ├── context_menu.rs   # Native right-click menu
│   │       ├── downloads.rs      # Download interception
│   │       ├── tray.rs           # System tray (desktop)
│   │       ├── global_shortcuts.rs  # Keyboard shortcuts (desktop)
│   │       ├── webnotifications.rs  # Notification permissions
│   │       ├── cloudfare.rs      # User-agent spoofing
│   │       └── universal_deep_link.rs  # Deep link handler
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── tauri.windows.conf.json
│   ├── tauri.macos.conf.json
│   ├── tauri.linux.conf.json
│   ├── tauri.ios.conf.json
│   └── tauri.android.conf.json
├── docs/
│   ├── index.md                  # You are here
│   ├── architecture.md           # Architecture deep-dive
│   ├── adblock.md                # Adblock system details
│   ├── features.md               # Feature documentation
│   └── development.md            # Building and contributing
├── AGENTS.md                     # AI agent instructions
├── CHANGELOG.md                  # Release history
└── README.md                     # Project overview
```

---

## 📖 Continue reading

| Page | What you'll find |
|---|---|
| [📐 Architecture](architecture.md) | How the app is structured, platform split, module layering |
| [🛡️ Adblock System](adblock.md) | Deep dive into the dual-layer adblock architecture |
| [✨ Features](features.md) | Detailed documentation of every feature |
| [🛠️ Development](development.md) | Building, testing, and contributing |
