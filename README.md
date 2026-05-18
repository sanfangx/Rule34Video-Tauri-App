# 🏠 Rule34Video Tauri App

**🚀 Current release:** [v1.0.4](CHANGELOG.md#v104---2026-05-18) • **Updated:** 2026-05-18

A privacy-forward native wrapper for [rule34video.com](https://rule34video.com) — built with **Tauri v2** and **Rust**.  
No separate browser. No extensions. No bloat.

---

## 🎯 What is this?

This app turns the rule34video.com website into a **desktop-class application** with native ad blocking, controlled navigation, secure downloads, and system tray integration — all while preserving the familiar web experience.

Under the hood, it uses a **dual-layer adblock architecture**: a Rust-native `adblock-rust` engine (same one Brave uses) combined with platform-specific WebView2 COM interception on Windows to catch ads at the native level before they even load.

---

## ✨ Features

| 🛡️ **Native ad blocking** | WebView2 COM-level request interception blocks ads before they load. JavaScript injection catches dynamically created elements. Combined with ~100K AdGuard filter rules + ~900 bundled fallback rules. [Full details →](docs/adblock.md) |
|---|---|
| 🔗 **Smart navigation** | `window.open()` and `target="_blank"` links are intercepted. Internal site links open in a managed child webview; external links go to your system browser. |
| 📥 **Safe downloads** | Filename sanitization, collision handling, and native OS notifications on completion or failure. |
| 🖥️ **System tray** | Minimize to tray with "Show" / "Quit" menu. Left-click to restore. (Desktop) |
| ⌨️ **Global shortcut** | `Ctrl+Shift+O` (`Cmd+Shift+O`) to show the window from anywhere. (Desktop) |
| 🔗 **Deep linking** | Open the app directly via `rule34video://` custom scheme or `rule34video.net` universal links. |
| 🔔 **Web notifications** | Automatic permission handling for site notifications. |
| 📋 **Native context menu** | Right-click links for "Open in Browser". (Desktop) |
| 📱 **Cross-platform** | Windows, macOS, Linux, Android, and iOS. |

---

## 🏗️ Tech stack

| Layer | Technology |
|---|---|
| **Shell** | Tauri v2 (`tauri-runtime-wry`) |
| **Webview** | WebView2 (Win), WKWebView (macOS/iOS), WebKitGTK (Linux), Android WebView |
| **Adblock engine** | `adblock-rust` v0.12 (Brave's engine) |
| **Filter sources** | AdGuard Base + Tracking Protection + bundled custom rules |
| **Native interception** | `webview2-com` + `windows` crates for WebView2 COM API |
| **Language** | Rust (edition 2021) |

---

## 📚 Documentation

| Page | Description |
|---|---|
| [📖 Introduction](docs/index.md) | Project overview, features, tech stack |
| [📐 Architecture](docs/architecture.md) | Layered architecture, platform split, module details |
| [🛡️ Adblock System](docs/adblock.md) | Deep dive into the dual-layer adblock architecture |
| [✨ Features](docs/features.md) | Detailed documentation of every feature |
| [🛠️ Development](docs/development.md) | Building, testing, debugging, release process |

---

## 🚀 Quick start

```bash
# 1. Install Tauri CLI
cargo install tauri-cli --version "^2" --locked

# 2. Clone and enter
git clone https://github.com/PhantomNimbi/Rule34Video-Tauri-App.git
cd Rule34Video-Tauri-App

# 3. Run in dev mode
cargo tauri dev

# 4. Build for production
cargo tauri build
```

### Platform-specific builds

| Platform | Command |
|---|---|
| Windows | `cargo tauri build` |
| macOS | `cargo tauri build` |
| Linux | `cargo tauri build` |
| Android | `cargo tauri android build --apk -t aarch64 armv7 i686 x86_64` |
| iOS | `cargo tauri ios build` |

---

## 📁 Project structure

```
rule34video-tauri-app/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs                 # App setup, command registration
│   │   ├── main.rs                # Binary entrypoint
│   │   └── ext/
│   │       ├── adblock.rs         # Adblock engine + JS injection
│   │       ├── adblock_bundled.txt  # ~900 bundled filter rules
│   │       ├── webview_intercept.rs  # WebView2 native interception (Win)
│   │       ├── navigation.rs      # Link handling + init script builder
│   │       ├── child_windows.rs   # Child webview windows (desktop)
│   │       ├── context_menu.rs    # Native right-click menu (desktop)
│   │       ├── downloads.rs       # Download interception
│   │       ├── tray.rs            # System tray (desktop)
│   │       ├── global_shortcuts.rs  # Global shortcuts (desktop)
│   │       ├── webnotifications.rs  # Notification permissions
│   │       ├── cloudfare.rs       # Anti-bot User-Agent
│   │       └── universal_deep_link.rs  # Deep link handler
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── tauri.windows.conf.json
│   ├── tauri.macos.conf.json
│   ├── tauri.linux.conf.json
│   ├── tauri.ios.conf.json
│   └── tauri.android.conf.json
└── docs/                           # You are here
```

---

## ✅ Supported platforms

| Platform | Target triple | Status |
|---|---|---|
| Windows | `x86_64-pc-windows-msvc` | ✅ Full support |
| macOS | `aarch64-apple-darwin` | ✅ Full support |
| Linux | `x86_64-unknown-linux-gnu` | ✅ Full support |
| Android | `aarch64-linux-android` + ARMv7 + i686 + x86_64 | ✅ Full support |
| iOS | `aarch64-apple-ios` | ✅ Full support |

---

## 📄 License

This repository includes a `LICENSE` file. Review it for details on usage, distribution, and contribution terms.
