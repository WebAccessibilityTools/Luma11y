// =============================================================================
// icc.rs - ICC Profile Management
// =============================================================================
//
// This module manages listing and selecting ICC profiles available on the system.
// On macOS, it uses NSColorSpace.availableColorSpaces to get the profiles.

// Import serde for JSON serialization
use serde::{Deserialize, Serialize};

// Import Mutex for thread-safe synchronization
use std::sync::Mutex;

// =============================================================================
// STRUCTURES
// =============================================================================

/// Structure representing an ICC profile
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ICCProfile {
    /// Profile name (unique identifier)
    pub name: String,

    /// Human-readable profile description
    pub description: String,

    /// Indicates if this profile is currently selected
    pub is_current: bool,
}

// =============================================================================
// GLOBAL STATE
// =============================================================================

/// Globally selected ICC profile (protected by Mutex)
static SELECTED_PROFILE: Mutex<Option<String>> = Mutex::new(None);

// =============================================================================
// IMPLÉMENTATION macOS
// =============================================================================

/// Lists all available ICC profiles on macOS via NSColorSpace
#[cfg(target_os = "macos")]
fn get_system_color_spaces() -> Vec<ICCProfile> {
    // Import required types for macOS
    use objc2_app_kit::NSColorSpace;
    use objc2_app_kit::NSColorSpaceModel;
    use objc2_foundation::NSArray;

    // Vector to store found profiles
    let mut profiles: Vec<ICCProfile> = Vec::new();

    // First add the "Auto" profile (automatic detection)
    profiles.push(ICCProfile {
        name: "Auto".to_string(),
        description: "Automatic color space detection".to_string(),
        is_current: false,
    });

    // Get the array of available RGB color spaces
    let color_spaces: objc2::rc::Retained<NSArray<NSColorSpace>> = 
        NSColorSpace::availableColorSpacesWithModel(
            // NSColorSpaceModelRGB for RGB spaces only
            NSColorSpaceModel::RGB,
        );

    // Get the number of elements in the array
    let count = color_spaces.count();

    // Iterate over each color space
    for i in 0..count {
        // Get the color space at index i via objectAtIndex:
        let color_space = color_spaces.objectAtIndex(i);

        // Get the localized name of the color space
        if let Some(name_ns) = color_space.localizedName() {
            // Convert NSString to Rust String
            let name = name_ns.to_string();

            // Create description (same as name for now)
            let description = name.clone();

            // Add profile to the list
            profiles.push(ICCProfile {
                name,
                description,
                is_current: false,
            });
        }
    }

    // Return the list of profiles
    profiles
}

/// Gets the NSColorSpace corresponding to the selected profile
///
/// # Returns
/// * `Option<Retained<NSColorSpace>>` - The color space or None if Auto/not found
#[cfg(target_os = "macos")]
pub fn get_selected_nscolorspace() -> Option<objc2::rc::Retained<objc2_app_kit::NSColorSpace>> {
    // Import required types for macOS
    use objc2_app_kit::NSColorSpace;
    use objc2_app_kit::NSColorSpaceModel;
    use objc2_foundation::NSArray;

    // Get the selected profile name
    let selected_name = if let Ok(selected) = SELECTED_PROFILE.lock() {
        // Clone the name to release the lock
        selected.clone()
    } else {
        // On error, return None (use Auto)
        return None;
    };

    // If no profile selected or "Auto", return None
    let profile_name = match selected_name {
        Some(name) if name != "Auto" => name,
        _ => return None,
    };

    // Get the array of available RGB color spaces
    let color_spaces: objc2::rc::Retained<NSArray<NSColorSpace>> = 
        NSColorSpace::availableColorSpacesWithModel(
            // NSColorSpaceModelRGB for RGB spaces only
            NSColorSpaceModel::RGB,
        );

    // Get the number of elements in the array
    let count = color_spaces.count();

    // Find the color space matching the name
    for i in 0..count {
        // Get the color space at index i
        let color_space = color_spaces.objectAtIndex(i);

        // Get the localized name of the color space
        if let Some(name_ns) = color_space.localizedName() {
            // Convert NSString to Rust String
            let name = name_ns.to_string();

            // Compare with the searched profile
            if name == profile_name {
                // Return the found color space
                return Some(color_space.clone());
            }
        }
    }

    // Profile not found
    None
}

/// Converts an RGB color from source color space to sRGB
///
/// # Arguments
/// * `r`, `g`, `b` - RGB components as u8 (0-255)
/// * `source_colorspace` - Source color space (or None for Auto)
///
/// # Returns
/// * `(u8, u8, u8)` - RGB components converted to sRGB
#[cfg(target_os = "macos")]
pub fn convert_color_to_srgb(r: u8, g: u8, b: u8, source_colorspace: Option<&objc2_app_kit::NSColorSpace>) -> (u8, u8, u8) {
    // Import required types
    use objc2_app_kit::{NSColor, NSColorSpace};
    use std::ptr::NonNull;

    // If no source color space, return colors unchanged
    let source_cs = match source_colorspace {
        Some(cs) => cs,
        None => return (r, g, b),
    };

    // Unsafe block for Objective-C calls
    unsafe {
        // Convert u8 values to CGFloat (0.0 - 1.0)
        let r_f: f64 = r as f64 / 255.0;
        let g_f: f64 = g as f64 / 255.0;
        let b_f: f64 = b as f64 / 255.0;
        let a_f: f64 = 1.0;

        // Create components array [R, G, B, A]
        let components: [f64; 4] = [r_f, g_f, b_f, a_f];

        // Create NonNull pointer to components
        let components_ptr = NonNull::new(components.as_ptr() as *mut f64);

        // Check that pointer is valid
        let components_ptr = match components_ptr {
            Some(ptr) => ptr,
            None => return (r, g, b), // Return colors unchanged on failure
        };

        // Create a color in the source color space
        let source_color = NSColor::colorWithColorSpace_components_count(
            source_cs,
            components_ptr,
            4, // 4 components (RGBA)
        );

        // Get the destination sRGB color space
        let srgb_cs = NSColorSpace::sRGBColorSpace();

        // Convert the color to sRGB
        let srgb_color = match source_color.colorUsingColorSpace(&srgb_cs) {
            Some(c) => c,
            None => return (r, g, b), // Return colors unchanged if conversion fails
        };

        // Extract components from the converted color
        let srgb_r = srgb_color.redComponent();
        let srgb_g = srgb_color.greenComponent();
        let srgb_b = srgb_color.blueComponent();

        // Convert CGFloat values (0.0 - 1.0) to u8 (0 - 255)
        let r_out = (srgb_r * 255.0).round().clamp(0.0, 255.0) as u8;
        let g_out = (srgb_g * 255.0).round().clamp(0.0, 255.0) as u8;
        let b_out = (srgb_b * 255.0).round().clamp(0.0, 255.0) as u8;

        (r_out, g_out, b_out)
    }
}

/// Lists all available ICC profiles on Windows
#[cfg(target_os = "windows")]
fn get_system_color_spaces() -> Vec<ICCProfile> {
    // On Windows, return a static list for now
    // TODO: Implement ICC profile retrieval via Windows API
    vec![
        ICCProfile {
            name: "Auto".to_string(),
            description: "Automatic color space detection".to_string(),
            is_current: false,
        },
        ICCProfile {
            name: "sRGB".to_string(),
            description: "sRGB IEC61966-2.1 (Standard web)".to_string(),
            is_current: false,
        },
        ICCProfile {
            name: "Adobe RGB".to_string(),
            description: "Adobe RGB (1998)".to_string(),
            is_current: false,
        },
    ]
}

/// Lists all available ICC profiles on Linux
#[cfg(target_os = "linux")]
fn get_system_color_spaces() -> Vec<ICCProfile> {
    // On Linux, return a static list for now
    // TODO: Implement ICC profile retrieval via colord or similar
    vec![
        ICCProfile {
            name: "Auto".to_string(),
            description: "Automatic color space detection".to_string(),
            is_current: false,
        },
        ICCProfile {
            name: "sRGB".to_string(),
            description: "sRGB IEC61966-2.1 (Standard web)".to_string(),
            is_current: false,
        },
    ]
}

/// Fallback for other platforms
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn get_system_color_spaces() -> Vec<ICCProfile> {
    // Return only Auto for unsupported platforms
    vec![ICCProfile {
        name: "Auto".to_string(),
        description: "Automatic color space detection".to_string(),
        is_current: false,
    }]
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Lists all available ICC profiles
///
/// # Returns
/// * `Vec<ICCProfile>` - List of profiles with their selection state
#[tauri::command]
pub fn list_icc_profiles() -> Vec<ICCProfile> {
    // Get system profiles
    let mut profiles = get_system_color_spaces();

    // Get the currently selected profile
    if let Ok(selected) = SELECTED_PROFILE.lock() {
        // Determine the selected profile name (Auto by default)
        let current_name = selected.as_deref().unwrap_or("Auto");

        // Mark the selected profile as current
        for profile in &mut profiles {
            // Compare profile name with selected profile
            profile.is_current = profile.name == current_name;
        }
    }

    // Return the list of profiles
    profiles
}

/// Selects an ICC profile
///
/// # Arguments
/// * `profile_name` - Name of the profile to select
///
/// # Returns
/// * `Result<(), String>` - Ok if success, Err with message if failure
#[tauri::command]
pub fn select_icc_profile(profile_name: String) -> Result<(), String> {
    // Try to lock the mutex
    if let Ok(mut selected) = SELECTED_PROFILE.lock() {
        // Update the selected profile
        *selected = Some(profile_name.clone());

        // Debug log
        println!("ICC Profile selected: {}", profile_name);

        // Return success
        Ok(())
    } else {
        // Return error if mutex cannot be locked
        Err("Failed to lock profile mutex / Échec du verrouillage du mutex".to_string())
    }
}

/// Gets the currently selected ICC profile
///
/// # Returns
/// * `Option<String>` - Selected profile name or None
#[tauri::command]
pub fn get_selected_icc_profile() -> Option<String> {
    // Lock the mutex and return a copy of the selected profile
    SELECTED_PROFILE.lock().ok().and_then(|s| s.clone())
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Returns the currently selected ICC profile name (internal use)
///
/// # Returns
/// * The selected profile name or "Auto" as default
#[allow(dead_code)]
pub fn get_current_profile_name() -> String {
    // Lock the mutex to access the selected profile
    if let Ok(selected) = SELECTED_PROFILE.lock() {
        // Return the selected profile or "Auto" as default
        selected.as_deref().unwrap_or("Auto").to_string()
    } else {
        // On lock error, return "Auto"
        "Auto".to_string()
    }
}
