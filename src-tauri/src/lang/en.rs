// =============================================================================
// lang/en.rs - English translations (canonical)
// =============================================================================

/// Returns the English translation for a key, or None if unknown
pub fn t(key: &str) -> Option<&'static str> {
    Some(match key {
        "about" => "About Luma11y",
        "hide" => "Hide",
        "hide_others" => "Hide Others",
        "show_all" => "Show All",
        "quit" => "Quit",
        "colour_profiles" => "Colour Profiles",
        "language" => "Language",
        "appearance" => "Appearance",
        "appearance_auto" => "Auto",
        "appearance_light" => "Light",
        "appearance_dark" => "Dark",
        "style_theme" => "Style",
        "style_modern" => "Modern",
        "style_classic" => "Classic",
        "settings" => "Settings\u{2026}",
        "settings_title" => "Settings",
        "edit" => "Edit",
        "copy_templates" => "Copy Templates",
        "window" => "Window",
        "minimize" => "Minimize",
        "zoom" => "Zoom",
        "close_window" => "Close Window",
        "fullscreen" => "Toggle Full Screen",
        "always_on_top" => "Always on Top",
        _ => return None,
    })
}
