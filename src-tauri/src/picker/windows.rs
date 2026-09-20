// =============================================================================
// COLOR PICKER - WINDOWS VERSION
// =============================================================================
// Fullscreen window displaying screen capture + magnifier
// =============================================================================

// -----------------------------------------------------------------------------
// IMPORTS - Configuration
// -----------------------------------------------------------------------------
use crate::config::{
    BORDER_WIDTH,          // Colored border thickness
    CAPTURED_PIXELS,       // Default captured pixels count
    INITIAL_ZOOM_FACTOR,   // Initial zoom factor
    SHIFT_MOVE_PIXELS,     // Pixels to move with Shift
    ZOOM_MIN,              // Minimum zoom
    ZOOM_MAX,              // Maximum zoom
    ZOOM_STEP,             // Zoom increment
};

// -----------------------------------------------------------------------------
// IMPORTS - Common types and functions
// -----------------------------------------------------------------------------
use super::common::{
    ColorPickerResult,         // Result structure with FG/BG
    should_use_dark_text,      // Determines black or white text
    format_labeled_hex_color,  // Formats "Label - #RRGGBB"
};

// -----------------------------------------------------------------------------
// IMPORTS - Windows API
// -----------------------------------------------------------------------------
use windows::{
    core::*,                                    // Windows core types
    Win32::{
        Foundation::*,                          // Fundamental types
        Graphics::Gdi::*,                       // GDI for 2D drawing
        Graphics::GdiPlus,                      // GDI+ for anti-aliasing
        System::LibraryLoader::GetModuleHandleW, // Current module handle
        UI::{
            Input::KeyboardAndMouse::*,         // Keyboard/mouse input
            WindowsAndMessaging::*,             // Messages and windows
        },
    },
};

// -----------------------------------------------------------------------------
// IMPORTS - Rust standard library
// -----------------------------------------------------------------------------
use std::sync::Mutex; // Mutex for thread-safe sync

// =============================================================================
// CONSTANTS
// =============================================================================

/// Minimum captured pixels count (max zoom)
const CAPTURED_PIXELS_MIN: f64 = 9.0;

/// Maximum captured pixels count (min zoom)
const CAPTURED_PIXELS_MAX: f64 = 21.0;

/// Increment for captured pixels count
const CAPTURED_PIXELS_STEP: f64 = 2.0;

/// Windows window class name prefix (will be made unique with timestamp)
const WINDOW_CLASS_PREFIX: &str = "ColorPickerFullscreen_";

/// Timer ID for refresh
const TIMER_ID: usize = 1;

// -----------------------------------------------------------------------------
// Variables statiques globales
// Global static variables
// -----------------------------------------------------------------------------

/// GDI+ token for initialization/shutdown
static GDIPLUS_TOKEN: Mutex<usize> = Mutex::new(0);

/// Window handle (stored separately because HWND is not Send)
static WINDOW_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Previous window handle to restore focus after closing
static PREVIOUS_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

// =============================================================================
// GLOBAL STATE
// =============================================================================

/// Global color picker state protected by Mutex
static STATE: Mutex<PickerState> = Mutex::new(PickerState::new());

/// Structure containing the complete color picker state
struct PickerState {
    cursor_x: i32,                      // Cursor X position (screen coords)
    cursor_y: i32,                      // Cursor Y position (screen coords)
    color: (u8, u8, u8),                // Color under cursor
    fg_color: Option<(u8, u8, u8)>,     // Selected FG color
    bg_color: Option<(u8, u8, u8)>,     // Selected BG color
    fg_mode: bool,                      // true = FG mode, false = BG mode
    continue_mode: bool,                // Continue mode enabled
    zoom: f64,                          // Current zoom factor
    captured: f64,                      // Number of captured pixels
    quit: bool,                         // Flag to quit application
    screen_width: i32,                  // Virtual desktop width
    screen_height: i32,                 // Virtual desktop height
    virtual_left: i32,                  // Virtual desktop X origin
    virtual_top: i32,                   // Virtual desktop Y origin
}

/// Screen capture bitmap handle (must be global for WM_PAINT)
static SCREEN_BITMAP: Mutex<Option<isize>> = Mutex::new(None);

/// Raw screen capture data (BGRA)
static SCREEN_DATA: Mutex<Vec<u8>> = Mutex::new(Vec::new());

// =============================================================================
// GDI+ INITIALIZATION
// =============================================================================

/// Initialize GDI+ for anti-aliasing and advanced drawing
fn init_gdiplus() {
    unsafe {
        let mut token: usize = 0;                              // Token returned by GDI+
        let input = GdiPlus::GdiplusStartupInput {
            GdiplusVersion: 1,                                 // GDI+ version
            DebugEventCallback: 0,                             // No debug callback (isize, not Option)
            SuppressBackgroundThread: FALSE,                   // Allow background thread
            SuppressExternalCodecs: FALSE,                     // Allow external codecs
        };
        
        // Start GDI+ and get the token
        let status = GdiPlus::GdiplusStartup(
            &mut token,                                        // Pointer to token
            &input,                                            // Input parameters
            std::ptr::null_mut()                               // No output
        );
        
        // If success (Status == 0), save the token
        if status == GdiPlus::Status(0) {
            if let Ok(mut t) = GDIPLUS_TOKEN.lock() {
                *t = token;                                    // Store token for shutdown
            }
        }
    }
}

/// Shutdown GDI+ and release resources
fn shutdown_gdiplus() {
    unsafe {
        if let Ok(token) = GDIPLUS_TOKEN.lock() {
            if *token != 0 {                                   // If GDI+ was initialized
                GdiPlus::GdiplusShutdown(*token);              // Shutdown GDI+
            }
        }
    }
}

/// PickerState implementation
impl PickerState {
    /// Creates a new state with default values (const fn for static initialization)
    const fn new() -> Self {
        Self {
            cursor_x: 0,                           // Initial X position
            cursor_y: 0,                           // Initial Y position
            color: (0, 0, 0),                      // Black by default
            fg_color: None,                        // No FG selected
            bg_color: None,                        // No BG selected
            fg_mode: true,                         // Start in FG mode
            continue_mode: false,                  // Continue mode disabled
            zoom: INITIAL_ZOOM_FACTOR,             // Initial zoom from config
            captured: CAPTURED_PIXELS,             // Captured pixels from config
            quit: false,                           // Don't quit
            screen_width: 0,                       // Will be set during capture
            screen_height: 0,                      // Will be set during capture
            virtual_left: 0,                       // Will be set during capture
            virtual_top: 0,                        // Will be set during capture
        }
    }
    
    /// Resets state to default values
    fn reset(&mut self) {
        self.cursor_x = 0;                         // Reset X position
        self.cursor_y = 0;                         // Reset Y position
        self.color = (0, 0, 0);                    // Reset color
        self.fg_color = None;                      // Clear selected FG
        self.bg_color = None;                      // Clear selected BG
        self.fg_mode = true;                       // Back to FG mode
        self.continue_mode = false;                // Disable continue mode
        self.zoom = INITIAL_ZOOM_FACTOR;           // Reset zoom
        self.captured = CAPTURED_PIXELS;           // Reset captured pixels
        self.quit = false;                         // Don't quit
    }
}

// =============================================================================
// SCREEN CAPTURE
// =============================================================================

/// Captures the entire virtual desktop (all monitors) into a bitmap and extracts pixel data
fn capture_screen() {
    unsafe {
        // Get virtual desktop dimensions (all screens combined)
        let virtual_left = GetSystemMetrics(SM_XVIRTUALSCREEN);   // X origin (can be < 0)
        let virtual_top = GetSystemMetrics(SM_YVIRTUALSCREEN);    // Y origin (can be < 0)
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);         // Total width
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);        // Total height
        
        // Create device contexts (DC) for copying
        let hdc_screen = GetDC(HWND::default());      // Screen DC
        let hdc_mem = CreateCompatibleDC(hdc_screen); // Compatible memory DC
        
        // Create a compatible bitmap to store the capture
        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        
        if !hbitmap.is_invalid() {
            // Select the bitmap into the memory DC
            SelectObject(hdc_mem, hbitmap);
            
            // Copy virtual desktop to bitmap (BitBlt = Bit Block Transfer)
            let _ = BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, virtual_left, virtual_top, SRCCOPY);
            
            // Store the bitmap handle for later use
            if let Ok(mut bmp) = SCREEN_BITMAP.lock() {
                *bmp = Some(hbitmap.0 as isize);       // Convert HBITMAP to isize
            }
            
            // Configure BITMAPINFO structure to extract raw data
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, // Structure size
                    biWidth: width,                    // Bitmap width
                    biHeight: -height,                 // Negative = top-down
                    biPlanes: 1,                       // Always 1
                    biBitCount: 32,                    // 32 bits per pixel (BGRA)
                    biCompression: BI_RGB.0,           // No compression
                    ..Default::default()               // Rest zeroed
                },
                ..Default::default()
            };
            
            // Allocate buffer for pixel data (4 bytes per pixel: BGRA)
            let mut data: Vec<u8> = vec![0; (width * height * 4) as usize];
            
            // Extract pixel data from the bitmap
            let _ = GetDIBits(
                hdc_mem,                               // Source DC
                hbitmap,                               // Source bitmap
                0,                                     // First scan line
                height as u32,                         // Number of lines
                Some(data.as_mut_ptr() as *mut _),     // Destination buffer
                &mut bmi,                              // Bitmap info
                DIB_RGB_COLORS,                        // RGB format
            );
            
            // Store pixel data for later reading
            if let Ok(mut screen_data) = SCREEN_DATA.lock() {
                *screen_data = data;
            }
            
            // Save virtual desktop dimensions in state
            if let Ok(mut state) = STATE.lock() {
                state.screen_width = width;
                state.screen_height = height;
                state.virtual_left = virtual_left;
                state.virtual_top = virtual_top;
            }
        }
        
        // Release GDI resources
        let _ = DeleteDC(hdc_mem);                     // Delete memory DC
        let _ = ReleaseDC(HWND::default(), hdc_screen); // Release screen DC
    }
}

/// Cleans up the capture bitmap and frees memory
fn cleanup_screen_bitmap() {
    // Delete the bitmap if present
    if let Ok(mut bmp) = SCREEN_BITMAP.lock() {
        if let Some(h) = bmp.take() {                  // takes and returns the value
            unsafe {
                let _ = DeleteObject(HBITMAP(h as *mut _)); // Delete GDI object
            }
        }
    }
    // Clear the data buffer
    if let Ok(mut data) = SCREEN_DATA.lock() {
        data.clear();                                  // Free memory
    }
}

/// Gets the RGB color of the pixel at screen coordinates (x, y)
/// 
/// # Arguments
/// * `x` - Pixel X position (screen coords, can be negative)
/// * `y` - Pixel Y position (screen coords, can be negative)
/// 
/// # Returns
/// Tuple (R, G, B) of pixel color
fn get_pixel_color(x: i32, y: i32) -> (u8, u8, u8) {
    // Get virtual desktop dimensions and origin
    let (width, height, virtual_left, virtual_top) = {
        if let Ok(state) = STATE.lock() {
            (state.screen_width, state.screen_height, state.virtual_left, state.virtual_top)
        } else {
            return (0, 0, 0);                          // Black if error
        }
    };
    
    // Convert screen coordinates to bitmap coordinates
    let bitmap_x = x - virtual_left;
    let bitmap_y = y - virtual_top;
    
    // Read color from captured data
    if let Ok(data) = SCREEN_DATA.lock() {
        // Check that coordinates are within bitmap bounds
        if bitmap_x >= 0 && bitmap_x < width && bitmap_y >= 0 && bitmap_y < height {
            // Calculate index in buffer (4 bytes per pixel: BGRA)
            let idx = ((bitmap_y * width + bitmap_x) * 4) as usize;
            if idx + 2 < data.len() {
                let b = data[idx];                     // Blue first (BGRA format)
                let g = data[idx + 1];                 // Green next
                let r = data[idx + 2];                 // Red last
                return (r, g, b);                      // Return in RGB order
            }
        }
    }
    (0, 0, 0)                                          // Black by default
}

// =============================================================================
// POSITION UPDATE
// =============================================================================

/// Updates cursor position and corresponding color
/// 
/// # Arguments
/// * `x` - New X position
/// * `y` - New Y position
fn update_cursor_pos(x: i32, y: i32) {
    let color = get_pixel_color(x, y);                 // Get color
    if let Ok(mut state) = STATE.lock() {
        state.cursor_x = x;                            // Update X
        state.cursor_y = y;                            // Update Y
        state.color = color;                           // Update color
    }
}

// =============================================================================
// CURVED TEXT DRAWING
// =============================================================================

/// Draws text following a circular arc with GDI+
/// 
/// If `show_continue_badge` is true, draws a red badge with "C" at the end
/// 
/// # Arguments
/// * `hdc` - Device context handle
/// * `text` - Text to draw
/// * `cx` - Circle center X
/// * `cy` - Circle center Y
/// * `radius` - Text arc radius
/// * `char_spacing` - Character spacing in pixels
/// * `upper` - true = upper arc, false = lower arc
/// * `color` - Text color (COLORREF)
/// * `show_continue_badge` - Show red "C" badge
fn draw_curved_text(
    hdc: HDC,                    // Windows DC handle
    text: &str,                  // Text to display
    cx: f64,                     // Center X in pixels
    cy: f64,                     // Center Y in pixels
    radius: f64,                 // Arc radius
    char_spacing: f64,           // Character spacing
    upper: bool,                 // Upper or lower arc
    color: COLORREF,             // Text color
    show_continue_badge: bool,   // Show continue badge
) {
    unsafe {
        // Create a GDI+ graphics context from the HDC
        let mut graphics: *mut GdiPlus::GpGraphics = std::ptr::null_mut();
        if GdiPlus::GdipCreateFromHDC(hdc, &mut graphics) != GdiPlus::Status(0) {
            return; // Creation failed
        }
        
        // Enable anti-aliasing for smooth text rendering
        let _ = GdiPlus::GdipSetTextRenderingHint(graphics, GdiPlus::TextRenderingHint(3)); // AntiAlias
        let _ = GdiPlus::GdipSetSmoothingMode(graphics, GdiPlus::SmoothingMode(4));         // AntiAlias
        
        // Extract RGB components from COLORREF (format: 0x00BBGGRR)
        let r = (color.0 & 0xFF) as u8;              // Red in bits 0-7
        let g = ((color.0 >> 8) & 0xFF) as u8;       // Green in bits 8-15
        let b = ((color.0 >> 16) & 0xFF) as u8;      // Blue in bits 16-23
        
        // Convert to ARGB format for GDI+ (format: 0xAARRGGBB)
        let argb = 0xFF000000u32 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        
        // Create a solid color brush for text
        let mut brush: *mut GdiPlus::GpBrush = std::ptr::null_mut();
        if GdiPlus::GdipCreateSolidFill(argb, &mut brush as *mut _ as *mut *mut GdiPlus::GpSolidFill) != GdiPlus::Status(0) {
            GdiPlus::GdipDeleteGraphics(graphics);   // Clean up on error
            return;
        }
        
        // Create "Segoe UI" font family
        let mut font_family: *mut GdiPlus::GpFontFamily = std::ptr::null_mut();
        let font_name: Vec<u16> = "Segoe UI".encode_utf16().chain(std::iter::once(0)).collect(); // UTF-16 + null
        let _ = GdiPlus::GdipCreateFontFamilyFromName(
            windows::core::PCWSTR(font_name.as_ptr()), // Font name
            std::ptr::null_mut(),                      // Font collection
            &mut font_family                           // Output pointer
        );
        
        // Create the font with specified size
        let mut font: *mut GdiPlus::GpFont = std::ptr::null_mut();
        if !font_family.is_null() {
            let _ = GdiPlus::GdipCreateFont(
                font_family,                           // Font family
                11.0,                                  // Size in pixels
                0,                                     // Style (0 = regular)
                GdiPlus::Unit(2),                      // Unit (2 = Pixel)
                &mut font                              // Output pointer
            );
        }
        
        // Check that font was created successfully
        if font.is_null() {
            GdiPlus::GdipDeleteBrush(brush);           // Free brush
            GdiPlus::GdipDeleteGraphics(graphics);     // Free context
            if !font_family.is_null() {
                GdiPlus::GdipDeleteFontFamily(font_family); // Free family
            }
            return;
        }
        
        // Calculate character count (+ space for badge if needed)
        let badge_space = if show_continue_badge { 2.0 } else { 0.0 }; // Space for badge
        let char_count = text.chars().count() as f64 + badge_space;
        let angle_step = char_spacing / radius;
        let total_arc = angle_step * (char_count - 1.0);
        
        // For each character
        for (i, c) in text.chars().enumerate() {
            let angle = if upper {
                // Upper arc: left to right, letters upright
                let start = std::f64::consts::FRAC_PI_2 + total_arc / 2.0;
                start - angle_step * (i as f64)
            } else {
                // Lower arc: left to right, letters upside down
                let start = -std::f64::consts::FRAC_PI_2 - total_arc / 2.0;
                start + angle_step * (i as f64)
            };
            
            // Position on circle
            let px = cx + radius * angle.cos();
            let py = cy - radius * angle.sin();
            
            // Rotation angle for the letter
            let rot_deg = if upper {
                // Top: perpendicular to radius, letters facing outward
                -(angle.to_degrees() - 90.0)
            } else {
                // Bottom: perpendicular to radius, letters facing outward (so inverted)
                -(angle.to_degrees() + 90.0)
            };
            
            // Save state, apply transform, draw, restore
            let _ = GdiPlus::GdipSaveGraphics(graphics, &mut 0u32);
            
            // Translate to point, rotate, then draw centered
            let _ = GdiPlus::GdipTranslateWorldTransform(graphics, px as f32, py as f32, GdiPlus::MatrixOrder(0));
            let _ = GdiPlus::GdipRotateWorldTransform(graphics, rot_deg as f32, GdiPlus::MatrixOrder(0));
            
            // Measure character to center
            let char_str: Vec<u16> = c.to_string().encode_utf16().chain(std::iter::once(0)).collect();
            let mut bbox = GdiPlus::RectF { X: 0.0, Y: 0.0, Width: 0.0, Height: 0.0 };
            let layout_rect = GdiPlus::RectF { X: 0.0, Y: 0.0, Width: 100.0, Height: 100.0 };
            let _ = GdiPlus::GdipMeasureString(
                graphics,
                windows::core::PCWSTR(char_str.as_ptr()),
                1,
                font,
                &layout_rect,
                std::ptr::null_mut(),
                &mut bbox,
                std::ptr::null_mut(),
                std::ptr::null_mut()
            );
            
            // Draw character centered
            let draw_rect = GdiPlus::RectF {
                X: -bbox.Width / 2.0,
                Y: -bbox.Height / 2.0,
                Width: bbox.Width,
                Height: bbox.Height,
            };
            
            let _ = GdiPlus::GdipDrawString(
                graphics,
                windows::core::PCWSTR(char_str.as_ptr()),
                1,
                font,
                &draw_rect,
                std::ptr::null_mut(),
                brush
            );
            
            // Restore transform
            let _ = GdiPlus::GdipResetWorldTransform(graphics);
        }
        
        // Draw "C" badge if needed
        if show_continue_badge {
            let text_len = text.chars().count() as f64;
            let badge_index = text_len + 1.0; // Position after text + space
            
            let angle = if upper {
                let start = std::f64::consts::FRAC_PI_2 + total_arc / 2.0;
                start - angle_step * badge_index
            } else {
                let start = -std::f64::consts::FRAC_PI_2 - total_arc / 2.0;
                start + angle_step * badge_index
            };
            
            let px = cx + radius * angle.cos();
            let py = cy - radius * angle.sin();
            
            let rot_deg = if upper {
                -(angle.to_degrees() - 90.0)
            } else {
                -(angle.to_degrees() + 90.0)
            };
            
            // Apply transform for badge
            let _ = GdiPlus::GdipTranslateWorldTransform(graphics, px as f32, py as f32, GdiPlus::MatrixOrder(0));
            let _ = GdiPlus::GdipRotateWorldTransform(graphics, rot_deg as f32, GdiPlus::MatrixOrder(0));
            
            // Draw red circle
            let badge_radius: f32 = 7.0;
            let mut red_brush: *mut GdiPlus::GpBrush = std::ptr::null_mut();
            let red_argb = 0xFFE63232u32; // Red
            let _ = GdiPlus::GdipCreateSolidFill(red_argb, &mut red_brush as *mut _ as *mut *mut GdiPlus::GpSolidFill);
            
            if !red_brush.is_null() {
                let _ = GdiPlus::GdipFillEllipse(
                    graphics,
                    red_brush,
                    -badge_radius,
                    -badge_radius,
                    badge_radius * 2.0,
                    badge_radius * 2.0,
                );
                let _ = GdiPlus::GdipDeleteBrush(red_brush);
            }
            
            // Draw "C" in white
            let mut white_brush: *mut GdiPlus::GpBrush = std::ptr::null_mut();
            let white_argb = 0xFFFFFFFFu32;
            let _ = GdiPlus::GdipCreateSolidFill(white_argb, &mut white_brush as *mut _ as *mut *mut GdiPlus::GpSolidFill);
            
            if !white_brush.is_null() {
                // Smaller font for C
                let mut small_font: *mut GdiPlus::GpFont = std::ptr::null_mut();
                let _ = GdiPlus::GdipCreateFont(font_family, 9.0, 1, GdiPlus::Unit(2), &mut small_font); // Bold
                
                if !small_font.is_null() {
                    let c_str: Vec<u16> = "C".encode_utf16().chain(std::iter::once(0)).collect();
                    let mut c_bbox = GdiPlus::RectF { X: 0.0, Y: 0.0, Width: 0.0, Height: 0.0 };
                    let c_layout_rect = GdiPlus::RectF { X: 0.0, Y: 0.0, Width: 100.0, Height: 100.0 };
                    let _ = GdiPlus::GdipMeasureString(
                        graphics,
                        windows::core::PCWSTR(c_str.as_ptr()),
                        1,
                        small_font,
                        &c_layout_rect,
                        std::ptr::null_mut(),
                        &mut c_bbox,
                        std::ptr::null_mut(),
                        std::ptr::null_mut()
                    );
                    
                    let c_rect = GdiPlus::RectF {
                        X: -c_bbox.Width / 2.0,
                        Y: -c_bbox.Height / 2.0,
                        Width: c_bbox.Width,
                        Height: c_bbox.Height,
                    };
                    
                    let _ = GdiPlus::GdipDrawString(
                        graphics,
                        windows::core::PCWSTR(c_str.as_ptr()),
                        1,
                        small_font,
                        &c_rect,
                        std::ptr::null_mut(),
                        white_brush
                    );
                    
                    let _ = GdiPlus::GdipDeleteFont(small_font);
                }
                
                let _ = GdiPlus::GdipDeleteBrush(white_brush);
            }
            
            let _ = GdiPlus::GdipResetWorldTransform(graphics);
        }
        
        // Nettoyage
        // Cleanup
        GdiPlus::GdipDeleteFont(font);
        GdiPlus::GdipDeleteFontFamily(font_family);
        GdiPlus::GdipDeleteBrush(brush);
        GdiPlus::GdipDeleteGraphics(graphics);
    }
}

// =============================================================================
// MAIN DRAWING
// =============================================================================

fn paint_window(_hwnd: HWND, hdc: HDC) {
    // Get current state
    let (cursor_x, cursor_y, color, fg_color, bg_color, fg_mode, continue_mode, zoom, captured, 
         screen_width, screen_height, virtual_left, virtual_top) = {
        let state = match STATE.lock() {
            Ok(s) => s,
            Err(_) => return,
        };
        (
            state.cursor_x, state.cursor_y, state.color,
            state.fg_color, state.bg_color,
            state.fg_mode, state.continue_mode,
            state.zoom, state.captured,
            state.screen_width, state.screen_height,
            state.virtual_left, state.virtual_top,
        )
    };
    
    // Convert screen coordinates to window (bitmap) coordinates
    let window_x = cursor_x - virtual_left;
    let window_y = cursor_y - virtual_top;
    
    // Get screen data
    let screen_data = match SCREEN_DATA.lock() {
        Ok(d) => d.clone(),
        Err(_) => return,
    };
    
    if screen_data.is_empty() { return; }
    
    unsafe {
        // Create a double buffer to avoid flickering
        let hdc_mem = CreateCompatibleDC(hdc);
        let hbitmap = CreateCompatibleBitmap(hdc, screen_width, screen_height);
        
        if hbitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            return;
        }
        
        SelectObject(hdc_mem, hbitmap);
        
        // Draw background (screen capture)
        if let Ok(bmp) = SCREEN_BITMAP.lock() {
            if let Some(h) = *bmp {
                let hdc_src = CreateCompatibleDC(hdc);
                SelectObject(hdc_src, HBITMAP(h as *mut _));
                let _ = BitBlt(hdc_mem, 0, 0, screen_width, screen_height, hdc_src, 0, 0, SRCCOPY);
                let _ = DeleteDC(hdc_src);
            }
        }
        
        // Magnifier parameters
        // Use window_x/window_y for drawing (coordinates relative to window)
        let mag_size = (captured * zoom) as i32;
        let zoom_i = zoom as i32;
        let captured_i = captured as i32;
        let half_cap = captured_i / 2;
        let border_f = BORDER_WIDTH as f32;
        let cx_f = window_x as f32;
        let cy_f = window_y as f32;
        let inner_radius_f = mag_size as f32 / 2.0;
        let outer_radius_f = inner_radius_f + border_f;
        
        // Inner radius of arcs reduced by 1px to cover the zoom edge
        let arc_inner_radius_f = inner_radius_f - 1.0;
        
        // =====================================================================
        // FG/BG COLOR CALCULATION
        // =====================================================================
        
        // Color for FG arc (foreground)
        // - If FG mode active: show current color (under cursor)
        // - Otherwise: show saved FG color (or gray if not captured yet)
        let (fg_r, fg_g, fg_b) = if fg_mode {
            color
        } else {
            fg_color.unwrap_or((128, 128, 128))
        };
        
        // Color for BG arc (background)
        // - If BG mode active: show current color (under cursor)
        // - Otherwise: show saved BG color (or gray if not captured yet)
        let (bg_r, bg_g, bg_b) = if !fg_mode {
            color
        } else {
            bg_color.unwrap_or((128, 128, 128))
        };
        
        // =====================================================================
        // MAIN GDI+ CONTEXT
        // =====================================================================
        
        let mut graphics: *mut GdiPlus::GpGraphics = std::ptr::null_mut();
        let status = GdiPlus::GdipCreateFromHDC(hdc_mem, &mut graphics);
        
        if status == GdiPlus::Status(0) && !graphics.is_null() {
            // Active l'anti-aliasing
            // Enable anti-aliasing
            let _ = GdiPlus::GdipSetSmoothingMode(graphics, GdiPlus::SmoothingMode(4)); // AntiAlias
            
            // =================================================================
            // STEP 1: DRAW ZOOMED PIXELS (with circular clip)
            // =================================================================
            
            // Create a circular path for clipping
            let mut clip_path: *mut GdiPlus::GpPath = std::ptr::null_mut();
            let _ = GdiPlus::GdipCreatePath(GdiPlus::FillMode(0), &mut clip_path);
            
            if !clip_path.is_null() {
                // Inner circle - same radius as inner edge of arcs
                let _ = GdiPlus::GdipAddPathEllipse(
                    clip_path,
                    cx_f - inner_radius_f,
                    cy_f - inner_radius_f,
                    inner_radius_f * 2.0,
                    inner_radius_f * 2.0,
                );
                
                let _ = GdiPlus::GdipSetClipPath(graphics, clip_path, GdiPlus::CombineMode(0)); // Replace
                
                // Disable anti-aliasing for pixels (avoids gaps)
                let _ = GdiPlus::GdipSetSmoothingMode(graphics, GdiPlus::SmoothingMode(0)); // None
                let _ = GdiPlus::GdipSetPixelOffsetMode(graphics, GdiPlus::PixelOffsetMode(3)); // PixelOffsetModeHalf
                
                // Starting position of pixels (integers to avoid gaps)
                let start_x = (cx_f - inner_radius_f).floor() as i32;
                let start_y = (cy_f - inner_radius_f).floor() as i32;
                
                // Draw each zoomed pixel
                for py in 0..captured_i {
                    for px in 0..captured_i {
                        // Calculate bitmap coordinates (relative to window)
                        let src_x = window_x - half_cap + px;
                        let src_y = window_y - half_cap + py;
                        
                        let (r, g, b) = if src_x >= 0 && src_x < screen_width && src_y >= 0 && src_y < screen_height {
                            let idx = ((src_y * screen_width + src_x) * 4) as usize;
                            if idx + 2 < screen_data.len() {
                                (screen_data[idx + 2], screen_data[idx + 1], screen_data[idx])
                            } else {
                                (128, 128, 128)
                            }
                        } else {
                            (64, 64, 64)
                        };
                        
                        // Integer position to avoid gaps between pixels
                        let dst_x = start_x + px * zoom_i;
                        let dst_y = start_y + py * zoom_i;
                        
                        // Create a brush for this pixel
                        let argb = 0xFF000000u32 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                        let mut pixel_brush: *mut GdiPlus::GpBrush = std::ptr::null_mut();
                        let _ = GdiPlus::GdipCreateSolidFill(argb, &mut pixel_brush as *mut _ as *mut *mut GdiPlus::GpSolidFill);
                        
                        if !pixel_brush.is_null() {
                            let _ = GdiPlus::GdipFillRectangleI(
                                graphics,
                                pixel_brush,
                                dst_x,
                                dst_y,
                                zoom_i,
                                zoom_i,
                            );
                            let _ = GdiPlus::GdipDeleteBrush(pixel_brush);
                        }
                    }
                }
                
                // Re-enable anti-aliasing for arcs
                let _ = GdiPlus::GdipSetSmoothingMode(graphics, GdiPlus::SmoothingMode(4)); // AntiAlias
                let _ = GdiPlus::GdipSetPixelOffsetMode(graphics, GdiPlus::PixelOffsetMode(0)); // Default
                
                // Reset clip
                let _ = GdiPlus::GdipResetClip(graphics);
                let _ = GdiPlus::GdipDeletePath(clip_path);
            }
            
            // =================================================================
            // STEP 2: DRAW ARCS ON TOP (covers edges)
            // =================================================================
            
            // Determine if each arc should be visible
            // - FG arc visible if: FG mode active OR FG color already captured
            // - BG arc visible if: BG mode active OR BG color already captured
            let show_fg_arc = fg_mode || fg_color.is_some();
            let show_bg_arc = !fg_mode || bg_color.is_some();
            
            // Draw upper arc (FG) with anti-aliasing
            if show_fg_arc {
                let mut fg_brush_gdi: *mut GdiPlus::GpBrush = std::ptr::null_mut();
                let fg_argb = 0xFF000000u32 | ((fg_r as u32) << 16) | ((fg_g as u32) << 8) | (fg_b as u32);
                let _ = GdiPlus::GdipCreateSolidFill(fg_argb, &mut fg_brush_gdi as *mut _ as *mut *mut GdiPlus::GpSolidFill);
                
                if !fg_brush_gdi.is_null() {
                    // Create a path for upper arc (half ring)
                    let mut path: *mut GdiPlus::GpPath = std::ptr::null_mut();
                    let _ = GdiPlus::GdipCreatePath(GdiPlus::FillMode(0), &mut path);
                    
                    if !path.is_null() {
                        // Outer arc (from 180° to 360°)
                        let _ = GdiPlus::GdipAddPathArc(
                            path,
                            cx_f - outer_radius_f,
                            cy_f - outer_radius_f,
                            outer_radius_f * 2.0,
                            outer_radius_f * 2.0,
                            180.0,
                            180.0,
                        );
                        
                        // Inner arc (from 360° to 180°) - reduced by 1px
                        let _ = GdiPlus::GdipAddPathArc(
                            path,
                            cx_f - arc_inner_radius_f,
                            cy_f - arc_inner_radius_f,
                            arc_inner_radius_f * 2.0,
                            arc_inner_radius_f * 2.0,
                            0.0,
                            -180.0,
                        );
                        
                        let _ = GdiPlus::GdipClosePathFigure(path);
                        let _ = GdiPlus::GdipFillPath(graphics, fg_brush_gdi, path);
                        let _ = GdiPlus::GdipDeletePath(path);
                    }
                    
                    let _ = GdiPlus::GdipDeleteBrush(fg_brush_gdi);
                }
            }
            
            // Draw lower arc (BG) with anti-aliasing
            if show_bg_arc {
                let mut bg_brush_gdi: *mut GdiPlus::GpBrush = std::ptr::null_mut();
                let bg_argb = 0xFF000000u32 | ((bg_r as u32) << 16) | ((bg_g as u32) << 8) | (bg_b as u32);
                let _ = GdiPlus::GdipCreateSolidFill(bg_argb, &mut bg_brush_gdi as *mut _ as *mut *mut GdiPlus::GpSolidFill);
                
                if !bg_brush_gdi.is_null() {
                    // Create a path for lower arc (half ring)
                    let mut path: *mut GdiPlus::GpPath = std::ptr::null_mut();
                    let _ = GdiPlus::GdipCreatePath(GdiPlus::FillMode(0), &mut path);
                    
                    if !path.is_null() {
                        // Outer arc (from 0° to 180°)
                        let _ = GdiPlus::GdipAddPathArc(
                            path,
                            cx_f - outer_radius_f,
                            cy_f - outer_radius_f,
                            outer_radius_f * 2.0,
                            outer_radius_f * 2.0,
                            0.0,
                            180.0,
                        );
                        
                        // Inner arc (from 180° to 0°) - reduced by 1px
                        let _ = GdiPlus::GdipAddPathArc(
                            path,
                            cx_f - arc_inner_radius_f,
                            cy_f - arc_inner_radius_f,
                            arc_inner_radius_f * 2.0,
                            arc_inner_radius_f * 2.0,
                            180.0,
                            -180.0,
                        );
                        
                        let _ = GdiPlus::GdipClosePathFigure(path);
                        let _ = GdiPlus::GdipFillPath(graphics, bg_brush_gdi, path);
                        let _ = GdiPlus::GdipDeletePath(path);
                    }
                    
                    let _ = GdiPlus::GdipDeleteBrush(bg_brush_gdi);
                }
            }
            
            let _ = GdiPlus::GdipDeleteGraphics(graphics);
        }
        
        // =====================================================================
        // DRAWING THE RETICLE
        // =====================================================================
        
        let ret_half = zoom_i / 2;
        // Use window coordinates for reticle
        let ret_x = window_x - ret_half;
        let ret_y = window_y - ret_half;
        let gray_pen = CreatePen(PS_SOLID, 1, COLORREF(0x606060));
        let old_pen = SelectObject(hdc_mem, gray_pen);
        let null_brush = GetStockObject(NULL_BRUSH);
        let old_brush = SelectObject(hdc_mem, null_brush);
        let _ = Rectangle(hdc_mem, ret_x, ret_y, ret_x + zoom_i, ret_y + zoom_i);
        let _ = SelectObject(hdc_mem, old_pen);
        let _ = SelectObject(hdc_mem, old_brush);
        let _ = DeleteObject(gray_pen);
        
        // Text arc radius (middle of border)
        let text_radius = (arc_inner_radius_f + outer_radius_f) as f64 / 2.0;
        let char_spacing = 8.0_f64; // Character spacing
        
        // Determine if each arc should be visible (same logic as for arcs)
        let show_fg_arc = fg_mode || fg_color.is_some();
        let show_bg_arc = !fg_mode || bg_color.is_some();
        
        // =====================================================================
        // FG TEXT IN UPPER ARC (CURVED)
        // =====================================================================
        
        if show_fg_arc {
            // Uses format_labeled_hex_color from common module
            let fg_hex = format_labeled_hex_color("Foreground", fg_r, fg_g, fg_b);
            // Uses should_use_dark_text from common module
            let fg_text_color = if should_use_dark_text(fg_r, fg_g, fg_b) { COLORREF(0) } else { COLORREF(0xFFFFFF) };
            
            // Show (C) badge if continue mode active and FG mode
            draw_curved_text(
                hdc_mem,
                &fg_hex,
                cx_f as f64,
                cy_f as f64,
                text_radius,
                char_spacing,
                true, // Upper arc
                fg_text_color,
                continue_mode && fg_mode, // Continue badge
            );
        }
        
        // =====================================================================
        // BG TEXT IN LOWER ARC (CURVED)
        // =====================================================================
        
        if show_bg_arc {
            // Uses format_labeled_hex_color from common module
            let bg_hex = format_labeled_hex_color("Background", bg_r, bg_g, bg_b);
            // Uses should_use_dark_text from common module
            let bg_text_color = if should_use_dark_text(bg_r, bg_g, bg_b) { COLORREF(0) } else { COLORREF(0xFFFFFF) };
            
            // Show (C) badge if continue mode active and BG mode
            draw_curved_text(
                hdc_mem,
                &bg_hex,
                cx_f as f64,
                cy_f as f64,
                text_radius,
                char_spacing,
                false, // Lower arc
                bg_text_color,
                continue_mode && !fg_mode, // Continue badge
            );
        }
        
        // Copy to screen
        let _ = BitBlt(hdc, 0, 0, screen_width, screen_height, hdc_mem, 0, 0, SRCCOPY);
        
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(hdc_mem);
    }
}

// =============================================================================
// EVENTS
// =============================================================================

fn handle_key(hwnd: HWND, vk: VIRTUAL_KEY) {
    let shift = unsafe { GetKeyState(VK_SHIFT.0 as i32) < 0 };
    
    match vk {
        VK_ESCAPE => {
            if let Ok(mut state) = STATE.lock() {
                state.quit = true;
            }
            unsafe { PostQuitMessage(0); }
        }
        VK_RETURN | VK_SPACE => select_color(),
        VK_C => {
            if let Ok(mut state) = STATE.lock() {
                state.continue_mode = !state.continue_mode;
            }
            unsafe { let _ = InvalidateRect(hwnd, None, FALSE); }
        }
        VK_I => {
            if let Ok(mut state) = STATE.lock() {
                if shift {
                    state.captured = (state.captured + CAPTURED_PIXELS_STEP).min(CAPTURED_PIXELS_MAX);
                } else {
                    state.zoom = (state.zoom + ZOOM_STEP).min(ZOOM_MAX);
                }
            }
            unsafe { let _ = InvalidateRect(hwnd, None, FALSE); }
        }
        VK_O => {
            if let Ok(mut state) = STATE.lock() {
                if shift {
                    state.captured = (state.captured - CAPTURED_PIXELS_STEP).max(CAPTURED_PIXELS_MIN);
                } else {
                    state.zoom = (state.zoom - ZOOM_STEP).max(ZOOM_MIN);
                }
            }
            unsafe { let _ = InvalidateRect(hwnd, None, FALSE); }
        }
        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN => {
            let amt = if shift { SHIFT_MOVE_PIXELS as i32 } else { 1 };
            unsafe {
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);
                match vk {
                    VK_LEFT => pt.x -= amt,
                    VK_RIGHT => pt.x += amt,
                    VK_UP => pt.y -= amt,
                    VK_DOWN => pt.y += amt,
                    _ => {}
                }
                let _ = SetCursorPos(pt.x, pt.y);
                update_cursor_pos(pt.x, pt.y);
                let _ = InvalidateRect(hwnd, None, FALSE);
            }
        }
        _ => {}
    }
}

fn handle_wheel(hwnd: HWND, delta: i16) {
    let shift = unsafe { GetKeyState(VK_SHIFT.0 as i32) < 0 };
    let up = delta > 0;
    
    if let Ok(mut state) = STATE.lock() {
        if shift {
            if up {
                state.captured = (state.captured + CAPTURED_PIXELS_STEP).min(CAPTURED_PIXELS_MAX);
            } else {
                state.captured = (state.captured - CAPTURED_PIXELS_STEP).max(CAPTURED_PIXELS_MIN);
            }
        } else {
            if up {
                state.zoom = (state.zoom + ZOOM_STEP).min(ZOOM_MAX);
            } else {
                state.zoom = (state.zoom - ZOOM_STEP).max(ZOOM_MIN);
            }
        }
    }
    unsafe { let _ = InvalidateRect(hwnd, None, FALSE); }
}

fn select_color() {
    // Indicates if we should quit after selection
    let should_quit;
    
    if let Ok(mut state) = STATE.lock() {
        // Get current color under cursor
        let color = state.color;
        
        if state.continue_mode {
            // Continue mode: capture both colors
            let has_other = if state.fg_mode {
                state.bg_color.is_some()
            } else {
                state.fg_color.is_some()
            };
            
            // Store color in appropriate slot
            if state.fg_mode {
                state.fg_color = Some(color);
            } else {
                state.bg_color = Some(color);
            }
            
            if has_other {
                // We have both colors, we can quit
                state.quit = true;
                should_quit = true;
            } else {
                // Switch to other mode
                state.fg_mode = !state.fg_mode;
                should_quit = false;
            }
        } else {
            // Normal mode: single color
            if state.fg_mode {
                state.fg_color = Some(color);
            } else {
                state.bg_color = Some(color);
            }
            state.quit = true;
            should_quit = true;
        }
    } else {
        return;
    }
    
    if should_quit {
        // Wait for mouse button to be released before quitting
        // This prevents the click from being propagated to the window below
        unsafe {
            // Wait for left mouse button release
            while (GetAsyncKeyState(VK_LBUTTON.0 as i32) & 0x8000u16 as i16) != 0 {
                // Process pending messages to avoid blocking
                let mut msg = MSG::default();
                if PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                // Small pause to avoid consuming too much CPU
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            
            // Now we can safely quit
            PostQuitMessage(0);
        }
    } else {
        // Force redraw to show captured color
        let hwnd_ptr = WINDOW_HWND.load(std::sync::atomic::Ordering::SeqCst);
        if hwnd_ptr != 0 {
            let hwnd = HWND(hwnd_ptr as *mut std::ffi::c_void);
            unsafe { 
                let _ = InvalidateRect(hwnd, None, FALSE); 
            }
        }
    }
}

// =============================================================================
// WINDOW PROCEDURE
// =============================================================================

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let _ = ShowCursor(false);
                let _ = SetTimer(hwnd, TIMER_ID, 16, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                let _ = ShowCursor(true);
                let _ = KillTimer(hwnd, TIMER_ID);
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                paint_window(hwnd, hdc);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_TIMER => {
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);
                update_cursor_pos(pt.x, pt.y);
                let _ = InvalidateRect(hwnd, None, FALSE);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                // WM_MOUSEMOVE coordinates are relative to the window
                // The window starts at (virtual_left, virtual_top)
                let window_x = (lp.0 & 0xFFFF) as i16 as i32;
                let window_y = ((lp.0 >> 16) & 0xFFFF) as i16 as i32;
                
                // Convert to screen coordinates
                let (virtual_left, virtual_top) = if let Ok(state) = STATE.lock() {
                    (state.virtual_left, state.virtual_top)
                } else {
                    (0, 0)
                };
                
                let screen_x = window_x + virtual_left;
                let screen_y = window_y + virtual_top;
                
                update_cursor_pos(screen_x, screen_y);
                let _ = InvalidateRect(hwnd, None, FALSE);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                // Select the color
                select_color();
                // Return 0 to indicate message was handled
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                // Capture click release to prevent propagation
                LRESULT(0)
            }
            WM_RBUTTONDOWN => {
                // Cancel and quit
                if let Ok(mut state) = STATE.lock() {
                    state.quit = true;
                }
                
                // Wait for right mouse button to be released before quitting
                while (GetAsyncKeyState(VK_RBUTTON.0 as i32) & 0x8000u16 as i16) != 0 {
                    // Process pending messages
                    let mut msg = MSG::default();
                    if PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                // Capture right click release to prevent propagation
                LRESULT(0)
            }
            WM_KEYDOWN => {
                handle_key(hwnd, VIRTUAL_KEY(wp.0 as u16));
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let delta = ((wp.0 >> 16) & 0xFFFF) as i16;
                handle_wheel(hwnd, delta);
                LRESULT(0)
            }
            WM_ERASEBKGND => {
                // Don't erase background (avoids flicker)
                LRESULT(1)
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp)
        }
    }
}

// =============================================================================
// PUBLIC API
// =============================================================================

pub fn run(fg: bool) -> ColorPickerResult {
    if let Ok(mut state) = STATE.lock() {
        state.reset();
        state.fg_mode = fg;
    }
    
    // Initialize GDI+ for anti-aliasing
    init_gdiplus();
    
    // Capture screen BEFORE creating window
    capture_screen();
    
    unsafe {
        let hinst = GetModuleHandleW(None).unwrap();
        
        // Save current active window to restore focus later
        let prev_window = GetForegroundWindow();
        PREVIOUS_HWND.store(prev_window.0 as isize, std::sync::atomic::Ordering::SeqCst);
        
        // Generate unique class name with timestamp to avoid conflicts
        let unique_class_name = format!("{}{}", WINDOW_CLASS_PREFIX, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0));
        let class_wide: Vec<u16> = unique_class_name.encode_utf16().chain(std::iter::once(0)).collect();
        let class_name = PCWSTR(class_wide.as_ptr());
        
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinst.into(),
            hCursor: HCURSOR::default(),
            lpszClassName: class_name,
            ..Default::default()
        };
        
        if RegisterClassExW(&wc) == 0 {
            cleanup_screen_bitmap();
            shutdown_gdiplus();
            return ColorPickerResult { foreground: None, background: None, continue_mode: false };
        }
        
        // Get virtual desktop dimensions (all screens combined)
        let virtual_left = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let virtual_top = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let virtual_width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let virtual_height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        
        // Fullscreen window covering all monitors, always on top
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!(""),
            WS_POPUP,
            virtual_left, virtual_top, virtual_width, virtual_height,
            None, None, hinst, None,
        );
        
        if hwnd.is_err() {
            let _ = UnregisterClassW(class_name, hinst);
            cleanup_screen_bitmap();
            shutdown_gdiplus();
            return ColorPickerResult { foreground: None, background: None, continue_mode: false };
        }
        
        let hwnd = hwnd.unwrap();
        
        // Save window handle
        WINDOW_HWND.store(hwnd.0 as isize, std::sync::atomic::Ordering::SeqCst);
        
        // Initial position
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        update_cursor_pos(pt.x, pt.y);
        
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(hwnd);
        let _ = SetCapture(hwnd);
        
        // Message loop
        let mut msg = MSG::default();
        loop {
            let quit = STATE.lock().map(|s| s.quit).unwrap_or(false);
            if quit { break; }
            
            if GetMessageW(&mut msg, HWND::default(), 0, 0).0 <= 0 {
                break;
            }
            
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        // Release mouse capture
        let _ = ReleaseCapture();
        
        // Destroy window and wait for it to be completely destroyed
        if DestroyWindow(hwnd).is_ok() {
            // Process remaining messages to ensure WM_DESTROY is handled
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    break;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        
        // Unregister window class
        let _ = UnregisterClassW(class_name, hinst);
        
        // Restore focus to previous window (Tauri application)
        let prev_hwnd_value = PREVIOUS_HWND.load(std::sync::atomic::Ordering::SeqCst);
        if prev_hwnd_value != 0 {
            let prev_hwnd = HWND(prev_hwnd_value as *mut std::ffi::c_void);
            if !prev_hwnd.is_invalid() {
                let _ = SetForegroundWindow(prev_hwnd);
                let _ = SetFocus(prev_hwnd);
            }
        }
    }
    
    cleanup_screen_bitmap();
    
    // Shutdown GDI+
    shutdown_gdiplus();
    
    if let Ok(state) = STATE.lock() {
        ColorPickerResult {
            foreground: state.fg_color,
            background: state.bg_color,
            continue_mode: state.continue_mode,
        }
    } else {
        ColorPickerResult { foreground: None, background: None, continue_mode: false }
    }
}