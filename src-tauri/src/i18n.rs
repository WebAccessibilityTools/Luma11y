// =============================================================================
// i18n.rs - Native menu translation dispatcher
// =============================================================================
//
// Per-language tables live in `lang/<locale>.rs`. This file only dispatches
// the request to the right table and applies the English fallback.

use crate::lang;

/// Returns the translation of a menu key for a given locale
pub fn menu_t(locale: &str, key: &str) -> &'static str {
    let lookup = match locale {
        "en" => lang::en::t(key),
        "fr" => lang::fr::t(key),
        "sk" => lang::sk::t(key),
        // Unknown locale: fall back straight to English
        _ => lang::en::t(key),
    };

    // If the key is missing in the requested locale, fall back to English
    lookup
        .or_else(|| lang::en::t(key))
        .unwrap_or("?")
}
