# 🤖 AI Agent Instructions for Rule34Video Tauri App

This document provides detailed instructions for AI agents working on the Rule34Video Tauri App. Follow these guidelines to ensure consistency with project standards and maintain code quality.

## 📋 Overview

### Current release

- Version: v1.0.4
- Date: 2026-05-18
- Summary: Bug fixes and adblock improvements.
## 🏗️ Architecture Guidelines

### Tauri V2 Standards
1. **Separation of Concerns**: Keep Rust backend logic separate from frontend concerns
2. **Command Pattern**: Use Tauri commands (`#[tauri::command]`) for all backend-frontend communication
3. **Plugin Architecture**: Leverage Tauri plugins for cross-platform functionality
4. **Event System**: Use Tauri's event system for loose coupling between modules
5. **Security First**: Always validate and sanitize inputs, especially from web content
6. **Platform-Specific Code**: Use conditional compilation (`#[cfg(desktop)]` and `#[cfg(not(desktop))]`) for code that only runs on specific platforms

### Code Organization
- `src-tauri/`: Contains platform-specific configuration and build scripts (single source for all platforms)
- `src-tauri/src/lib.rs`: Main application entry point and setup for desktop and mobile (desktop features plus `#[tauri::mobile_entry_point]` support)
- Feature modules: Each major feature gets its own file in `src-tauri/src/ext/` (adblock.rs, webview_intercept.rs, downloads.rs, etc.)
- Shared logic: Put in `src-tauri/src/ext/` or create new shared modules as needed
- Platform-specific UI features: Tray, child windows, and global shortcuts are desktop-only
- Platform-specific request interception: `webview_intercept.rs` is Windows-only via `#[cfg(windows)]`

### Adblock Architecture
- **Dual-layer approach**: Combines a Rust-native `adblock-rust` engine (Brave's library, v0.12) with JS injection for webview resource blocking.
- **Engine**: `AdblockManager` wraps `adblock::engine::Engine` inside `Mutex<Engine>` with `unsafe impl Send+Sync` (Engine uses `Rc`/`RefCell` internally; Mutex ensures single-threaded access, making it safe to share across threads).
- **Bundled rules** (`adblock_bundled.txt`): ~900 rules compiled into the binary via `include_str!` so the engine is never empty on first launch.
- **Runtime updates**: Asynchronously downloads AdGuard Base + Tracking Protection lists on startup, stores a serialized engine cache (`engine_cache.bin`) for fast subsequent loads.
- **Filter syntax**: Uses Adblock Plus / uBlock Origin syntax (`||domain.com^$third-party`, `rule34video.com##.ad-class` for cosmetic hiding).
- **On-navigation blocking**: `on_navigation()` callback blocks top-level navigations to ad/document URLs via the engine.
- **Native resource interception** (`webview_intercept.rs`, Windows-only): Uses WebView2's `ICoreWebView2.AddWebResourceRequestedFilter` + `add_WebResourceRequested` COM events to intercept ALL sub-resource HTTP requests (scripts, images, media, XHR, etc.) at the native level, before the browser processes them. This is the only reliable way to block `<script src="...">` tags parsed from raw HTML — JS injection alone cannot catch these.
- **JS injection** (`adblock_script()`): Fallback for dynamically created elements. Includes fetch/XHR interception, `createElement` src descriptor for media/script elements, lightweight MutationObserver (per-element check, no querySelectorAll), immediate CSS hiding, and single cosmetic filter load on page idle. No `setAttribute` override, no `data-src` interception, no 5-second setInterval — those caused freezes.
- **Performance**: The JS injection is intentionally minimal to avoid freezing. Heavy interception mechanisms (setAttribute override, querySelectorAll in MutationObserver, constant IPC polling) were removed after they caused app freezes from DOM mutation overhead.
- **Platform-specific `webview_intercept`**: Only enabled on Windows via `#[cfg(windows)]`. On other platforms, JS injection + `on_navigation` are the only blockers.
- **Debugging tip**: If ads still show, check whether they're loaded via HTML-parsed `<script src>` tags (need WebView2 interception to block) vs dynamically created elements (need JS injection + engine rules). Add the ad URL/domain to `adblock_bundled.txt` if it's not caught.

## 🔧 Development Workflow

### Setting Up
```bash
# Clone repository
git clone https://github.com/PhantomNimbi/Rule34Video-Tauri-App.git
cd Rule34Video-Tauri-App

# Install Tauri CLI
cargo install tauri-cli --version "^2" --locked

# Desktop development
cargo tauri dev

# Android development
cargo tauri dev --target <android-target>

# iOS development
cargo tauri dev --target <ios-target>

# For release builds (desktop)
cargo tauri build

# For release builds (mobile)
cargo tauri build --target <target-triple>
```

### Making Changes
1. Create a descriptive branch: `git checkout -b feat/your-feature-name`
2. Make focused changes following existing code style
3. Update documentation if your change affects documented behavior — the project has a `docs/` directory with five pages (`index.md`, `architecture.md`, `adblock.md`, `features.md`, `development.md`)
4. When adding or modifying features, update the relevant `docs/*.md` page(s) to keep them in sync
5. Test thoroughly on target platforms
6. Commit using Conventional Commits format
7. Open pull request against main branch

## 🚀 Release Automation
- AI agents should publish a new GitHub release for the repository on every push to `main` when a version or release-worthy change is present.
- Before publishing, always run `.github/scripts/auto-version.sh --commit --push` to calculate and update the correct version and release number in manifests and documentation.
- The auto-version workflow is configured in `.github/workflows/release-manager.yml` and runs automatically on `main` pushes.
- The workflow uses `secrets.PAT_TOKEN` for push and release authentication, so this secret must be configured in the repository.
- Use the tag created by the script as the release tag.
- Ensure the published release is treated as the latest release on GitHub, even if the build semantics are prerelease in nature.
- Keep `README.md`, `CHANGELOG.md`, and `docs/Development.md` in sync with the release metadata.

### Code Style
- Follow existing Rust idioms and patterns
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and under 50 lines when possible
- Handle errors appropriately using `Result` types
- Use `tracing` or `println!` for debug output (remove in production)

## 🧪 Testing

### Unit Tests
- Write unit tests for business logic in each module
- Use the `[cfg(test)]` module pattern
- Mock external dependencies when necessary
- Aim for high test coverage on core functionality

### Integration Testing
- Test end-to-end flows on target platforms
- Verify inter-module communication works correctly
- Test edge cases and error conditions

## 📁 File Structure Guidelines

### Adding New Features
1. Create new module in `src-tauri/src/ext/` (e.g., `new_feature.rs`)
2. Add module declaration to `src-tauri/src/lib.rs`
3. Implement Tauri commands for frontend communication
4. Register commands in the invoke handler in `lib.rs::run()`
5. Add any necessary setup in the setup closure
6. Update documentation in appropriate `.md` file

### Modifying Existing Features
1. Locate the relevant feature module in `src-tauri/src/ext/`
2. Make minimal, focused changes
3. Ensure backward compatibility where possible
4. Update associated tests
5. Verify no regressions in related functionality

## ⚠️ Common Pitfalls to Avoid

### Tauri-Specific Issues
- Don't perform heavy computations on the main thread
- Avoid blocking async operations in command handlers
- Be mindful of webview security considerations
- Don't expose unsafe APIs to the frontend
- Remember that commands run on a thread pool, not the main thread
- When publishing a version that must be the latest release, do not mark it as a GitHub prerelease. The published release should be the latest release regardless of whether it is a prerelease build.

### Security Considerations
- Always validate URLs and inputs from web content
- Sanitize filenames to prevent path traversal
- Use Tauri's built-in security features (CSP, etc.)
- Be careful with deep link handling
- Validate all data coming from the webview
- When accessing the native WebView2 COM API (`ICoreWebView2`), always use `unsafe` blocks and handle `Result` errors gracefully — COM calls can fail if the webview is not fully initialized

### Performance
- Don't block the UI thread with long-running operations
- Use async/await for I/O operations
- Consider using Tauri's async command support
- Be mindful of memory usage with webviews
- Cache expensive computations when appropriate

## 📚 Resources

- [Tauri V2 Documentation](https://tauri.app/v2/guides/)
- [Rust Programming Language](https://www.rust-lang.org/)
- [Tauri Plugins](https://tauri.app/v2/plugins/)
- [Conventional Commits](https://www.conventionalcommits.org/)

## 🆘 Getting Help

If you're unsure about something:
1. Check existing code for similar patterns
2. Look at the documentation in `docs/`
3. Ask in the project discussions
4. Review open issues and pull requests for context

Remember: When in doubt, keep it simple and follow existing patterns in the codebase.