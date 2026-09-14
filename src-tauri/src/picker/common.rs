//! =============================================================================
//! COMMON.RS - Shared code between platforms
//! =============================================================================
//!
//! This module contains types and functions used by both macOS and Windows.

use serde::Serialize;

// =============================================================================
// RESULT STRUCTURES
// =============================================================================

/// Result returned by the color picker
/// 
/// Contains selected colors for foreground and background.
#[derive(Serialize, Clone, Debug, Default)]
pub struct ColorPickerResult {
    /// Foreground color - RGB
    pub foreground: Option<(u8, u8, u8)>,
    
    /// Background color - RGB
    pub background: Option<(u8, u8, u8)>,
    
    /// Indicates if continue mode was enabled
    pub continue_mode: bool,
}

// =============================================================================
// COLOR CALCULATION FUNCTIONS
// =============================================================================

/// Calculates the relative luminance of an RGB color
/// 
/// Uses the standard ITU-R BT.601 formula:
/// Y = 0.299 * R + 0.587 * G + 0.114 * B
/// 
/// # Arguments
/// * `r` - Red component (0-255)
/// * `g` - Green component (0-255)
/// * `b` - Blue component (0-255)
/// 
/// # Returns
/// Luminance between 0.0 (black) and 255.0 (white)
#[inline]
fn calculate_luminance(r: u8, g: u8, b: u8) -> f64 {
    0.299 * (r as f64) + 0.587 * (g as f64) + 0.114 * (b as f64)
}

/// Determines if text should be black or white based on background color
/// 
/// # Arguments
/// * `r`, `g`, `b` - Background color
/// 
/// # Returns
/// `true` if text should be black, `false` if white
#[inline]
pub fn should_use_dark_text(r: u8, g: u8, b: u8) -> bool {
    calculate_luminance(r, g, b) > 128.0
}

// =============================================================================
// FORMATTING FUNCTIONS
// =============================================================================

/// Formats an RGB color as a hex string
/// 
/// # Arguments
/// * `r`, `g`, `b` - RGB components
/// 
/// # Returns
/// String in "#RRGGBB" format
#[inline]
pub fn format_hex_color(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// Formats a color with a prefix (Foreground/Background)
/// 
/// # Arguments
/// * `prefix` - Prefix ("Foreground" or "Background")
/// * `r`, `g`, `b` - RGB components
/// 
/// # Returns
/// String in "Prefix - #RRGGBB" format
#[inline]
pub fn format_labeled_hex_color(prefix: &str, r: u8, g: u8, b: u8) -> String {
    format!("{} - #{:02X}{:02X}{:02X}", prefix, r, g, b)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_luminance() {
        // Black
        assert!((calculate_luminance(0, 0, 0) - 0.0).abs() < 0.001);
        // White
        assert!((calculate_luminance(255, 255, 255) - 255.0).abs() < 0.001);
        // Pure red
        assert!((calculate_luminance(255, 0, 0) - 76.245).abs() < 0.001);
    }
    
    #[test]
    fn test_dark_text() {
        // White background -> black text (dark)
        assert!(should_use_dark_text(255, 255, 255));
        // Black background -> white text (not dark)
        assert!(!should_use_dark_text(0, 0, 0));
    }
    
    #[test]
    fn test_format_hex() {
        assert_eq!(format_hex_color(255, 0, 128), "#FF0080");
        assert_eq!(format_hex_color(0, 0, 0), "#000000");
    }
    
    #[test]
    fn test_format_labeled() {
        assert_eq!(format_labeled_hex_color("Foreground", 255, 0, 0), "Foreground - #FF0000");
        assert_eq!(format_labeled_hex_color("Background", 0, 255, 0), "Background - #00FF00");
    }
}