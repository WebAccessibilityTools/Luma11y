// =============================================================================
// store.rs - Store management module
// =============================================================================

use tauri::{AppHandle, Emitter};
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use crate::config;
use crate::picker;
use crate::color;
use crate::color_names;
use crate::wcag_contrast;

// =============================================================================
// STORE - État global partagé
// STORE - Shared global state
// =============================================================================

/// Structure du store - contient toutes les données réactives
/// Store structure - contains all reactive data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResultStore {
    /// Plateforme actuelle (macos, windows, linux)
    /// Current platform (macos, windows, linux)
    pub platform: &'static str,

    /// Couleur de premier plan au format RGB (r, g, b)
    /// Foreground color in RGB format (r, g, b)
    pub foreground_rgb: (u8, u8, u8),

    /// Chaîne CSS d'affichage du RGB : "rgb(r, g, b)" ou "rgb(r g b / NN%)".
    /// CSS display string for RGB: "rgb(r, g, b)" or "rgb(r g b / NN%)".
    pub foreground_rgb_css: String,

    /// Couleur de premier plan au format hexadécimal
    /// Foreground color in hexadecimal format
    pub foreground_hex: String,

    /// Couleur de premier plan au format HSL "hsl(h, s%, l%)"
    /// Foreground color in HSL format "hsl(h, s%, l%)"
    pub foreground_hsl: String,

    /// Couleur de premier plan au format HSV "hsv(h, s%, v%)"
    /// Foreground color in HSV format "hsv(h, s%, v%)"
    pub foreground_hsv: String,

    /// Couleur de premier plan au format CIE L*a*b* "lab(l, a, b)"
    /// Foreground color in CIE L*a*b* format "lab(l, a, b)"
    pub foreground_lab: String,

    /// Couleur de premier plan au format OKLCH "oklch(l c h)"
    /// Foreground color in OKLCH format "oklch(l c h)"
    pub foreground_oklch: String,

    /// Si la couleur est sombre
    /// If the colour is dark
    pub foreground_is_dark: bool,

    /// Opacité du premier plan.
    /// Foreground opacity.
    pub foreground_alpha: f64,

    /// Hex opaque du premier plan aplati sur l'arrière-plan (composition alpha).
    /// Opaque hex of the foreground flattened over the background (alpha compositing).
    pub foreground_composited_hex: String,

    /// Couleur d'arrière-plan au format RGB (r, g, b)
    /// Background color in RGB format (r, g, b)
    pub background_rgb: (u8, u8, u8),

    /// Chaîne CSS d'affichage du RGB : "rgb(r, g, b)"
    /// CSS display string for RGB: "rgb(r, g, b)"
    pub background_rgb_css: String,

    /// Couleur d'arrière-plan au format hexadécimal
    /// Background color in hexadecimal format
    pub background_hex: String,

    /// Couleur d'arrière-plan au format HSL "hsl(h, s%, l%)"
    /// Background color in HSL format "hsl(h, s%, l%)"
    pub background_hsl: String,

    /// Couleur d'arrière-plan au format HSV "hsv(h, s%, v%)"
    /// Background color in HSV format "hsv(h, s%, v%)"
    pub background_hsv: String,

    /// Couleur d'arrière-plan au format CIE L*a*b* "lab(l, a, b)"
    /// Background color in CIE L*a*b* format "lab(l, a, b)"
    pub background_lab: String,

    /// Couleur d'arrière-plan au format OKLCH "oklch(l c h)"
    /// Background color in OKLCH format "oklch(l c h)"
    pub background_oklch: String,

    /// Si la couleur est sombre
    /// If the colour is dark
    pub background_is_dark: bool,

    /// Mode continue activé
    /// Continue mode enabled
    pub continue_mode: bool,

    // Contast Ratio value, not rounded
    // Valeur du Ratio de Contraste, non arrondi
    #[serde(skip)]
    pub contrast_ratio_raw: f64,

    // Contast Ratio value, rounded
    // Valeur du Ratio de Contraste, arrondi
    pub contrast_ratio_rounded: f64,

    /// Ratio de contraste entre l'arrière-plan et le blanc / noir.
    /// Contrast ratio between background and white / black.
    /// (Fixes #9)
    pub background_contrast_with_white: f64,
    pub background_contrast_with_black: f64,
}

impl Default for ResultStore {
    fn default() -> Self {
        let (fr, fg, fb) = config::DEFAULT_FOREGROUND_RGB;
        let (br, bg, bb) = config::DEFAULT_BACKGROUND_RGB;
        let contrast_ratio_raw = color::contrast_ratio((fr, fg, fb), (br, bg, bb));
        let contrast_ratio_rounded = wcag_contrast::truncate_ratio(contrast_ratio_raw);
        let background_contrast_with_white = color::contrast_ratio((br, bg, bb), (255, 255, 255));
        let background_contrast_with_black = color::contrast_ratio((br, bg, bb), (0, 0, 0));
        Self {
            // Plateforme détectée à la compilation
            // Platform detected at compile time
            #[cfg(target_os = "macos")]
            platform: "macos",
            #[cfg(target_os = "windows")]
            platform: "windows",
            #[cfg(target_os = "linux")]
            platform: "linux",
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            platform: "unknown",
            foreground_rgb: config::DEFAULT_FOREGROUND_RGB,
            foreground_rgb_css: color::rgb_to_css_string((fr, fg, fb), 1.0),
            foreground_hex: format!("#{:02X}{:02X}{:02X}", fr, fg, fb),
            foreground_hsl: color::rgb_to_hsl_string((fr, fg, fb), 1.0),
            foreground_hsv: color::rgb_to_hsv_string((fr, fg, fb), 1.0),
            foreground_lab: color::rgb_to_lab_string((fr, fg, fb), 1.0),
            foreground_oklch: color::rgb_to_oklch_string((fr, fg, fb), 1.0),
            foreground_is_dark: color::is_dark((fr, fg, fb)),
            foreground_alpha: 1.0,
            foreground_composited_hex: format!("#{:02X}{:02X}{:02X}", fr, fg, fb),
            background_rgb: config::DEFAULT_BACKGROUND_RGB,
            background_rgb_css: color::rgb_to_css_string((br, bg, bb), 1.0),
            background_hex: format!("#{:02X}{:02X}{:02X}", br, bg, bb),
            background_hsl: color::rgb_to_hsl_string((br, bg, bb), 1.0),
            background_hsv: color::rgb_to_hsv_string((br, bg, bb), 1.0),
            background_lab: color::rgb_to_lab_string((br, bg, bb), 1.0),
            background_oklch: color::rgb_to_oklch_string((br, bg, bb), 1.0),
            background_is_dark: color::is_dark((br, bg, bb)),
            continue_mode: false,
            contrast_ratio_raw,
            contrast_ratio_rounded,
            background_contrast_with_white,
            background_contrast_with_black,
        }
    }
}

/// Modèle de copie (template + raccourci)
/// Copy template (template + shortcut)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CopyTemplate {
    #[serde(default)]
    pub name: String,
    pub template: String,
    pub shortcut: String,
}

/// État de l'application wrappé dans un Mutex pour thread-safety
/// Application state wrapped in Mutex for thread-safety
pub struct AppState {
    pub store: Mutex<ResultStore>,
    pub locale: Mutex<String>,
    pub templates: Mutex<Vec<CopyTemplate>>,
    pub appearance: Mutex<String>,
    pub style_theme: Mutex<String>,
    pub always_on_top: Mutex<bool>,
}

// =============================================================================
// COMMANDES TAURI
// TAURI COMMANDS
// =============================================================================

/// Récupère l'état actuel du store
/// Gets the current store state
#[tauri::command]
pub fn get_store(state: tauri::State<AppState>) -> ResultStore {
    // Verrouille le mutex et clone le contenu
    // Lock the mutex and clone the content
    state.store.lock().unwrap().clone()
}

/// Lance le color picker et met à jour le store automatiquement
/// Launches the color picker and automatically updates the store
#[tauri::command]
pub fn pick_color(app: AppHandle, state: tauri::State<AppState>, fg: bool) {
    // Lance le picker natif
    // Launch the native picker
    let result = picker::run(fg);

    // Met à jour le store avec les couleurs sélectionnées
    // Update the store with selected colors
    {
        // Verrouille le mutex
        // Lock the mutex
        let mut store = state.store.lock().unwrap();

        // Met à jour les couleurs à partir du résultat du picker
        // Update colors from picker result
        color::update_results_from_picker(&mut store, &result);

        // Met à jour le mode continue
        // Update continue mode
        store.continue_mode = result.continue_mode;

        // Émet l'événement "store-updated" avec le nouveau state
        // Emit "store-updated" event with the new state
        let _ = app.emit("store-updated", store.clone());
    }
}

/// Applique une couleur RGB (+ alpha) à la clé donnée puis recalcule les dérivés
/// (hex, hsl, sombre, ratios de contraste). Retourne false si la clé est inconnue.
///
/// Applies an RGB color (+ alpha) to the given key then recomputes the derived values
/// (hex, hsl, dark, contrast ratios). Returns false on an unknown key.
pub(crate) fn apply_color(store: &mut ResultStore, key: &str, rgb: (u8, u8, u8), alpha: f64, source: Option<(&str, String)>) -> bool {
    let alpha = if key == "background" { 1.0 } else { alpha.clamp(0.0, 1.0) };

    let rgb_css = color::rgb_to_css_string(rgb, alpha);
    let hex = color::rgb_to_hex_string(rgb, alpha);
    let mut hsl = color::rgb_to_hsl_string(rgb, alpha);
    let mut hsv = color::rgb_to_hsv_string(rgb, alpha);
    let mut lab = color::rgb_to_lab_string(rgb, alpha);
    let mut oklch = color::rgb_to_oklch_string(rgb, alpha);

    // Conserve la chaîne saisie pour le format édité (verbatim).
    // Keep the entered string for the edited format (verbatim).
    if let Some((fmt, s)) = source {
        match fmt {
            "hsl" => hsl = s,
            "hsv" => hsv = s,
            "lab" => lab = s,
            "oklch" => oklch = s,
            _ => {}
        }
    }

    match key {
        "foreground" => {
            store.foreground_rgb = rgb;
            store.foreground_rgb_css = rgb_css;
            store.foreground_hex = hex;
            store.foreground_hsl = hsl;
            store.foreground_hsv = hsv;
            store.foreground_lab = lab;
            store.foreground_oklch = oklch;
            store.foreground_alpha = alpha;
            // Sombre : évalué sur la couleur composée sur l'arrière-plan.
            // Dark: evaluated on the color composited over the background.
            store.foreground_is_dark =
                color::is_dark(color::composite_over(rgb, store.background_rgb, alpha));
        }
        "background" => {
            store.background_rgb = rgb;
            store.background_rgb_css = rgb_css;
            store.background_hex = hex;
            store.background_hsl = hsl;
            store.background_hsv = hsv;
            store.background_lab = lab;
            store.background_oklch = oklch;
            store.background_is_dark = color::is_dark(rgb);
        }
        _ => return false, // Clé inconnue / Unknown key
    }

    // Recalcule le ratio de contraste (brut + arrondi vers le bas) ; le premier plan
    // semi-transparent est d'abord composé sur l'arrière-plan.
    // Recalculate the contrast ratio (raw + floor-rounded); the semi-transparent
    // foreground is first composited over the background.
    let blended_fg = color::composite_over(
        store.foreground_rgb,
        store.background_rgb,
        store.foreground_alpha,
    );
    // Hex opaque du premier plan aplati
    // Opaque hex of the flattened foregroundÒ
    store.foreground_composited_hex = color::rgb_to_hex_string(blended_fg, 1.0);
    store.contrast_ratio_raw = color::contrast_ratio(blended_fg, store.background_rgb);
    store.contrast_ratio_rounded = wcag_contrast::truncate_ratio(store.contrast_ratio_raw);
    // Recalcule les contrastes arrière-plan vs blanc / noir
    // Recalculate background-vs-white / black contrasts
    store.background_contrast_with_white = color::contrast_ratio(store.background_rgb, (255, 255, 255));
    store.background_contrast_with_black = color::contrast_ratio(store.background_rgb, (0, 0, 0));

    true
}

/// Met à jour une couleur depuis des composantes RGB (chemin RGB et inversion).
/// Updates a color from RGB components (RGB path and swap).
#[tauri::command]
pub fn update_store_rgb(app: AppHandle, state: tauri::State<AppState>, key: String, r: u8, g: u8, b: u8, alpha: Option<f64>) {
    let mut store = state.store.lock().unwrap();
    if apply_color(&mut store, &key, (r, g, b), alpha.unwrap_or(1.0), None) {
        let _ = app.emit("store-updated", store.clone());
    }
}

/// Met à jour une couleur depuis des composantes HSL.
/// Updates a color from HSL components
#[tauri::command]
pub fn update_store_hsl(app: AppHandle, state: tauri::State<AppState>, key: String, h: u16, s: u8, l: u8, alpha: Option<f64>) {
    let rgb = color::hsl_to_rgb(h, s, l);
    let alpha = alpha.unwrap_or(1.0);
    let display = format!("hsl({}, {}%, {}%{})", h, s, l, color::alpha_suffix(alpha));
    let mut store = state.store.lock().unwrap();
    if apply_color(&mut store, &key, rgb, alpha, Some(("hsl", display))) {
        let _ = app.emit("store-updated", store.clone());
    }
}

/// Met à jour une couleur depuis des composantes HSV.
/// Updates a color from HSV components
#[tauri::command]
pub fn update_store_hsv(app: AppHandle, state: tauri::State<AppState>, key: String, h: u16, s: u8, v: u8, alpha: Option<f64>) {
    let rgb = color::hsv_to_rgb(h, s, v);
    let alpha = alpha.unwrap_or(1.0);
    let display = format!("hsv({}, {}%, {}%{})", h, s, v, color::alpha_suffix(alpha));
    let mut store = state.store.lock().unwrap();
    if apply_color(&mut store, &key, rgb, alpha, Some(("hsv", display))) {
        let _ = app.emit("store-updated", store.clone());
    }
}

/// Met à jour une couleur depuis des composantes CIE L*a*b*
/// (a et b peuvent être négatifs.)
/// Updates a color from CIE L*a*b* components
/// (a and b may be negative.)
#[tauri::command]
pub fn update_store_lab(app: AppHandle, state: tauri::State<AppState>, key: String, l: i16, a: i16, b: i16, alpha: Option<f64>) {
    let rgb = color::lab_to_rgb(l, a, b);
    let alpha = alpha.unwrap_or(1.0);
    let display = format!("lab({}, {}, {}{})", l, a, b, color::alpha_suffix(alpha));
    let mut store = state.store.lock().unwrap();
    if apply_color(&mut store, &key, rgb, alpha, Some(("lab", display))) {
        let _ = app.emit("store-updated", store.clone());
    }
}

/// Met à jour une couleur depuis des composantes OKLCH (l: 0-1, c, h: degrés).
/// Updates a color from OKLCH components (l: 0-1, c, h: degrees).
#[tauri::command]
pub fn update_store_oklch(app: AppHandle, state: tauri::State<AppState>, key: String, l: f64, c: f64, h: f64, alpha: Option<f64>) {
    let rgb = color::oklch_to_rgb(l, c, h);
    let alpha = alpha.unwrap_or(1.0);
    let display = format!("oklch({:.3} {:.3} {:.0}{})", l, c, h, color::alpha_suffix(alpha));
    let mut store = state.store.lock().unwrap();
    if apply_color(&mut store, &key, rgb, alpha, Some(("oklch", display))) {
        let _ = app.emit("store-updated", store.clone());
    }
}

/// Met à jour une couleur depuis une saisie hexadécimale
/// Updates a color from a hexadecimal input
#[tauri::command]
pub fn update_store_hex(app: AppHandle, state: tauri::State<AppState>, key: String, hex: String, alpha: Option<f64>) {
    let Some((rgb, hex_alpha)) = color::hex_to_rgb(&hex) else { return };
    // Si le hex est en 6 chiffres (hex_alpha == 1.0), on retombe sur `alpha` si fourni.
    // When the hex is 6-digit (hex_alpha == 1.0), fall back to `alpha` if provided.
    let alpha = if hex_alpha < 1.0 { hex_alpha } else { alpha.unwrap_or(1.0) };
    let mut store = state.store.lock().unwrap();
    if apply_color(&mut store, &key, rgb, alpha, None) {
        let _ = app.emit("store-updated", store.clone());
    }
}

/// Retourne le nom de couleur CSS exact pour une valeur RGB, ou vide
/// Returns the exact CSS color name for a given RGB value, or empty
#[tauri::command]
pub fn get_color_name(r: u8, g: u8, b: u8) -> String {
    color_names::exact_color_name(r, g, b)
        .unwrap_or("")
        .to_string()
}

/// Efface le store
/// Clears the store
#[tauri::command]
pub fn clear_store(app: AppHandle, state: tauri::State<AppState>) {
    {
        let mut store = state.store.lock().unwrap();
        *store = ResultStore::default();
        let _ = app.emit("store-updated", store.clone());
    }
}
