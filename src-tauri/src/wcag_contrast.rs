// =============================================================================
// wcag_contrast.rs - Ratio de contraste WCAG 2.2 strictement normatif
// wcag_contrast.rs - Strictly normative WCAG 2.2 contrast ratio
// =============================================================================
//
// Ce module implémente l'algorithme WCAG 2.x, tel que défini dans la spec
//
// This module implements the WCAG 2.x algorithm, as defined in the spec

// Fix #51 using the **rounded** luminance coefficients (0.2126 / 0.7152 / 0.0722), as `palette`'s sRGB→XYZ conversion, uses full-precision coefficients ~0.2126729 / 0.7151522 / 0.0721750)

use crate::config;

/// Linéarise un canal sRGB 8 bits ([0..255]) vers l'espace linéaire ([0..1]).
///
/// Linearizes an 8-bit sRGB channel ([0..255]) into linear space ([0..1]).
fn linearize(channel: u8) -> f64 {
    let c = channel as f64 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Luminance relative WCAG d'une couleur RGB opaque.
///
/// WCAG relative luminance of an opaque RGB color.
fn relative_luminance(rgb: [u8; 3]) -> f64 {
    0.2126 * linearize(rgb[0]) + 0.7152 * linearize(rgb[1]) + 0.0722 * linearize(rgb[2])
}

/// Ratio de contraste WCAG entre deux couleurs opaques : (L_clair + 0.05) /
/// (L_sombre + 0.05). Indépendant de l'ordre des arguments.
///
/// WCAG contrast ratio between two opaque colors: (L_lighter + 0.05) /
/// (L_darker + 0.05). Independent of argument order.
pub fn contrast_ratio(a: [u8; 3], b: [u8; 3]) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (lighter, darker) = if la >= lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Adaptateur pour les tuples RGB utilisés dans le reste du code.
/// Adapter for the RGB tuples used across the rest of the codebase.
pub fn contrast_ratio_u8(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    contrast_ratio([a.0, a.1, a.2], [b.0, b.1, b.2])
}

/// Tronque un ratio à la précision de `config::ROUNDING_FACTOR`.
///
/// Truncates a ratio to `config::ROUNDING_FACTOR` precision.
/// See https://github.com/w3c/wcag/issues/200
pub fn truncate_ratio(raw: f64) -> f64 {
    (raw * config::ROUNDING_FACTOR as f64).trunc() / config::ROUNDING_FACTOR as f64
}

/// Formate un ratio pour l'affichage (seulement, par pour la validation), ex. "4.47:1".
///
/// Formats a ratio for display (only, not for validation), e.g. "4.47:1".
#[allow(dead_code)]
pub fn format_ratio(ratio: f64) -> String {
    let decimals = (config::ROUNDING_FACTOR as f64).log10().round() as usize;
    format!("{:.*}:1", decimals, truncate_ratio(ratio))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE: [u8; 3] = [255, 255, 255];
    const BLACK: [u8; 3] = [0, 0, 0];

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn orange_on_white_passes_three_to_one() {
        // #FE6300 sur #FFFFFF : cas limite où palette échoue mais la formule
        // normative passe le seuil 3:1.
        let ratio = contrast_ratio([0xFE, 0x63, 0x00], WHITE);
        assert!(
            approx(ratio, 3.0004705615178744, 1e-9),
            "got {ratio}"
        );
        assert!(ratio >= 3.0, "must pass the 3.0 threshold, got {ratio}");
    }

    #[test]
    fn gray_on_white_fails_four_point_five() {
        // #777777 sur #FFFFFF : ~4.478, échoue au seuil 4.5.
        let ratio = contrast_ratio([0x77, 0x77, 0x77], WHITE);
        assert!(approx(ratio, 4.478, 1e-3), "got {ratio}");
        assert!(ratio < 4.5, "must fail the 4.5 threshold, got {ratio}");
        assert_eq!(format_ratio(ratio), "4.47:1");
    }

    #[test]
    fn black_on_white_is_twenty_one() {
        let ratio = contrast_ratio(BLACK, WHITE);
        assert!(approx(ratio, 21.0, 1e-9), "got {ratio}");
    }

    #[test]
    fn identity_is_one() {
        let c = [0x12, 0x9A, 0xF3];
        assert_eq!(contrast_ratio(c, c), 1.0);
    }

    #[test]
    fn symmetry() {
        let a = [0xFE, 0x63, 0x00];
        let b = [0x33, 0x88, 0xCC];
        assert_eq!(contrast_ratio(a, b), contrast_ratio(b, a));
    }
}
