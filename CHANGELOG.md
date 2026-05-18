## v1.0.4 - 2026-05-18

### ✨ What's new

- **🔒 Native ad blocking at the WebView2 level** — Ads loaded via raw HTML `<script src>` tags are now intercepted before the browser even processes them, using the Windows WebView2 COM API (`ICoreWebView2.AddWebResourceRequested`). This catches what JS-based adblocking never could.
- **🚀 Performance overhaul** — The injected JavaScript interception layer was rewritten from scratch. Gone are the `setAttribute` override, `querySelectorAll` in MutationObserver, `data-src` interception, and the 5-second `setInterval` that were causing the app to freeze under DOM mutation pressure.
- **📋 30+ new filter rules** — Added coverage for adult ad networks discovered on the site (TrafficJunky, ExoClick, JuicyAds, Spankurbate, and more), plus first-party ad path patterns and cosmetic hiding rules for affiliate buttons and ad cards.
- **🛡️ Discovered and blocked the primary ad domain** — `guidepaparazzisurface.com` was identified as the main ad script server, along with affiliate/tracking domains (`bawafx.com`, `hrtyj.com`, `happyleafmotion.com`).

### 🐛 Fixed

- **App no longer freezes** when scrolling the video grid or navigating the site — the MutationObserver no longer scans every subtree with `querySelectorAll`.
- **Video ads and ad widgets now actually block** — the native WebView2 interceptor catches all sub-resource requests at the process level, including HTML-parsed `<script>` tags that JS interception could never touch.
- **Sidebar affiliate links** (Spankurbate, ThePornDude, rule34comic.party) and the animated "AI Jerk Off" button are now hidden via cosmetic filter rules.

### 🏗️ Behind the scenes

- Native WebView2 resource interception (`webview_intercept.rs`) runs on Windows via `ICoreWebView2_4::add_WebResourceRequested` — filters all resource contexts and returns a 204 No Content response for blocked URLs.
- The adblock engine (`adblock-rust` v0.12) continues to power both the `on_navigation` callback and the new native interceptor, using the same bundled + runtime-downloaded filter lists.
- JS injection was trimmed from ~320 lines to ~90 lines, keeping only what's strictly needed: `fetch`/`XHR` interception, a lightweight `createElement` `src` descriptor, and a minimal MutationObserver.

## v1.0.3 - 2026-05-17

### ✨ What changed

- Initial release of the desktop and mobile wrapper for rule34video.com.

### 📝 Details

- Native, privacy-minded browser wrapper for rule34video.com built with Tauri v2 and Rust.
- Network- and DOM-level ad blocking to reduce distractions and improve page load behavior. *Currently minimal at best. Could possibly benefit from being replaced with adguard based ad blocking for a better and more complete ad block support*.
- Secure download interception with filename sanitization and safe save-path handling.
- System tray integration on desktop platforms for background operation and quick restore.
- Custom deep linking support via `rule43video://` and mobile app links.
- Child window and popup management on desktop to keep navigation controlled and secure.
- Cross-platform support across Windows, macOS, Linux, and Android.
