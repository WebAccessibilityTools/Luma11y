// =============================================================================
// picker/mod.rs - Color picker module
// =============================================================================

/// Common code between platforms (types, utility functions)
pub mod common;

/// macOS implementation
#[cfg(target_os = "macos")]
pub mod macos;

/// Windows implementation
#[cfg(target_os = "windows")]
pub mod windows;

/// Linux implementation (not implemented)
#[cfg(target_os = "linux")]
pub mod linux;

// =============================================================================
// PUBLIC FUNCTION
// =============================================================================

/// Launches the native color picker based on the platform
///
/// # Arguments
/// * `fg` - true for foreground, false for background
///
/// # Returns
/// * `ColorPickerResult` - Result with the selected colors
pub fn run(fg: bool) -> common::ColorPickerResult {
    #[cfg(target_os = "macos")]
    {
        macos::run(fg)
    }

    #[cfg(target_os = "windows")]
    {
        windows::run(fg)
    }

    #[cfg(target_os = "linux")]
    {
        linux::run(fg)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        common::ColorPickerResult::default()
    }
}
