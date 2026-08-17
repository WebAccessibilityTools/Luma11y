// =============================================================================
// lib.rs - Backend Tauri avec store réactif
// lib.rs - Tauri backend with reactive store
// =============================================================================

// Import de Mutex pour la synchronisation thread-safe
// Import Mutex for thread-safe synchronization
use std::sync::Mutex;
use tauri::Manager;
use tauri::WebviewWindowBuilder;
use tauri::WebviewUrl;

// =============================================================================
// MODULES
// =============================================================================

/// Configuration partagée (constantes)
/// Shared configuration (constants)
mod config;

/// Module du color picker (code commun et implémentations par plateforme)
/// Color picker module (common code and platform implementations)
mod picker;

/// Gestion du store et des commandes associées
/// Store management and associated commands
mod store;

/// Fonctions de manipulation de couleurs
/// Color manipulation functions
mod color;

/// Ratio de contraste WCAG 2.2
/// WCAG 2.2 contrast ratio
mod wcag_contrast;

/// Noms de couleurs CSS (W3C CSS Color Module Level 4)
/// CSS named colors (W3C CSS Color Module Level 4)
mod color_names;

/// Gestion des profils ICC
/// ICC profile management
mod icc;

/// Internationalisation des menus
/// Menu internationalization
mod i18n;

/// Tables de traduction par langue
/// Per-language translation tables
mod lang;

/// Vérification des permissions système (capture d'écran macOS)
/// System permission checks (macOS screen capture)
mod permissions;

// =============================================================================
// INITIALISATION
// INITIALIZATION
// =============================================================================
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Import pour le système de menu
// Import for the menu system
use tauri::menu::{CheckMenuItemBuilder, Menu, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder, AboutMetadata};

// Import pour l'émission d'événements
// Import for event emission
use tauri::Emitter;

/// Préfixe utilisé pour les IDs des éléments de menu ICC
/// Prefix used for ICC menu item IDs
const ICC_MENU_PREFIX: &str = "icc_profile_";

/// Convertit un nom de profil en ID de menu
/// Converts a profile name to a menu ID
///
/// # Arguments
/// * `name` - Nom du profil ICC / ICC profile name
///
/// # Returns
/// * ID de menu formaté / Formatted menu ID
fn profile_name_to_menu_id(name: &str) -> String {
    // Concatène le préfixe avec le nom en minuscules et espaces remplacés par underscores
    // Concatenate prefix with lowercase name and spaces replaced by underscores
    format!("{}{}", ICC_MENU_PREFIX, name.to_lowercase().replace(' ', "_"))
}

/// Extrait le nom du profil depuis un ID de menu
/// Extracts profile name from a menu ID
///
/// # Arguments
/// * `menu_id` - ID de l'élément de menu / Menu item ID
///
/// # Returns
/// * Option contenant le nom du profil si trouvé / Option containing profile name if found
fn menu_id_to_profile_name(menu_id: &str) -> Option<String> {
    // Vérifie si l'ID commence par le préfixe ICC
    // Check if ID starts with ICC prefix
    if menu_id.starts_with(ICC_MENU_PREFIX) {
        // Récupère la liste des profils pour trouver le nom exact
        // Get profile list to find exact name
        let profiles = icc::list_icc_profiles();

        // Cherche le profil dont l'ID correspond
        // Find profile whose ID matches
        for profile in profiles {
            // Compare l'ID généré avec l'ID reçu
            // Compare generated ID with received ID
            if profile_name_to_menu_id(&profile.name) == menu_id {
                // Retourne le nom du profil
                // Return profile name
                return Some(profile.name);
            }
        }
    }

    // Aucun profil trouvé
    // No profile found
    None
}

/// Crée le sous-menu ICC avec tous les profils disponibles
/// Creates the ICC submenu with all available profiles
///
/// # Arguments
/// * `app` - Handle de l'application Tauri / Tauri application handle
/// * `locale` - Locale courante / Current locale
///
/// # Returns
/// * `Result<Submenu<tauri::Wry>, tauri::Error>` - Le sous-menu ICC créé
fn create_icc_submenu<R: tauri::Runtime>(app: &tauri::AppHandle<R>, locale: &str) -> Result<Submenu<R>, tauri::Error> {
    // Récupère la liste des profils ICC disponibles sur le système
    // Get the list of ICC profiles available on the system
    let profiles = icc::list_icc_profiles();

    // Crée le constructeur du sous-menu ICC
    // Create the ICC submenu builder
    let mut icc_submenu_builder = SubmenuBuilder::new(app, i18n::menu_t(locale, "colour_profiles"));

    // Itère sur chaque profil pour créer un élément de menu
    // Iterate over each profile to create a menu item
    for profile in &profiles {
        // Génère un ID unique pour l'élément de menu
        // Generate a unique ID for the menu item
        let menu_id = profile_name_to_menu_id(&profile.name);

        // Crée un élément de menu avec case à cocher
        // Create a check menu item
        let menu_item = CheckMenuItemBuilder::with_id(menu_id, &profile.name)
            // Coche l'élément si c'est le profil actuel
            // Check item if it's the current profile
            .checked(profile.is_current)
            // Construit l'élément de menu
            // Build the menu item
            .build(app)?;

        // Ajoute l'élément au sous-menu
        // Add item to submenu
        icc_submenu_builder = icc_submenu_builder.item(&menu_item);
    }

    // Log le nombre de profils chargés
    // Log the number of loaded profiles
    println!("Loaded {} ICC profiles into menu", profiles.len());

    // Construit et retourne le sous-menu ICC
    // Build and return the ICC submenu
    icc_submenu_builder.build()
}

/// Construit et applique le menu complet de l'application
/// Builds and applies the full application menu
///
/// # Arguments
/// * `app` - Handle de l'application Tauri / Tauri application handle
/// * `locale` - Locale courante / Current locale
fn rebuild_menu(app: &tauri::AppHandle, locale: &str) -> Result<(), tauri::Error> {
    // === MENU APPLICATION (premier menu sur macOS) ===
    // === APPLICATION MENU (first menu on macOS) ===
    // Crée l'élément "À propos" avec métadonnées / Create "About" item with metadata
    let about = PredefinedMenuItem::about(
        app,
        Some(i18n::menu_t(locale, "about")), // Titre / Title
        Some(AboutMetadata {
            name: Some("Luma11y".to_string()),    // Nom de l'app / App name
            version: Some("1.0.0".to_string()),           // Version / Version
            copyright: Some("xxx Licence".to_string()), // Copyright / Copyright
            authors: Some(vec!["Cédric Trévisan".to_string()]), // Auteurs / Authors
            ..Default::default()                          // Autres champs par défaut / Other fields default
        }),
    )?;

    // Élément Settings avec raccourci Cmd+, / Settings item with Cmd+, shortcut
    let settings_item = MenuItemBuilder::with_id("settings", i18n::menu_t(locale, "settings"))
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    // Éléments standards du menu Application / Standard Application menu items
    // Note : hide/hide_others/show_all sont réservés à macOS car Windows/Linux n'ont
    // pas de menubar persistante pour réafficher l'application.
    // Note: hide/hide_others/show_all are macOS-only since Windows/Linux have no
    // persistent menubar to bring the app back.
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = PredefinedMenuItem::quit(app, Some(i18n::menu_t(locale, "quit")))?;

    // === SOUS-MENU APPARENCE ===
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

    // === SOUS-MENU STYLE ===
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

    // Construit le sous-menu Application / Build Application submenu
    // Sur macOS on ajoute hide/hide_others/show_all ; ailleurs on les omet.
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

    // === MENU ÉDITION ===
    // === EDIT MENU ===
    // Items natifs requis sur macOS pour que ⌘C/⌘V/⌘X/⌘A soient routés vers la
    // webview (sans eux, l'OS n'associe pas la combinaison à une action).
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

    // Ajoute les modèles de copie avec leurs raccourcis
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

    // === SOUS-MENU FENÊTRE ===
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
        // Crée le sous-menu ICC avec les profils
        // Create the ICC submenu with profiles
        let icc_submenu = create_icc_submenu(app, locale)?;

        // Crée le menu de l'application
        // Get the application menu
        let root_menu = Menu::with_items(app, &[
            &app_menu,
            &edit_submenu,
            &icc_submenu,
            &window_submenu,
        ])?;
        // Applique le menu à l'application
        // Apply menu to the application
        app.set_menu(root_menu)?;
    }

    // Sur Windows/Linux, pas de menu natif
    // On Windows/Linux, no native menu
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        // silence le warning à propos des vars non utilisées
        // silence the unused warnings
        let _ = app_menu;
        let _ = edit_submenu;
        let _ = window_submenu;
    }

    Ok(())
}

/// Commande Tauri pour mettre à jour les modèles de copie depuis le frontend
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

/// Ouvre ou focus la fenêtre Settings, avec config plateforme-spécifique.
/// Opens or focuses the Settings window with platform-specific config.
fn open_settings_window_impl(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_focus();
        return;
    }

    let settings_title = {
        let state = app.state::<store::AppState>();
        let locale = state.locale.lock().unwrap();
        i18n::menu_t(&locale, "settings_title").to_string()
    };

    // Hérite l'état always-on-top de l'app : si la main est pinned,
    // Settings doit l'être aussi sinon elle s'ouvre derrière la principale
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
    .inner_size(500.0, 700.0)
    .resizable(true)
    .maximizable(false)
    .min_inner_size(400.0, 700.0)
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
        // visible(false) : on masque la fenêtre pendant son init pour éviter
        // le flash de re-layout
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

/// Commande Tauri pour ouvrir la fenêtre Settings depuis le frontend (menu toolbar).
/// Tauri command to open the Settings window from the frontend (toolbar menu).
#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) {
    let inner = app.clone();
    let _ = app.run_on_main_thread(move || {
        open_settings_window_impl(&inner);
    });
}

/// Applique l'état always-on-top.
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

/// Commande Tauri pour basculer always-on-top depuis le frontend
/// Tauri command to toggle always-on-top from frontend
#[tauri::command]
fn set_always_on_top(app: tauri::AppHandle, value: bool) {
    apply_always_on_top(&app, value);
}

/// Commande Tauri pour synchroniser le mode d'apparence depuis le frontend
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

/// Commande Tauri pour synchroniser le thème de style depuis le frontend
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

/// Commande Tauri pour changer la locale depuis le frontend
/// Tauri command to change locale from frontend
#[tauri::command]
fn set_locale(app: tauri::AppHandle, state: tauri::State<store::AppState>, locale: String) {
    // Met à jour la locale dans l'état
    // Update locale in state
    {
        let mut current_locale = state.locale.lock().unwrap();
        if *current_locale == locale {
            return;
        }
        *current_locale = locale.clone();
    }

    // Reconstruit le menu avec la nouvelle locale
    // Rebuild menu with new locale
    let _ = rebuild_menu(&app, &locale);

    // Émet l'événement pour notifier toutes les fenêtres
    // Emit event to notify all windows
    let _ = app.emit("locale-changed", &locale);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialise le plugin OS pour la détection de locale
        // Initialize OS plugin for locale detection
        .plugin(tauri_plugin_os::init())
        // Plugin pour les raccourcis clavier globaux (système-wide)
        // Plugin for global (system-wide) keyboard shortcuts
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Initialise l'état global du color store
        // Initialize global color store state
        .manage(store::AppState {
            store: Mutex::new(store::ResultStore::default()),
            locale: Mutex::new("en".to_string()),
            templates: Mutex::new(Vec::new()),
            appearance: Mutex::new("auto".to_string()),
            style_theme: Mutex::new("modern".to_string()),
            always_on_top: Mutex::new(false),
        })
        // Configure le menu de l'application
        // Configure the application menu
        .setup(|app| {
            // Récupère le handle de l'application
            // Get the application handle
            let handle = app.handle();

            // Construit le menu initial avec la locale par défaut
            // Build initial menu with default locale
            rebuild_menu(handle, "en")?;

            // Retourne Ok pour indiquer le succès
            // Return Ok to indicate success
            Ok(())
        })
        // Gestionnaire d'événements de menu
        // Menu event handler
        .on_menu_event(|app, event| {
            // Récupère l'ID de l'élément de menu cliqué
            // Get the clicked menu item ID
            let menu_id = event.id().as_ref();

            // === Gestion du changement de langue ===
            // === Language change handling ===
            // Gestion des modèles de copie
            // Copy template handling
            if menu_id.starts_with("copy_template_") {
                if let Ok(index) = menu_id["copy_template_".len()..].parse::<usize>() {
                    let _ = app.emit("copy-template", index);
                }
                return;
            }

            match menu_id {
                "settings" => {
                    open_settings_window_impl(app);
                    return;
                }
                "appearance_auto" | "appearance_light" | "appearance_dark" => {
                    let mode = match menu_id {
                        "appearance_light" => "light",
                        "appearance_dark" => "dark",
                        _ => "auto",
                    };

                    // Met à jour le mode dans l'état et reconstruit le menu
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

                    // Met à jour le style theme dans l'état et reconstruit le menu
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
                    // Bascule via le helper partagé
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

            // Tente d'extraire le nom du profil depuis l'ID
            // Try to extract profile name from ID
            if let Some(profile_name) = menu_id_to_profile_name(menu_id) {
                // Met à jour le profil sélectionné dans le backend
                // Update the selected profile in the backend
                let _ = icc::select_icc_profile(profile_name.clone());

                // Récupère tous les profils disponibles
                // Get all available profiles
                let profiles = icc::list_icc_profiles();

                // Déselectionne tous les profils d'abord
                // Deselect all profiles first
                for profile in &profiles {
                    let id = profile_name_to_menu_id(&profile.name);

                    // Essaie de trouver l'item dans le menu principal
                    // Try to find item in main menu
                    if let Some(menu) = app.menu() {
                        // Cherche d'abord dans le menu principal
                        // First search in main menu
                        if let Some(item) = menu.get(&id) {
                            if let Some(check_item) = item.as_check_menuitem() {
                                let _ = check_item.set_checked(false);
                            }
                        }
                        // Sinon, cherche dans tous les items du menu récursivement
                        // Otherwise, search recursively in all menu items
                        else if let Ok(items) = menu.items() {
                            for menu_item in items {
                                // Si c'est un sous-menu, cherche dedans
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

                // Maintenant, coche uniquement le profil sélectionné
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

                // Émet un événement pour notifier le frontend du changement
                // Emit event to notify frontend of the change
                let _ = app.emit("icc-profile-changed", &profile_name);

                // Log le changement de profil
                // Log profile change
                println!("ICC Profile changed via menu: {}", profile_name);
            }
        })
        // Enregistre les commandes Tauri
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
            permissions::check_screen_recording_permission,
            permissions::request_screen_recording_permission,
            permissions::open_screen_recording_settings,
        ])
        // Lance l'application Tauri
        // Run the Tauri application
        .run(tauri::generate_context!())
        // Affiche un message d'erreur si le lancement échoue
        // Display error message if launch fails
        .expect("error while running tauri application");
}
