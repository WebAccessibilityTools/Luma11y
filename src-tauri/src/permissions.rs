//! =============================================================================
//! PERMISSIONS.RS - System permission checks
//! =============================================================================
//!
//! On macOS, screen capture (the color picker) requires the
//! "Screen & System Audio Recording" (Screen Recording) authorization.

// =============================================================================
// macOS
// =============================================================================
#[cfg(target_os = "macos")]
mod platform {
    // Bindings C bruts du framework CoreGraphics.
    // Raw C bindings from the CoreGraphics framework.
    //
    // - CGPreflightScreenCaptureAccess: returns true if the app already has
    //   access, without triggering any prompt or system dialog.
    // - CGRequestScreenCaptureAccess: triggers the system request (adds the app
    //   to the Screen Recording list) and returns the current authorization state.
    //
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }

    /// Checks (without prompting) whether the app has screen recording access.
    pub fn has_screen_recording_access() -> bool {
        // FFI call: reads a system state, with no side effect.
        unsafe { CGPreflightScreenCaptureAccess() }
    }

    /// Requests screen recording access (triggers the system dialog on first
    /// call and registers the app in the list).
    pub fn request_screen_recording_access() -> bool {
        unsafe { CGRequestScreenCaptureAccess() }
    }

    /// Opens System Settings › Privacy & Security › Screen Recording.
    pub fn open_settings() -> Result<(), String> {
        // System Settings URL scheme targeting the correct pane directly.
        const URL: &str =
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";

        std::process::Command::new("open")
            .arg(URL)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

// =============================================================================
// Other platforms (no equivalent restriction here)
// =============================================================================
#[cfg(not(target_os = "macos"))]
mod platform {
    pub fn has_screen_recording_access() -> bool {
        true
    }

    pub fn request_screen_recording_access() -> bool {
        true
    }

    pub fn open_settings() -> Result<(), String> {
        Ok(())
    }
}

// =============================================================================
// COMMANDES TAURI
// TAURI COMMANDS
// =============================================================================

/// Returns `true` if the app can capture the screen. Always `true` off macOS.
#[tauri::command]
pub fn check_screen_recording_permission() -> bool {
    platform::has_screen_recording_access()
}

/// Triggers the system authorization request and returns the resulting state.
#[tauri::command]
pub fn request_screen_recording_permission() -> bool {
    platform::request_screen_recording_access()
}

/// Opens the matching System Settings pane.
#[tauri::command]
pub fn open_screen_recording_settings() -> Result<(), String> {
    platform::open_settings()
}
