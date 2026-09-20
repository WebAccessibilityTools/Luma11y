// =============================================================================
// main.ts - Frontend application entry point
// =============================================================================

// Import invoke function to call Tauri commands from the frontend
import { invoke } from "@tauri-apps/api/core";

// Import listen function to listen to events emitted by Tauri
import { listen } from "@tauri-apps/api/event";

// Import current window for resizing
import { getCurrentWindow, PhysicalSize } from "@tauri-apps/api/window";

// Import Alpine.js for user interface reactivity
import Alpine from 'alpinejs';

// Import store and interfaces from store.ts
import { UIStore, BackendStore } from './store';

// Import i18n module
import { initLocale, onLocaleChange, setLocale, t as i18nT } from './i18n';
import { check, type Update } from '@tauri-apps/plugin-updater';

// Import system locale detection via Tauri plugin OS
import { locale as getSystemLocale } from '@tauri-apps/plugin-os';

// Import theme module
import { initTheme, applyTheme, getThemePreference, setThemePreference, getCurrentTheme, onThemeChange } from './theme';

// Import style theme module
import { initStyleTheme, applyStyleTheme, getStyleTheme, setStyleTheme } from './styleTheme';

// Import for window creation
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

// Platform helper import
import { IS_MAC } from './lib/platform';

// Global (system-wide) shortcuts
import {
  register as registerGlobalShortcut,
  unregisterAll as unregisterAllGlobalShortcuts,
  type ShortcutEvent,
} from "@tauri-apps/plugin-global-shortcut";

// Import Webcomponents
import './components/ColorControls';
import './components/ProgressBar';
import './components/SvgIcon';
import './components/AppTitleBar';
import './components/WindowResizeGrips';
import './components/AppMenubar';

// =============================================================================
// AUTOMATIC WINDOW RESIZING
// See https://github.com/tauri-apps/tauri/issues/12420
// =============================================================================

// Adjusts Tauri window size to match content using PhysicalSize
let lastSetHeight = 0;

async function resizeWindow() {
  const currentWindow = getCurrentWindow();
  const factor = window.devicePixelRatio;
  const currentSize = await currentWindow.innerSize();
  const width = currentSize.width;

  // Observe body so header (titlebar) + main are both included.
  const bodyHeight = document.body.getBoundingClientRect().height;
  const bufferCssPx = 10; /* Add some space to the bottom of the app. (Fixes #11) */
  const newHeight = Math.ceil((bodyHeight + bufferCssPx) * factor);

  // Avoid resize loops on Windows
  if (Math.abs(newHeight - lastSetHeight) < 2) return;
  lastSetHeight = newHeight;

  await currentWindow.setSize(new PhysicalSize(width, newHeight));
}

// Initialize ResizeObserver on <body> after DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  const observer = new ResizeObserver(() => {
    resizeWindow();
  });
  observer.observe(document.body);

  // The window is created hidden (tauri.conf.json `visible: false`): Show it once the first size adjustment is done.
  resizeWindow()
    .catch(() => {})
    .finally(() => {
      getCurrentWindow().show();
    });
});

// Block the webview's native context menu in production builds.
// In dev (vite dev), we keep it so the inspector stays reachable via right-click.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

// =============================================================================
// KEYBOARD SHORTCUT TO COPY RESULTS
// =============================================================================

function eventToShortcut(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.metaKey) parts.push('Cmd');
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
  return parts.join('+');
}

function formatTemplate(template: string, store: UIStore): string {
  return template
    .replace(/%f\.hex%/g, store.foregroundHex)
    .replace(/%b\.hex%/g, store.backgroundHex)
    .replace(/%f\.hsl%/g, store.foregroundHsl)
    .replace(/%b\.hsl%/g, store.backgroundHsl)
    .replace(/%f\.hsv%/g, store.foregroundHsv)
    .replace(/%b\.hsv%/g, store.backgroundHsv)
    .replace(/%f\.lab%/g, store.foregroundLab)
    .replace(/%b\.lab%/g, store.backgroundLab)
    .replace(/%f\.oklch%/g, store.foregroundOklch)
    .replace(/%b\.oklch%/g, store.backgroundOklch)
    .replace(/%cr%/g, store.contrastRatio)
    .replace(/%crr%/g, store.contrastRatio)
    .replace(/%1\.4\.3%/g, store.level143Regular ? 'Pass' : 'Fail')
    .replace(/%1\.4\.6%/g, store.level146Regular ? 'Pass' : 'Fail')
    .replace(/%1\.4\.11%/g, store.level1411 ? 'Pass' : 'Fail');
}

let toastTimeout: ReturnType<typeof setTimeout>;
function showCopyToast(text: string) {
  const toast = document.getElementById('copy-toast');
  if (!toast) return;
  const duration = parseInt(localStorage.getItem('luma11y-toast-duration') ?? '3', 10);

  toast.textContent = text;

  // Manual mode (duration === 0): add a close button
  if (duration === 0) {
    const btn = document.createElement('button');
    btn.textContent = '\u00d7';
    btn.className = 'toast-close';
    btn.onclick = () => toast.classList.remove('visible');
    toast.appendChild(btn);
  }

  toast.classList.add('visible');
  clearTimeout(toastTimeout);
  if (duration > 0) {
    toastTimeout = setTimeout(() => toast.classList.remove('visible'), duration * 1000);
  }
}

interface CopyTemplate {
  name: string;
  template: string;
  shortcut: string;
}

interface AppShortcut {
  id: string;
  key: string;
}

function loadCopyTemplates(): CopyTemplate[] {
  try {
    const raw = localStorage.getItem('luma11y-copy-templates');
    if (raw) return JSON.parse(raw);
  } catch {}
  const defaultShortcut = navigator.platform.includes('Mac') ? 'Cmd+S' : 'Ctrl+S';
  return [{ name: i18nT('settings.default_template_name'), template: '%f.hex% / %b.hex% = %cr%:1', shortcut: defaultShortcut }];
}

// Load all shortcuts
function loadShortcuts(): AppShortcut[] {
  try {
    const raw = localStorage.getItem('luma11y-shortcuts');
    if (raw) return JSON.parse(raw);
  } catch {}
  return [
    { id: 'pick_fg', key: 'F11' },
    { id: 'pick_bg', key: 'F12' },
  ];
}

// Listener for app shortcuts (Not global shortcuts).
document.addEventListener('keydown', (e) => {
  // We ignore auto-repeat via e.repeat to avoid multiple triggers.
  if (e.repeat) return;

  const pressed = eventToShortcut(e);

  const templates = loadCopyTemplates();
  for (const tpl of templates) {
    if (tpl.shortcut && tpl.shortcut === pressed) {
      e.preventDefault();
      const store = Alpine.store('uiStore') as UIStore;
      const text = formatTemplate(tpl.template, store);
      navigator.clipboard.writeText(text);
      showCopyToast(text);
      return;
    }
  }
});

// Registers picker shortcuts as system-wide hotkeys (work even when the app
// is not focused).
async function registerPickerShortcuts(): Promise<void> {
  try {
    await unregisterAllGlobalShortcuts();
  } catch (err) {
    console.error('Error unregistering global shortcuts:', err);
  }

  for (const sc of loadShortcuts()) {
    if (!sc.key) continue;
    if (sc.id !== 'pick_fg' && sc.id !== 'pick_bg') continue;
    try {
      await registerGlobalShortcut(sc.key, (event: ShortcutEvent) => {
        // Trigger on key release to avoid reacting to auto-repeat events
        // when the key is held down.
        if (event.state !== 'Released') return;
        const store = Alpine.store('uiStore') as UIStore;
        store.pickColor(sc.id === 'pick_fg');
      });
    } catch (err) {
      // Combination already taken by another app, or invalid syntax
      console.error(`Failed to register global shortcut "${sc.key}" for ${sc.id}:`, err);
    }
  }
}

// =============================================================================
// PERMISSION CHECK (macOS: screen recording)
// =============================================================================

// Checks at launch that the app can capture the screen (required by the color
// picker).
async function checkScreenRecordingPermission(): Promise<void> {
  if (!IS_MAC) return;

  let granted = false;
  try {
    granted = await invoke<boolean>('check_screen_recording_permission');
  } catch (err) {
    console.error('Error checking screen recording permission:', err);
    return;
  }

  // Permission already granted: nothing to do.
  if (granted) return;

  new WebviewWindow('permission', {
    url: 'permission.html',
    title: i18nT('permissions.screen_recording_title'),
    width: 480,
    height: 340,
    resizable: false,
    maximizable: false,
    minimizable: false,
    center: true,
    ...(IS_MAC
      ? { titleBarStyle: 'overlay' as const, hiddenTitle: true }
      : { decorations: false, transparent: true }),
  });
}

// =============================================================================
// ALPINE.JS STORE CONFIGURATION
// =============================================================================

// Register the store in Alpine.js with the name 'uiStore'
Alpine.store('uiStore', UIStore);

// Update banner: one check at startup
type UpdaterStore = { update: Update | null; later(): void; view(): void };
const updaterStore: UpdaterStore = {
  update: null,
  // Dismiss for this session; the check runs again at next startup.
  later() {
    this.update = null;
  },
  // Open the Updates tab of the settings.
  view() {
    this.update = null;
    invoke('open_settings_window', { tab: 'update' }).catch(() => {});
  },
};
Alpine.store('updater', updaterStore);

// =============================================================================
// BIDIRECTIONAL i18n SYNCHRONIZATION
// =============================================================================

// When locale changes on frontend (setLocale), sync Alpine and Rust
onLocaleChange((locale) => {
  const alpineStore = Alpine.store('uiStore') as UIStore;
  alpineStore.locale = locale;

  // Notify Rust backend to rebuild menus
  invoke('set_locale', { locale }).catch((err) => {
    console.error('Error setting locale in backend:', err);
  });
});

// =============================================================================
// INITIALIZATION
// =============================================================================

// Initialize Alpine.js and activate reactivity in the DOM
Alpine.start();

// Dev only: create access to the stores from the webview console (e.g. to fake an update)
if (import.meta.env.DEV) (window as unknown as { Alpine: typeof Alpine }).Alpine = Alpine;

initTheme();
initStyleTheme();

// Keep the current theme reactive in the Alpine store for reactive getters
// (e.g. backgroundContrastWithSurrounding).
(Alpine.store('uiStore') as UIStore).currentTheme = getCurrentTheme();
onThemeChange((theme) => {
  (Alpine.store('uiStore') as UIStore).currentTheme = theme;
});

// =============================================================================
// LAST COLOUR COMBINATION PERSISTENCE
// =============================================================================

const RESTORE_COLORS_KEY = 'luma11y-restore-colors';
const LAST_COLORS_KEY = 'luma11y-last-colors';
const ALWAYS_ON_TOP_KEY = 'luma11y-always-on-top';

interface LastColors {
  fg: [number, number, number];
  bg: [number, number, number];
  fgAlpha: number;
}

// True once init is done: avoids persisting the default state on load (which
// would overwrite the remembered last combination).
let persistColorsReady = false;

// Split the "r, g, b" string into 3 numeric values
function parseRgbTriplet(value: string): [number, number, number] | null {
  const parts = value.split(',').map((v) => parseInt(v.trim(), 10));
  if (parts.length === 3 && parts.every((n) => Number.isFinite(n))) {
    return [parts[0], parts[1], parts[2]];
  }
  return null;
}

// Remembers the current colour combination
function saveLastColors(): void {
  const store = Alpine.store('uiStore') as UIStore;
  const fg = parseRgbTriplet(store.foregroundRgb);
  const bg = parseRgbTriplet(store.backgroundRgb);
  if (!fg || !bg) return;
  const data: LastColors = { fg, bg, fgAlpha: store.foregroundAlpha };
  try {
    localStorage.setItem(LAST_COLORS_KEY, JSON.stringify(data));
  } catch (err) {
    console.error('Error saving last colours:', err);
  }
}

// Re-applies the last combination and recomputes all derived values
async function restoreLastColors(): Promise<void> {
  if (localStorage.getItem(RESTORE_COLORS_KEY) !== 'true') return;
  let data: LastColors;
  try {
    const raw = localStorage.getItem(LAST_COLORS_KEY);
    if (!raw) return;
    data = JSON.parse(raw);
  } catch (err) {
    console.error('Error reading last colours:', err);
    return;
  }
  if (!Array.isArray(data.fg) || !Array.isArray(data.bg)) return;
  try {
    await invoke('update_store_rgb', { key: 'background', r: data.bg[0], g: data.bg[1], b: data.bg[2] });
    await invoke('update_store_rgb', { key: 'foreground', r: data.fg[0], g: data.fg[1], b: data.fg[2], alpha: data.fgAlpha ?? 1 });
  } catch (err) {
    console.error('Error restoring last colours:', err);
  }
}

// Remembers the current always-on-top state
function saveLastAlwaysOnTop(value: boolean): void {
  try {
    localStorage.setItem(ALWAYS_ON_TOP_KEY, String(value));
  } catch (err) {
    console.error('Error saving always-on-top state:', err);
  }
}

// Re-applies the always-on-top state from the last session
async function restoreLastAlwaysOnTop(): Promise<void> {
  if (localStorage.getItem(ALWAYS_ON_TOP_KEY) !== 'true') return;
  try {
    await invoke('set_always_on_top', { value: true });
  } catch (err) {
    console.error('Error restoring always-on-top state:', err);
  }
}

// Immediately Invoked Async Function Expression (IIFE) for Tauri synchronization
(async () => {
  // Step 0: Detect system locale and initialize i18n
  let detectedLocale = 'en';
  try {
    const systemLocale = await getSystemLocale();
    detectedLocale = initLocale(systemLocale ?? undefined);
  } catch (error) {
    console.error('Error detecting system locale:', error);
    detectedLocale = initLocale();
  }

  // Sync locale into Alpine store
  const alpineStore = Alpine.store('uiStore') as UIStore;
  alpineStore.locale = detectedLocale;

  // Send initial locale to backend
  try {
    await invoke('set_locale', { locale: detectedLocale });
  } catch (error) {
    console.error('Error setting initial locale:', error);
  }

  // Sync appearance (auto/light/dark) with native menu
  try {
    await invoke('set_appearance', { appearance: getThemePreference() });
  } catch (error) {
    console.error('Error setting initial appearance:', error);
  }

  // Sync style theme (modern/classic) with native menu
  try {
    await invoke('set_style_theme', { style: getStyleTheme() });
  } catch (error) {
    console.error('Error setting initial style theme:', error);
  }

  // Step 0b: Open style chooser on first launch
  if (!localStorage.getItem('luma11y-style-theme')) {
    const chooser = new WebviewWindow('style-chooser', {
      url: 'style-chooser.html',
      title: i18nT('style_chooser.title'),
      width: 620,
      height: 400,
      resizable: false,
      maximizable: false,
      center: true,
      ...(IS_MAC
        ? { titleBarStyle: 'overlay' as const, hiddenTitle: true }
        : { decorations: false, transparent: true }),
    });

    // Wait for user to choose a style before continuing
    await new Promise<void>((resolve) => {
      listen<string>('style-chosen', (event) => {
        applyStyleTheme(event.payload as any);
        resolve();
      });
      // If window is closed without choosing, apply default
      chooser.onCloseRequested(() => {
        if (!localStorage.getItem('luma11y-style-theme')) {
          localStorage.setItem('luma11y-style-theme', 'modern');
          applyStyleTheme('modern');
        }
        resolve();
      });
    });
  }

  // Step 1: Fetch initial Tauri store state on page load
  try {
    // Call get_store command to get current backend state
    const initialStore = await invoke<BackendStore>('get_store');

    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Synchronize Alpine store with Tauri's initial state
    store.updateFromTauriStore(initialStore);
  } catch (error) {
    // Display error if initial load fails
    console.error('Error loading initial store:', error);
  }

  // Step 2: Continuously listen for Tauri store updates
  await listen<BackendStore>('store-updated', (event) => {
    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Synchronize Alpine store with new payload received from Tauri
    // This makes the interface reactive to backend changes
    store.updateFromTauriStore(event.payload);

    // Remember the colors combination
    if (persistColorsReady) saveLastColors();
  });

  // Step 2b: Restore the last colour combination if the option is enabled
  await restoreLastColors();
  persistColorsReady = true;

  // Step 2c: Remember every always-on-top change and restore the last state
  await listen<boolean>('always-on-top-changed', (event) => {
    saveLastAlwaysOnTop(event.payload);
  });
  await restoreLastAlwaysOnTop();

  // Step 3: Listen for ICC profile changes from the menu
  await listen<string>('icc-profile-changed', (event) => {
    // Get the selected ICC profile name
    const profileName = event.payload;

    // Display selected profile in console (for debug)
    console.log('ICC Profile changed to:', profileName);

    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Update ICC profile in Alpine store
    store.currentICCProfile = profileName;
  });

  // Step 3b: Listen for appearance changes from the native menu
  await listen<string>('appearance-changed', (event) => {
    setThemePreference(event.payload as 'auto' | 'light' | 'dark');
  });

  // Step 3c: Listen for style theme changes from the native menu
  await listen<string>('style-theme-changed', (event) => {
    setStyleTheme(event.payload as 'modern' | 'classic');
  });

  // Step 3d: Register pickers as system-wide hotkeys
  // and re-register when Settings saves
  await registerPickerShortcuts();
  await listen('shortcuts-changed', () => {
    registerPickerShortcuts();
  });

  // Step 4: Listen for locale changes from native Rust menu
  await listen<string>('locale-changed', (event) => {
    const locale = event.payload;

    // Calls setLocale which updates the i18n module and triggers onLocaleChange
    // Note: onLocaleChange will invoke invoke('set_locale') but backend is already up to date,
    // so it's a no-op on Rust side (locale is already correct)
    setLocale(locale);
  });

  // Step 5: Listen for focus-main event from settings window
  await listen('focus-main', () => {
    getCurrentWindow().setFocus();
    // Re-apply theme in case it changed in settings
    applyTheme();
    applyStyleTheme();
    // Reload the enabled color formats
    (Alpine.store('uiStore') as UIStore).refreshEnabledFormats();
  });

  // Step 5b: Send copy templates to backend for Edit menu
  try {
    const templates = loadCopyTemplates();
    await invoke('set_copy_templates', { templates });
  } catch (error) {
    console.error('Error sending templates to backend:', error);
  }

  // Step 5c: Listen for copy template clicks from native menu
  await listen<number>('copy-template', (event) => {
    const index = event.payload;
    const templates = loadCopyTemplates();
    if (index < templates.length) {
      const store = Alpine.store('uiStore') as UIStore;
      const text = formatTemplate(templates[index].template, store);
      navigator.clipboard.writeText(text);
      showCopyToast(text);
    }
  });

  // Step 5d: Check screen recording permission (macOS)
  await checkScreenRecordingPermission();

  // Step 6: Get initial ICC profile
  try {
    // Call command to get currently selected ICC profile
    const currentProfile = await invoke<string | null>('get_selected_icc_profile');

    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Update ICC profile in store (or 'Auto' as default)
    store.currentICCProfile = currentProfile || 'Auto';
  } catch (error) {
    // Display error if retrieval fails
    console.error('Error loading ICC profile:', error);
  }
  // Step 7: automatic update check
  if (localStorage.getItem('luma11y-auto-update') !== 'false') {
    try {
      (Alpine.store('updater') as UpdaterStore).update = await check();
    } catch (err) {
      console.warn('Update check failed:', err);
    }
  }
})();
