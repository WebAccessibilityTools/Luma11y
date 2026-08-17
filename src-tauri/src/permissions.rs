//! =============================================================================
//! PERMISSIONS.RS - Vérification des permissions système
//! PERMISSIONS.RS - System permission checks
//! =============================================================================
//!
//! Sur macOS, la capture d'écran (color picker) nécessite l'autorisation
//! « Screen & System Audio Recording » (Screen Recording).
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
    // - CGPreflightScreenCaptureAccess : renvoie true si l'app a déjà l'accès,
    //   sans déclencher de demande ni de dialogue système.
    // - CGRequestScreenCaptureAccess : déclenche la demande système (ajoute l'app
    //   à la liste Screen Recording) et renvoie l'état courant de l'autorisation.
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

    /// Vérifie (sans demander) si l'app a l'autorisation d'enregistrement d'écran.
    /// Checks (without prompting) whether the app has screen recording access.
    pub fn has_screen_recording_access() -> bool {
        // Appel FFI : lecture d'un état système, sans effet de bord.
        // FFI call: reads a system state, with no side effect.
        unsafe { CGPreflightScreenCaptureAccess() }
    }

    /// Demande l'autorisation d'enregistrement d'écran (déclenche le dialogue
    /// système au premier appel et inscrit l'app dans la liste).
    /// Requests screen recording access (triggers the system dialog on first
    /// call and registers the app in the list).
    pub fn request_screen_recording_access() -> bool {
        unsafe { CGRequestScreenCaptureAccess() }
    }

    /// Ouvre Réglages Système › Confidentialité et sécurité › Enregistrement de
    /// l'écran (volet Screen Recording).
    /// Opens System Settings › Privacy & Security › Screen Recording.
    pub fn open_settings() -> Result<(), String> {
        // URL scheme des Réglages Système ciblant directement le bon volet.
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
// Autres plateformes (pas de restriction équivalente ici)
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

/// Renvoie `true` si l'app peut capturer l'écran. Toujours `true` hors macOS.
/// Returns `true` if the app can capture the screen. Always `true` off macOS.
#[tauri::command]
pub fn check_screen_recording_permission() -> bool {
    platform::has_screen_recording_access()
}

/// Déclenche la demande d'autorisation système et renvoie l'état résultant.
/// Triggers the system authorization request and returns the resulting state.
#[tauri::command]
pub fn request_screen_recording_permission() -> bool {
    platform::request_screen_recording_access()
}

/// Ouvre le volet Réglages Système correspondant.
/// Opens the matching System Settings pane.
#[tauri::command]
pub fn open_screen_recording_settings() -> Result<(), String> {
    platform::open_settings()
}
