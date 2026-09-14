// =============================================================================
// style-chooser.ts - Style chooser window entry point
//
// Shown on first launch if no style has been chosen yet.
// =============================================================================

import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import Alpine from 'alpinejs';
import { locale as getSystemLocale } from '@tauri-apps/plugin-os';
import { initLocale, t as i18nT } from './i18n';
import { initTheme } from './theme';
import './components/AppTitleBar';

// =============================================================================
// STORE ALPINE
// =============================================================================

Alpine.store('chooser', {
  locale: 'en',

  // Reactive translation
  t(key: string): string {
    void (this as any).locale;
    return i18nT(key);
  },

  // Select a style, save and close the window
  async choose(style: 'modern' | 'classic'): Promise<void> {
    localStorage.setItem('luma11y-style-theme', style);
    await emit('style-chosen', style);
    getCurrentWindow().close();
  },
});

// =============================================================================
// INITIALISATION
// =============================================================================

// Block the webview's native context menu in production builds.
// In dev (vite dev), we keep it so the inspector stays reachable via right-click.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

Alpine.start();
initTheme();

(async () => {
  // Detect system locale
  let systemLocale: string | undefined;
  try {
    systemLocale = (await getSystemLocale()) ?? undefined;
  } catch {}

  const detectedLocale = initLocale(systemLocale);
  const store = Alpine.store('chooser') as any;
  store.locale = detectedLocale;
})();
