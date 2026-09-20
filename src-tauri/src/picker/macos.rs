//! =============================================================================
//! MACOS.RS - macOS implementation of the Color Picker
//! =============================================================================
//!
//! This module contains all macOS-specific code using Cocoa and Core Graphics.
//! It creates a full-screen overlay window that captures the screen and displays
//! a magnified view of the pixels around the cursor.
//!
//! # Architecture
//! - ColorPickerView: Custom NSView that handles drawing and events
//! - KeyableWindow: Custom window that can become the key window
//! - Global state: Mutex to share data between callbacks
//!
//! # Execution flow
//! 1. run() creates the application and the overlay windows
//! 2. Mouse/keyboard events are captured by ColorPickerView
//! 3. On each movement, the color is extracted and the view redrawn
//! 4. Click or Enter terminates the application and returns the color

// =============================================================================
// IMPORTS
// =============================================================================

// -----------------------------------------------------------------------------
// Modern Objective-C bindings (objc2 crate)
// -----------------------------------------------------------------------------
// Modern type-safe API for declaring Objective-C classes in Rust
use objc2::{define_class, msg_send, ClassType, MainThreadOnly}; // Class declaration macros
use objc2::rc::{Allocated, Retained};  // Smart pointers for ObjC objects

// Foundation types (equivalent of the ObjC standard library)
use objc2_foundation::{
    MainThreadMarker,    // Marker to guarantee execution on the main thread
    NSAffineTransform,    // 2D transforms (rotation, translation, scale)
    NSCopying,           // Copy protocol
    NSPoint,             // 2D point (x, y)
    NSRect,              // Rectangle (origin + size)
    NSSize,              // 2D size (width, height)
    NSString,            // Objective-C string
};

// AppKit types (macOS UI framework)
use objc2_app_kit::{
    NSAffineTransformNSAppKitAdditions,   // AppKit extensions for NSAffineTransform
    NSApplication,                       // Main application
    NSApplicationActivationOptions,      // Activation options (ActivateAllWindows, etc.)
    NSApplicationActivationPolicy,       // Activation policy (Regular, Accessory, etc.)
    NSBezierPath,                        // Vector paths for drawing
    NSColor,                             // Colors
    NSCursor,                            // Mouse cursor
    NSEvent,                             // Events (mouse, keyboard, etc.)
    NSEventModifierFlags,                 // Modifiers (Shift, Ctrl, etc.)
    NSFont,                              // Fonts
    NSGraphicsContext,                   // Drawing context
    NSRunningApplication,                // Running application
    NSScreen,                            // Screen (to get dimensions)
    NSStringDrawing,                     // Extension to draw text
    NSView,                              // Base view
    NSWindow as NSWindow2,               // Window (renamed to avoid conflict)
    NSWindowCollectionBehavior,          // Behavior with respect to Spaces / fullscreen
    NSWindowSharingType,                 // Window sharing type (None, ReadOnly, ReadWrite)
    NSWindowStyleMask,                   // Window styles (Borderless, etc.)
};

// -----------------------------------------------------------------------------
// Core Graphics (screen capture and image manipulation)
// -----------------------------------------------------------------------------
use core_graphics::display::CGDisplay; // Screen access
use core_graphics::image::CGImage;     // Bitmap images

// -----------------------------------------------------------------------------
// Rust standard library
// -----------------------------------------------------------------------------
use std::sync::Mutex; // Mutex for thread-safe synchronization

// -----------------------------------------------------------------------------
// Shared configuration
// -----------------------------------------------------------------------------
// Import all constants from the config module
use crate::config::*;

// -----------------------------------------------------------------------------
// Common code shared between platforms
// -----------------------------------------------------------------------------
use super::common::{
    ColorPickerResult,
    should_use_dark_text,
    format_hex_color,
    format_labeled_hex_color,
};

// =============================================================================
// TYPE ALIASES AND CONSTANTS
// =============================================================================

/// Bool type from objc2 for Objective-C booleans
/// Replaces objc::runtime::BOOL which is less type-safe
use objc2::runtime::Bool;

// =============================================================================
// UTILITY FUNCTIONS (declared before define_class! to be accessible)
// =============================================================================

/// Gets the scale factor of the screen at the given position
///
/// # Arguments
/// * `screen_x` - X coordinate in global Cocoa coordinates
/// * `screen_y` - Y coordinate in global Cocoa coordinates
///
/// # Returns
/// * `f64` - The scale factor (2.0 for Retina, 1.0 otherwise)
fn get_scale_factor_at_position(screen_x: f64, screen_y: f64) -> f64 {
    // Get main thread marker
    let Some(mtm) = MainThreadMarker::new() else {
        return 2.0; // Default Retina
    };
    
    // Get list of screens
    let screens = NSScreen::screens(mtm);
    let count = screens.count();
    
    // Iterate through all screens to find the one containing the point
    for i in 0..count {
        unsafe {
            let screen: Retained<NSScreen> = msg_send![&*screens, objectAtIndex: i];
            let frame = screen.frame();
            let scale = screen.backingScaleFactor();
            
            // Check if point is in this screen
            // In Cocoa, frame.origin is the bottom-left corner
            if screen_x >= frame.origin.x 
                && screen_x <= frame.origin.x + frame.size.width
                && screen_y >= frame.origin.y 
                && screen_y <= frame.origin.y + frame.size.height 
            {
                return scale;
            }
        }
    }
    
    // Fallback: return main screen factor
    if let Some(main_screen) = NSScreen::mainScreen(mtm) {
        return main_screen.backingScaleFactor();
    }
    
    2.0 // Default Retina
}

/// Smart refresh of picker overlay windows
/// 
/// Instead of refreshing all windows (causing flicker),
/// this function only refreshes windows affected by the change
fn refresh_picker_windows_smart(old_screen_x: f64, old_screen_y: f64, new_screen_x: f64, new_screen_y: f64) {
    // Get main thread marker
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    
    // Get shared application
    let app = NSApplication::sharedApplication(mtm);
    
    // Get all windows and screens
    let windows = app.windows();
    let window_count = windows.count();
    let screens = NSScreen::screens(mtm);
    let screen_count = screens.count();
    
    // Find screens containing old and new positions
    let mut old_screen_index: Option<usize> = None;
    let mut new_screen_index: Option<usize> = None;
    
    for i in 0..screen_count {
        unsafe {
            let screen: Retained<NSScreen> = msg_send![&*screens, objectAtIndex: i];
            let frame = screen.frame();
            
            // Check if old position is in this screen
            if old_screen_x >= frame.origin.x 
                && old_screen_x <= frame.origin.x + frame.size.width
                && old_screen_y >= frame.origin.y 
                && old_screen_y <= frame.origin.y + frame.size.height 
            {
                old_screen_index = Some(i);
            }
            
            // Check if new position is in this screen
            if new_screen_x >= frame.origin.x 
                && new_screen_x <= frame.origin.x + frame.size.width
                && new_screen_y >= frame.origin.y 
                && new_screen_y <= frame.origin.y + frame.size.height 
            {
                new_screen_index = Some(i);
            }
        }
    }
    
    // If on same screen, only refresh that window
    if old_screen_index == new_screen_index {
        if let Some(screen_idx) = new_screen_index {
            // Only refresh window for this screen
            unsafe {
                if screen_idx < window_count {
                    let window: Retained<NSWindow2> = msg_send![&*windows, objectAtIndex: screen_idx];
                    if window.level() == 1000 {
                        if let Some(content_view) = window.contentView() {
                            content_view.setNeedsDisplay(true);
                            content_view.displayIfNeeded();
                        }
                    }
                }
            }
        }
    } else {
        // Screen change: refresh old AND new
        for idx in [old_screen_index, new_screen_index].iter().filter_map(|&x| x) {
            unsafe {
                if idx < window_count {
                    let window: Retained<NSWindow2> = msg_send![&*windows, objectAtIndex: idx];
                    if window.level() == 1000 {
                        if let Some(content_view) = window.contentView() {
                            content_view.setNeedsDisplay(true);
                            content_view.displayIfNeeded();
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// CUSTOM OBJECTIVE-C CLASSES
// =============================================================================

// -----------------------------------------------------------------------------
// ColorPickerView - Custom view for the color picker
// -----------------------------------------------------------------------------

// New define_class! macro syntax for objc2 0.6+
define_class!(
    // SAFETY:
    // - The superclass NSView does not have any subclassing requirements that we violate.
    // - ColorPickerView does not implement Drop.
    #[unsafe(super = NSView)]                    // Inherit from NSView (parent class)
    #[thread_kind = MainThreadOnly]              // Can only be used on the main thread
    #[name = "ColorPickerView"]                  // Objective-C class name

    /// Custom view that handles all rendering and events for the color picker
    pub struct ColorPickerView;

    // Implementation of Objective-C methods
    impl ColorPickerView {
        // ---------------------------------------------------------------------
        // acceptsFirstResponder - Allows the view to receive keyboard events
        // ---------------------------------------------------------------------
        /// Indicates that this view can become the "first responder"
        /// Required to receive keyboard events
        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true // Yes, this view accepts being the first responder
        }

        // ---------------------------------------------------------------------
        // acceptsFirstMouse: - Accepts first mouse click without activation
        // ---------------------------------------------------------------------
        /// Indicates that this view accepts the first mouse click
        /// Without this, the first click would only activate the window,
        /// and a second click would be needed to trigger mouseDown:
        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _event: &NSEvent) -> bool {
            true // Yes, always accept the first mouse click
        }

        // ---------------------------------------------------------------------
        // mouseDown: - Handles mouse clicks
        // ---------------------------------------------------------------------
        /// Called when the user clicks with the mouse
        /// In normal mode: saves the current color and terminates the application
        /// In continue mode: saves the current color and toggles fg/bg
        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, _event: &NSEvent) {
            // Check if continue mode is enabled
            let is_continue_mode = if let Ok(mode) = CONTINUE_MODE.lock() {
                *mode // Copy the boolean value
            } else {
                false // Default to disabled if lock fails
            };

            // Get the current fg mode
            let is_fg_mode = if let Ok(mode) = FG_MODE.lock() {
                *mode // Copy the boolean value
            } else {
                true // Default to fg if lock fails
            };

            // Check if the opposite color has already been selected (= we already toggled)
            // If fg_mode=true, check BG_COLOR. If fg_mode=false, check FG_COLOR.
            let has_already_toggled = if is_fg_mode {
                // We're in fg mode, check if bg was already selected
                if let Ok(bg) = BG_COLOR.lock() {
                    bg.is_some() // True if background color exists
                } else {
                    false
                }
            } else {
                // We're in bg mode, check if fg was already selected
                if let Ok(fg) = FG_COLOR.lock() {
                    fg.is_some() // True if foreground color exists
                } else {
                    false
                }
            };

            // Lock the mutex to access the mouse state
            if let Ok(state) = MOUSE_STATE.lock() {
                // If we have information about the current color
                if let Some(ref info) = *state {
                    // Store the color in the appropriate variable based on fg_mode
                    if is_fg_mode {
                        // Store in FG_COLOR
                        if let Ok(mut fg_color) = FG_COLOR.lock() {
                            *fg_color = Some((info.r, info.g, info.b));
                        }
                    } else {
                        // Store in BG_COLOR
                        if let Ok(mut bg_color) = BG_COLOR.lock() {
                            *bg_color = Some((info.r, info.g, info.b));
                        }
                    }
                }
            }

            if is_continue_mode && !has_already_toggled {
                // Continue mode, first click: toggle between fg and bg
                if let Ok(mut fg_mode) = FG_MODE.lock() {
                    *fg_mode = !*fg_mode; // Toggle fg mode
                }
                // Request a refresh to update the display
                self.setNeedsDisplay(true);
            } else {
                // Normal mode OR continue mode after toggle: stop the application
                stop_application();
            }
        }

        // ---------------------------------------------------------------------
        // mouseMoved: - Handles mouse movements
        // ---------------------------------------------------------------------
        /// Called when the mouse moves
        /// Updates the position and color, then redraws
        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            // Get the mouse position in window coordinates
            let location: NSPoint = event.locationInWindow();

            // Get the parent window of this view
            let window_opt: Option<Retained<NSWindow2>> = self.window();

            // If we have a valid window
            if let Some(window) = window_opt {
                // Convert window coordinates to screen coordinates
                let screen_location: NSPoint = window.convertPointToScreen(location);

                // MULTI-SCREEN FIX:
                // Use the scale factor of the screen where the cursor is located,
                // not the one from the window receiving the event
                let scale_factor: f64 = get_scale_factor_at_position(screen_location.x, screen_location.y);

                // Get captured pixels count for capture size
                let captured_pixels = match CURRENT_CAPTURED_PIXELS.lock() {
                    Ok(p) => *p,
                    Err(_) => CAPTURED_PIXELS,
                };
                
                // Capture size in points (adjusted for Retina)
                let capture_size = captured_pixels / scale_factor;

                // Capture the area and extract the center pixel color
                if let Some((_image, r, g, b)) = capture_and_get_center_color(screen_location.x, screen_location.y, capture_size, captured_pixels) {
                    // Format the color in hexadecimal (#RRGGBB)
                    // Uses format_hex_color from common module
                    let hex_color = format_hex_color(r, g, b);

                    // Update the global state
                    if let Ok(mut state) = MOUSE_STATE.lock() {
                        // Create the new state structure
                        *state = Some(MouseColorInfo {
                            x: location.x,           // X position in window
                            y: location.y,           // Y position in window
                            screen_x: screen_location.x, // X position on screen
                            screen_y: screen_location.y, // Y position on screen
                            r,                       // Red component [0-255]
                            g,                       // Green component [0-255]
                            b,                       // Blue component [0-255]
                            hex_color: hex_color.clone(), // Hex code "#RRGGBB"
                            scale_factor,            // Retina scale factor
                        });
                    }

                    // Request a display refresh
                    self.setNeedsDisplay(true);
                }
            }
        }

        // ---------------------------------------------------------------------
        // scrollWheel: - Handles scroll wheel
        // ---------------------------------------------------------------------
        /// Called when the user uses the scroll wheel
        /// Without Shift: adjusts zoom level
        /// With Shift: adjusts captured pixels count
        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            // Get the vertical delta of the scroll wheel
            let delta_y: f64 = event.deltaY();

            // If the wheel moved
            if delta_y != 0.0 {
                // Get modifier flags to check for Shift
                let modifier_flags: NSEventModifierFlags = event.modifierFlags();
                let shift_pressed = modifier_flags.contains(NSEventModifierFlags::Shift);

                if shift_pressed {
                    // Shift + wheel: adjust captured pixels count
                    if let Ok(mut pixels) = CURRENT_CAPTURED_PIXELS.lock() {
                        // Calculate new value (inverted direction for intuitive UX)
                        let direction = if delta_y > 0.0 { 1.0 } else { -1.0 };
                        let new_pixels = *pixels + direction * CAPTURED_PIXELS_STEP;
                        // Clamp between min and max
                        *pixels = new_pixels.clamp(CAPTURED_PIXELS_MIN, CAPTURED_PIXELS_MAX);
                    }
                } else {
                    // Wheel alone: adjust zoom
                    if let Ok(mut zoom) = CURRENT_ZOOM.lock() {
                        // Calculate new zoom by adding delta * zoom step
                        let new_zoom = *zoom + delta_y * ZOOM_STEP;
                        // Clamp zoom between ZOOM_MIN and ZOOM_MAX
                        *zoom = new_zoom.clamp(ZOOM_MIN, ZOOM_MAX);
                    }
                }

                // Request a refresh to display the change
                self.setNeedsDisplay(true);
            }
        }

        // ---------------------------------------------------------------------
        // keyDown: - Handles keyboard keys
        // ---------------------------------------------------------------------
        /// Called when a key is pressed
        /// Handles ESC (cancel), Enter (confirm), and arrows (move)
        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            // Get the key code of the pressed key
            let key_code: u16 = event.keyCode();
            // Get the modifiers (Shift, Ctrl, etc.)
            let modifier_flags: NSEventModifierFlags = event.modifierFlags();

            // Check if Shift is pressed
            // In objc2-app-kit 0.3, the constant is NSEventModifierFlags::Shift
            let shift_pressed = modifier_flags.contains(NSEventModifierFlags::Shift);
            
            // Get the scale factor to adjust movement for Retina displays
            // On Retina (scale_factor=2.0), 1 pixel = 0.5 point
            let scale_factor = if let Ok(state) = MOUSE_STATE.lock() {
                if let Some(ref info) = *state {
                    info.scale_factor
                } else {
                    1.0
                }
            } else {
                1.0
            };
            
            // Determine movement distance in points
            // 1 pixel = 1/scale_factor points
            // Without Shift: 1 pixel, with Shift: SHIFT_MOVE_PIXELS pixels
            let pixels_to_move = if shift_pressed { SHIFT_MOVE_PIXELS } else { 1.0 };
            let move_amount = pixels_to_move / scale_factor;

            // Key codes: ESC = 53, Enter/Return = 36, C = 8, I = 34, O = 31
            if key_code == 53 {
                // ESC - Cancel the selection
                stop_application();
            } else if key_code == 36 {
                // Enter - Confirm the selection and exit
                // Get the current fg mode
                let is_fg_mode = if let Ok(mode) = FG_MODE.lock() {
                    *mode
                } else {
                    true
                };

                if let Ok(state) = MOUSE_STATE.lock() {
                    if let Some(ref info) = *state {
                        // Store the color in the appropriate variable based on fg_mode
                        if is_fg_mode {
                            if let Ok(mut fg_color) = FG_COLOR.lock() {
                                *fg_color = Some((info.r, info.g, info.b));
                            }
                        } else {
                            if let Ok(mut bg_color) = BG_COLOR.lock() {
                                *bg_color = Some((info.r, info.g, info.b));
                            }
                        }
                    }
                }
                stop_application();
            } else if key_code == 8 {
                // C key - Toggle continue mode
                if let Ok(mut continue_mode) = CONTINUE_MODE.lock() {
                    *continue_mode = !*continue_mode; // Toggle the mode
                }
                // Request a refresh to update the display
                self.setNeedsDisplay(true);
            } else if key_code == 34 {
                // I key - Zoom in or increase captured pixels
                if shift_pressed {
                    // Shift+I: increase captured pixels count
                    if let Ok(mut pixels) = CURRENT_CAPTURED_PIXELS.lock() {
                        *pixels = (*pixels + CAPTURED_PIXELS_STEP).min(CAPTURED_PIXELS_MAX);
                    }
                } else {
                    // I alone: zoom in
                    if let Ok(mut zoom) = CURRENT_ZOOM.lock() {
                        *zoom = (*zoom + ZOOM_STEP).min(ZOOM_MAX);
                    }
                }
                // Request a refresh to update the display
                self.setNeedsDisplay(true);
            } else if key_code == 31 {
                // O key - Zoom out or decrease captured pixels
                if shift_pressed {
                    // Shift+O: decrease captured pixels count
                    if let Ok(mut pixels) = CURRENT_CAPTURED_PIXELS.lock() {
                        *pixels = (*pixels - CAPTURED_PIXELS_STEP).max(CAPTURED_PIXELS_MIN);
                    }
                } else {
                    // O alone: zoom out
                    if let Ok(mut zoom) = CURRENT_ZOOM.lock() {
                        *zoom = (*zoom - ZOOM_STEP).max(ZOOM_MIN);
                    }
                }
                // Request a refresh to update the display
                self.setNeedsDisplay(true);
            } else {
                // Arrow key codes: left=123, right=124, down=125, up=126
                let (dx, dy): (f64, f64) = match key_code {
                    123 => (-move_amount, 0.0),  // Left: move left
                    124 => (move_amount, 0.0),   // Right: move right
                    125 => (0.0, -move_amount),  // Down: move down
                    126 => (0.0, move_amount),   // Up: move up
                    _ => (0.0, 0.0),             // Other key: no movement
                };

                // If movement is requested
                if dx != 0.0 || dy != 0.0 {
                    // Move the cursor and update the state
                    if let Ok(state) = MOUSE_STATE.lock() {
                        if let Some(ref info) = *state {
                            // Copy needed values BEFORE releasing the lock
                            let old_screen_x = info.screen_x;
                            let old_screen_y = info.screen_y;
                            
                            // Calculate the new position (in points)
                            let new_x = old_screen_x + dx;
                            let new_y = old_screen_y + dy;

                            // Get scale factor of the screen at the NEW position
                            let scale_factor = get_scale_factor_at_position(new_x, new_y);

                            // Get captured pixels count for capture size
                            let captured_pixels = match CURRENT_CAPTURED_PIXELS.lock() {
                                Ok(p) => *p,
                                Err(_) => CAPTURED_PIXELS,
                            };
                            
                            // Capture size in points (adjusted for Retina)
                            let capture_size = captured_pixels / scale_factor;

                            // Get main screen height for Y conversion
                            // CGEvent uses global coordinates based on main screen (screens[0])
                            let screen_height_points = if let Some(mtm) = objc2_foundation::MainThreadMarker::new() {
                                let screens = NSScreen::screens(mtm);
                                if screens.count() > 0 {
                                    unsafe {
                                        let main_screen: Retained<NSScreen> = msg_send![&*screens, objectAtIndex: 0usize];
                                        main_screen.frame().size.height
                                    }
                                } else if let Some(main_screen) = NSScreen::mainScreen(mtm) {
                                    main_screen.frame().size.height
                                } else {
                                    let main_display = CGDisplay::main();
                                    main_display.pixels_high() as f64 / 2.0
                                }
                            } else {
                                let main_display = CGDisplay::main();
                                main_display.pixels_high() as f64 / 2.0
                            };

                            // Convert Cocoa coordinates (origin bottom-left, in points) to 
                            // Core Graphics coordinates (origin top-left, in points)
                            // CGEvent uses POINTS, not pixels
                            let cg_x = new_x;
                            let cg_y = screen_height_points - new_y;

                            // Move the mouse cursor using CGEvent (more reliable than warp)
                            use core_graphics::event::{CGEvent, CGEventType, CGMouseButton};
                            use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
                            use core_graphics::geometry::CGPoint as CGPointCG;
                            
                            if let Ok(source) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
                                let point = CGPointCG::new(cg_x, cg_y);
                                if let Ok(event) = CGEvent::new_mouse_event(
                                    source,
                                    CGEventType::MouseMoved,
                                    point,
                                    CGMouseButton::Left
                                ) {
                                    event.post(core_graphics::event::CGEventTapLocation::HID);
                                }
                            }

                            // Release the lock before getting the new color
                            drop(state);

                            // Capture the area and extract the center pixel color
                            if let Some((_image, r, g, b)) = capture_and_get_center_color(new_x, new_y, capture_size, captured_pixels) {
                                // Uses format_hex_color from common module
                                let hex_color = format_hex_color(r, g, b);

                                // Update the state with the new position and color
                                if let Ok(mut state) = MOUSE_STATE.lock() {
                                    if let Some(window) = self.window() {
                                        // Convert screen coordinates to window coordinates
                                        let screen_point = NSPoint::new(new_x, new_y);
                                        let window_point: NSPoint = window.convertPointFromScreen(screen_point);

                                        // Update the state
                                        *state = Some(MouseColorInfo {
                                            x: window_point.x,
                                            y: window_point.y,
                                            screen_x: new_x,
                                            screen_y: new_y,
                                            r,
                                            g,
                                            b,
                                            hex_color,
                                            scale_factor,
                                        });
                                    }
                                }

                                // CORRECTION CLIGNOTEMENT : Utilise un refresh intelligent
                                // FLICKER FIX: Use smart refresh
                                // Only refresh affected windows (old and new position)
                                refresh_picker_windows_smart(old_screen_x, old_screen_y, new_x, new_y);
                            }
                        }
                    }
                }
            }
        }

        // ---------------------------------------------------------------------
        // drawRect: - Draws the view content
        // ---------------------------------------------------------------------
        /// Called by the system when the view needs to be redrawn
        /// Delegates to the draw_view() function
        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _rect: NSRect) {
            // Call the main drawing function
            draw_view(self);
        }
    }
);

// -----------------------------------------------------------------------------
// KeyableWindow - Window that can receive keyboard events
// -----------------------------------------------------------------------------

// New define_class! macro syntax for objc2 0.6+
define_class!(
    // SAFETY:
    // - The superclass NSWindow does not have any subclassing requirements that we violate.
    // - KeyableWindow does not implement Drop.
    #[unsafe(super = NSWindow2)]                 // Inherit from NSWindow (parent class)
    #[thread_kind = MainThreadOnly]              // Can only be used on the main thread
    #[name = "KeyableWindow"]                    // Objective-C class name

    /// Custom window that can become the key window
    /// By default, borderless windows cannot become key windows
    pub struct KeyableWindow;

    impl KeyableWindow {
        // Override canBecomeKeyWindow to return true
        // Allows this borderless window to receive keyboard events
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true // Yes, this window can become the key window
        }
    }
);

// =============================================================================
// GLOBAL STATE
// =============================================================================

/// Global state protected by Mutex for mouse position and color
/// Mutex allows thread-safe access from the different callbacks
static MOUSE_STATE: Mutex<Option<MouseColorInfo>> = Mutex::new(None);

/// Global state for the current zoom level
/// Initialized with the default zoom factor
static CURRENT_ZOOM: Mutex<f64> = Mutex::new(INITIAL_ZOOM_FACTOR);

/// Global state for captured pixels count
/// Initialized with default value from config
static CURRENT_CAPTURED_PIXELS: Mutex<f64> = Mutex::new(CAPTURED_PIXELS);

/// Minimum captured pixels (must be odd)
const CAPTURED_PIXELS_MIN: f64 = 9.0;

/// Maximum captured pixels (must be odd)
const CAPTURED_PIXELS_MAX: f64 = 21.0;

/// Increment step for captured pixels (2 to stay odd)
const CAPTURED_PIXELS_STEP: f64 = 2.0;

/// Stores the selected foreground color
static FG_COLOR: Mutex<Option<(u8, u8, u8)>> = Mutex::new(None);

/// Stores the selected background color
static BG_COLOR: Mutex<Option<(u8, u8, u8)>> = Mutex::new(None);

/// Display mode: true = top arc (foreground), false = bottom arc (background)
static FG_MODE: Mutex<bool> = Mutex::new(true);

/// Continue mode: true = enabled, false = disabled
/// When enabled, a red "C" badge is displayed before the hex text
static CONTINUE_MODE: Mutex<bool> = Mutex::new(false);

// ColorPickerResult is now defined in common.rs

/// Structure containing all information about current position and color
struct MouseColorInfo {
    x: f64,          // X position in window coordinates
    y: f64,          // Y position in window coordinates
    screen_x: f64,   // X position in screen coordinates
    screen_y: f64,   // Y position in screen coordinates
    r: u8,           // Red component (0-255)
    g: u8,           // Green component (0-255)
    b: u8,           // Blue component (0-255)
    hex_color: String, // Hexadecimal color code (#RRGGBB)
    scale_factor: f64, // Screen scale factor (2.0 for Retina)
}

// =============================================================================
// SCREEN CAPTURE FUNCTIONS
// =============================================================================

/// Captures a square area of pixels around the given coordinates
///
/// # Arguments
/// * `x` - X coordinate of the center (Cocoa coordinates in points, origin at bottom-left)
/// * `y` - Y coordinate of the center (Cocoa coordinates in points)
/// * `size` - Size of the square to capture (in points)
///
/// # Returns
/// * `Some(CGImage)` - The captured image if capture succeeded
/// * `None` - If capture failed
fn capture_zoom_area(x: f64, y: f64, size: f64) -> Option<CGImage> {
    // Import Core Graphics geometry types
    use core_graphics::geometry::{CGRect, CGPoint as CGPointStruct, CGSize};

    // MULTI-SCREEN FIX: Find the screen that contains the given coordinates
    let mtm = objc2_foundation::MainThreadMarker::new()?;
    let screens = NSScreen::screens(mtm);
    let count = screens.count();
    
    // Variables to store the found screen information
    let mut target_screen_frame: Option<NSRect> = None;
    let mut target_display_id: Option<u32> = None;
    
    // Iterate through all screens to find the one containing the point
    for i in 0..count {
        unsafe {
            let screen: Retained<NSScreen> = msg_send![&*screens, objectAtIndex: i];
            let frame = screen.frame();
            
            // Check if point (x, y) is within this screen frame
            if x >= frame.origin.x 
                && x <= frame.origin.x + frame.size.width
                && y >= frame.origin.y 
                && y <= frame.origin.y + frame.size.height 
            {
                target_screen_frame = Some(frame);
                
                // Get the Core Graphics display ID for this screen
                // deviceDescription contains a dictionary with the display ID
                let device_desc = screen.deviceDescription();
                let ns_screen_number_key = NSString::from_str("NSScreenNumber");
                if let Some(screen_number) = device_desc.objectForKey(&ns_screen_number_key) {
                    // NSNumber to u32 via msg_send (dereference with &*)
                    let display_id: u32 = msg_send![&*screen_number, unsignedIntValue];
                    target_display_id = Some(display_id);
                }
                break;
            }
        }
    }
    
    // If no screen was found containing the point, use main screen as fallback
    let (screen_frame, display) = if let (Some(frame), Some(display_id)) = (target_screen_frame, target_display_id) {
        (frame, CGDisplay::new(display_id))
    } else {
        // Fallback to main screen
        let main_screen = NSScreen::mainScreen(mtm)?;
        let frame = main_screen.frame();
        (frame, CGDisplay::main())
    };
    
    // Height of the found screen in points (for coordinate conversion)
    let screen_height_points = screen_frame.size.height;
    
    // Convert global Cocoa coordinates to local screen coordinates
    // In Cocoa: origin at bottom-left, global coordinates
    // In CG: origin at top-left, local to screen
    
    // Local X coordinate to the screen (relative to screen origin)
    let local_x = x - screen_frame.origin.x;
    
    // Local Y coordinate, converted from Cocoa (bottom) to CG (top)
    let local_y_cocoa = y - screen_frame.origin.y;
    let local_y_cg = screen_height_points - local_y_cocoa;

    // Capture size in points
    let capture_size = size;
    let half_size = capture_size / 2.0;

    // Create capture rectangle centered on the point (local CG coordinates)
    let rect = CGRect::new(
        &CGPointStruct::new(local_x - half_size, local_y_cg - half_size),
        &CGSize::new(capture_size, capture_size)
    );

    // Capture the image in the specified rectangle on the correct screen
    display.image_for_rect(rect)
}

/// Extracts the center pixel color from a CGImage
/// and applies ICC conversion if a profile is selected
///
/// # Arguments
/// * `image` - The captured image
/// * `target_pixels` - Target pixel count for the center computation
///
/// # Returns
/// * `Some((r, g, b))` - RGB components as u8 [0-255] converted to sRGB
/// * `None` - If extraction failed
fn get_center_pixel_from_image(image: &CGImage, target_pixels: f64) -> Option<(u8, u8, u8)> {
    // Get image dimensions
    let img_width = image.width() as f64;
    let img_height = image.height() as f64;
    
    // Calculate crop offset like in draw_view
    // In draw_view, crop_x and crop_y are used for NSImage.drawInRect:fromRect:
    // where origin is at BOTTOM-left (Cocoa convention)
    // But CGImage data is stored with origin at TOP-left
    
    let crop_x = if img_width > target_pixels {
        ((img_width - target_pixels) / 2.0).floor()
    } else {
        0.0
    };
    
    // For NSImage (draw_view), crop_y is from bottom
    // For CGImage (data), we need to calculate from top
    let crop_y_from_bottom = if img_height > target_pixels {
        ((img_height - target_pixels) / 2.0).floor()
    } else {
        0.0
    };
    
    // Effective size after crop
    let use_width = if img_width > target_pixels { target_pixels } else { img_width };
    let use_height = if img_height > target_pixels { target_pixels } else { img_height };
    
    // Center pixel in X (same convention)
    let center_x = (crop_x + use_width / 2.0).floor() as usize;
    
    // Center pixel in Y: convert from "from bottom" to "from top"
    // In NSImage: center is at crop_y_from_bottom + use_height/2 from bottom
    // In CGImage: we want distance from top
    let center_y_from_bottom = crop_y_from_bottom + use_height / 2.0;
    let center_y = (img_height - center_y_from_bottom).floor() as usize;
    
    // Get raw image data
    let data = image.data();
    let bytes_per_row = image.bytes_per_row() as usize;
    let bits_per_pixel = image.bits_per_pixel() as usize;
    let bytes_per_pixel = bits_per_pixel / 8;
    
    // Calculate center pixel offset in data
    let offset = (center_y * bytes_per_row) + (center_x * bytes_per_pixel);
    
    // Check we have enough data
    let data_len = data.len() as usize;
    if offset + bytes_per_pixel <= data_len {
        // Data is in BGRA format (Blue, Green, Red, Alpha)
        let b = data[offset];
        let g = data[offset + 1];
        let r = data[offset + 2];
        
        // Apply ICC conversion if a profile is selected
        let (r_out, g_out, b_out) = apply_icc_conversion(r, g, b);
        
        Some((r_out, g_out, b_out))
    } else {
        None
    }
}

/// Applies ICC conversion from selected profile to sRGB
///
/// # Arguments
/// * `r`, `g`, `b` - Raw captured RGB components
///
/// # Returns
/// * `(u8, u8, u8)` - RGB components converted to sRGB
fn apply_icc_conversion(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    // Import icc module for conversion
    use crate::icc;
    
    // Get the selected color space
    let source_colorspace = icc::get_selected_nscolorspace();
    
    // Convert color to sRGB using the selected profile
    icc::convert_color_to_srgb(r, g, b, source_colorspace.as_deref())
}

/// Captures an area and returns both the image and the center pixel color
///
/// # Arguments
/// * `x` - X coordinate of the center (Cocoa coordinates in points)
/// * `y` - Y coordinate of the center (Cocoa coordinates in points)
/// * `size` - Size of the square to capture (in points)
/// * `target_pixels` - Target pixel count for the crop (used to find the center)
///
/// # Returns
/// * `Some((CGImage, r, g, b))` - The image and the RGB components of the center pixel
/// * `None` - If capture failed
fn capture_and_get_center_color(x: f64, y: f64, size: f64, target_pixels: f64) -> Option<(CGImage, u8, u8, u8)> {
    // Capture the area
    let image = capture_zoom_area(x, y, size)?;
    
    // Extract the center pixel color (taking the crop into account)
    let (r, g, b) = get_center_pixel_from_image(&image, target_pixels)?;
    
    Some((image, r, g, b))
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Global flag to signal picker stop
static SHOULD_STOP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Helper function to stop the picker and show cursor again
fn stop_application() {
    // Signal stop via the atomic flag
    SHOULD_STOP.store(true, std::sync::atomic::Ordering::SeqCst);
    
    // Show the mouse cursor again
    NSCursor::unhide();
}

/// Runs the color picker application on macOS
///
/// # Arguments
/// * `fg` - If true, starts in foreground mode. If false, starts in background mode.
///
/// # Returns
/// * `ColorPickerResult` with foreground and/or background filled based on selections
/// * Both fields are None if user pressed ESC to cancel
pub fn run(fg: bool) -> ColorPickerResult {
    // Store the fg mode in the global variable
    if let Ok(mut mode) = FG_MODE.lock() {
        *mode = fg; // Set the display mode (true = top arc, false = bottom arc)
    }

    // Reset the selected colors
    if let Ok(mut color) = FG_COLOR.lock() {
        *color = None; // Clear any previously selected foreground color
    }
    if let Ok(mut color) = BG_COLOR.lock() {
        *color = None; // Clear any previously selected background color
    }

    // Reset the continue mode
    if let Ok(mut mode) = CONTINUE_MODE.lock() {
        *mode = false; // Disable continue mode at start
    }

    // Reset captured pixels to default value
    if let Ok(mut pixels) = CURRENT_CAPTURED_PIXELS.lock() {
        *pixels = CAPTURED_PIXELS; // Reset to default from config
    }

    // Reset zoom to default value
    if let Ok(mut zoom) = CURRENT_ZOOM.lock() {
        *zoom = INITIAL_ZOOM_FACTOR; // Reset to default from config
    }

    // Reset the stop flag
    SHOULD_STOP.store(false, std::sync::atomic::Ordering::SeqCst);

    // Get main thread marker - required for UI operations
    let mtm = MainThreadMarker::new().expect("Must be called from main thread");

    // Get the shared application instance
    let app = NSApplication::sharedApplication(mtm);

    // During the picker activation, switch to Accessory: lets us activate the app
    // without leaving the Space of another app's full-screen. The policy is
    // restored to Regular at the end of this function. (Fixes #26)
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Create overlay windows for each screen
    unsafe {
        // Get the list of all screens using native objc2 API
        let screens = NSScreen::screens(mtm); // Returns Retained<NSArray<NSScreen>>

        // Iterate over each screen in the array via native API
        // Note: NSArray doesn't have a get() method in objc2, use objectAtIndex: via msg_send
        let count: usize = screens.count(); // Get the number of screens
        for i in 0..count {
            // Get the screen at index i via objectAtIndex:
            let screen: Retained<NSScreen> = msg_send![&*screens, objectAtIndex: i];
            // Get the screen dimensions using native objc2 API
            let frame: NSRect = screen.frame(); // Returns NSRect directly

            // Create KeyableWindow using native objc2 API
            // For MainThreadOnly classes, use mtm.alloc::<Class>() pattern
            let window: Retained<KeyableWindow> = {
                // Allocate the window object using MainThreadMarker for MainThreadOnly classes
                let allocated: Allocated<KeyableWindow> = mtm.alloc(); // Allocate memory for the window
                // NSBackingStoreType: 0 = Retained, 1 = Nonretained, 2 = Buffered
                const NS_BACKING_STORE_BUFFERED: u64 = 2; // NSBackingStoreBuffered
                // Initialize with content rect, style mask, backing store type, and defer flag
                // Use msg_send! for init methods that return Retained
                let initialized: Retained<KeyableWindow> = {
                    msg_send![
                        allocated,
                        initWithContentRect: frame,                      // Window frame rectangle
                        styleMask: NSWindowStyleMask::Borderless,        // No border style
                        backing: NS_BACKING_STORE_BUFFERED,              // Buffered backing store
                        defer: Bool::NO                                  // Don't defer window creation
                    ]
                };
                initialized // Return the initialized window
            };

            // Cast KeyableWindow to NSWindow2 to access NSWindow methods
            // KeyableWindow inherits from NSWindow2 so this cast is safe
            let window_as_nswindow: &NSWindow2 = &window; // Deref coercion to parent class

            // Configure the window using NSWindow2 methods
            window_as_nswindow.setLevel(1000);                        // Very high level (above everything)

            // 256 = FullScreenAuxiliary. (Fixes #26)
            // Lets the window show on every Space (including full-screen apps').
            // 1 = CanJoinAllSpaces, 16 = Stationary, 256 = FullScreenAuxiliary. (Fixes #26)
            window_as_nswindow.setCollectionBehavior(NSWindowCollectionBehavior(1 | 16 | 256));

            let clear_color = NSColor::clearColor();                  // Transparent color
            window_as_nswindow.setBackgroundColor(Some(&clear_color)); // Transparent background

            window_as_nswindow.setOpaque(false);                      // Non-opaque
            window_as_nswindow.setHasShadow(false);                   // No shadow
            window_as_nswindow.setIgnoresMouseEvents(false);          // Receives mouse events
            window_as_nswindow.setAcceptsMouseMovedEvents(true);      // Receives mouseMoved
            
            // DOUBLE-CLICK FIX: Configure window to accept first mouse click
            // Without this, the first click just activates the window,
            // and a second click is needed to trigger the action
            // Note: setAcceptsFirstMouse is not available via objc2-app-kit,
            //       but makeKeyAndOrderFront + hidesOnDeactivate solves the issue
            window_as_nswindow.setHidesOnDeactivate(false);           // Does not hide when deactivated
            
            // Disable window content sharing (prevents screen capture of this window)
            // NSWindowSharingType: 0 = None, 1 = ReadOnly, 2 = ReadWrite
            window_as_nswindow.setSharingType(NSWindowSharingType(0));

            // Create ColorPickerView using native objc2 API
            // For MainThreadOnly classes, use mtm.alloc::<Class>() pattern
            let view: Retained<ColorPickerView> = {
                // Allocate the view object using MainThreadMarker for MainThreadOnly classes
                let allocated: Allocated<ColorPickerView> = mtm.alloc(); // Allocate memory for the view
                // Initialize with frame - use msg_send! for init methods that return Retained
                let initialized: Retained<ColorPickerView> = {
                    msg_send![allocated, initWithFrame: frame] // Initialize with frame
                };
                initialized // Return the initialized view
            };

            // Cast ColorPickerView to NSView for setContentView and makeFirstResponder
            // ColorPickerView inherits from NSView so this cast is safe
            let view_as_nsview: &NSView = &view; // Deref coercion to parent class

            // FLICKER FIX: Enable layer-backing for smooth rendering
            // Layer-backing uses Core Animation for automatic double-buffering
            view_as_nsview.setWantsLayer(true);
            
            // The view is opaque (no transparency in the view itself)
            // This allows AppKit to optimize rendering
            // Note: The semi-transparent overlay is drawn INSIDE the view, the view itself is opaque
            let _: () = msg_send![view_as_nsview, setOpaque: Bool::YES];

            // Configure the window with the view
            window_as_nswindow.setContentView(Some(view_as_nsview));  // Set the content view
            window_as_nswindow.makeKeyAndOrderFront(None);            // Show and bring to front
            window_as_nswindow.makeFirstResponder(Some(view_as_nsview)); // View receives events
        } // End of for loop
    } // End of unsafe block

    // Activate the application using native objc2 API
    {
        // Get the running application instance
        let running_app = NSRunningApplication::currentApplication();
        // Activate the application with default options (empty = standard activation)
        running_app.activateWithOptions(NSApplicationActivationOptions::empty());
    }

    // Initialize MOUSE_STATE with the current mouse position
    {
        use core_graphics::event::CGEvent;
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
        
        // Get current mouse position via Core Graphics
        if let Ok(source) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
            let cg_event = CGEvent::new(source);
            if let Ok(event) = cg_event {
                // CGEvent.location() returns coordinates in POINTS (Global Display Coordinates)
                // with origin at top-left
                let cg_point = event.location();
                
                // Get the scale factor and height in points
                let scale_factor = if let Some(main_screen) = NSScreen::mainScreen(mtm) {
                    main_screen.backingScaleFactor()
                } else {
                    2.0 // Default to Retina
                };
                
                let screen_height_points = if let Some(main_screen) = NSScreen::mainScreen(mtm) {
                    main_screen.frame().size.height
                } else {
                    let main_display = CGDisplay::main();
                    main_display.pixels_high() as f64 / scale_factor
                };
                
                // Convert CG (origin at top) to Cocoa (origin at bottom)
                let cocoa_x = cg_point.x;
                let cocoa_y = screen_height_points - cg_point.y;
                
                // Get captured pixels count for capture size
                let captured_pixels = match CURRENT_CAPTURED_PIXELS.lock() {
                    Ok(p) => *p,
                    Err(_) => CAPTURED_PIXELS,
                };
                
                // Capture size in points (adjusted for Retina)
                let capture_size = captured_pixels / scale_factor;
                
                // Capture the area and extract the center pixel color
                if let Some((_image, r, g, b)) = capture_and_get_center_color(cocoa_x, cocoa_y, capture_size, captured_pixels) {
                    // Uses format_hex_color from common module
                    let hex_color = format_hex_color(r, g, b);
                    
                    // Initialise MOUSE_STATE
                    // Initialize MOUSE_STATE
                    if let Ok(mut state) = MOUSE_STATE.lock() {
                        *state = Some(MouseColorInfo {
                            x: cocoa_x,        // X position in window coordinates
                            y: cocoa_y,        // Y position in window coordinates
                            screen_x: cocoa_x, // X position in screen coordinates
                            screen_y: cocoa_y, // Y position in screen coordinates
                            r,
                            g,
                            b,
                            hex_color,
                            scale_factor,
                        });
                    }
                }
            }
        }
    }

    // Hide the mouse cursor
    NSCursor::hide();

    // Custom event loop (instead of app.run() which would close Tauri)
    unsafe {
        use objc2_foundation::NSDate;

        while !SHOULD_STOP.load(std::sync::atomic::Ordering::SeqCst) {
            // Short timeout to check the flag regularly
            let timeout: Retained<NSDate> = msg_send![
                NSDate::class(),
                dateWithTimeIntervalSinceNow: 0.016f64  // ~60fps
            ];

            let event = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                objc2_app_kit::NSEventMask::Any,
                Some(&timeout),
                objc2_foundation::NSDefaultRunLoopMode,
                true
            );

            if let Some(event) = event {
                app.sendEvent(&event);
            }

            app.updateWindows();
        }
    }

    // Close all application windows that are at level 1000 (our picker windows)
    unsafe {
        let windows = app.windows();
        let count: usize = windows.count();
        for i in 0..count {
            let win: Retained<NSWindow2> = msg_send![&*windows, objectAtIndex: i];
            if win.level() == 1000 {
                win.orderOut(None);
            }
        }
    }

    // Restore the regular policy (Fixes #26)
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    // Get the selected colors
    let fg_color = if let Ok(color) = FG_COLOR.lock() {
        color.clone() // Clone the Option<(u8, u8, u8)>
    } else {
        None // Return None if lock fails
    };

    let bg_color = if let Ok(color) = BG_COLOR.lock() {
        color.clone() // Clone the Option<(u8, u8, u8)>
    } else {
        None // Return None if lock fails
    };

    // Get the continue mode state
    let was_continue_mode = if let Ok(mode) = CONTINUE_MODE.lock() {
        *mode // Copy the boolean value
    } else {
        false // Default to false if lock fails
    };

    // Build the result with both colors and continue mode
    ColorPickerResult {
        foreground: fg_color,       // Foreground color (may be None)
        background: bg_color,       // Background color (may be None)
        continue_mode: was_continue_mode, // Whether continue mode was enabled
    }
}

// =============================================================================
// DRAWING
// =============================================================================

/// Main drawing function called from ColorPickerView's drawRect
///
/// This function draws:
/// 1. A semi-transparent overlay over the whole screen
/// 2. The circular magnifier with the enlarged pixels
/// 3. The central reticle
/// 4. The colored border
/// 5. The hexadecimal text on an arc
fn draw_view(view: &NSView) {
    // -------------------------------------------------------------------------
    // Draw the semi-transparent overlay
    // -------------------------------------------------------------------------
    // Create a black color with 5% opacity
    let overlay_color = NSColor::colorWithCalibratedWhite_alpha(0.0, 0.0);
    // Set as fill color
    overlay_color.set();

    // Get the view bounds
    let bounds: NSRect = view.bounds();
    // Create a rectangular path covering the whole view
    let bounds_path = NSBezierPath::bezierPathWithRect(bounds);
    // Fill with the overlay color
    bounds_path.fill();

    // -------------------------------------------------------------------------
    // Draw the magnifier if we have mouse information
    // -------------------------------------------------------------------------
    if let Ok(state) = MOUSE_STATE.lock() {
        if let Some(ref info) = *state {
            // MULTI-SCREEN FIX: Check if cursor is in this window's screen
            let should_draw_magnifier = if let Some(window) = view.window() {
                if let Some(screen) = window.screen() {
                    let screen_frame = screen.frame();
                    // Check if cursor position is within this screen frame
                    info.screen_x >= screen_frame.origin.x
                        && info.screen_x <= screen_frame.origin.x + screen_frame.size.width
                        && info.screen_y >= screen_frame.origin.y
                        && info.screen_y <= screen_frame.origin.y + screen_frame.size.height
                } else {
                    true // Fallback: draw if we can't determine the screen
                }
            } else {
                true // Fallback: draw if we can't get the window
            };

            // Only draw magnifier if cursor is in this screen
            if should_draw_magnifier {
            // Get the current zoom
            let current_zoom = match CURRENT_ZOOM.lock() {
                Ok(z) => *z,
                Err(_) => INITIAL_ZOOM_FACTOR,
            };

            // Get the current captured pixels count
            let captured_pixels = match CURRENT_CAPTURED_PIXELS.lock() {
                Ok(p) => *p,
                Err(_) => CAPTURED_PIXELS, // Fallback to default constant
            };

            // Calculate the magnifier size to display
            // mag_size = captured pixels count × zoom factor
            let mag_size = captured_pixels * current_zoom;
            // Capture size adjusted for the Retina scale factor
            let capture_size = captured_pixels / info.scale_factor;

            // Capture the pixel area around the cursor
            if let Some(cg_image) = capture_zoom_area(info.screen_x, info.screen_y, capture_size) {
                // Captured image dimensions
                let img_width = cg_image.width() as f64;
                let img_height = cg_image.height() as f64;
                let target_pixels = captured_pixels;

                // Calculate the offset to center the crop
                let crop_x = if img_width > target_pixels {
                    ((img_width - target_pixels) / 2.0).floor()
                } else {
                    0.0
                };
                let crop_y = if img_height > target_pixels {
                    ((img_height - target_pixels) / 2.0).floor()
                } else {
                    0.0
                };

                // Effective size to use
                let use_width = if img_width > target_pixels { target_pixels } else { img_width };
                let use_height = if img_height > target_pixels { target_pixels } else { img_height };

                unsafe {
                    // -------------------------------------------------------------
                    // Create an NSImage from CGImage
                    // Note: initWithCGImage:size: is not directly available in objc2-app-kit,
                    // so we use raw msg_send! with proper type handling
                    // -------------------------------------------------------------
                    use objc2_app_kit::NSImage;
                    use objc2::runtime::AnyObject;
                    use objc2::ClassType;
                    use objc2::encode::{Encoding, RefEncode};
                    
                    // Define a wrapper type for CGImage with proper Objective-C encoding
                    // This represents the opaque CGImage struct (not the pointer)
                    #[repr(C)]
                    struct OpaqueImage {
                        _private: [u8; 0], // Zero-sized opaque type
                    }
                    
                    // Implement RefEncode to tell objc2 the correct type encoding
                    // When passed as *const OpaqueImage, this becomes "^{CGImage=}"
                    unsafe impl RefEncode for OpaqueImage {
                        const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("CGImage", &[]));
                    }
                    
                    // Get the CGImage pointer and cast it to our opaque type
                    let cg_image_ref: *const OpaqueImage = {
                        // CGImage from core-graphics is a wrapper around CFTypeRef
                        // We need to extract the raw pointer
                        let ptr_addr = &cg_image as *const CGImage as *const *const OpaqueImage;
                        *ptr_addr // Dereference to get the raw CGImageRef
                    };

                    // Use msg_send! to call alloc on NSImage class
                    // This returns a raw pointer to the allocated object
                    let ns_image_alloc: *mut AnyObject = msg_send![NSImage::class(), alloc];
                    
                    // Initialize NSImage with CGImage using msg_send!
                    // The initWithCGImage:size: method takes a CGImageRef and NSSize
                    let full_size = NSSize::new(img_width, img_height);   // Full image size
                    
                    // Use msg_send! to call initWithCGImage:size:
                    // This consumes the allocated object and returns the initialized object
                    let ns_image_ptr: *mut AnyObject = msg_send![ns_image_alloc, initWithCGImage: cg_image_ref, size: full_size];
                    
                    // Wrap in Retained - the init method returns a retained object
                    // SAFETY: initWithCGImage:size: returns a retained +1 object
                    let ns_image: Retained<NSImage> = Retained::from_raw(ns_image_ptr as *mut NSImage)
                        .expect("NSImage initWithCGImage:size: returned nil");
                    let cropped_size = NSSize::new(use_width, use_height); // Size to use after cropping

                    // Calculate magnifier position (centered on cursor)
                    let mag_x = info.x - mag_size / 2.0;                  // X position
                    let mag_y = info.y - mag_size / 2.0;                  // Y position

                    // Destination rectangle for the magnifier
                    let mag_rect = NSRect::new(
                        NSPoint::new(mag_x, mag_y),     // Origin point
                        NSSize::new(mag_size, mag_size) // Size (square)
                    );

                    // Create a circular path for clipping
                    let circular_clip = NSBezierPath::bezierPathWithOvalInRect(mag_rect);

                    // -------------------------------------------------------------
                    // Draw the image inside the circle
                    // -------------------------------------------------------------
                    // Save current graphics state
                    NSGraphicsContext::saveGraphicsState_class();

                    // Disable interpolation for pixelated rendering
                    if let Some(graphics_context) = NSGraphicsContext::currentContext() {
                        graphics_context.setImageInterpolation(objc2_app_kit::NSImageInterpolation::None);
                    }

                    // Apply the circular clip
                    circular_clip.addClip();

                    // Source rectangle in the image (defines the portion to draw from)
                    let from_rect = NSRect::new(
                        NSPoint::new(crop_x, crop_y), // Origin of source rectangle
                        cropped_size                   // Size of source rectangle
                    );

                    // Draw the image from source rect to destination rect
                    // Use NSImage's drawInRect:fromRect:operation:fraction: method
                    // operation: 2 = NSCompositingOperationSourceOver (standard alpha blending)
                    // fraction: 1.0 = full opacity (no transparency)
                    const NS_COMPOSITING_OPERATION_SOURCE_OVER: usize = 2; // NSCompositingOperationSourceOver constant
                    let _: () = msg_send![
                        &*ns_image,
                        drawInRect: mag_rect,
                        fromRect: from_rect,
                        operation: NS_COMPOSITING_OPERATION_SOURCE_OVER,
                        fraction: 1.0_f64
                    ];

                    // Restore graphics state
                    NSGraphicsContext::restoreGraphicsState_class();

                    // -------------------------------------------------------------
                    // Draw the central reticle
                    // -------------------------------------------------------------
                    // Center of the magnifier
                    let center_x = mag_x + mag_size / 2.0;
                    let center_y = mag_y + mag_size / 2.0;

                    // Reticle size: FIXED, based only on current_zoom
                    let reticle_size = current_zoom;
                    let half_reticle = reticle_size / 2.0;

                    // The reticle is always centered in the magnifier
                    let reticle_center_x = center_x;
                    let reticle_center_y = center_y;

                    // Reticle rectangle
                    let square_rect = NSRect::new(
                        NSPoint::new(reticle_center_x - half_reticle, reticle_center_y - half_reticle),
                        NSSize::new(reticle_size, reticle_size)
                    );

                    // Gray color for the reticle
                    let gray_color = NSColor::colorWithCalibratedRed_green_blue_alpha(0.5, 0.5, 0.5, 1.0);
                    gray_color.setStroke();

                    // Draw the reticle square
                    let reticle_path = NSBezierPath::bezierPathWithRect(square_rect);
                    reticle_path.setLineWidth(1.0);
                    reticle_path.stroke();
                    
                    // Keep use_width for reference if needed
                    let _actual_pixels = use_width;

                    // -------------------------------------------------------------
                    // Draw the colored border (top or bottom arc based on fg_mode)
                    // -------------------------------------------------------------
                    // Parse the current hex color
                    let hex = &info.hex_color[1..]; // Remove the #
                    let r_val = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0; // Red component
                    let g_val = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0; // Green component
                    let b_val = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0; // Blue component

                    // Get the fg mode from the global variable
                    let fg_mode = if let Ok(mode) = FG_MODE.lock() {
                        *mode // Copy the boolean value
                    } else {
                        true // Default to top arc if lock fails
                    };
                    
                    // Get continue mode
                    let is_continue_mode = if let Ok(mode) = CONTINUE_MODE.lock() {
                        *mode
                    } else {
                        false
                    };
                    
                    // Get already captured colors
                    let captured_fg = if let Ok(color) = FG_COLOR.lock() {
                        *color
                    } else {
                        None
                    };
                    let captured_bg = if let Ok(color) = BG_COLOR.lock() {
                        *color
                    } else {
                        None
                    };

                    // Radius of the border circle (center of the border thickness)
                    // Subtract 0.5px so the inner edge slightly overlaps the magnifier
                    let border_radius = mag_size / 2.0 + BORDER_WIDTH / 2.0 - 0.5;

                    // In continue mode, first draw the arc for the already captured color (if it exists)
                    if is_continue_mode {
                        // If we're in fg mode (top arc), draw the captured bg color at bottom
                        // If we're in bg mode (bottom arc), draw the captured fg color at top
                        let (captured_color, captured_start, captured_end) = if fg_mode {
                            // Capturing fg (top), so show captured bg (bottom) if exists
                            (captured_bg, 180.0_f64, 360.0_f64)
                        } else {
                            // Capturing bg (bottom), so show captured fg (top) if exists
                            (captured_fg, 0.0_f64, 180.0_f64)
                        };
                        
                        if let Some((cap_r, cap_g, cap_b)) = captured_color {
                            // Draw the arc for the captured color
                            let cap_r_val = cap_r as f64 / 255.0;
                            let cap_g_val = cap_g as f64 / 255.0;
                            let cap_b_val = cap_b as f64 / 255.0;
                            
                            let captured_color_ns = NSColor::colorWithCalibratedRed_green_blue_alpha(
                                cap_r_val, cap_g_val, cap_b_val, 1.0
                            );
                            captured_color_ns.setStroke();
                            
                            let captured_arc_path = NSBezierPath::bezierPath();
                            let _: () = msg_send![
                                &*captured_arc_path,
                                appendBezierPathWithArcWithCenter: NSPoint::new(center_x, center_y),
                                radius: border_radius,
                                startAngle: captured_start,
                                endAngle: captured_end,
                                clockwise: Bool::NO
                            ];
                            captured_arc_path.setLineWidth(BORDER_WIDTH);
                            captured_arc_path.stroke();
                        }
                    }

                    // Border color = current pixel color
                    let border_color = NSColor::colorWithCalibratedRed_green_blue_alpha(r_val, g_val, b_val, 1.0);
                    border_color.setStroke(); // Set as stroke color

                    // Create the path for the current arc (top or bottom based on fg_mode)
                    let arc_path = NSBezierPath::bezierPath(); // Create empty bezier path
                    
                    // Angles for arcs (in degrees, counter-clockwise from positive X axis)
                    // Top arc: from 0° to 180° (upper half-circle)
                    // Bottom arc: from 180° to 360° (lower half-circle)
                    let (start_angle, end_angle) = if fg_mode {
                        (0.0_f64, 180.0_f64) // Top arc (foreground)
                    } else {
                        (180.0_f64, 360.0_f64) // Bottom arc (background)
                    };

                    // Add the arc to the path
                    // appendBezierPathWithArcWithCenter:radius:startAngle:endAngle:clockwise:
                    // Note: In Cocoa, clockwise=NO means counter-clockwise (positive mathematical direction)
                    let _: () = msg_send![
                        &*arc_path,
                        appendBezierPathWithArcWithCenter: NSPoint::new(center_x, center_y), // Center point
                        radius: border_radius,    // Arc radius
                        startAngle: start_angle,  // Start angle in degrees
                        endAngle: end_angle,      // End angle in degrees
                        clockwise: Bool::NO       // Counter-clockwise direction
                    ];

                    arc_path.setLineWidth(BORDER_WIDTH); // Set the line width
                    arc_path.stroke(); // Draw the arc

                    // Create system font for text
                    let font: Retained<NSFont> = NSFont::systemFontOfSize(HEX_FONT_SIZE);

                    // -------------------------------------------------------------
                    // Draw arc text for the captured color arc (if exists)
                    // -------------------------------------------------------------
                    if is_continue_mode {
                        let (captured_color_for_text, captured_fg_mode_for_text) = if fg_mode {
                            (captured_bg, false) // Show bg at bottom
                        } else {
                            (captured_fg, true) // Show fg at top
                        };
                        
                        if let Some((cap_r, cap_g, cap_b)) = captured_color_for_text {
                            // Uses format_labeled_hex_color from common module
                            let cap_label = if captured_fg_mode_for_text {
                                format_labeled_hex_color("Foreground", cap_r, cap_g, cap_b)
                            } else {
                                format_labeled_hex_color("Background", cap_r, cap_g, cap_b)
                            };
                            
                            // Text color based on captured color luminance
                            // Uses should_use_dark_text from common module
                            let cap_text_color = if should_use_dark_text(cap_r, cap_g, cap_b) {
                                NSColor::colorWithCalibratedRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0)
                            } else {
                                NSColor::colorWithCalibratedRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0)
                            };
                            
                            // Draw the captured color text (without C badge)
                            draw_arc_text(
                                &cap_label,
                                center_x, center_y,
                                border_radius,
                                captured_fg_mode_for_text,
                                &font,
                                &cap_text_color,
                                false, // No C badge for the captured color
                            );
                        }
                    }

                    // -------------------------------------------------------------
                    // Draw the hex text on arc (top or bottom based on fg_mode)
                    // -------------------------------------------------------------
                    // Text color based on luminance
                    // Uses should_use_dark_text from common module
                    let text_color = if should_use_dark_text(info.r, info.g, info.b) {
                        NSColor::colorWithCalibratedRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0)
                    } else {
                        NSColor::colorWithCalibratedRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0)
                    };

                    // Build text with Foreground/Background label
                    // Uses format_labeled_hex_color from common module
                    let label = if fg_mode {
                        format_labeled_hex_color("Foreground", info.r, info.g, info.b)
                    } else {
                        format_labeled_hex_color("Background", info.r, info.g, info.b)
                    };
                    
                    // Draw the current color text (with C badge if continue mode)
                    draw_arc_text(
                        &label,
                        center_x, center_y,
                        border_radius,
                        fg_mode,
                        &font,
                        &text_color,
                        is_continue_mode, // C badge if continue mode enabled
                    );
                } // End of unsafe block
            } // End of if let Some(cg_image)
            } // End of if should_draw_magnifier
        } // End of if let Some(ref info)
    }
}

/// Draw text along an arc around a circle
///
/// # Arguments
/// * `text` - The text to draw
/// * `center_x`, `center_y` - Center of the circle
/// * `radius` - Radius of the text arc
/// * `is_top_arc` - true for top arc, false for bottom arc
/// * `font` - Font to use
/// * `text_color` - Text color
/// * `show_badge` - Show the "C" badge at the end
fn draw_arc_text(
    text: &str,
    center_x: f64,
    center_y: f64,
    radius: f64,
    is_top_arc: bool,
    font: &NSFont,
    text_color: &NSColor,
    show_badge: bool,
) {
    use objc2_foundation::NSDictionary;
    use objc2::runtime::AnyObject;
    
    // Character count + space for badge if needed
    let badge_extra_chars = if show_badge { 2.0 } else { 0.0 };
    let char_count = text.len() as f64 + badge_extra_chars;
    
    // Calculate angle between each character
    let angle_step = CHAR_SPACING_PIXELS / radius;
    
    // Total arc occupied by text
    let total_arc = angle_step * (char_count - 1.0);
    
    // Start angle based on arc (top or bottom)
    let text_start_angle: f64 = if is_top_arc {
        std::f64::consts::PI / 2.0 + total_arc / 2.0
    } else {
        -std::f64::consts::PI / 2.0 - total_arc / 2.0
    };

    // Save graphics state
    NSGraphicsContext::saveGraphicsState_class();

    // Current character index
    let mut char_index: f64 = 0.0;

    // Draw each character of the text
    for c in text.chars() {
        // Angle for this character
        let angle = if is_top_arc {
            text_start_angle - angle_step * char_index
        } else {
            text_start_angle + angle_step * char_index
        };

        char_index += 1.0;

        // Position on the arc
        let char_x = center_x + radius * angle.cos();
        let char_y = center_y + radius * angle.sin();

        // Convert character to NSString
        let char_str = c.to_string();
        let ns_char = NSString::from_str(&char_str);

        // Create the attribute dictionary for text
        let font_attr_key = NSString::from_str("NSFont");
        let color_attr_key = NSString::from_str("NSColor");
        let keys: &[&NSString] = &[&font_attr_key, &color_attr_key];
        let values: &[&AnyObject] = unsafe {
            &[
                &*(font as *const NSFont as *const AnyObject),
                &*(text_color as *const NSColor as *const AnyObject),
            ]
        };
        let attributes = NSDictionary::from_slices(keys, values);

        // Measure character size
        let char_size: NSSize = unsafe { ns_char.sizeWithAttributes(Some(&attributes)) };

        // Create a transform to position and rotate the character
        let transform = NSAffineTransform::transform();
        transform.translateXBy_yBy(char_x, char_y);

        // Rotation based on arc
        let rotation_angle = if is_top_arc {
            angle - std::f64::consts::PI / 2.0
        } else {
            angle + std::f64::consts::PI / 2.0
        };
        transform.rotateByRadians(rotation_angle);
        transform.concat();

        // Draw the character centered
        let draw_point = NSPoint::new(-char_size.width / 2.0, -char_size.height / 2.0);
        unsafe { ns_char.drawAtPoint_withAttributes(draw_point, Some(&attributes)) };

        // Invert the transform
        let inverse = transform.copy();
        inverse.invert();
        inverse.concat();
    }

    // Draw the "C" badge at the end if requested
    if show_badge {
        // Advance by one space
        char_index += 1.0;
        
        // Angle for the badge (after the text)
        let badge_angle = if is_top_arc {
            text_start_angle - angle_step * char_index
        } else {
            text_start_angle + angle_step * char_index
        };

        // Badge position on the arc
        let badge_x = center_x + radius * badge_angle.cos();
        let badge_y = center_y + radius * badge_angle.sin();

        // Badge size
        let badge_radius = HEX_FONT_SIZE * 0.7;

        // Draw the red background circle
        let badge_rect = NSRect::new(
            NSPoint::new(badge_x - badge_radius, badge_y - badge_radius),
            NSSize::new(badge_radius * 2.0, badge_radius * 2.0)
        );
        let red_color = NSColor::colorWithCalibratedRed_green_blue_alpha(0.9, 0.1, 0.1, 1.0);
        red_color.setFill();
        let badge_circle = NSBezierPath::bezierPathWithOvalInRect(badge_rect);
        badge_circle.fill();

        // Draw the letter "C" in white
        let white_color = NSColor::colorWithCalibratedRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
        let ns_c = NSString::from_str("C");

        let font_attr_key = NSString::from_str("NSFont");
        let color_attr_key = NSString::from_str("NSColor");
        let badge_keys: &[&NSString] = &[&font_attr_key, &color_attr_key];
        let badge_values: &[&AnyObject] = unsafe {
            &[
                &*(font as *const NSFont as *const AnyObject),
                &*(white_color.as_ref() as *const NSColor as *const AnyObject),
            ]
        };
        let badge_attributes = NSDictionary::from_slices(badge_keys, badge_values);

        let c_size: NSSize = unsafe { ns_c.sizeWithAttributes(Some(&badge_attributes)) };

        let badge_transform = NSAffineTransform::transform();
        badge_transform.translateXBy_yBy(badge_x, badge_y);

        let badge_rotation = if is_top_arc {
            badge_angle - std::f64::consts::PI / 2.0
        } else {
            badge_angle + std::f64::consts::PI / 2.0
        };
        badge_transform.rotateByRadians(badge_rotation);
        badge_transform.concat();

        let c_draw_point = NSPoint::new(-c_size.width / 2.0, -c_size.height / 2.0);
        unsafe { ns_c.drawAtPoint_withAttributes(c_draw_point, Some(&badge_attributes)) };

        let badge_inverse = badge_transform.copy();
        badge_inverse.invert();
        badge_inverse.concat();
    }

    // Restore graphics state
    NSGraphicsContext::restoreGraphicsState_class();
}