// =============================================================================
// settings.ts - Point d'entrée de la fenêtre Settings
// settings.ts - Settings window entry point
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
  isDefaultTemplateName,
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

// =============================================================================
// DÉTECTION LOCALE SYSTÈME
// SYSTEM LOCALE DETECTION
// =============================================================================

let systemLocale: string | undefined;

// =============================================================================
// STORE ALPINE POUR SETTINGS
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
    if (raw) {
      const parsed = JSON.parse(raw) as CopyTemplate[];
      // Migre le nom du modèle par défaut sauvegardé avant que sa traduction
      // n'existe pour la locale courante (jamais les modèles renommés par l'utilisateur).
      // Migrate a default template name saved before its translation existed
      // for the current locale (never templates the user actually renamed).
      for (const tpl of parsed) {
        if (tpl.template === '%f.hex% / %b.hex% = %cr%:1' && isDefaultTemplateName(tpl.name)) {
          tpl.name = i18nT('settings.default_template_name');
        }
      }
      return parsed;
    }
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

// Résout la locale depuis localStorage avant de construire le store, pour que
// les valeurs par défaut générées ci-dessous (ex. nom du modèle par défaut)
// soient déjà dans la bonne langue.
// Resolve the locale from localStorage before building the store, so default
// values generated below (e.g. default template name) are already in the
// right language.
initLocale();

Alpine.store('settings', {
  // Préférence actuelle / Current preference
  preference: 'auto' as LocalePreference,

  // Locale résolue pour réactivité Alpine / Resolved locale for Alpine reactivity
  locale: 'en',

  // Raccourcis clavier / Keyboard shortcuts
  shortcuts: loadShortcuts() as AppShortcut[],

  // Liste des modèles de copie / Copy templates list
  templates: loadTemplates() as CopyTemplate[],

  // Thème light/dark/auto / Theme light/dark/auto
  theme: getThemePreference() as ThemePreference,

  // Thème de style (moderne/classique) / Style theme (modern/classic)
  styleTheme: getStyleTheme() as StyleTheme,

  // Durée du toast en secondes (0 = manuel) / Toast duration in seconds (0 = manual)
  toastDuration: parseInt(localStorage.getItem('luma11y-toast-duration') ?? '3', 10),

  // Formats de couleur activables (hors hex) et ceux activés
  // Toggleable color formats (excluding hex) and the enabled ones
  selectableFormats: selectableFormats as string[],
  enabledFormats: loadEnabledFormats() as string[],

  // Active/désactive un format
  // Toggles a format on/off
  toggleFormat(id: string): void {
    const list = (this as any).enabledFormats as string[];
    const i = list.indexOf(id);
    if (i >= 0) list.splice(i, 1);
    else list.push(id);
  },

  // Traduction réactive / Reactive translation
  t(key: string, ...args: (string | number)[]): string {
    void (this as any).locale;
    return i18nT(key, ...args);
  },

  // Change le thème (preview uniquement, persisté à la sauvegarde)
  // Change theme (preview only, persisted on save)
  setTheme(pref: ThemePreference): void {
    (this as any).theme = pref;
    applyTheme(pref);
  },

  // Change le thème de style (preview uniquement, persisté à la sauvegarde)
  // Change style theme (preview only, persisted on save)
  setStyleTheme(theme: StyleTheme): void {
    (this as any).styleTheme = theme;
    applyStyleTheme(theme);
  },

  // Change la préférence de locale (preview uniquement, persisté à la sauvegarde)
  // Change locale preference (preview only, persisted on save)
  apply(pref: LocalePreference): void {
    (this as any).preference = pref;
    previewLocalePreference(pref, systemLocale);
  },

  // Met à jour un raccourci / Update a shortcut
  updateShortcut(index: number, event: KeyboardEvent): void {
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return;
    (this as any).shortcuts[index].key = keyboardEventToShortcut(event);
  },

  // Ajoute un modèle / Add a template
  addTemplate(): void {
    (this as any).templates.push({ name: '', template: '', shortcut: '' });
  },

  // Supprime un modèle / Remove a template
  removeTemplate(index: number): void {
    (this as any).templates.splice(index, 1);
  },

  // Met à jour le raccourci d'un modèle / Update a template's shortcut
  updateTemplateShortcut(index: number, event: KeyboardEvent): void {
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return;
    (this as any).templates[index].shortcut = keyboardEventToShortcut(event);
  },

  // Sauvegarde les préférences / Save preferences
  async save(): Promise<void> {
    // Filtre les modèles sans nom / Filter out templates without a name
    (this as any).templates = (this as any).templates.filter((t: CopyTemplate) => t.name.trim() !== '');
    localStorage.setItem('luma11y-copy-templates', JSON.stringify((this as any).templates));
    localStorage.setItem('luma11y-shortcuts', JSON.stringify((this as any).shortcuts));
    localStorage.setItem('luma11y-toast-duration', String((this as any).toastDuration));
    localStorage.setItem('luma11y-enabled-formats', JSON.stringify((this as any).enabledFormats));

    // Persiste le thème, le style et la locale
    // Persist theme, style theme and locale
    setThemePreference((this as any).theme);
    setStyleTheme((this as any).styleTheme);
    setLocalePreference((this as any).preference, systemLocale);

    // Synchronise avec le backend (menu natif)
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

    // Notifie la fenêtre principale pour ré-enregistrer les hotkeys pickers
    // Notify main window to re-register picker hotkeys
    await emit('shortcuts-changed');

    await emit('focus-main');
    getCurrentWindow().close();
  },

  // Annule les modifications : ferme simplement la fenêtre.
  // Le contexte de Settings est détruit à la fermeture, et la prochaine
  // ouverture re-lit localStorage (= valeurs sauvegardées).
  // Cancel changes: just close the window.
  // The Settings context is destroyed on close, and the next opening
  // re-reads localStorage (= saved values).
  async cancel(): Promise<void> {
    await emit('focus-main');
    getCurrentWindow().close();
  },
});

// =============================================================================
// SYNCHRONISATION
// SYNCHRONIZATION
// =============================================================================

// Quand la locale change, met à jour la locale réactive du store (pour t())
// When locale changes, update the reactive locale on the store (used by t()).
onLocaleChange((locale) => {
  const store = Alpine.store('settings') as any;
  store.locale = locale;
});

// Bloque le menu contextuel natif de la webview en build de production.
// En dev (vite dev), on le laisse pour pouvoir ouvrir l'inspecteur via clic droit.
// Block the webview's native context menu in production builds.
// In dev (vite dev), we keep it so the inspector stays reachable via right-click.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

// =============================================================================
// INITIALISATION
// INITIALIZATION
// =============================================================================

Alpine.start();
initTheme();
initStyleTheme();

// Place le focus clavier sur la première tab, dès l'ouverture
// Give keyboard focus to the first tab on opening
requestAnimationFrame(() => {
  const firstTab = document.querySelector<HTMLElement>('[role=tab][data-tab=general]');
  firstTab?.focus({ preventScroll: true });
  getCurrentWindow().show();
});

(async () => {
  // Détecte la locale système / Detect system locale
  try {
    systemLocale = (await getSystemLocale()) ?? undefined;
  } catch (error) {
    console.error('Error detecting system locale:', error);
  }

  // Initialise i18n / Initialize i18n
  const detectedLocale = initLocale(systemLocale);

  // Synchronise le store Alpine / Sync Alpine store
  const store = Alpine.store('settings') as any;
  store.locale = detectedLocale;
  store.preference = getLocalePreference();

  // Écoute les changements de locale depuis le menu natif ou d'autres fenêtres
  // Listen for locale changes from native menu or other windows
  await listen<string>('locale-changed', (event) => {
    setLocale(event.payload);
    // Relit la préférence car elle a pu être mise à jour par l'autre fenêtre
    // Re-read preference as it may have been updated by the other window
    const s = Alpine.store('settings') as any;
    s.preference = getLocalePreference();
  });
})();
