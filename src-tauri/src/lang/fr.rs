// =============================================================================
// lang/fr.rs - French translations
// =============================================================================

/// Returns the French translation for a key, or None if unknown
pub fn t(key: &str) -> Option<&'static str> {
    Some(match key {
        "about" => "À propos de Luma11y",
        "hide" => "Masquer",
        "hide_others" => "Masquer les autres",
        "show_all" => "Tout afficher",
        "quit" => "Quitter",
        "colour_profiles" => "Profils de couleurs",
        "language" => "Langue",
        "appearance" => "Apparence",
        "appearance_auto" => "Auto",
        "appearance_light" => "Clair",
        "appearance_dark" => "Sombre",
        "style_theme" => "Style",
        "style_modern" => "Moderne",
        "style_classic" => "Classique",
        "settings" => "Pr\u{00e9}f\u{00e9}rences\u{2026}",
        "settings_title" => "Pr\u{00e9}f\u{00e9}rences",
        "edit" => "\u{00c9}dition",
        "copy_templates" => "Mod\u{00e8}les de copie",
        "window" => "Fen\u{00ea}tre",
        "minimize" => "R\u{00e9}duire",
        "zoom" => "Zoom",
        "close_window" => "Fermer la fen\u{00ea}tre",
        "fullscreen" => "Activer/d\u{00e9}sactiver le plein \u{00e9}cran",
        "always_on_top" => "Toujours au premier plan",
        _ => return None,
    })
}
