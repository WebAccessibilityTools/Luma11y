//! =============================================================================
//! LINUX_X11.RS - Linux (X11) implementation of the Color Picker
//! =============================================================================
//!
//! This module is the Linux counterpart of `macos.rs`. It uses X11 (through the
//! pure-Rust `x11rb` crate) to capture the screen and create a fullscreen overlay
//! window, and Cairo to draw the magnifier.

//! Unlike macOS where the overlay is truly transparent, we capture the screen
//! ONCE at startup (like the Windows version) and store it in a server pixmap
//! used as the window background. Only the magnifier is rendered client-side
//! and sent on every move; the old one is erased server-side with CopyArea.
//! No dependency on a compositor (no ARGB visual required).

// -----------------------------------------------------------------------------
// Shared configuration
// -----------------------------------------------------------------------------
use crate::config::{
    BORDER_WIDTH,        // Colored border thickness
    CAPTURED_PIXELS,     // Default captured pixels
    CHAR_SPACING_PIXELS, // Spacing between characters
    HEX_FONT_SIZE,       // Hex text font size
    INITIAL_ZOOM_FACTOR, // Initial zoom factor
    SHIFT_MOVE_PIXELS,   // Pixels to move with Shift
    ZOOM_MAX,            // Zoom maximum / Maximum zoom
    ZOOM_MIN,            // Zoom minimum / Minimum zoom
    ZOOM_STEP,           // Zoom increment
};

// -----------------------------------------------------------------------------
// Common types and functions
// -----------------------------------------------------------------------------
use super::common::{
    format_labeled_hex_color, // Formate "Label - #RRGGBB" / Formats "Label - #RRGGBB"
    should_use_dark_text,     // Black or white text
    ColorPickerResult,        // FG/BG result structure
};

// -----------------------------------------------------------------------------
// IMPORTS - X11 (x11rb) and Cairo
// -----------------------------------------------------------------------------
use cairo::{Context, Filter, Format, ImageSurface, SurfacePattern};
use std::error::Error;
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Minimum captured pixels (must stay odd)
const CAPTURED_PIXELS_MIN: f64 = 9.0;

/// Maximum captured pixels (must stay odd)
const CAPTURED_PIXELS_MAX: f64 = 21.0;

/// Increment step for captured pixels (2 to stay odd)
const CAPTURED_PIXELS_STEP: f64 = 2.0;

/// Number of keyboard/pointer grab attempts before giving up
const GRAB_RETRIES: u32 = 20;

/// Delay between two grab attempts
const GRAB_RETRY_DELAY: Duration = Duration::from_millis(25);

// -----------------------------------------------------------------------------
// X11 keysyms used (standard values from <X11/keysymdef.h>)
// -----------------------------------------------------------------------------
const XK_ESCAPE: u32 = 0xff1b;
const XK_RETURN: u32 = 0xff0d;
const XK_KP_ENTER: u32 = 0xff8d;
const XK_SPACE: u32 = 0x0020;
const XK_LEFT: u32 = 0xff51;
const XK_UP: u32 = 0xff52;
const XK_RIGHT: u32 = 0xff53;
const XK_DOWN: u32 = 0xff54;
const XK_C: u32 = 0x0063; // 'c'
const XK_I: u32 = 0x0069; // 'i'
const XK_O: u32 = 0x006f; // 'o'

/// Mathematical constant 2π for Cairo arcs
const TWO_PI: f64 = std::f64::consts::PI * 2.0;
const PI: f64 = std::f64::consts::PI;

// =============================================================================
// STATE
// =============================================================================

/// Simple rectangle (x, y, width, height) in screen coordinates
#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// Complete color picker state, manipulated in the event loop
struct PickerState {
    cursor_x: i32,                  // Cursor X position
    cursor_y: i32,                  // Cursor Y position
    color: (u8, u8, u8),            // Color under cursor
    fg_color: Option<(u8, u8, u8)>, // Selected FG color
    bg_color: Option<(u8, u8, u8)>, // Selected BG color
    fg_mode: bool,                  // true = FG mode (top arc)
    continue_mode: bool,            // Continue mode enabled
    zoom: f64,                      // Current zoom factor
    captured: f64,                  // Captured pixels count
    quit: bool,                     // Quit request
    /// Button whose release we wait for before ungrabbing, so the
    /// ButtonRelease does not land on the application underneath.
    pending_release: Option<u8>,
}

/// Captured screen: raw pixels + metadata for color decoding
struct Screen {
    data: Vec<u8>,    // Raw pixels (server format)
    width: i32,       // Largeur en pixels / Width in pixels
    height: i32,      // Hauteur en pixels / Height in pixels
    stride: usize,    // Bytes per row
    bpp: usize,       // Bytes per pixel
    lsb_first: bool,  // LSB-first byte order
    r_shift: u32,     // Red channel shift
    g_shift: u32,     // Green channel shift
    b_shift: u32,     // Blue channel shift
    r_max: u32,       // Red channel max value
    g_max: u32,       // Green channel max value
    b_max: u32,       // Blue channel max value
}

impl Screen {
    /// Reads a pixel's raw value (respecting byte order)
    #[inline]
    fn raw_pixel(&self, x: i32, y: i32) -> u32 {
        let off = y as usize * self.stride + x as usize * self.bpp;
        // Chemin rapide 32 bpp (cas courant) / 32 bpp fast path (common case)
        if self.bpp == 4 {
            let bytes = [
                self.data[off],
                self.data[off + 1],
                self.data[off + 2],
                self.data[off + 3],
            ];
            return if self.lsb_first {
                u32::from_le_bytes(bytes)
            } else {
                u32::from_be_bytes(bytes)
            };
        }
        let mut px: u32 = 0;
        if self.lsb_first {
            for i in 0..self.bpp {
                px |= (self.data[off + i] as u32) << (8 * i);
            }
        } else {
            for i in 0..self.bpp {
                px = (px << 8) | self.data[off + i] as u32;
            }
        }
        px
    }

    /// Extracts the (R, G, B) color of a pixel using the visual's masks
    #[inline]
    fn pixel(&self, x: i32, y: i32) -> (u8, u8, u8) {
        // Clamp coordinates within the screen
        let x = x.clamp(0, self.width - 1);
        let y = y.clamp(0, self.height - 1);
        let px = self.raw_pixel(x, y);

        // Scale each channel to 0..255 according to its mask width
        let r = (((px >> self.r_shift) & self.r_max) * 255 / self.r_max.max(1)) as u8;
        let g = (((px >> self.g_shift) & self.g_max) * 255 / self.g_max.max(1)) as u8;
        let b = (((px >> self.b_shift) & self.b_max) * 255 / self.b_max.max(1)) as u8;
        (r, g, b)
    }

    /// True when the X pixel layout is byte-identical to Cairo's Rgb24 format
    /// (32 bpp, 8-bit channels at the standard shifts, little-endian). This is
    /// the usual 24-bit TrueColor case: a plain row copy is enough.
    fn is_cairo_rgb24_compatible(&self) -> bool {
        self.bpp == 4
            && self.lsb_first
            && self.r_shift == 16
            && self.g_shift == 8
            && self.b_shift == 0
            && self.r_max == 0xFF
            && self.g_max == 0xFF
            && self.b_max == 0xFF
    }
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Runs the color picker on Linux/X11
///
/// * `fg` - true to start in foreground mode, false for background
///
/// # Returns
/// * `ColorPickerResult` with selected colors (None if cancelled via ESC)
pub fn run(fg: bool) -> ColorPickerResult {
    match run_inner(fg) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Linux color picker error: {e}");
            ColorPickerResult::default()
        }
    }
}

/// Inner implementation that can fail (propagates X11/Cairo errors)
fn run_inner(fg: bool) -> Result<ColorPickerResult, Box<dyn Error>> {
    // -------------------------------------------------------------------------
    //    Explicit refusal under Wayland (see Limitations at the top of the file)
    // -------------------------------------------------------------------------
    if is_wayland_session() {
        return Err("Wayland session detected: the X11 color picker cannot capture \
                    native Wayland windows (the XWayland root would be blank). \
                    Log in to an X11 session to use the picker."
            .into());
    }

    // -------------------------------------------------------------------------
    //    Connect to the X server and gather screen info
    // -------------------------------------------------------------------------
    let (conn, screen_num) = x11rb::connect(None)?;
    let setup = conn.setup();
    let x_screen = &setup.roots[screen_num];
    let root = x_screen.root;
    let depth = x_screen.root_depth;
    let visual_id = x_screen.root_visual;

    // Full desktop geometry (spans all side-by-side monitors)
    let geom = conn.get_geometry(root)?.reply()?;
    let width = geom.width as i32;
    let height = geom.height as i32;

    // Byte order of images returned by the server
    let lsb_first = setup.image_byte_order == ImageOrder::LSB_FIRST;

    // Bytes per pixel for the screen depth (from the pixmap formats)
    let bpp = setup
        .pixmap_formats
        .iter()
        .find(|f| f.depth == depth)
        .map(|f| (f.bits_per_pixel / 8) as usize)
        .unwrap_or(4);

    // RGB masks of the root visual (to decode colors)
    let (r_mask, g_mask, b_mask) = find_visual_masks(x_screen, visual_id)
        .ok_or("Could not find the root visual's color masks")?;

    // -------------------------------------------------------------------------
    //    Capture the screen BEFORE creating our window
    // -------------------------------------------------------------------------
    let image = conn
        .get_image(
            ImageFormat::Z_PIXMAP,
            root,
            0,
            0,
            width as u16,
            height as u16,
            !0u32, // all planes
        )?
        .reply()?;

    let stride = image.data.len() / height as usize;
    let screen = Screen {
        data: image.data,
        width,
        height,
        stride,
        bpp,
        lsb_first,
        r_shift: r_mask.trailing_zeros(),
        g_shift: g_mask.trailing_zeros(),
        b_shift: b_mask.trailing_zeros(),
        r_max: r_mask >> r_mask.trailing_zeros(),
        g_max: g_mask >> g_mask.trailing_zeros(),
        b_max: b_mask >> b_mask.trailing_zeros(),
    };

    // -------------------------------------------------------------------------
    //    Server-side frozen background: a Pixmap receives the capture ONCE, as
    //    is (server format and padding, no conversion). The server repaints
    //    exposed areas itself and erases the old magnifier with CopyArea; the
    //    client never sends more than the magnifier.
    // -------------------------------------------------------------------------
    let bg_pixmap = conn.generate_id()?;
    conn.create_pixmap(depth, bg_pixmap, root, width as u16, height as u16)?;

    // GC shared by pixmap and window (same depth). No GraphicsExpose: CopyArea
    // sources are a pixmap, always fully available.
    let gc = conn.generate_id()?;
    conn.create_gc(gc, bg_pixmap, &CreateGCAux::new().graphics_exposures(0))?;
    upload_pixmap(&conn, bg_pixmap, gc, &screen, depth)?;

    // Working buffer limited to the magnifier (instead of the whole screen)
    let work_size = max_magnifier_size();
    let mut work = ImageSurface::create(Format::Rgb24, work_size, work_size)?;

    // -------------------------------------------------------------------------
    //    Invisible cursor, overlay window (background = pixmap)
    // -------------------------------------------------------------------------
    let invisible_cursor = create_invisible_cursor(&conn, root)?;

    let win = conn.generate_id()?;
    let win_aux = CreateWindowAux::new()
        .override_redirect(1) // Bypass the WM
        .background_pixmap(bg_pixmap)
        .event_mask(
            EventMask::EXPOSURE
                | EventMask::KEY_PRESS
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::STRUCTURE_NOTIFY,
        )
        .cursor(invisible_cursor);
    conn.create_window(
        depth,
        win,
        root,
        0,
        0,
        width as u16,
        height as u16,
        0,
        WindowClass::INPUT_OUTPUT,
        visual_id,
        &win_aux,
    )?;

    conn.map_window(win)?;
    // Raise the window above everything
    conn.configure_window(
        win,
        &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
    )?;

    // Grab keyboard and pointer to receive all events. Check the status and
    // retry: another client (screen locker, compositor, open menu) may hold a
    // grab for a few ms. Without an effective keyboard grab ESC would never
    // reach us and the user would be stuck.
    grab_with_retry("keyboard", || {
        Ok(conn
            .grab_keyboard(true, win, x11rb::CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC)?
            .reply()?
            .status)
    })?;
    grab_with_retry("pointer", || {
        Ok(conn
            .grab_pointer(
                true,
                win,
                EventMask::POINTER_MOTION | EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
                root,
                invisible_cursor,
                x11rb::CURRENT_TIME,
            )?
            .reply()?
            .status)
    })?;

    // Keysym table to translate keycode → keysym
    let min_kc = setup.min_keycode;
    let mapping = conn
        .get_keyboard_mapping(min_kc, setup.max_keycode - min_kc + 1)?
        .reply()?;
    let per = mapping.keysyms_per_keycode as usize;
    let keysyms = mapping.keysyms;

    // -------------------------------------------------------------------------
    //    Initial state: current pointer position + color
    // -------------------------------------------------------------------------
    let pointer = conn.query_pointer(root)?.reply()?;
    let start_x = (pointer.root_x as i32).clamp(0, width - 1);
    let start_y = (pointer.root_y as i32).clamp(0, height - 1);

    let mut state = PickerState {
        cursor_x: start_x,
        cursor_y: start_y,
        color: screen.pixel(start_x, start_y),
        fg_color: None,
        bg_color: None,
        fg_mode: fg,
        continue_mode: false,
        zoom: INITIAL_ZOOM_FACTOR,
        captured: CAPTURED_PIXELS,
        quit: false,
        pending_release: None,
    };

    // Rectangle of the magnifier currently on screen (None before the first draw)
    let mut shown_rect: Option<Rect> = None;

    // First draw: the server already painted the background, only send the magnifier
    redraw(
        &conn,
        win,
        gc,
        bg_pixmap,
        &mut work,
        &screen,
        &state,
        depth,
        width,
        height,
        &mut shown_rect,
    )?;

    // -------------------------------------------------------------------------
    //    Event loop
    // -------------------------------------------------------------------------
    conn.flush()?;
    while !state.quit {
        let mut needs_redraw = false;

        // Handle the blocking event then drain EVERYTHING pending before
        // redrawing: a fast mouse can send hundreds of MotionNotify per second
        // and only the last position matters.
        let mut next = Some(conn.wait_for_event()?);
        while let Some(event) = next {
            match event {
                // The server repainted the background itself (pixmap); only the
                // magnifier, still in the buffer, needs to be shown again.
                Event::Expose(e) => {
                    if e.count == 0 {
                        if let Some(rect) = shown_rect {
                            put_surface(&conn, win, gc, &mut work, depth, rect)?;
                        }
                    }
                }

                // Mouse movement: update position and color
                Event::MotionNotify(e) => {
                    state.cursor_x = (e.root_x as i32).clamp(0, width - 1);
                    state.cursor_y = (e.root_y as i32).clamp(0, height - 1);
                    state.color = screen.pixel(state.cursor_x, state.cursor_y);
                    needs_redraw = true;
                }

                // Clicks and wheel
                Event::ButtonPress(e) => {
                    let shift = e.state.contains(KeyButMask::SHIFT);
                    match e.detail {
                        // Left click: select
                        1 => {
                            select_color(&mut state);
                            needs_redraw = true;
                        }
                        // Right click: cancel (like Windows)
                        3 => cancel(&mut state),
                        // Wheel up
                        4 => {
                            wheel(&mut state, shift, 1.0);
                            needs_redraw = true;
                        }
                        // Wheel down
                        5 => {
                            wheel(&mut state, shift, -1.0);
                            needs_redraw = true;
                        }
                        // Other buttons (middle, horizontal scroll): ignored
                        _ => {}
                    }
                    if state.quit {
                        state.pending_release = Some(e.detail);
                    }
                }

                // Clavier / Keyboard
                Event::KeyPress(e) => {
                    let shift = e.state.contains(KeyButMask::SHIFT);
                    // checked_sub: a keycode < min_keycode must not panic
                    let keysym = e
                        .detail
                        .checked_sub(min_kc)
                        .and_then(|i| keysyms.get(i as usize * per))
                        .copied()
                        .unwrap_or(0);
                    needs_redraw |= handle_key(&conn, root, &screen, &mut state, keysym, shift)?;
                }

                _ => {}
            }

            if state.quit {
                break;
            }
            next = conn.poll_for_event()?;
        }

        // Redraw only the affected region (anti-flicker)
        if needs_redraw && !state.quit {
            redraw(
                &conn,
                win,
                gc,
                bg_pixmap,
                &mut work,
                &screen,
                &state,
                depth,
                width,
                height,
                &mut shown_rect,
            )?;
            conn.flush()?;
        }
    }

    // -------------------------------------------------------------------------
    // 7) Nettoyage / Cleanup
    // -------------------------------------------------------------------------
    // If the pick ended with a mouse button, wait for its release BEFORE
    // ungrabbing: otherwise the ButtonRelease would be delivered to the
    // application under the cursor (same logic as the Windows version).
    if let Some(button) = state.pending_release {
        loop {
            if let Event::ButtonRelease(e) = conn.wait_for_event()? {
                if e.detail == button {
                    break;
                }
            }
        }
    }

    conn.ungrab_pointer(x11rb::CURRENT_TIME)?;
    conn.ungrab_keyboard(x11rb::CURRENT_TIME)?;
    conn.destroy_window(win)?;
    conn.free_cursor(invisible_cursor)?;
    conn.free_gc(gc)?;
    conn.free_pixmap(bg_pixmap)?;
    conn.flush()?;

    Ok(ColorPickerResult {
        foreground: state.fg_color,
        background: state.bg_color,
        continue_mode: state.continue_mode,
    })
}

// =============================================================================
// INPUT LOGIC
// =============================================================================

/// Selects the current color (left click or Enter)
///
/// Mirrors the macOS/Windows logic: in continue mode the first pick toggles
/// fg/bg; the second one finishes. In normal mode it finishes immediately.
fn select_color(state: &mut PickerState) {
    let color = state.color;

    if state.continue_mode {
        // Did we already capture the opposite color?
        let has_other = if state.fg_mode {
            state.bg_color.is_some()
        } else {
            state.fg_color.is_some()
        };

        // Store in the right slot
        if state.fg_mode {
            state.fg_color = Some(color);
        } else {
            state.bg_color = Some(color);
        }

        if has_other {
            // Both colors picked → quit
            state.quit = true;
        } else {
            // Switch to the other mode
            state.fg_mode = !state.fg_mode;
        }
    } else {
        // Normal mode: single color then quit
        if state.fg_mode {
            state.fg_color = Some(color);
        } else {
            state.bg_color = Some(color);
        }
        state.quit = true;
    }
}

/// Cancels the selection: no color is returned (ESC or right click)
fn cancel(state: &mut PickerState) {
    state.fg_color = None;
    state.bg_color = None;
    state.quit = true;
}

/// Adjusts zoom (wheel only) or captured pixels count (Shift+wheel)
fn wheel(state: &mut PickerState, shift: bool, direction: f64) {
    if shift {
        let new = state.captured + direction * CAPTURED_PIXELS_STEP;
        state.captured = new.clamp(CAPTURED_PIXELS_MIN, CAPTURED_PIXELS_MAX);
    } else {
        let new = state.zoom + direction * ZOOM_STEP;
        state.zoom = new.clamp(ZOOM_MIN, ZOOM_MAX);
    }
}

/// Handles a keyboard key. Returns true if a redraw is needed.
fn handle_key(
    conn: &impl Connection,
    root: Window,
    screen: &Screen,
    state: &mut PickerState,
    keysym: u32,
    shift: bool,
) -> Result<bool, Box<dyn Error>> {
    match keysym {
        // cancel everything
        XK_ESCAPE => {
            cancel(state);
            Ok(false)
        }
        // Enter / Space: same logic as a left click (honors continue mode,
        // like the Windows version)
        XK_RETURN | XK_KP_ENTER | XK_SPACE => {
            select_color(state);
            Ok(true)
        }
        // toggle continue mode
        XK_C => {
            state.continue_mode = !state.continue_mode;
            Ok(true)
        }
        // zoom in, or Shift+I = more pixels
        XK_I => {
            if shift {
                state.captured = (state.captured + CAPTURED_PIXELS_STEP).min(CAPTURED_PIXELS_MAX);
            } else {
                state.zoom = (state.zoom + ZOOM_STEP).min(ZOOM_MAX);
            }
            Ok(true)
        }
        // zoom out, or Shift+O = fewer pixels
        XK_O => {
            if shift {
                state.captured = (state.captured - CAPTURED_PIXELS_STEP).max(CAPTURED_PIXELS_MIN);
            } else {
                state.zoom = (state.zoom - ZOOM_STEP).max(ZOOM_MIN);
            }
            Ok(true)
        }
        // Arrows: move the cursor by one pixel (50 with Shift)
        XK_LEFT | XK_RIGHT | XK_UP | XK_DOWN => {
            let step = if shift { SHIFT_MOVE_PIXELS as i32 } else { 1 };
            let (dx, dy) = match keysym {
                XK_LEFT => (-step, 0),
                XK_RIGHT => (step, 0),
                XK_UP => (0, -step),
                XK_DOWN => (0, step),
                _ => (0, 0),
            };
            state.cursor_x = (state.cursor_x + dx).clamp(0, screen.width - 1);
            state.cursor_y = (state.cursor_y + dy).clamp(0, screen.height - 1);
            state.color = screen.pixel(state.cursor_x, state.cursor_y);

            // Actually warp the X pointer to stay consistent
            conn.warp_pointer(
                x11rb::NONE,
                root,
                0,
                0,
                0,
                0,
                state.cursor_x as i16,
                state.cursor_y as i16,
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

// =============================================================================
// DRAWING
// =============================================================================

/// Computes the rectangle (screen coords) covering the magnifier + border + text
fn magnifier_rect(state: &PickerState, screen_w: i32, screen_h: i32) -> Rect {
    let half = magnifier_half(state.captured, state.zoom);
    let x = (state.cursor_x - half).max(0);
    let y = (state.cursor_y - half).max(0);
    let x2 = (state.cursor_x + half).min(screen_w);
    let y2 = (state.cursor_y + half).min(screen_h);
    Rect {
        x,
        y,
        w: (x2 - x).max(0),
        h: (y2 - y).max(0),
    }
}

/// Magnifier half-extent: radius + border + margin for the curved text
fn magnifier_half(captured: f64, zoom: f64) -> i32 {
    (captured * zoom / 2.0 + BORDER_WIDTH + HEX_FONT_SIZE * 2.0 + 6.0).ceil() as i32
}

/// Working buffer size: the magnifier at max zoom and max captured pixels
fn max_magnifier_size() -> i32 {
    magnifier_half(CAPTURED_PIXELS_MAX, ZOOM_MAX) * 2 + 2
}

/// Intersection of two rectangles, None if empty
fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let x2 = (a.x + a.w).min(b.x + b.w);
    let y2 = (a.y + a.h).min(b.y + b.h);
    (x2 > x && y2 > y).then(|| Rect {
        x,
        y,
        w: x2 - x,
        h: y2 - y,
    })
}

/// Parts of `a` not covered by `b` (at most 4 rectangles)
fn subtract(a: Rect, b: Rect) -> Vec<Rect> {
    let Some(i) = intersect(a, b) else {
        return vec![a];
    };
    let mut out = Vec::with_capacity(4);
    // Top and bottom bands, full width
    if i.y > a.y {
        out.push(Rect { x: a.x, y: a.y, w: a.w, h: i.y - a.y });
    }
    let (a_bottom, i_bottom) = (a.y + a.h, i.y + i.h);
    if a_bottom > i_bottom {
        out.push(Rect { x: a.x, y: i_bottom, w: a.w, h: a_bottom - i_bottom });
    }
    // Left and right sides, at the intersection's height
    if i.x > a.x {
        out.push(Rect { x: a.x, y: i.y, w: i.x - a.x, h: i.h });
    }
    let (a_right, i_right) = (a.x + a.w, i.x + i.w);
    if a_right > i_right {
        out.push(Rect { x: i_right, y: i.y, w: a_right - i_right, h: i.h });
    }
    out
}

/// Draws the magnifier into the local buffer, sends it to the window, then
/// erases server-side (CopyArea from the pixmap) what is left of the old one.
/// The new magnifier is sent BEFORE erasing so it never flickers.
#[allow(clippy::too_many_arguments)]
fn redraw(
    conn: &impl Connection,
    win: Window,
    gc: Gcontext,
    bg_pixmap: Pixmap,
    work: &mut ImageSurface,
    screen: &Screen,
    state: &PickerState,
    depth: u8,
    screen_w: i32,
    screen_h: i32,
    shown_rect: &mut Option<Rect>,
) -> Result<(), Box<dyn Error>> {
    let new_rect = magnifier_rect(state, screen_w, screen_h);

    // Frozen background of the rectangle, copied from the raw capture
    fill_background(work, screen, new_rect)?;
    {
        let cr = Context::new(&*work)?;
        // The buffer is magnifier-local: shift the screen coordinate system
        cr.translate(-new_rect.x as f64, -new_rect.y as f64);
        draw_magnifier(&cr, screen, state)?;
    }
    put_surface(conn, win, gc, work, depth, new_rect)?;

    // Erase what the old magnifier left uncovered, without uploading anything
    if let Some(old) = *shown_rect {
        for r in subtract(old, new_rect) {
            conn.copy_area(
                bg_pixmap,
                win,
                gc,
                r.x as i16,
                r.y as i16,
                r.x as i16,
                r.y as i16,
                r.w as u16,
                r.h as u16,
            )?;
        }
    }
    *shown_rect = Some(new_rect);
    Ok(())
}

/// Draws the complete magnifier: zoomed pixels, reticle, arcs, text
fn draw_magnifier(
    cr: &Context,
    screen: &Screen,
    state: &PickerState,
) -> Result<(), Box<dyn Error>> {
    let n = state.captured as i32; // Captured pixels (odd)
    let mag = state.captured * state.zoom; // Magnifier diameter
    let cx = state.cursor_x as f64;
    let cy = state.cursor_y as f64;

    // -------------------------------------------------------------------------
    // Zoomed pixels, clipped to a circle, pixelated rendering (Nearest)
    // -------------------------------------------------------------------------
    let zoom_surface = make_zoom_surface(screen, state.cursor_x, state.cursor_y, n)?;
    cr.save()?;
    cr.arc(cx, cy, mag / 2.0, 0.0, TWO_PI);
    cr.clip();
    cr.translate(cx - mag / 2.0, cy - mag / 2.0);
    let scale = mag / n as f64;
    cr.scale(scale, scale);
    let pattern = SurfacePattern::create(&zoom_surface);
    pattern.set_filter(Filter::Nearest);
    cr.set_source(&pattern)?;
    cr.paint()?;
    cr.restore()?;

    // -------------------------------------------------------------------------
    // Central reticle (gray square the size of one zoomed pixel)
    // -------------------------------------------------------------------------
    let reticle = state.zoom;
    cr.set_source_rgb(0.5, 0.5, 0.5);
    cr.set_line_width(1.0);
    cr.rectangle(cx - reticle / 2.0, cy - reticle / 2.0, reticle, reticle);
    cr.stroke()?;

    // -------------------------------------------------------------------------
    // Colored border (arc) + text. Radius centered on the border thickness.
    // -------------------------------------------------------------------------
    let border_radius = mag / 2.0 + BORDER_WIDTH / 2.0 - 0.5;

    // Font for all text
    cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(HEX_FONT_SIZE);

    // In continue mode: first draw the arc of the already-picked opposite color
    if state.continue_mode {
        let (other, other_is_top) = if state.fg_mode {
            (state.bg_color, false) // On capture le fg (haut) → montre le bg (bas)
        } else {
            (state.fg_color, true) // On capture le bg (bas) → montre le fg (haut)
        };
        if let Some((r, g, b)) = other {
            draw_arc(cr, cx, cy, border_radius, other_is_top, r, g, b)?;
            let label = if other_is_top {
                format_labeled_hex_color("Foreground", r, g, b)
            } else {
                format_labeled_hex_color("Background", r, g, b)
            };
            set_text_color(cr, r, g, b);
            draw_arc_text(cr, &label, cx, cy, border_radius, other_is_top, false)?;
        }
    }

    // Current color arc (top if fg_mode, bottom otherwise)
    let (r, g, b) = state.color;
    draw_arc(cr, cx, cy, border_radius, state.fg_mode, r, g, b)?;

    // Current color hex text
    let label = if state.fg_mode {
        format_labeled_hex_color("Foreground", r, g, b)
    } else {
        format_labeled_hex_color("Background", r, g, b)
    };
    set_text_color(cr, r, g, b);
    draw_arc_text(cr, &label, cx, cy, border_radius, state.fg_mode, state.continue_mode)?;

    Ok(())
}

/// Draws a half-circle (top or bottom arc) of the given color
#[allow(clippy::too_many_arguments)]
fn draw_arc(
    cr: &Context,
    cx: f64,
    cy: f64,
    radius: f64,
    is_top: bool,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), Box<dyn Error>> {
    cr.set_line_width(BORDER_WIDTH);
    cr.set_source_rgb(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    cr.new_path();
    // In Cairo coordinates (Y down): top = [π, 2π], bottom = [0, π]
    if is_top {
        cr.arc(cx, cy, radius, PI, TWO_PI);
    } else {
        cr.arc(cx, cy, radius, 0.0, PI);
    }
    cr.stroke()?;
    Ok(())
}

/// Sets the text color (black or white) based on background luminance
fn set_text_color(cr: &Context, r: u8, g: u8, b: u8) {
    if should_use_dark_text(r, g, b) {
        cr.set_source_rgb(0.0, 0.0, 0.0);
    } else {
        cr.set_source_rgb(1.0, 1.0, 1.0);
    }
}

/// Draws curved text along an arc (top or bottom)
///
/// Each character is positioned and oriented tangentially to the arc, like the
/// macOS `draw_arc_text`, but in Cairo coordinates (Y down).
fn draw_arc_text(
    cr: &Context,
    text: &str,
    cx: f64,
    cy: f64,
    radius: f64,
    is_top: bool,
    show_badge: bool,
) -> Result<(), Box<dyn Error>> {
    let badge_extra = if show_badge { 2.0 } else { 0.0 };
    let char_count = text.chars().count() as f64 + badge_extra;

    // Angle between two characters (constant pixel spacing)
    let angle_step = CHAR_SPACING_PIXELS / radius;
    let total = angle_step * (char_count - 1.0);

    for (i, ch) in text.chars().enumerate() {
        let idx = i as f64;
        // Character angle. Top center = -π/2, bottom center = +π/2.
        let angle = if is_top {
            -PI / 2.0 - total / 2.0 + angle_step * idx
        } else {
            PI / 2.0 + total / 2.0 - angle_step * idx
        };
        draw_glyph(cr, &ch.to_string(), cx, cy, radius, angle, is_top)?;
    }

    // Red "C" badge at the end of the text if continue mode is active
    if show_badge {
        let idx = text.chars().count() as f64 + 1.0; // skip one space
        let angle = if is_top {
            -PI / 2.0 - total / 2.0 + angle_step * idx
        } else {
            PI / 2.0 + total / 2.0 - angle_step * idx
        };
        let bx = cx + radius * angle.cos();
        let by = cy + radius * angle.sin();
        let badge_radius = HEX_FONT_SIZE * 0.7;

        // Cercle rouge / Red circle
        cr.set_source_rgb(0.9, 0.1, 0.1);
        cr.new_path();
        cr.arc(bx, by, badge_radius, 0.0, TWO_PI);
        cr.fill()?;

        // White "C", oriented like the rest
        cr.set_source_rgb(1.0, 1.0, 1.0);
        draw_glyph(cr, "C", cx, cy, radius, angle, is_top)?;
    }

    Ok(())
}

/// Draws a single character, centered and rotated tangentially to the arc
fn draw_glyph(
    cr: &Context,
    s: &str,
    cx: f64,
    cy: f64,
    radius: f64,
    angle: f64,
    is_top: bool,
) -> Result<(), Box<dyn Error>> {
    let x = cx + radius * angle.cos();
    let y = cy + radius * angle.sin();
    // Rotation so text is upright and readable along the arc
    let rotation = if is_top {
        angle + PI / 2.0
    } else {
        angle - PI / 2.0
    };

    cr.save()?;
    cr.translate(x, y);
    cr.rotate(rotation);
    // Center the glyph on the origin
    let ext = cr.text_extents(s)?;
    cr.move_to(
        -ext.width() / 2.0 - ext.x_bearing(),
        -ext.height() / 2.0 - ext.y_bearing(),
    );
    cr.show_text(s)?;
    cr.restore()?;
    Ok(())
}

/// Builds a small n×n Cairo surface with the pixels around the cursor
fn make_zoom_surface(
    screen: &Screen,
    cx: i32,
    cy: i32,
    n: i32,
) -> Result<ImageSurface, Box<dyn Error>> {
    let mut surface = ImageSurface::create(Format::Rgb24, n, n)?;
    let stride = surface.stride() as usize;
    {
        let mut data = surface.data()?;
        let half = n / 2;
        for j in 0..n {
            for i in 0..n {
                let (r, g, b) = screen.pixel(cx - half + i, cy - half + j);
                let o = j as usize * stride + i as usize * 4;
                data[o] = b;
                data[o + 1] = g;
                data[o + 2] = r;
            }
        }
    }
    Ok(surface)
}

// =============================================================================
// IMAGE TRANSFER (Cairo → X11 window)
// =============================================================================

/// Sends the raw capture into the server pixmap, in bands of full rows: format
/// and padding are the server's own, no conversion needed.
fn upload_pixmap(
    conn: &impl Connection,
    pixmap: Pixmap,
    gc: Gcontext,
    screen: &Screen,
    depth: u8,
) -> Result<(), Box<dyn Error>> {
    // Max rows per request (margin for the request header)
    let max_bytes = conn.maximum_request_bytes();
    let max_rows = ((max_bytes.saturating_sub(64)) / screen.stride.max(1)).max(1) as i32;

    let mut row = 0i32;
    while row < screen.height {
        let band_h = (screen.height - row).min(max_rows);
        let start = row as usize * screen.stride;
        let end = (row + band_h) as usize * screen.stride;
        conn.put_image(
            ImageFormat::Z_PIXMAP,
            pixmap,
            gc,
            screen.width as u16,
            band_h as u16,
            0,
            row as i16,
            0, // left_pad
            depth,
            &screen.data[start..end],
        )?;
        row += band_h;
    }
    Ok(())
}

/// Copies the frozen background of `rect` (screen coordinates) to the top-left
/// of the working buffer.
fn fill_background(
    work: &mut ImageSurface,
    screen: &Screen,
    rect: Rect,
) -> Result<(), Box<dyn Error>> {
    work.flush();
    let cairo_stride = work.stride() as usize;
    {
        let mut data = work.data()?;
        if screen.is_cairo_rgb24_compatible() {
            // Fast path: same layout (BGRX LE), row copy
            let row_bytes = rect.w as usize * 4;
            for y in 0..rect.h as usize {
                let src = (rect.y as usize + y) * screen.stride + rect.x as usize * 4;
                data[y * cairo_stride..y * cairo_stride + row_bytes]
                    .copy_from_slice(&screen.data[src..src + row_bytes]);
            }
        } else {
            // Generic path: decode through the visual's masks
            for y in 0..rect.h {
                for x in 0..rect.w {
                    let (r, g, b) = screen.pixel(rect.x + x, rect.y + y);
                    let o = y as usize * cairo_stride + x as usize * 4;
                    // Cairo Rgb24 = native 0x00RRGGBB → bytes B, G, R, X on little-endian
                    data[o] = b;
                    data[o + 1] = g;
                    data[o + 2] = r;
                }
            }
        }
    }
    // Pixels were written behind Cairo's back: tell it
    work.mark_dirty();
    Ok(())
}

/// Sends the top-left (rect.w × rect.h) of the working buffer to screen
/// position (rect.x, rect.y), in bands to respect the max X request size.
fn put_surface(
    conn: &impl Connection,
    win: Window,
    gc: Gcontext,
    work: &mut ImageSurface,
    depth: u8,
    rect: Rect,
) -> Result<(), Box<dyn Error>> {
    if rect.w <= 0 || rect.h <= 0 {
        return Ok(());
    }
    work.flush();
    let cairo_stride = work.stride() as usize;
    let data = work.data()?;

    let row_bytes = rect.w as usize * 4;
    // Max rows per request (margin for the request header)
    let max_bytes = conn.maximum_request_bytes();
    let max_rows = ((max_bytes.saturating_sub(64)) / row_bytes.max(1)).max(1);

    let mut row = 0i32;
    while row < rect.h {
        let band_h = (rect.h - row).min(max_rows as i32);
        // Build a contiguous buffer (rows of row_bytes, no internal padding)
        let mut buf = Vec::with_capacity(band_h as usize * row_bytes);
        for r in 0..band_h {
            let start = (row + r) as usize * cairo_stride;
            buf.extend_from_slice(&data[start..start + row_bytes]);
        }
        conn.put_image(
            ImageFormat::Z_PIXMAP,
            win,
            gc,
            rect.w as u16,
            band_h as u16,
            rect.x as i16,
            (rect.y + row) as i16,
            0, // left_pad
            depth,
            &buf,
        )?;
        row += band_h;
    }
    Ok(())
}

// =============================================================================
// X11 HELPERS
// =============================================================================

/// True when the graphical session is Wayland. `XDG_SESSION_TYPE` is set by
/// logind; `WAYLAND_DISPLAY` covers sessions started otherwise. `DISPLAY` is
/// not a criterion: XWayland sets it too.
pub fn is_wayland_session() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|v| v.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
        || std::env::var("WAYLAND_DISPLAY")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
}

/// Attempts a grab until the server answers SUCCESS, with a few spaced
/// retries. Fails if another client keeps the grab.
fn grab_with_retry<F>(what: &str, mut try_grab: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut() -> Result<GrabStatus, Box<dyn Error>>,
{
    for _ in 0..GRAB_RETRIES {
        if try_grab()? == GrabStatus::SUCCESS {
            return Ok(());
        }
        std::thread::sleep(GRAB_RETRY_DELAY);
    }
    Err(format!("Could not grab the {what} (held by another X client)").into())
}

/// Finds the (red, green, blue) masks of the given visual
fn find_visual_masks(screen: &Screen11, visual_id: Visualid) -> Option<(u32, u32, u32)> {
    for depth in &screen.allowed_depths {
        for v in &depth.visuals {
            if v.visual_id == visual_id {
                return Some((v.red_mask, v.green_mask, v.blue_mask));
            }
        }
    }
    None
}

/// Creates an invisible cursor (empty 1×1 pixmap) to hide the pointer
fn create_invisible_cursor(
    conn: &impl Connection,
    root: Window,
) -> Result<Cursor, Box<dyn Error>> {
    let pixmap = conn.generate_id()?;
    conn.create_pixmap(1, pixmap, root, 1, 1)?;
    let cursor = conn.generate_id()?;
    // source = mask = empty pixmap → fully transparent cursor
    conn.create_cursor(cursor, pixmap, pixmap, 0, 0, 0, 0, 0, 0, 0, 0)?;
    conn.free_pixmap(pixmap)?;
    Ok(cursor)
}

// Alias for x11rb's Screen type (avoids the clash with our own Screen struct)
use x11rb::protocol::xproto::Screen as Screen11;
