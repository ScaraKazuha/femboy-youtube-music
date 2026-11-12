#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod policy;

use policy::{destination, Destination};
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::NewWindowResponse,
    window::Color,
    AppHandle, Manager, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::UpdaterExt;

const HOME: &str = "https://music.youtube.com";
const TITLE: &str = "Femboy YouTube Music";
const SPLASH_TIME: std::time::Duration = std::time::Duration::from_millis(2500);

fn restore(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn open_browser(app: &AppHandle, url: &url::Url) {
    // Never send file:, javascript:, custom protocols, or credential-bearing
    // addresses to the Windows shell. No shell command is constructed here.
    if destination(url) != Destination::Blocked {
        if let Err(error) = webbrowser::open(url.as_str()) {
            app.dialog()
                .message(format!(
                    "Could not open your browser: {error}\nOpen {HOME} manually."
                ))
                .title(TITLE)
                .show(|_| {});
        }
    }
}

fn sign_in_help(app: &AppHandle) {
    let handle = app.clone();
    app.dialog()
        .message("This is an unofficial app, not affiliated with Google or YouTube.\n\nGoogle may block sign-in in embedded browsers with a 'browser or app may not be secure' message. This app cannot bypass that restriction. Use YouTube Music in your default browser if sign-in is blocked.\n\nThe app keeps its own WebView2 profile. Signing in in your browser does not sign you in here. Accounts, subscriptions, ads, regional availability and playback restrictions are controlled by YouTube.\n\nClosing the window hides it to the tray and keeps the player alive. Choose Quit to stop it.")
        .title("About and sign-in help — Femboy YouTube Music")
        .buttons(MessageDialogButtons::OkCancelCustom("Open in browser".into(), "Stay in app".into()))
        .show(move |open| {
            if open {
                open_browser(&handle, &HOME.parse().expect("static HTTPS URL"));
            }
        });
}

fn notify(app: &AppHandle, message: impl Into<String>) {
    app.dialog().message(message).title(TITLE).show(|_| {});
}

/// Checks the signed GitHub release feed. Runs entirely in native code: the
/// remote page gets no updater permission. A silent check (at startup) only
/// speaks up when an update exists; a manual one also reports "up to date"
/// and errors.
fn check_for_updates(app: AppHandle, manual: bool) {
    tauri::async_runtime::spawn(async move {
        let checked = match app.updater() {
            Ok(updater) => updater.check().await,
            Err(error) => Err(error),
        };
        match checked {
            Ok(Some(update)) => {
                let message = format!(
                    "Version {} is available (installed: {}).\n\nInstall it now? Playback stops and the app restarts.",
                    update.version, update.current_version
                );
                let handle = app.clone();
                app.dialog()
                    .message(message)
                    .title(TITLE)
                    .buttons(MessageDialogButtons::OkCancelCustom(
                        "Update now".into(),
                        "Later".into(),
                    ))
                    .show(move |install| {
                        if !install {
                            return;
                        }
                        tauri::async_runtime::spawn(async move {
                            match update.download_and_install(|_, _| {}, || {}).await {
                                Ok(()) => handle.restart(),
                                Err(error) => notify(&handle, format!("Update failed: {error}")),
                            }
                        });
                    });
            }
            Ok(None) if manual => notify(&app, "You have the latest version."),
            Err(error) if manual => notify(&app, format!("Could not check for updates: {error}")),
            _ => {}
        }
    });
}

fn main() {
    tauri::Builder::default()
        // Register first, so a second process never creates another profile/window.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| restore(app)))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // No invoke_handler, JS-facing capabilities, or remote plugin grants.
        .setup(|app| {
            let profile = app.path().app_local_data_dir()?.join("WebView2");
            std::fs::create_dir_all(&profile)?;
            let navigation_app = app.handle().clone();
            let popup_app = app.handle().clone();
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::External(HOME.parse()?))
                    .title(TITLE)
                    // Stays hidden behind the splash window for a moment.
                    .visible(false)
                    .inner_size(1180.0, 800.0)
                    .min_inner_size(640.0, 480.0)
                    .data_directory(profile)
                    .incognito(false)
                    .devtools(cfg!(debug_assertions))
                    // Dark frame and backdrop like the site; no native menu bar.
                    .theme(Some(Theme::Dark))
                    .background_color(Color(3, 3, 3, 255))
                    .on_navigation(move |url| {
                        if policy::sign_in_rejected(url) {
                            sign_in_help(&navigation_app);
                        }
                        match destination(url) {
                            Destination::Embedded => true,
                            Destination::Browser => {
                                open_browser(&navigation_app, url);
                                false
                            }
                            Destination::Blocked => false,
                        }
                    })
                    .on_new_window(move |url, _| {
                        match destination(&url) {
                            Destination::Embedded => {
                                // Keep authentication within the same persistent profile.
                                if let Some(window) = popup_app.get_webview_window("main") {
                                    let _ = window.navigate(url);
                                }
                            }
                            Destination::Browser => open_browser(&popup_app, &url),
                            Destination::Blocked => {}
                        }
                        NewWindowResponse::Deny
                    })
                    .on_download(|_, _| false)
                    .build()?;
            // Local, permission-less splash page shown while the site loads.
            let splash =
                WebviewWindowBuilder::new(app, "splash", WebviewUrl::App("splash.html".into()))
                    .title(TITLE)
                    .inner_size(400.0, 470.0)
                    .center()
                    .resizable(false)
                    .decorations(false)
                    .skip_taskbar(true)
                    .theme(Some(Theme::Dark))
                    .background_color(Color(3, 3, 3, 255))
                    .build()?;
            std::thread::spawn({
                let window = window.clone();
                move || {
                    std::thread::sleep(SPLASH_TIME);
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = splash.close();
                }
            });
            window.on_window_event({
                let window = window.clone();
                move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        // Hide, never destroy/navigate/suspend the playing webview.
                        let _ = window.hide();
                    }
                }
            });
            let tray_menu = MenuBuilder::new(app)
                .text("show", "Show")
                .text("home", "YouTube Music home")
                .text("browser", "Open in browser")
                .separator()
                .text("update", "Check for updates")
                .text("help", "About / sign-in help")
                .separator()
                .text("quit", "Quit")
                .build()?;
            TrayIconBuilder::with_id("player")
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/icon.png"
                ))?)
                .tooltip("Femboy YouTube Music — right-click for Show / Quit")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        restore(tray.app_handle());
                    }
                })
                .build(app)?;
            check_for_updates(app.handle().clone(), false);
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => restore(app),
            "quit" => app.exit(0),
            "help" => sign_in_help(app),
            "update" => check_for_updates(app.clone(), true),
            "browser" => open_browser(app, &HOME.parse().expect("static HTTPS URL")),
            "home" => {
                restore(app);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.navigate(HOME.parse().expect("static HTTPS URL"));
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("Unable to start Femboy YouTube Music");
}
