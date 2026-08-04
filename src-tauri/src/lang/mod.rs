// =============================================================================
// lang/mod.rs - Tables de traduction par langue
// lang/mod.rs - Per-language translation tables
// =============================================================================
//
// Chaque sous-module expose `pub fn t(key: &str) -> Option<&'static str>` qui
// retourne Some pour les clés connues, None sinon. Le dispatch et le fallback
// sont gérés dans i18n.rs.
//
// Each submodule exposes `pub fn t(key: &str) -> Option<&'static str>` returning
// Some for known keys, None otherwise. Dispatch and fallback are handled in i18n.rs.

pub mod en;
pub mod fr;
pub mod sk;
