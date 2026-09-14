// =============================================================================
// lib.rs - Tauri backend with reactive store
// =============================================================================

// Import Mutex for thread-safe synchronization
use std::sync::Mutex;
use tauri::Manager;
use tauri::WebviewWindowBuilder;
use tauri::WebviewUrl;

// =============================================================================
// MODULES
// =============================================================================

/// Shared configuration (constants)
mod config;

/// Color picker module (common code and platform implementations)
mod picker;

/// Store management and associated commands
mod store;

/// Color manipulation functions
mod color;

/// WCAG 2.2 contrast ratio
mod wcag_contrast;

/// CSS named colors (W3C CSS Color Module Level 4)
mod color_names;

/// ICC profile management
mod icc;

/// Menu internationalization
mod i18n;

/// Per-language translation tables
mod lang;

/// System permission checks (macOS screen capture)
mod permissions;

// =============================================================================
// INITIALIZATION
// =============================================================================
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Import for the menu system
use tauri::menu::{CheckMenuItemBuilder, Menu, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder};

// Import for event emission
use tauri::Emitter;

/// Prefix used for ICC menu item IDs
const ICC_MENU_PREFIX: &str = "icc_profile_";

/// Converts a profile name to a menu ID
///
/// # Arguments
/// * `name` - ICC profile name
///
/// # Returns
/// * Formatted menu ID
fn profile_name_to_menu_id(name: &str) -> String {
    // Concatenate prefix with lowercase name and spaces replaced by underscores
    format!("{}{}", ICC_MENU_PREFIX, name.to_lowercase().replace(' ', "_"))
}

/// Extracts profile name from a menu ID
///
/// # Arguments
/// * `menu_id` - Menu item ID
///
/// # Returns
/// * Option containing profile name if found
fn menu_id_to_profile_name(menu_id: &str) -> Option<String> {
    // Check if ID starts with ICC prefix
    if menu_id.starts_with(ICC_MENU_PREFIX) {
        // Get profile list to find exact name
        let profiles = icc::list_icc_profiles();

        // Find profile whose ID matches
        for profile in profiles {
            // Compare generated ID with received ID
            if profile_name_to_menu_id(&profile.name) == menu_id {
                // Return profile name
                return Some(profile.name);
            }
        }
    }

    // No profile found
    None
}

/// Creates the ICC submenu with all available profiles
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `locale` - Current locale
///
/// # Returns
/// * `Result<Submenu<tauri::Wry>, tauri::Error>` - The created ICC submenu
fn create_icc_submenu<R: tauri::Runtime>(app: &tauri::AppHandle<R>, locale: &str) -> Result<Submenu<R>, tauri::Error> {
    // Get the list of ICC profiles available on the system
    let profiles = icc::list_icc_profiles();

    // Create the ICC submenu builder
    let mut icc_submenu_builder = SubmenuBuilder::new(app, i18n::menu_t(locale, "colour_profiles"));

    // Iterate over each profile to create a menu item
    for profile in &profiles {
        // Generate a unique ID for the menu item
        let menu_id = profile_name_to_menu_id(&profile.name);

        // Create a check menu item
        let menu_item = CheckMenuItemBuilder::with_id(menu_id, &profile.name)
            // Check item if it's the current profile
            .checked(profile.is_current)
            // Build the menu item
            .build(app)?;

        // Add item to submenu
        icc_submenu_builder = icc_submenu_builder.item(&menu_item);
    }

    // Log the number of loaded profiles
    println!("Loaded {} ICC profiles into menu", profiles.len());

    // Build and return the ICC submenu
    icc_submenu_builder.build()
}

/// Builds and applies the full application menu
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `locale` - Current locale
fn rebuild_menu(app: &tauri::AppHandle, locale: &str) -> Result<(), tauri::Error> {
    // === APPLICATION MENU (first menu on macOS) ===
    // "About" item: opens the dedicated Settings tab.
    let about = MenuItemBuilder::with_id("about", i18n::menu_t(locale, "about")).build(app)?;

    // Settings item with Cmd+, shortcut
    let settings_item = MenuItemBuilder::with_id("settings", i18n::menu_t(locale, "settings"))
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    // Standard Application menu items
    // Note: hide/hide_others/show_all are macOS-only since Windows/Linux have no
    // persistent menubar to bring the app back.
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = PredefinedMenuItem::quit(app, Some(i18n::menu_t(locale, "quit")))?;

    // === APPEARANCE SUBMENU ===
    let appearance = {
        let state = app.state::<store::AppState>();
        let value = state.appearance.lock().unwrap().clone();
        value
    };
    let appearance_auto = CheckMenuItemBuilder::with_id("appearance_auto", i18n::menu_t(locale, "appearance_auto"))
        .checked(appearance == "auto")
        .build(app)?;
    let appearance_light = CheckMenuItemBuilder::with_id("appearance_light", i18n::menu_t(locale, "appearance_light"))
        .checked(appearance == "light")
        .build(app)?;
    let appearance_dark = CheckMenuItemBuilder::with_id("appearance_dark", i18n::menu_t(locale, "appearance_dark"))
        .checked(appearance == "dark")
        .build(app)?;

    let appearance_submenu = SubmenuBuilder::new(app, i18n::menu_t(locale, "appearance"))
        .item(&appearance_auto)
        .item(&appearance_light)
        .item(&appearance_dark)
        .build()?;

    // === STYLE SUBMENU ===
    let style_theme = {
        let state = app.state::<store::AppState>();
        let value = state.style_theme.lock().unwrap().clone();
        value
    };
    let style_modern = CheckMenuItemBuilder::with_id("style_modern", i18n::menu_t(locale, "style_modern"))
        .checked(style_theme == "modern")
        .build(app)?;
    let style_classic = CheckMenuItemBuilder::with_id("style_classic", i18n::menu_t(locale, "style_classic"))
        .checked(style_theme == "classic")
        .build(app)?;

    let style_submenu = SubmenuBuilder::new(app, i18n::menu_t(locale, "style_theme"))
        .item(&style_modern)
        .item(&style_classic)
        .build()?;

    // Build Application submenu
    // On macOS we include hide/hide_others/show_all; elsewhere we omit them.
    let app_menu;
    #[cfg(target_os = "macos")]
    {
        let separator1 = PredefinedMenuItem::separator(app)?;
        let hide = PredefinedMenuItem::hide(app, Some(i18n::menu_t(locale, "hide")))?;
        let hide_others = PredefinedMenuItem::hide_others(app, Some(i18n::menu_t(locale, "hide_others")))?;
        let show_all = PredefinedMenuItem::show_all(app, Some(i18n::menu_t(locale, "show_all")))?;

        app_menu = Submenu::with_items(
            app,
            "Luma11y",
            true,
            &[
                &about,
                &PredefinedMenuItem::separator(app)?,
                &settings_item,
                &separator1,
                &hide,
                &hide_others,
                &show_all,
                &separator2,
                &appearance_submenu,
                &style_submenu,
                &PredefinedMenuItem::separator(app)?,
                &quit,
            ],
        )?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        app_menu = Submenu::with_items(
            app,
            "Luma11y",
            true,
            &[
                &about,
                &PredefinedMenuItem::separator(app)?,
                &settings_item,
                &separator2,
                &appearance_submenu,
                &style_submenu,
                &PredefinedMenuItem::separator(app)?,
                &quit,
            ],
        )?;
    }

    // === EDIT MENU ===
    // Native items required on macOS so ⌘C/⌘V/⌘X/⌘A are forwarded to the
    // webview (without them, the OS doesn't associate the combo to an action).
    let edit_undo = PredefinedMenuItem::undo(app, None)?;
    let edit_redo = PredefinedMenuItem::redo(app, None)?;
    let edit_sep = PredefinedMenuItem::separator(app)?;
    let edit_cut = PredefinedMenuItem::cut(app, None)?;
    let edit_copy = PredefinedMenuItem::copy(app, None)?;
    let edit_paste = PredefinedMenuItem::paste(app, None)?;
    let edit_select_all = PredefinedMenuItem::select_all(app, None)?;

    let mut edit_builder = SubmenuBuilder::new(app, i18n::menu_t(locale, "edit"))
        .item(&edit_undo)
        .item(&edit_redo)
        .item(&edit_sep)
        .item(&edit_cut)
        .item(&edit_copy)
        .item(&edit_paste)
        .item(&edit_select_all);

    // Add copy templates with their shortcuts
    let state = app.state::<store::AppState>();
    let templates = state.templates.lock().unwrap().clone();

    if !templates.is_empty() {
        let tpl_sep = PredefinedMenuItem::separator(app)?;
        edit_builder = edit_builder.item(&tpl_sep);

        let mut tpl_submenu_builder = SubmenuBuilder::new(app, i18n::menu_t(locale, "copy_templates"));

        for (i, tpl) in templates.iter().enumerate() {
            let menu_id = format!("copy_template_{}", i);
            let name = if tpl.name.is_empty() { format!("Template {}", i + 1) } else { tpl.name.clone() };

            let item = if !tpl.shortcut.is_empty() {
                match MenuItemBuilder::with_id(&menu_id, &name)
                    .accelerator(&tpl.shortcut)
                    .build(app) {
                    Ok(item) => item,
                    Err(_) => MenuItemBuilder::with_id(&menu_id, &name).build(app)?,
                }
            } else {
                MenuItemBuilder::with_id(&menu_id, &name).build(app)?
            };

            tpl_submenu_builder = tpl_submenu_builder.item(&item);
        }

        let tpl_submenu = tpl_submenu_builder.build()?;
        edit_builder = edit_builder.item(&tpl_submenu);
    }

    let edit_submenu = edit_builder.build()?;

    // === WINDOW SUBMENU ===
    let win_minimize = PredefinedMenuItem::minimize(app, Some(i18n::menu_t(locale, "minimize")))?;
    let win_close = PredefinedMenuItem::close_window(app, Some(i18n::menu_t(locale, "close_window")))?;
    let win_sep = PredefinedMenuItem::separator(app)?;
    let always_on_top = {
        let state = app.state::<store::AppState>();
        let value = *state.always_on_top.lock().unwrap();
        value
    };
    let win_always_on_top = CheckMenuItemBuilder::with_id("always_on_top", i18n::menu_t(locale, "always_on_top"))
        .checked(always_on_top)
        .build(app)?;

    let window_submenu = SubmenuBuilder::new(app, i18n::menu_t(locale, "window"))
        .item(&win_minimize)
        .item(&win_sep)
        .item(&win_always_on_top)
        .item(&win_close)
        .build()?;

    #[cfg(target_os = "macos")]
    {
        // Create the ICC submenu with profiles
        let icc_submenu = create_icc_submenu(app, locale)?;

        // Get the application menu
        let root_menu = Menu::with_items(app, &[
            &app_menu,
            &edit_submenu,
            &icc_submenu,
            &window_submenu,
        ])?;
        // Apply menu to the application
        app.set_menu(root_menu)?;
    }

    // On Windows/Linux, no native menu
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        // silence the unused warnings
        let _ = app_menu;
        let _ = edit_submenu;
        let _ = window_submenu;
    }

    Ok(())
}

/// Tauri command to update copy templates from frontend
#[tauri::command]
fn set_copy_templates(app: tauri::AppHandle, state: tauri::State<store::AppState>, templates: Vec<store::CopyTemplate>) {
    {
        let mut tpls = state.templates.lock().unwrap();
        *tpls = templates;
    }
    let locale = state.locale.lock().unwrap().clone();
    let _ = rebuild_menu(&app, &locale);
}

/// Opens or focuses the Settings window with platform-specific config.
fn open_settings_window_impl(app: &tauri::AppHandle, tab: &str) {
    // Remember the target tab: the window reads it on init via
    // `get_settings_initial_tab`.
    {
        let state = app.state::<store::AppState>();
        *state.settings_tab.lock().unwrap() = tab.to_string();
    }

    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_focus();
        // Already open: tell the window to switch tab.
        let _ = window.emit("settings-navigate", tab);
        return;
    }

    let settings_title = {
        let state = app.state::<store::AppState>();
        let locale = state.locale.lock().unwrap();
        i18n::menu_t(&locale, "settings_title").to_string()
    };

    // Inherit the always-on-top state: if the main window is pinned,
    // Settings must also be pinned, otherwise it opens behind main
    let always_on_top = {
        let state = app.state::<store::AppState>();
        let v = *state.always_on_top.lock().unwrap();
        v
    };

    let mut builder = WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("settings.html".into()),
    )
    .title(settings_title)
    .inner_size(640.0, 700.0)
    .resizable(true)
    .maximizable(false)
    .min_inner_size(520.0, 700.0)
    .always_on_top(always_on_top)
    .center();

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true);
    }

    #[cfg(not(target_os = "macos"))]
    {
        // visible(false): hide the window during init to avoid flash of
        // re-layout
        builder = builder
            .decorations(false)
            .transparent(true)
            .visible(false);
    }

    if let Ok(window) = builder.build() {
        #[cfg(not(target_os = "macos"))]
        {
            let _ = window.set_decorations(false);
        }
        let _ = window;
    }
}

/// Tauri command to open the Settings window from the frontend (toolbar menu).
#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) {
    let inner = app.clone();
    let _ = app.run_on_main_thread(move || {
        open_settings_window_impl(&inner, "general");
    });
}

/// Returns the tab to activate when the Settings window opens.
#[tauri::command]
fn get_settings_initial_tab(state: tauri::State<store::AppState>) -> String {
    state.settings_tab.lock().unwrap().clone()
}

/// Application metadata for the "About" tab.
#[derive(serde::Serialize)]
struct AppInfo {
    name: String,
    version: String,
    authors: String,
    description: String,
}

/// Returns the app metadata (name, version, authors, description).
#[tauri::command]
fn get_app_info(app: tauri::AppHandle) -> AppInfo {
    let info = app.package_info();
    AppInfo {
        name: info.name.clone(),
        version: info.version.to_string(),
        authors: info.authors.to_string(),
        description: info.description.to_string(),
    }
}

/// Applies always-on-top state
fn apply_always_on_top(app: &tauri::AppHandle, value: bool) {
    let locale = {
        let state = app.state::<store::AppState>();
        {
            let mut current = state.always_on_top.lock().unwrap();
            *current = value;
        }
        let locale = state.locale.lock().unwrap().clone();
        locale
    };

    for (_, window) in app.webview_windows() {
        let _ = window.set_always_on_top(value);
    }

    let _ = rebuild_menu(app, &locale);
    let _ = app.emit("always-on-top-changed", value);
}

/// Tauri command to toggle always-on-top from frontend
#[tauri::command]
fn set_always_on_top(app: tauri::AppHandle, value: bool) {
    apply_always_on_top(&app, value);
}

/// Tauri command to synchronize appearance mode from frontend
#[tauri::command]
fn set_appearance(app: tauri::AppHandle, state: tauri::State<store::AppState>, appearance: String) {
    let locale = {
        let mut current = state.appearance.lock().unwrap();
        if *current == appearance {
            return;
        }
        *current = appearance;
        state.locale.lock().unwrap().clone()
    };
    let _ = rebuild_menu(&app, &locale);
}

/// Tauri command to synchronize style theme from frontend
#[tauri::command]
fn set_style_theme(app: tauri::AppHandle, state: tauri::State<store::AppState>, style: String) {
    let locale = {
        let mut current = state.style_theme.lock().unwrap();
        if *current == style {
            return;
        }
        *current = style;
        state.locale.lock().unwrap().clone()
    };
    let _ = rebuild_menu(&app, &locale);
}

/// Tauri command to change locale from frontend
#[tauri::command]
fn set_locale(app: tauri::AppHandle, state: tauri::State<store::AppState>, locale: String) {
    // Update locale in state
    {
        let mut current_locale = state.locale.lock().unwrap();
        if *current_locale == locale {
            return;
        }
        *current_locale = locale.clone();
    }

    // Rebuild menu with new locale
    let _ = rebuild_menu(&app, &locale);

    // Emit event to notify all windows
    let _ = app.emit("locale-changed", &locale);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialize OS plugin for locale detection
        .plugin(tauri_plugin_os::init())
        // Plugin for global (system-wide) keyboard shortcuts
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Plugin to open URLs/files with the default app
        .plugin(tauri_plugin_opener::init())
        // Initialize global color store state
        .manage(store::AppState {
            store: Mutex::new(store::ResultStore::default()),
            locale: Mutex::new("en".to_string()),
            templates: Mutex::new(Vec::new()),
            appearance: Mutex::new("auto".to_string()),
            style_theme: Mutex::new("modern".to_string()),
            always_on_top: Mutex::new(false),
            settings_tab: Mutex::new("general".to_string()),
        })
        // Configure the application menu
        .setup(|app| {
            // Get the application handle
            let handle = app.handle();

            // Build initial menu with default locale
            rebuild_menu(handle, "en")?;

            // Return Ok to indicate success
            Ok(())
        })
        // Menu event handler
        .on_menu_event(|app, event| {
            // Get the clicked menu item ID
            let menu_id = event.id().as_ref();

            // === Language change handling ===
            // Copy template handling
            if menu_id.starts_with("copy_template_") {
                if let Ok(index) = menu_id["copy_template_".len()..].parse::<usize>() {
                    let _ = app.emit("copy-template", index);
                }
                return;
            }

            match menu_id {
                "settings" => {
                    open_settings_window_impl(app, "general");
                    return;
                }
                "about" => {
                    // Replaces the native "About" dialog with the dedicated tab.
                    open_settings_window_impl(app, "about");
                    return;
                }
                "appearance_auto" | "appearance_light" | "appearance_dark" => {
                    let mode = match menu_id {
                        "appearance_light" => "light",
                        "appearance_dark" => "dark",
                        _ => "auto",
                    };

                    // Update mode in state and rebuild menu
                    let state = app.state::<store::AppState>();
                    let locale = {
                        let mut appearance = state.appearance.lock().unwrap();
                        *appearance = mode.to_string();
                        state.locale.lock().unwrap().clone()
                    };

                    let _ = rebuild_menu(app, &locale);
                    let _ = app.emit("appearance-changed", mode);
                    return;
                }
                "style_modern" | "style_classic" => {
                    let theme = if menu_id == "style_classic" { "classic" } else { "modern" };

                    // Update style theme in state and rebuild menu
                    let state = app.state::<store::AppState>();
                    let locale = {
                        let mut style = state.style_theme.lock().unwrap();
                        *style = theme.to_string();
                        state.locale.lock().unwrap().clone()
                    };

                    let _ = rebuild_menu(app, &locale);
                    let _ = app.emit("style-theme-changed", theme);
                    return;
                }
                "always_on_top" => {
                    // Toggle via shared helper
                    let current = {
                        let state = app.state::<store::AppState>();
                        let v = *state.always_on_top.lock().unwrap();
                        v
                    };
                    apply_always_on_top(app, !current);
                    return;
                }
                _ => {}
            }

            // Try to extract profile name from ID
            if let Some(profile_name) = menu_id_to_profile_name(menu_id) {
                // Update the selected profile in the backend
                let _ = icc::select_icc_profile(profile_name.clone());

                // Get all available profiles
                let profiles = icc::list_icc_profiles();

                // Deselect all profiles first
                for profile in &profiles {
                    let id = profile_name_to_menu_id(&profile.name);

                    // Try to find item in main menu
                    if let Some(menu) = app.menu() {
                        // First search in main menu
                        if let Some(item) = menu.get(&id) {
                            if let Some(check_item) = item.as_check_menuitem() {
                                let _ = check_item.set_checked(false);
                            }
                        }
                        // Otherwise, search recursively in all menu items
                        else if let Ok(items) = menu.items() {
                            for menu_item in items {
                                // If it's a submenu, search inside
                                if let Some(submenu) = menu_item.as_submenu() {
                                    if let Some(subitem) = submenu.get(&id) {
                                        if let Some(check_item) = subitem.as_check_menuitem() {
                                            let _ = check_item.set_checked(false);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Now, check only the selected profile
                let selected_id = profile_name_to_menu_id(&profile_name);
                if let Some(menu) = app.menu() {
                    if let Some(item) = menu.get(&selected_id) {
                        if let Some(check_item) = item.as_check_menuitem() {
                            let _ = check_item.set_checked(true);
                        }
                    } else if let Ok(items) = menu.items() {
                        for menu_item in items {
                            if let Some(submenu) = menu_item.as_submenu() {
                                if let Some(subitem) = submenu.get(&selected_id) {
                                    if let Some(check_item) = subitem.as_check_menuitem() {
                                        let _ = check_item.set_checked(true);
                                    }
                                }
                            }
                        }
                    }
                }

                // Emit event to notify frontend of the change
                let _ = app.emit("icc-profile-changed", &profile_name);

                // Log profile change
                println!("ICC Profile changed via menu: {}", profile_name);
            }
        })
        // Register Tauri commands
        .invoke_handler(tauri::generate_handler![
            store::get_store,
            store::pick_color,
            store::update_store_rgb,
            store::update_store_hsl,
            store::update_store_hsv,
            store::update_store_lab,
            store::update_store_oklch,
            store::update_store_hex,
            store::clear_store,
            store::get_color_name,
            icc::list_icc_profiles,
            icc::select_icc_profile,
            icc::get_selected_icc_profile,
            set_locale,
            set_appearance,
            set_style_theme,
            set_copy_templates,
            set_always_on_top,
            open_settings_window,
            get_settings_initial_tab,
            get_app_info,
            permissions::check_screen_recording_permission,
            permissions::request_screen_recording_permission,
            permissions::open_screen_recording_settings,
        ])
        // Run the Tauri application
        .run(tauri::generate_context!())
        // Display error message if launch fails
        .expect("error while running tauri application");
}
