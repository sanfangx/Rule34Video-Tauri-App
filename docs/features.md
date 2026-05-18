# ✨ Features

> Detailed documentation of every feature in the Rule34Video Tauri App.

---

## 🛡️ Ad Blocking

| Aspect | Detail |
|---|---|
| **Approach** | Dual-layer: native WebView2 COM interception (Windows) + JS injection (all platforms) |
| **Engine** | `adblock-rust` v0.12 (Brave's engine) — supports Adblock Plus / uBlock Origin syntax |
| **Filter lists** | AdGuard Base + Tracking Protection (~100K rules) downloaded on startup |
| **Bundled fallback** | ~900 custom rules compiled into binary for instant protection on first launch |
| **Cosmetic filters** | CSS element hiding via engine's `url_cosmetic_resources()` + scriptlets |
| **On-navigation blocking** | `on_navigation()` callback prevents navigating to ad/document URLs |
| **Performance** | JS injection is minimal (~90 lines) to avoid freezing; heavy observers removed |

[Full details →](adblock.md)

---

## 🔗 Smart Navigation

Internal links (`rule34video.net` domain) open inside the app in a managed child webview window. External links open directly in your system browser.

### How it works

1. `window.open()` is overridden to intercept all popup requests
2. Anchor clicks with `target="_blank"` or modifier keys are captured
3. The URL is checked against `rule34video.net` — internal or external?
4. **Internal** → Opens in a child webview (desktop) or navigates in-place (mobile)
5. **External** → Opens in the default system browser via `tauri-plugin-opener`

### Why?

- Prevents ad popups from spawning new browser windows
- Keeps the app experience contained and controlled
- External links still work seamlessly — they just open where they belong

---

## 🪟 Child Windows (Desktop-only)

When you open an internal link in a new tab (or via `window.open`), it opens in a dedicated child webview window:

- Labeled `"child"` — only one child window exists; reusing it for subsequent opens
- Shares the same initialization scripts (navigation handling, adblock, context menu)
- Positioned and sized independently from the main window
- Supports `postMessage` communication back to the main window (for the site's attachment manager)

---

## 📥 Download Handling

All file downloads are intercepted and managed:

1. **Filename sanitization** — strips path separators (`/`, `\`, `..`), control characters, leading dots/spaces
2. **Length limiting** — filenames capped at 200 characters
3. **Collision handling** — if a file with the same name exists, appends `(1)`, `(2)`, etc.
4. **Native notifications** — on download completion or failure, a system notification is shown

---

## 🖥️ System Tray (Desktop-only)

The app minimizes to the system tray instead of closing:

- **Tray icon** with "Show" and "Quit" context menu
- **Left-click** on the tray icon shows/focuses the main window
- No background processes — the app continues running normally in the tray

---

## ⌨️ Global Shortcut (Desktop-only)

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+O` (Windows/Linux) | Show and focus the main window |
| `Cmd+Shift+O` (macOS) | Show and focus the main window |

Works even when the app is minimized to the tray or behind other windows.

---

## 🔗 Deep Linking

Open the app directly from a custom URI scheme or universal link:

| Scheme | Example |
|---|---|
| `rule34video://` | `rule34video://video/12345` |
| HTTPS universal | `https://rule34video.net/video/12345` |

Deep links are handled by `universal_deep_link.rs` — they map the custom scheme or `rule34video.net` domain to the equivalent `rule34video.com` URL and navigate the main webview there.

---

## 🔔 Web Notifications

The site's notification system (for upload alerts, messages, etc.) is supported:

- On startup, the app checks the notification permission state
- If still in the `Prompt` or `PromptWithRationale` state, it requests permission automatically
- This prevents the browser-style permission prompt that would normally appear

---

## 📋 Native Context Menu (Desktop-only)

Right-clicking on links shows a native OS context menu with "Open Link in Browser":

- Captures right-clicks on anchor elements
- Extracts the link URL and any selected text
- Shows a native popup menu at the cursor position
- "Open Link in Browser" passes the URL to `tauri-plugin-opener`

---

## ☁️ Cloudflare / Anti-Bot Bypass

The webview sends a realistic Chrome 120 User-Agent string to avoid triggering Cloudflare's bot detection or basic anti-bot challenges that would otherwise block or delay page loads.
