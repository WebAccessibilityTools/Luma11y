// =============================================================================
// store.ts - Alpine.js store configuration
// =============================================================================

// Import invoke function to call Tauri commands
import { invoke } from "@tauri-apps/api/core";

// Import i18n module
import { t as i18nT, setLocale } from './i18n';

// Import the color parser and the format registry
import { parseColor, colorFormats, loadEnabledFormats } from './colors';

// Color format and its value
export interface ColorFormatValue {
  // Format identifier (hex, rgb, hsl, ...), used as the radio value
  id: string;

  label: string;
  value: string;
}

// Builds the id -> displayed value map.
function colorValues(hex: string, rgb: string, hsl: string, hsv: string, lab: string, oklch: string): Record<string, string> {
  return { hex, rgb, hsl, hsv, lab, oklch };
}

// Interface for Tauri store (global state on backend side)
export interface BackendStore {
  // Platform
  platform: string;

  // Foreground color in RGB format [r, g, b]
  foreground_rgb: [number, number, number];

  // CSS display string for RGB: "rgb(r, g, b)" or "rgb(r g b / NN%)"
  foreground_rgb_css: string;

  // Foreground color in Hexa format
  foreground_hex: string;

  // Foreground color in HSL format "hsl(h, s%, l%)"
  foreground_hsl: string;

  // Foreground color in HSV format "hsv(h, s%, v%)"
  foreground_hsv: string;

  // Foreground color in CIE L*a*b* format "lab(l, a, b)"
  foreground_lab: string;

  // Foreground color in OKLCH format "oklch(l c h)"
  foreground_oklch: string;

  /// If the colour is dark
  foreground_is_dark: boolean;

  // Foreground opacity
  foreground_alpha: number;

  // Opaque hex of the foreground flattened over the background (alpha compositing)
  foreground_composited_hex: string;

  // Background color in RGB format [r, g, b]
  background_rgb: [number, number, number];

  // CSS display string for RGB: "rgb(r, g, b)"
  background_rgb_css: string;

  // Background color in Hexa format
  background_hex: string;

  // Background color in HSL format "hsl(h, s%, l%)"
  background_hsl: string;

  // Background color in HSV format "hsv(h, s%, v%)"
  background_hsv: string;

  // Background color in CIE L*a*b* format "lab(l, a, b)"
  background_lab: string;

  // Background color in OKLCH format "oklch(l c h)"
  background_oklch: string;

  /// If the colour is dark
  background_is_dark: boolean;

  // Contrast Ratio (Rounded)
  contrast_ratio_rounded: number;

  // Contrast ratio between background and white / black
  background_contrast_with_white: number;
  background_contrast_with_black: number;

  // Indicates if continue mode is enabled
  continue_mode: boolean;
}

// Interface for Alpine.js color picker store (local state on frontend side)
export interface UIStore {
  // Platform
  platform: string;

  // Current locale
  locale: string;

  // Translation
  t(key: string, ...args: (string | number)[]): string;

  // Switch locale
  switchLocale(locale: string): void;

  // Indicates if a color selection is in progress
  isPicking: boolean;

  // Foreground color in RGB format "r, g, b" (raw form for the sliders)
  foregroundRgb: string;

  // CSS display string for RGB ("rgb(r, g, b)" / "rgb(r g b / NN%)"), from the backend
  foregroundRgbCss: string;

  // Foreground color in hexadecimal format
  foregroundHex: string;

  // Foreground color in HSL format "hsl(h, s%, l%)"
  foregroundHsl: string;

  // Foreground color in HSV format "hsv(h, s%, v%)"
  foregroundHsv: string;

  // Foreground color in CIE L*a*b* format "lab(l, a, b)"
  foregroundLab: string;

  // Foreground color in OKLCH format "oklch(l c h)"
  foregroundOklch: string;

  // CSS name of foreground color (empty if no exact match)
  foregroundName: string;

  /// If the colour is dark
  foregroundIsDark: boolean;

  // Foreground opacity
  // Drives the 4th alpha channel and the " / NN%" suffix of the formats
  foregroundAlpha: number;

  // Opaque hex of the foreground flattened over the background (alpha compositing).
  // Shown under the current value when alpha < 1
  foregroundCompositedHex: string;

  // Background color in RGB format "r, g, b" (raw form for the sliders)
  backgroundRgb: string;

  // CSS display string for RGB ("rgb(r, g, b)"), from the backend
  backgroundRgbCss: string;

  // Background color in hexadecimal format
  backgroundHex: string;

  // Background color in HSL format "hsl(h, s%, l%)"
  backgroundHsl: string;

  // Background color in HSV format "hsv(h, s%, v%)"
  backgroundHsv: string;

  // Background color in CIE L*a*b* format "lab(l, a, b)"
  backgroundLab: string;

  // Background color in OKLCH format "oklch(l c h)"
  backgroundOklch: string;

  // CSS name of background color (empty if no exact match)
  backgroundName: string;

  /// If the colour is dark
  backgroundIsDark: boolean;

  // Contrast Ratio Rounded
  contrastRatio: string;

  // Background contrast ratios vs white and vs black
  backgroundContrastWithWhite: number;
  backgroundContrastWithBlack: number;

  // Current theme, kept in sync from theme.ts
  currentTheme: 'light' | 'dark';

  // Background contrast vs the app's surrounding background color
  // (white in light theme, black in dark theme)
  backgroundContrastWithSurrounding: number;

  // List of available formats and their values, for each color
  foregroundFormats: ColorFormatValue[];
  backgroundFormats: ColorFormatValue[];

  // Selected format (id) for display in each color's preview
  foregroundFormat: string;
  backgroundFormat: string;

  // Enabled formats in the interface.
  enabledFormats: string[];

  // Re-reads the enabled formats from localStorage
  refreshEnabledFormats(): void;

  // Value to show in the color preview, depending on the selected format
  foregroundDisplayValue: string;
  backgroundDisplayValue: string;

  // Changes the displayed format for a color ('foreground' | 'background')
  setColorFormat(key: 'foreground' | 'background', format: string): void;

  // Error message shown when the typed hex is invalid (empty otherwise)
  foregroundHexError: string;
  backgroundHexError: string;

  // Currently selected ICC profile
  currentICCProfile: string;

  // Progress bar display mode ('levels' or 'ratios')
  progressLabels: 'levels' | 'ratios';

  // Toggle progress bar display mode
  toggleProgressLabels(): void;

  // WCAG Levels
  level143Regular: boolean;
  level143Large: boolean;
  level146Regular: boolean;
  level146Large: boolean;
  level1411: boolean;

  // Method to launch the color picker
  pickColor(fg: boolean): Promise<void>;

  // Method to swap foreground and background colors
  switchColor(): Promise<void>;

  // Applies a color change emitted by the sliders
  applyColorChange(key: 'foreground' | 'background', detail: { command: string; args: Record<string, number> }): Promise<void>;

  // Update a color from a free-typed text input
  updateColorFromText(key: 'foreground' | 'background', text: string, live?: boolean): Promise<void>;

  // Method to update Alpine store from Tauri store
  updateFromTauriStore(store: BackendStore): void;
}

// =============================================================================
// STORE CONFIGURATION
// =============================================================================

// Alpine.js store configuration exported for use in main.ts
export const UIStore = {
  platform: 'unknown',

  // Current locale (initialized by main.ts)
  locale: 'en',

  // Translation: reads this.locale to create an Alpine reactive dependency
  t(this: UIStore, key: string, ...args: (string | number)[]): string {
    // Reading this.locale creates an Alpine dependency, forcing re-render
    void this.locale;
    return i18nT(key, ...args);
  },

  // Switch locale from the frontend
  switchLocale(_this: UIStore, locale: string): void {
    setLocale(locale);
  },

  // Initial state: no selection in progress
  isPicking: false,

  // Initial state: empty foreground RGB
  foregroundRgb: '',
  foregroundRgbCss: '',

  // Initial state: no foreground color
  foregroundHex: '',

  // Initial state: empty foreground HSL
  foregroundHsl: '',

  // Initial state: empty foreground HSV
  foregroundHsv: '',

  // Initial state: empty foreground Lab
  foregroundLab: '',

  // Initial state: empty foreground OKLCH
  foregroundOklch: '',

  // CSS name of foreground color
  foregroundName: '',

  /// If the colour is dark
  foregroundIsDark: true,

  // Initial state: opaque foreground
  foregroundAlpha: 1,

  // Initial state: empty flattened value (filled on the first store-updated)
  foregroundCompositedHex: '',

  // Initial state: empty background RGB
  backgroundRgb: '',
  backgroundRgbCss: '',

  // Initial state: no background color
  backgroundHex: '',

  // Initial state: empty background HSL
  backgroundHsl: '',

  // Initial state: empty background HSV
  backgroundHsv: '',

  // Initial state: empty background Lab
  backgroundLab: '',

  // Initial state: empty background OKLCH
  backgroundOklch: '',

  // CSS name of background color
  backgroundName: '',

  /// If the colour is dark
  backgroundIsDark: false,

  // Initial state: Contrast ratio
  contrastRatio: '0',

  // Background contrast ratios vs white / vs black
  // (Fixes #9)
  backgroundContrastWithWhite: 1,
  backgroundContrastWithBlack: 1,

  // Current theme (updated from main.ts via onThemeChange)
  currentTheme: 'light' as 'light' | 'dark',

  // Background contrast against the app surrounding bg, depending on theme
  get backgroundContrastWithSurrounding(): number {
    const self = this as unknown as UIStore;
    return self.currentTheme === 'dark'
      ? self.backgroundContrastWithBlack
      : self.backgroundContrastWithWhite;
  },

  // List of {label, value} for the available formats of each color, derived from
  // the colorFormats registry
  get foregroundFormats(): ColorFormatValue[] {
    const self = this as unknown as UIStore;
    const values = colorValues(self.foregroundHex, self.foregroundRgbCss, self.foregroundHsl, self.foregroundHsv, self.foregroundLab, self.foregroundOklch);
    return colorFormats
      .filter((f) => f.id === 'hex' || self.enabledFormats.includes(f.id))
      .map((f) => ({ id: f.id, label: self.t(`color.${f.id}`), value: values[f.id] ?? '' }));
  },

  get backgroundFormats(): ColorFormatValue[] {
    const self = this as unknown as UIStore;
    const values = colorValues(self.backgroundHex, self.backgroundRgbCss, self.backgroundHsl, self.backgroundHsv, self.backgroundLab, self.backgroundOklch);
    return colorFormats
      .filter((f) => f.id === 'hex' || self.enabledFormats.includes(f.id))
      .map((f) => ({ id: f.id, label: self.t(`color.${f.id}`), value: values[f.id] ?? '' }));
  },

  // Value shown in the color preview, depending on the selected format radio.
  get foregroundDisplayValue(): string {
    const self = this as unknown as UIStore;
    const values = colorValues(self.foregroundHex, self.foregroundRgbCss, self.foregroundHsl, self.foregroundHsv, self.foregroundLab, self.foregroundOklch);
    return values[self.foregroundFormat] ?? self.foregroundHex;
  },

  get backgroundDisplayValue(): string {
    const self = this as unknown as UIStore;
    const values = colorValues(self.backgroundHex, self.backgroundRgbCss, self.backgroundHsl, self.backgroundHsv, self.backgroundLab, self.backgroundOklch);
    return values[self.backgroundFormat] ?? self.backgroundHex;
  },

  // Initial state: default displayed format (hexadecimal)
  foregroundFormat: 'hex',
  backgroundFormat: 'hex',

  // Initial state: enabled formats (read from localStorage)
  enabledFormats: loadEnabledFormats(),

  // Changes the displayed format for a color
  setColorFormat(this: UIStore, key: 'foreground' | 'background', format: string) {
    if (key === 'foreground') this.foregroundFormat = format;
    else this.backgroundFormat = format;
  },

  // Re-reads the enabled formats and, if the selected format is no longer
  // available, falls back to hex (always on).
  refreshEnabledFormats(this: UIStore) {
    this.enabledFormats = loadEnabledFormats();
    const available = (id: string) => id === 'hex' || this.enabledFormats.includes(id);
    if (!available(this.foregroundFormat)) this.foregroundFormat = 'hex';
    if (!available(this.backgroundFormat)) this.backgroundFormat = 'hex';
  },

  // Hex input error messages (empty when the value is valid)
  foregroundHexError: '',
  backgroundHexError: '',

  // Initial state: default ICC profile (Auto)
  currentICCProfile: 'Auto',

  // Progress bar display mode
  progressLabels: (localStorage.getItem('luma11y-progress-labels') === 'ratios' ? 'ratios' : 'levels') as 'levels' | 'ratios',

  // Toggle display mode
  toggleProgressLabels(this: UIStore) {
    this.progressLabels = this.progressLabels === 'levels' ? 'ratios' : 'levels';
    localStorage.setItem('luma11y-progress-labels', this.progressLabels);
  },

  // WCAG Levels
  level143Regular: true,
  level143Large: true,
  level146Regular: true,
  level146Large: true,
  level1411: true,

  // Asynchronous method to launch the color picker
  async pickColor(this: UIStore, fg: boolean = true) {
    // Prevent opening a second eyedropper (via shortcuts) when one is already active (Fixes #39)
    if (this.isPicking) return;

    // Enable picking indicator (disables button)
    this.isPicking = true;

    try {
      // Calls Tauri pick_color command with fg parameter (true = foreground, false = background)
      // The call automatically updates Tauri store on backend side
      // and emits "store-updated" event which will be captured by the listener below
      await invoke('pick_color', { fg });
    } catch (error) {
      // Display error in console if selection fails
      console.error('Error:', error);
    } finally {
      // Disable picking indicator (re-enable button)
      this.isPicking = false;
    }
  },

  // Method to swap foreground and background colors
  async switchColor(this: UIStore) {
    // Save current foreground RGB values
    const fgRgb = this.foregroundRgb;
    const bgRgb = this.backgroundRgb;

    // Parse RGB values from "r, g, b" strings
    const [fr, fg, fb] = fgRgb.split(',').map(v => parseInt(v.trim()));
    const [br, bg, bb] = bgRgb.split(',').map(v => parseInt(v.trim()));

    try {
      // Update foreground with old background values
      await invoke('update_store_rgb', { key: 'foreground', r: br, g: bg, b: bb });

      // Update background with old foreground values
      await invoke('update_store_rgb', { key: 'background', r: fr, g: fg, b: fb });
    } catch (error) {
      console.error('Error switching colors:', error);
    }
  },

  // Applies a change emitted by the sliders.
  async applyColorChange(this: UIStore, key: 'foreground' | 'background', detail: { command: string; args: Record<string, number> }) {
    // The conversion is done on the backend side
    // we only send the command and its components.
    try {
      await invoke(detail.command, { key, ...detail.args });
    } catch (error) {
      console.error('Error updating color:', error);
    }
  },

  // Update a color from a free-typed text input.
  // Tries each registered format (hex, rgb, ...) via parseColor.
  // Sets the matching error field when no format recognizes the format.
  async updateColorFromText(this: UIStore, key: 'foreground' | 'background', text: string, live = false) {
    const errorKey = key === 'foreground' ? 'foregroundHexError' : 'backgroundHexError';

    // Empty input: not an error.
    if (text.trim() === '') {
      if (!live) this[errorKey] = '';
      return;
    }

    // Delegates parsing to the format registry (see src/colors/index.ts).
    // The format returns the backend path to invoke for commit and conversion.
    const commit = parseColor(text);
    if (!commit) {
      // We ignore invalid intermediate states (e.g. "255," while
      // typing an rgb); the error only shows on commit.
      if (!live) this[errorKey] = this.t('color.invalid_value');
      return;
    }

    this[errorKey] = '';
    try {
      await invoke(commit.command, { key, ...commit.args });
    } catch (error) {
      console.error('Error updating color from text:', error);
    }
  },

  // Method to synchronize Alpine store with Tauri store
  updateFromTauriStore(this: UIStore, store: BackendStore) {
    this.platform = store.platform;

    // Destructure RGB tuple of foreground color
    const [fr, fg, fb] = store.foreground_rgb;

    // Store the raw RGB version (for the sliders) + the CSS display string
    this.foregroundRgb = `${fr}, ${fg}, ${fb}`;
    this.foregroundRgbCss = store.foreground_rgb_css;

    // Update foreground color (hex format)
    this.foregroundHex = store.foreground_hex;

    // Update foreground color (HSL format)
    this.foregroundHsl = store.foreground_hsl;

    // Update foreground color (HSV format)
    this.foregroundHsv = store.foreground_hsv;

    // Update foreground color (Lab format)
    this.foregroundLab = store.foreground_lab;

    // Update foreground color (OKLCH format)
    this.foregroundOklch = store.foreground_oklch;

    // Look up exact CSS name
    invoke<string>('get_color_name', { r: fr, g: fg, b: fb }).then((name) => {
      this.foregroundName = name;
    });

    /// If the colour is dark
    this.foregroundIsDark = store.foreground_is_dark;

    // Foreground opacity (drives the 4th alpha channel and the " / NN%" suffix)
    this.foregroundAlpha = store.foreground_alpha;

    // Foreground flattened over the background (alpha compositing)
    this.foregroundCompositedHex = store.foreground_composited_hex;

    // Destructure RGB tuple of background color
    const [br, bg, bb] = store.background_rgb;

    // Store the raw RGB version (for the sliders) + the CSS display string
    this.backgroundRgb = `${br}, ${bg}, ${bb}`;
    this.backgroundRgbCss = store.background_rgb_css;

    // Update background color (hex format)
    this.backgroundHex = store.background_hex;

    // Update background color (HSL format)
    this.backgroundHsl = store.background_hsl;

    // Update background color (HSV format)
    this.backgroundHsv = store.background_hsv;
    this.backgroundLab = store.background_lab;
    this.backgroundOklch = store.background_oklch;

    // Look up exact CSS name
    invoke<string>('get_color_name', { r: br, g: bg, b: bb }).then((name) => {
      this.backgroundName = name;
    });

    /// If the colour is dark
    this.backgroundIsDark = store.background_is_dark;

    this.contrastRatio = `${store.contrast_ratio_rounded}`;
    this.backgroundContrastWithWhite = store.background_contrast_with_white;
    this.backgroundContrastWithBlack = store.background_contrast_with_black;

    // WCAG levels from the raw ratio value
    this.level143Regular = true;
    this.level143Large = true;
    this.level146Regular = true;
    this.level146Large = true;
    this.level1411 = true;

    if (store.contrast_ratio_rounded < 7) {
      this.level146Regular = false;
    }
    if (store.contrast_ratio_rounded < 4.5) {
      this.level143Regular = false;
      this.level146Large = false;
    }
    if (store.contrast_ratio_rounded < 3) {
      this.level143Large = false;
      this.level1411 = false;
    }
  }
};