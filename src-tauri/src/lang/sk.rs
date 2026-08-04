// =============================================================================
// lang/sk.rs - Traductions slovaques
// lang/sk.rs - Slovak translations
// =============================================================================

/// Retourne la traduction slovaque d'une clé, ou None si inconnue
/// Returns the Slovak translation for a key, or None if unknown
pub fn t(key: &str) -> Option<&'static str> {
    Some(match key {
        "about" => "O aplik\u{00e1}cii Luma11y",
        "hide" => "Skry\u{0165}",
        "hide_others" => "Skry\u{0165} ostatn\u{00e9}",
        "show_all" => "Zobrazi\u{0165} v\u{0161}etko",
        "quit" => "Ukon\u{010d}i\u{0165}",
        "colour_profiles" => "Farebn\u{00e9} profily",
        "language" => "Jazyk",
        "appearance" => "Vzh\u{013e}ad",
        "appearance_auto" => "Automaticky",
        "appearance_light" => "Svetl\u{00fd}",
        "appearance_dark" => "Tmav\u{00fd}",
        "style_theme" => "\u{0160}t\u{00fd}l",
        "style_modern" => "Modern\u{00fd}",
        "style_classic" => "Klasick\u{00fd}",
        "settings" => "Nastavenia\u{2026}",
        "settings_title" => "Nastavenia",
        "edit" => "Upravi\u{0165}",
        "copy_templates" => "\u{0160}abl\u{00f3}ny kop\u{00ed}rovania",
        "window" => "Okno",
        "minimize" => "Minimalizova\u{0165}",
        "zoom" => "Pribl\u{00ed}\u{017e}i\u{0165}",
        "close_window" => "Zavrie\u{0165} okno",
        "fullscreen" => "Prepn\u{00fa}\u{0165} na cel\u{00fa} obrazovku",
        "always_on_top" => "V\u{017e}dy navrchu",
        _ => return None,
    })
}
