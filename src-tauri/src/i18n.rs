// =============================================================================
// i18n.rs - Dispatcher de traductions des menus natifs
// i18n.rs - Native menu translation dispatcher
// =============================================================================
//
// Les tables par langue vivent dans `lang/<locale>.rs`. Ce fichier ne fait
// que dispatcher la requête vers la bonne table et appliquer le fallback
// vers l'anglais.
//
// Per-language tables live in `lang/<locale>.rs`. This file only dispatches
// the request to the right table and applies the English fallback.

use crate::lang;

/// Retourne la traduction d'une clé de menu pour une locale donnée
/// Returns the translation of a menu key for a given locale
pub fn menu_t(locale: &str, key: &str) -> &'static str {
    let lookup = match locale {
        "en" => lang::en::t(key),
        "fr" => lang::fr::t(key),
        "sk" => lang::sk::t(key),
        // Locale inconnue : fallback direct vers l'anglais
        // Unknown locale: fall back straight to English
        _ => lang::en::t(key),
    };

    // Si la clé n'existe pas dans la locale demandée, fallback vers l'anglais
    // If the key is missing in the requested locale, fall back to English
    lookup
        .or_else(|| lang::en::t(key))
        .unwrap_or("?")
}
