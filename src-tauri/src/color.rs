// =============================================================================
// color.rs - Color manipulation and store update functions
// =============================================================================

use palette::{FromColor, Hsl, Hsv, Lab, Oklch, Srgb};
use crate::store::ResultStore;
use crate::picker::common::ColorPickerResult;
use crate::wcag_contrast;

/// Convertit un tuple RGB u8 en `Srgb<f64>` (composantes [0..1]).
/// Convert a u8 RGB tuple into `Srgb<f64>` (components in [0..1]).
fn srgb_from_u8(rgb: (u8, u8, u8)) -> Srgb<f64> {
    let (r, g, b) = rgb;
    Srgb::<u8>::new(r, g, b).into_format::<f64>()
}

/// Ratio de contraste WCAG entre deux couleurs sRGB opaques. Délègue au module
/// `wcag_contrast`
///
/// WCAG contrast ratio between two opaque sRGB colors. Delegates to the
/// `wcag_contrast` module
pub fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f64 {
    wcag_contrast::contrast_ratio_u8(fg, bg)
}

/// Compose un premier-plan semi-transparent (alpha) sur un background opaque
/// Sert au contraste et à `is_dark`
///
/// Composites a semi-transparent foreground (alpha) over an opaque
/// Used for contrast and `is_dark`
/// fg·a + bg·(1−a)
pub fn composite_over(fg: (u8, u8, u8), bg: (u8, u8, u8), alpha: f64) -> (u8, u8, u8) {
    let a = alpha.clamp(0.0, 1.0);
    let mix = |f: u8, b: u8| (f as f64 * a + b as f64 * (1.0 - a)).round() as u8;
    (mix(fg.0, bg.0), mix(fg.1, bg.1), mix(fg.2, bg.2))
}

/// Suffixe alpha en notation CSS moderne (` / NN%`)
/// Alpha suffix in modern CSS notation (` / NN%`)
pub fn alpha_suffix(alpha: f64) -> String {
    if alpha >= 1.0 {
        String::new()
    } else {
        format!(" / {}%", (alpha * 100.0).round() as u8)
    }
}

/// Sérialise une couleur RGB en chaîne CSS d'affichage : `rgb(r, g, b / NN%)` si alpha,
/// sinon `rgb(r g b)`
///
/// Serializes an RGB color into a CSS display string: `rgb(r, g, b / NN%)` when alpha,
/// otherwise `rgb(r g b)`
pub fn rgb_to_css_string(rgb: (u8, u8, u8), alpha: f64) -> String {
    let (r, g, b) = rgb;
    if alpha >= 1.0 {
        format!("rgb({}, {}, {})", r, g, b)
    } else {
        format!("rgb({} {} {}{})", r, g, b, alpha_suffix(alpha))
    }
}

/// Sérialise une couleur RGB en chaîne HSL CSS "hsl(h, s%, l%)" + ` / NN%` si alpha.
///
/// Serializes an RGB color into a CSS HSL string "hsl(h, s%, l%)" + ` / NN%` when alpha
pub fn rgb_to_hsl_string(rgb: (u8, u8, u8), alpha: f64) -> String {
    let hsl = Hsl::from_color(srgb_from_u8(rgb));
    // Teinte ramenée dans [0, 360), saturation et luminosité en pourcentage.
    // Hue brought back into [0, 360), saturation and lightness as percentages.
    let h = (hsl.hue.into_positive_degrees().round() as u16) % 360;
    let s = (hsl.saturation * 100.0).round() as u8;
    let l = (hsl.lightness * 100.0).round() as u8;
    format!("hsl({}, {}%, {}%{})", h, s, l, alpha_suffix(alpha))
}

/// Convertit une couleur HSL (h: 0-360, s/l: 0-100) en composantes RGB.
///
/// Converts an HSL color (h: 0-360, s/l: 0-100) into RGB components.
pub fn hsl_to_rgb(h: u16, s: u8, l: u8) -> (u8, u8, u8) {
    let hsl = Hsl::new(h as f64, s as f64 / 100.0, l as f64 / 100.0);
    let rgb = Srgb::from_color(hsl).into_format::<u8>();
    (rgb.red, rgb.green, rgb.blue)
}

/// Sérialise une couleur RGB en chaîne HSV "hsv(h, s%, v%)" (+ ` / NN%` si alpha).
///
/// Serializes an RGB color into an HSV string "hsv(h, s%, v%)" (+ ` / NN%` when alpha).
pub fn rgb_to_hsv_string(rgb: (u8, u8, u8), alpha: f64) -> String {
    let hsv = Hsv::from_color(srgb_from_u8(rgb));
    // Teinte ramenée dans [0, 360), saturation et valeur en pourcentage.
    // Hue brought back into [0, 360), saturation and value as percentages.
    let h = (hsv.hue.into_positive_degrees().round() as u16) % 360;
    let s = (hsv.saturation * 100.0).round() as u8;
    let v = (hsv.value * 100.0).round() as u8;
    format!("hsv({}, {}%, {}%{})", h, s, v, alpha_suffix(alpha))
}

/// Convertit une couleur HSV (h: 0-360, s/v: 0-100) en composantes RGB.
///
/// Converts an HSV color (h: 0-360, s/v: 0-100) into RGB components.
pub fn hsv_to_rgb(h: u16, s: u8, v: u8) -> (u8, u8, u8) {
    let hsv = Hsv::new(h as f64, s as f64 / 100.0, v as f64 / 100.0);
    let rgb = Srgb::from_color(hsv).into_format::<u8>();
    (rgb.red, rgb.green, rgb.blue)
}

/// Sérialise une couleur RGB en chaîne CIE L*a*b* "lab(l, a, b)" (composantes
/// arrondies ; a/b peuvent être négatifs).
///
/// Serializes an RGB color into a CIE L*a*b* string "lab(l, a, b)" (rounded
/// components; a/b may be negative).
pub fn rgb_to_lab_string(rgb: (u8, u8, u8), alpha: f64) -> String {
    let lab = Lab::from_color(srgb_from_u8(rgb)); // (point blanc D65)
    let l = lab.l.round() as i16;
    let a = lab.a.round() as i16;
    let b = lab.b.round() as i16;
    format!("lab({}, {}, {}{})", l, a, b, alpha_suffix(alpha))
}

/// Convertit une couleur CIE L*a*b* (l: 0-100, a/b ~ -128..127) en composantes RGB.
/// La couleur résultante est ramenée dans le gamut sRGB.
///
/// Converts a CIE L*a*b* color (l: 0-100, a/b ~ -128..127) into RGB components.
/// The resulting color is clamped to the sRGB gamut.
pub fn lab_to_rgb(l: i16, a: i16, b: i16) -> (u8, u8, u8) {
    let lab = Lab::new(l as f64, a as f64, b as f64);
    let rgb = Srgb::from_color(lab).into_format::<u8>();
    (rgb.red, rgb.green, rgb.blue)
}

/// Sérialise une couleur RGB en chaîne CSS OKLCH "oklch(l c h)".
///
/// Serializes an RGB color into a CSS OKLCH string "oklch(l c h)".
pub fn rgb_to_oklch_string(rgb: (u8, u8, u8), alpha: f64) -> String {
    let oklch = Oklch::from_color(srgb_from_u8(rgb));
    // Teinte indéfinie pour un gris (chroma ≈ 0) : on retombe sur 0.
    // Hue is undefined for a gray (chroma ≈ 0): fall back to 0.
    let h = oklch.hue.into_positive_degrees();
    let h = if h.is_finite() { h } else { 0.0 };
    format!("oklch({:.3} {:.3} {:.0}{})", oklch.l, oklch.chroma, h, alpha_suffix(alpha))
}

/// Convertit une couleur OKLCH (l: 0-1, c: chroma, h: degrés) en composantes RGB.
///
/// Converts an OKLCH color (l: 0-1, c: chroma, h: degrees) into RGB components.
pub fn oklch_to_rgb(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    let oklch = Oklch::new(l, c, h);
    let rgb = Srgb::from_color(oklch).into_format::<u8>();
    (rgb.red, rgb.green, rgb.blue)
}

/// Sérialise une couleur RGB en chaîne hex : `#RRGGBB` si opaque, sinon
/// `#RRGGBBAA` (alpha sur deux chiffres).
///
/// Serializes an RGB color into a hex string: `#RRGGBB` when opaque, otherwise
/// `#RRGGBBAA` (two-digit alpha).
pub fn rgb_to_hex_string(rgb: (u8, u8, u8), alpha: f64) -> String {
    let (r, g, b) = rgb;
    if alpha >= 1.0 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, (alpha * 255.0).round() as u8)
    }
}

/// Convertit une saisie hexadécimale en RGB + alpha. Accepte 3 (#abc), 4 (#abcd),
/// 6 (#aabbcc) et 8 (#aabbccdd) chiffres ; le # est optionnel.
///
/// Converts a hexadecimal input into RGB + alpha. Accepts 3 (#abc), 4 (#abcd),
/// 6 (#aabbcc) and 8 (#aabbccdd) digits; the # is optional.
pub fn hex_to_rgb(hex: &str) -> Option<((u8, u8, u8), f64)> {
    let cleaned = hex.trim().trim_start_matches('#');
    let byte = |s: &str| u8::from_str_radix(s, 16).ok();
    let dup = |c: char| byte(&format!("{0}{0}", c));

    match cleaned.len() {
        // Notation courte : chaque chiffre est doublé (#abc -> #aabbcc).
        // Short notation: each digit is doubled (#abc -> #aabbcc).
        3 => {
            let c: Vec<char> = cleaned.chars().collect();
            Some(((dup(c[0])?, dup(c[1])?, dup(c[2])?), 1.0))
        }
        // Notation courte avec alpha (#abcd -> #aabbccdd).
        // Short notation with alpha (#abcd -> #aabbccdd).
        4 => {
            let c: Vec<char> = cleaned.chars().collect();
            Some(((dup(c[0])?, dup(c[1])?, dup(c[2])?), dup(c[3])? as f64 / 255.0))
        }
        // Notation longue : deux chiffres par composante.
        // Long notation: two digits per component.
        6 => Some(((byte(&cleaned[0..2])?, byte(&cleaned[2..4])?, byte(&cleaned[4..6])?), 1.0)),
        // Notation longue avec alpha (#RRGGBBAA).
        // Long notation with alpha (#RRGGBBAA).
        8 => Some((
            (byte(&cleaned[0..2])?, byte(&cleaned[2..4])?, byte(&cleaned[4..6])?),
            byte(&cleaned[6..8])? as f64 / 255.0,
        )),
        _ => None,
    }
}

/// Considère une couleur comme sombre si son contraste avec le noir est < 4.5
/// A color is considered dark when its contrast against black is < 4.5
pub fn is_dark(rgb: (u8, u8, u8)) -> bool {
    contrast_ratio(rgb, (0, 0, 0)) < 4.5
}

/// Met à jour les résultats du store à partir du résultat du picker
/// Updates the store results from picker result
///
/// # Arguments
/// * `store` - Le store à mettre à jour / The store to update
/// * `result` - Le résultat du color picker / The color picker result
pub fn update_results_from_picker(store: &mut ResultStore, result: &ColorPickerResult) {
    // Recalcule tous les formats dérivés (hex, hsl, hsv, lab)
    // et les ratios de contraste depuis le RGB capturé.
    // Recomputes all derived formats (hex, hsl, hsv, lab)
    // and the contrast ratios from the captured RGB.
    if let Some(rgb) = result.foreground {
        crate::store::apply_color(store, "foreground", rgb, 1.0, None);
    }
    if let Some(rgb) = result.background {
        crate::store::apply_color(store, "background", rgb, 1.0, None);
    }
}
