# 🛡️ Adblock System

> The most complex subsystem — a dual-layer approach combining a native Rust adblock engine with platform-specific WebView2 interception.

---

## 🎯 How it works (bird's eye view)

```sql
         User requests page
               │
               ▼
    ┌──────────────────────┐
    │  on_navigation()     │ ← Blocks top-level navigations to ad URLs
    └──────────┬───────────┘
               │
               ▼
    Page starts loading HTML
               │
     ┌─────────┴───────────┐
     │                     │
     ▼                     ▼
┌──────────────┐   ┌───────────────┐
│ Native       │   │ JS Injection  │
│ WebView2     │   │ (fallback)    │
│ Interception │   │               │
│ (Windows)    │   │ • fetch/XHR   │
│              │   │ • createEl    │
│ Blocks ALL   │   │ • MutationObs │
│ sub-resource │   │ • CSS hiding  │
│ requests at  │   │ • Cosmetic    │
│ COM level    │   │   filters     │
└──────────────┘   └───────────────┘
       │                   │
       ▼                   ▼
   Blocked? ──yes──► 204 No Content
       │
       no
       ▼
   Request proceeds
```

---

## ⚙️ The Adblock Engine (`adblock-rust`)

The engine is **Brave's `adblock-rust`** library (v0.12.5), the same engine that powers Brave Browser's native ad blocking.

### AdblockManager

```rs
pub struct AdblockManager {
    engine: Mutex<Engine>,
}

unsafe impl Send for AdblockManager {}
unsafe impl Sync for AdblockManager {}
```

- `adblock::engine::Engine` uses `Rc`/`RefCell` internally — it's `!Send + !Sync`
- We wrap it in `Mutex<Engine>` and manually impl `Send + Sync`, which is safe because the `Mutex` guarantees single-threaded access
- The `Mutex` is held only during the `is_blocked()` check (microseconds)

### Startup flow

```sql
App starts
    │
    ▼
new_with_bundled() ←── Creates engine from ~900 bundled rules
    │                    (compiled into binary via include_str!)
    │
    ▼
Filter download task spawned (async)
    │
    ├── Try loading engine_cache.bin (serialized from last run)
    │
    └── Download AdGuard Base + Tracking Protection lists
         │
         ├── Merge with bundled rules
         ├── Create new engine
         ├── Serialize to engine_cache.bin
         └── Replace the running engine
```

### Why bundled rules?

The async filter download may take seconds. Without bundled rules, the adblock engine starts **empty** — meaning the first page load sees zero blocked ads. Bundled rules ensure protection is active from the very first millisecond.

### Filter sources

| Source | Rules | Location |
|---|---|---|
| Bundled rules | ~900 | `adblock_bundled.txt` (compiled into binary) |
| AdGuard Base | ~80,000 | Downloaded from `filters.adtidy.org` |
| AdGuard Tracking | ~20,000 | Downloaded from `filters.adtidy.org` |

Filter cache is stored at `{app_data_dir}/adblock-filters/engine_cache.bin`.

---

## 🪟 Native WebView2 Resource Interception (Windows)

### The problem

Ads loaded via `<script src="//ad-server.com/ad.js">` in raw HTML **cannot be blocked by JavaScript**. The browser's HTML parser encounters the tag and starts fetching the resource before any JS injection runs. `on_navigation` only catches top-level navigations — not sub-resource loads like scripts, images, or iframes.

### The solution

We use WebView2's COM API directly:

```rs
// Pseudocode — see webview_intercept.rs
let core: ICoreWebView2 = controller.CoreWebView2()?;

// Intercept ALL resource types
core.AddWebResourceRequestedFilter("*", ALL_RESOURCES)?;

// Register callback
core.add_WebResourceRequested(handler)?;

// In the handler:
fn on_request(args) {
    let url = args.Request()?.Uri()?;
    if engine.is_blocked(url, ...) {
        let response = env.CreateWebResourceResponse(
            None, 204, "No Content", ""
        )?;
        args.SetResponse(&response)?;
    }
}
```

Key details:
- The filter catches **every sub-resource request**: scripts, images, media, XHR, fetch, fonts, stylesheets, websockets, etc.
- Returns a **204 No Content** response for blocked URLs — the resource never loads
- Runs at the **native COM level**, before any JavaScript in the page executes
- Only available on **Windows** (`#[cfg(windows)]`) — on other platforms, JS injection is the sole sub-resource blocker

### Why not just use JS interception everywhere?

| Interception method | Catches HTML-parsed tags | Catches dynamic elements | Performance impact |
|---|---|---|---|
| `on_navigation` | ✅ (top-nav only) | ❌ | Minimal |
| WebView2 native | ✅ **all sub-resources** | ✅ | Minimal |
| JS `fetch` override | ❌ | ✅ | Minimal |
| JS `createElement` override | ❌ | ✅ | Low |
| JS MutationObserver | ❌ (too late) | ✅ (too late for load) | Low |
| JS `setAttribute` override | ❌ | ✅ | ❌ **High** |
| JS `querySelectorAll` observer | ❌ | ✅ | ❌ **Very High** |

---

## 📜 JS Injection (`adblock_script()`)

A minimal, performant fallback that runs inside every page:

### What it does (in order)

1. **`fetch` interception** — Wraps `window.fetch` to check URLs against the engine via IPC. Returns `Promise.reject()` if blocked.

2. **`XMLHttpRequest` interception** — Wraps `open`/`send` to check URLs. Aborts the request if blocked.

3. **`createElement` `src` descriptor** — Overrides the `src` property setter on dynamically created `<img>`, `<iframe>`, `<video>`, `<source>`, `<embed>`, `<object>` elements. Checks the URL before allowing the set.

4. **Minimal MutationObserver** — Observes `childList` changes on `documentElement`. Checks each added node individually (tag name + src). No `querySelectorAll`, no attribute observation.

5. **Immediate CSS hiding** — Injects a `<style>` tag that hides common ad selectors (`ins.adsbygoogle`, `[id^="ad-"]`, `[class*="ad-"]`, etc.).

6. **Cosmetic filters (once)** — Calls `get_page_cosmetic_filters` via IPC after the page is fully loaded (with 100ms delay), applying engine-generated hide selectors and scriptlets.

### What it does NOT do

- ❌ `setAttribute` override — called on every attribute set everywhere; caused app freezes
- ❌ `data-src` interception — extremely rare case, not worth the overhead
- ❌ Periodic `setInterval` — was polling cosmetic filters every 5 seconds, causing constant IPC churn
- ❌ `querySelectorAll` in MutationObserver — scanning every subtree addition with a complex selector was the #1 cause of freezing
- ❌ Post-load scan — redundant with MutationObserver

---

## 🔍 Debugging ad issues

### Ads still showing? Check how they load:

1. **Is the ad domain in our filter list?**
   - Check `adblock_bundled.txt` — add `||domain.com^$third-party` if missing
   - The full AdGuard lists (downloaded at startup) cover most common networks

2. **Does the ad load via an HTML-parsed `<script src>` tag?**
   - ✅ Needs WebView2 native interception (Windows)
   - Check `webview_intercept.rs` is properly set up
   - On other platforms, these ads cannot be blocked at the network level

3. **Does the ad load via dynamic JavaScript (fetch/XHR)?**
   - ✅ Our JS injection should catch it
   - Check the URL matches a filter rule
   - Verify `check_url_blocked` IPC command is registered

4. **Is it a text-link or inline ad?**
   - ✅ Needs a cosmetic filter rule: `domain.com##a[href*="ad-domain"]`
   - Add the CSS selector to `adblock_bundled.txt`

### Common ad domains we've already blocked

```sql
guidepaparazzisurface.com   ← Main ad script server
spankurbate.com             ← Header text-link ad
trafficjunky.com            ← Adult ad network
trafficjunky.net
exoclick.com                ← Adult ad network
juicyads.com
clickadilla.com
bawafx.com                  ← Affiliate/tracking
hrtyj.com                   ← Affiliate/tracking
happyleafmotion.com         ← AI game affiliate
theporndude.com             ← Header affiliate link
rule34comic.party           ← Sidebar affiliate link
```
