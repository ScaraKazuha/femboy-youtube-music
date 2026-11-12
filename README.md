# Femboy YouTube Music for Windows

An unofficial YouTube Music desktop app.

A small Rust + Tauri 2 desktop wrapper around **https://music.youtube.com**, using the installed Windows WebView2 runtime. **Unofficial: not affiliated with, endorsed by, or sponsored by Google or YouTube.** YouTube and YouTube Music are their owners' trademarks. The icon and splash artwork are fan art of a mascot holding the YouTube Music logo; the logo remains Google's trademark and is used only to show which service this wrapper opens.

## Use

- Search, playlists, account features and playback use the official website directly.
- Closing the window (including Alt+F4) hides it to the system tray. The same webview stays alive, including playback. Windows may place the tray icon in its overflow area.
- Left-click the tray icon or choose **Show** to restore it. Launching the app again also restores the existing window, including when minimized.
- The window has a dark frame and no menu bar; every command is in the tray icon's right-click menu.
- Choose **Quit** in the tray menu to stop playback and exit.
- **About / sign-in help** explains sign-in restrictions and offers **Open in browser**. A separate **Open in browser** item is always available.
- **YouTube Music home** recovers from an auth/error page or retries after a network interruption.
- **Check for updates** looks for a newer release. The app also checks once at startup and asks before installing.

## Automatic updates and releases

The app reads `latest.json` from this repository's latest GitHub release and only installs an installer whose signature matches the public key in `src-tauri/tauri.conf.json`. The check runs in native Rust; web pages get no updater permission.

To publish a new version:

1. Set the same new version in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, run `cargo check` so `Cargo.lock` follows, and commit.
2. Tag and push: `git tag v0.2.1` then `git push origin main --tags`.

The **Release** workflow builds the installer, signs it with the `TAURI_SIGNING_PRIVATE_KEY` repository secret, and publishes the release with `latest.json`. Installed copies offer the update on their next start. Keep the private key (outside the repository) safe: without it, existing installs cannot be updated.

The profile lives in `%LOCALAPPDATA%\io.github.unofficial-youtube-music.desktop\WebView2`. Cookies and local storage persist across launches; this is not an incognito profile. Session lifetime still depends on Google, cookies, account policies and WebView2. The profile is separate from Edge/Chrome. Signing in externally does **not** transfer a session into this app. To reset the app profile, quit first, then delete only that app's `WebView2` directory. Uninstall may offer to remove application data; retain it if you want to keep the profile.

## Google sign-in limitation

Google may reject embedded browsers with “This browser or app may not be secure” or an equivalent error. This app does not spoof a browser identity, collect credentials, extract browser cookies, or bypass that restriction. Use the browser fallback when blocked. Known rejected/denied Google sign-in navigation also opens the native help dialog; the menu remains available for other error formats. Google describes its [embedded-webview OAuth restrictions](https://developers.googleblog.com/upcoming-security-changes-to-googles-oauth-20-authorization-endpoint-in-embedded-webviews/).

Some functionality requires an account, Premium, or an eligible region. Ads and service restrictions remain those of the official site. No downloads, ad blocking, custom playback engine or YouTube API keys are provided.

## Windows prerequisites

- Windows 11 x64.
- [Evergreen WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). The NSIS installer uses Tauri's download-bootstrapper mode to install the runtime if absent; that case requires internet access.
- To build: [Rust via rustup](https://rustup.rs/), including the `x86_64-pc-windows-msvc` target, rustfmt and Clippy. `rust-toolchain.toml` pins the tested toolchain.
- Visual Studio 2022 or newer Build Tools with **Desktop development with C++**, MSVC x64 tools and a Windows 10/11 SDK. Tauri's [Windows prerequisites](https://v2.tauri.app/start/prerequisites/) provide setup details.
- Tauri CLI 2.12.4: `cargo install tauri-cli --version 2.12.4 --locked`.
- Internet access for crates and NSIS packaging tools on the first build. Node.js is not required; there is no JavaScript application bundle.

## Build and checks

From the repository root in PowerShell:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo tauri build --target x86_64-pc-windows-msvc --bundles nsis -- --locked
```

Or run `./scripts/build.ps1`, which performs those checks and copies the installer to `outputs/`.

Installer: `target/x86_64-pc-windows-msvc/release/bundle/nsis/Femboy YouTube Music_<version>_x64-setup.exe`.

Delivery copy: `outputs/Femboy YouTube Music_<version>_x64-setup.exe`.

For development, `cargo tauri dev`. The release build disables devtools. The installer is per-user (no application-install elevation), and not code-signed (updates are verified with the updater's own signature instead). An unsigned installer may be identified as an unknown publisher by Windows. Downloaded WebView2 installation follows Microsoft's runtime installer behavior.

If the default NSIS compiler fails to launch with Windows error 623 (DLL relocation), run `. ./scripts/use-x64-nsis.ps1` in the same PowerShell session, then rerun the build. This optional workaround downloads a checksum-pinned x64 NSIS compiler from the official electron-builder toolset and replaces only the workspace's compiler cache. It does not add Electron to the app. The installer still contains an x64 Rust application; NSIS's installer stub is x86, as is normal for Tauri NSIS packages.

## Security boundaries

- `app.security.capabilities` is explicitly empty, no capability files are provided, no custom invoke handler is registered, and `withGlobalTauri` is false. Remote documents receive **no native command/plugin permissions**. The framework's internal JS may still be present; removing the public global alone is not the security boundary. Tauri's [capability ACL](https://v2.tauri.app/security/capabilities/) is the boundary.
- Single-instance, dialog and updater plugins are used only from native Rust. No shell, filesystem, opener, HTTP or other privileged plugin permissions are granted to any page.
- Top-level HTTPS navigation is limited to exact `music.youtube.com`, `accounts.google.com`, `accounts.youtube.com`, `consent.google.com` and `consent.youtube.com` origins on port 443, plus the official site's observed `www.youtube.com/signin` callback path. Other YouTube pages and all unrelated HTTP(S) destinations go to the default browser. Credentials in URLs and non-web schemes are blocked. Site resources/CDNs are handled normally by WebView2.
- Popups cannot create a new unrestricted window. Related site/auth popups navigate the existing window; unrelated HTTP(S) popups use the browser. Embedded downloads are declined.
- There is no injected player script, native media-command bridge, remote debugging switch, certificate bypass or WebView2 feature-flag override in the shipped app. The tray menu remains usable independently of page JavaScript.
- The local bundled fallback document has a restrictive CSP. It does not change or replace Google's own remote-page CSP.

## Media keys and Windows now-playing

The app relies on WebView2 and the official site's Media Session API implementation. It does not register global shortcuts or publish a second System Media Transport Controls (SMTC) session, avoiding competing playback handlers. Play/pause, next/previous, metadata and artwork can vary with WebView2, the site, account state, active media session and other playing apps. Hiding the window does not suspend or destroy the webview; sleep, OS resource pressure and site policies can still interrupt playback.

Microsoft documents a hardware-media-key feature flag but [advises against shipping browser flags in production](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags). This project retains the runtime defaults. Actual observations and unperformed checks are recorded in `VALIDATION.md`; do not infer successful media integration from API availability alone.

## Manual release checklist

1. Install the x64 NSIS package and launch it. Check the official site renders.
2. Play a track with an account/region that allows it; verify audible playback and advancing progress. Pause/resume, search, and open a playlist.
3. Close the window while playing. Confirm audio/progress continues and **Show** restores the same player. Repeat with a second launch and a minimized window. Confirm only one host process remains.
4. Check Windows now-playing title/artist/artwork and play/pause/next/previous media keys while visible and hidden. Test while another media app is open too.
5. Open native sign-in help and its browser button. Check an unrelated external link opens in the default browser, and no unrestricted popup appears.
6. Quit through the tray; verify the process exits and audio stops. Relaunch and check session persistence if signed in.
7. Test a clean machine without WebView2, installer upgrade/uninstall, and an offline launch separately before broader distribution.
