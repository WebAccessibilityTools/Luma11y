// =============================================================================
// lang/mod.rs - Per-language translation tables
// =============================================================================
//
// Each submodule exposes `pub fn t(key: &str) -> Option<&'static str>` returning
// Some for known keys, None otherwise. Dispatch and fallback are handled in i18n.rs.

pub mod en;
pub mod fr;
pub mod sk;
