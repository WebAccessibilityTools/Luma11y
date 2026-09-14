// =============================================================================
// settings.ts - Settings window entry point
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import Alpine from 'alpinejs';
import { locale as getSystemLocale } from '@tauri-apps/plugin-os';
import {
  initLocale,
  getLocale,
  onLocaleChange,
  setLocale,
  setLocalePreference,
  previewLocalePreference,
  getLocalePreference,
  t as i18nT,
  type LocalePreference,
} from './i18n';
import {
  initTheme,
  applyTheme,
  getThemePreference,
  setThemePreference,
  type ThemePreference,
} from './theme';
import {
  initStyleTheme,
  applyStyleTheme,
  getStyleTheme,
  setStyleTheme,
  type StyleTheme,
} from './styleTheme';
import {
  selectableFormats,
  loadEnabledFormats,
} from './colors';
import './components/AppTitleBar';
import './components/WindowResizeGrips';
import './components/SvgIcon';

// =============================================================================
// SYSTEM LOCALE DETECTION
// =============================================================================

let systemLocale: string | undefined;

// =============================================================================
// ALPINE STORE FOR SETTINGS
// =============================================================================

interface CopyTemplate {
  name: string;
  template: string;
  shortcut: string;
}

interface AppShortcut {
  id: string;
  key: string;
}

const DEFAULT_SHORTCUTS: AppShortcut[] = [
  { id: 'pick_fg', key: 'F11' },
  { id: 'pick_bg', key: 'F12' },
];

function loadShortcuts(): AppShortcut[] {
  try {
    const raw = localStorage.getItem('luma11y-shortcuts');
    if (raw) return JSON.parse(raw);
  } catch {}
  return structuredClone(DEFAULT_SHORTCUTS);
}

const DEFAULT_SHORTCUT = navigator.platform.includes('Mac') ? 'Cmd+S' : 'Ctrl+S';

function loadTemplates(): CopyTemplate[] {
  try {
    const raw = localStorage.getItem('luma11y-copy-templates');
    if (raw) return JSON.parse(raw);
  } catch {}
  return [{ name: i18nT('settings.default_template_name'), template: '%f.hex% / %b.hex% = %cr%:1', shortcut: DEFAULT_SHORTCUT }];
}

function keyboardEventToShortcut(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.metaKey) parts.push('Cmd');
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  parts.push(event.key.length === 1 ? event.key.toUpperCase() : event.key);
  return parts.join('+');
}

// Resolve the locale from localStorage before building the store, so default
// values generated below (e.g. default template name) are already in the
// right language.
initLocale();

Alpine.store('settings', {
  // Current preference
  preference: 'auto' as LocalePreference,

  // Resolved locale for Alpine reactivity
  locale: 'en',

  // Keyboard shortcuts
  shortcuts: loadShortcuts() as AppShortcut[],

  // Copy templates list
  templates: loadTemplates() as CopyTemplate[],

  // Theme light/dark/auto
  theme: getThemePreference() as ThemePreference,

  // Style theme (modern/classic)
  styleTheme: getStyleTheme() as StyleTheme,

  // Toast duration in seconds (0 = manual)
  toastDuration: parseInt(localStorage.getItem('luma11y-toast-duration') ?? '3', 10),

  // Restore the last colour combination on startup
  restoreColors: localStorage.getItem('luma11y-restore-colors') === 'true',

  // Toggleable color formats (excluding hex) and the enabled ones
  selectableFormats: selectableFormats as string[],
  enabledFormats: loadEnabledFormats() as string[],

  // App metadata for the "About" tab
  appInfo: { name: 'Luma11y', version: '', authors: '', description: '' },

  // Translator credits, per language (endonyms).
  translators: [
    { language: 'Français', names: 'Cédric Trévisan' },
    { language: 'Slovenčina', names: 'Radoslav Ďurač' },
  ] as { language: string; names: string }[],

  // Opens a URL in the default browser
  openExternal(url: string): void {
    openUrl(url).catch((err) => console.error('Error opening URL:', err));
  },

  // Toggles a format on/off
  toggleFormat(id: string): void {
    const list = (this as any).enabledFormats as string[];
    const i = list.indexOf(id);
    if (i >= 0) list.splice(i, 1);
    else list.push(id);
  },

  // Reactive translation
  t(key: string, ...args: (string | number)[]): string {
    void (this as any).locale;
    return i18nT(key, ...args);
  },

  // Change theme (preview only, persisted on save)
  setTheme(pref: ThemePreference): void {
    (this as any).theme = pref;
    applyTheme(pref);
  },

  // Change style theme (preview only, persisted on save)
  setStyleTheme(theme: StyleTheme): void {
    (this as any).styleTheme = theme;
    applyStyleTheme(theme);
  },

  // Change locale preference (preview only, persisted on save)
  apply(pref: LocalePreference): void {
    (this as any).preference = pref;
    previewLocalePreference(pref, systemLocale);
  },

  // Update a shortcut
  updateShortcut(index: number, event: KeyboardEvent): void {
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return;
    (this as any).shortcuts[index].key = keyboardEventToShortcut(event);
  },

  // Add a template
  addTemplate(): void {
    (this as any).templates.push({ name: '', template: '', shortcut: '' });
  },

  // Remove a template
  removeTemplate(index: number): void {
    (this as any).templates.splice(index, 1);
  },

  // Update a template's shortcut
  updateTemplateShortcut(index: number, event: KeyboardEvent): void {
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return;
    (this as any).templates[index].shortcut = keyboardEventToShortcut(event);
  },

  // Save preferences
  async save(): Promise<void> {
    // Filter out templates without a name
    (this as any).templates = (this as any).templates.filter((t: CopyTemplate) => t.name.trim() !== '');
    localStorage.setItem('luma11y-copy-templates', JSON.stringify((this as any).templates));
    localStorage.setItem('luma11y-shortcuts', JSON.stringify((this as any).shortcuts));
    localStorage.setItem('luma11y-toast-duration', String((this as any).toastDuration));
    localStorage.setItem('luma11y-enabled-formats', JSON.stringify((this as any).enabledFormats));
    localStorage.setItem('luma11y-restore-colors', String((this as any).restoreColors));

    // Persist theme, style theme and locale
    setThemePreference((this as any).theme);
    setStyleTheme((this as any).styleTheme);
    setLocalePreference((this as any).preference, systemLocale);

    // Sync with backend (native menu)
    try {
      await Promise.all([
        invoke('set_copy_templates', { templates: (this as any).templates }),
        invoke('set_appearance', { appearance: (this as any).theme }),
        invoke('set_style_theme', { style: (this as any).styleTheme }),
        invoke('set_locale', { locale: getLocale() }),
      ]);
    } catch (error) {
      console.error('Error syncing settings to backend:', error);
    }

    // Notify main window to re-register picker hotkeys
    await emit('shortcuts-changed');

    await emit('focus-main');
    getCurrentWindow().close();
  },

  // Cancel changes: just close the window.
  // The Settings context is destroyed on close, and the next opening
  // re-reads localStorage (= saved values).
  async cancel(): Promise<void> {
    await emit('focus-main');
    getCurrentWindow().close();
  },
});

// =============================================================================
// SYNCHRONIZATION
// =============================================================================

// When locale changes, update the reactive locale on the store (used by t()).
onLocaleChange((locale) => {
  const store = Alpine.store('settings') as any;
  store.locale = locale;
});

// Block the webview's native context menu in production builds.
// In dev (vite dev), we keep it so the inspector stays reachable via right-click.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

// =============================================================================
// INITIALIZATION
// =============================================================================

Alpine.start();
initTheme();
initStyleTheme();

// Give keyboard focus to the first tab on opening
requestAnimationFrame(() => {
  const firstTab = document.querySelector<HTMLElement>('[role=tab][data-tab=general]');
  firstTab?.focus({ preventScroll: true });
  getCurrentWindow().show();
});

(async () => {
  // Detect system locale
  try {
    systemLocale = (await getSystemLocale()) ?? undefined;
  } catch (error) {
    console.error('Error detecting system locale:', error);
  }

  // Initialize i18n
  const detectedLocale = initLocale(systemLocale);

  // Sync Alpine store
  const store = Alpine.store('settings') as any;
  store.locale = detectedLocale;
  store.preference = getLocalePreference();

  // Listen for locale changes from native menu or other windows
  await listen<string>('locale-changed', (event) => {
    setLocale(event.payload);
    // Re-read preference as it may have been updated by the other window
    const s = Alpine.store('settings') as any;
    s.preference = getLocalePreference();
  });

  // Fetch app metadata for the "About" tab
  try {
    const info = await invoke<{ name: string; version: string; authors: string; description: string }>('get_app_info');
    (Alpine.store('settings') as any).appInfo = info;
  } catch (error) {
    console.error('Error loading app info:', error);
  }

  // Switch to the requested tab when the window is already open
  await listen<string>('settings-navigate', (event) => {
    window.dispatchEvent(new CustomEvent('settings-navigate', { detail: event.payload }));
  });

  // Initial tab
  try {
    const initialTab = await invoke<string>('get_settings_initial_tab');
    if (initialTab && initialTab !== 'general') {
      window.dispatchEvent(new CustomEvent('settings-navigate', { detail: initialTab }));
    }
  } catch (error) {
    console.error('Error loading initial settings tab:', error);
  }
})();
