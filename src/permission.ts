// =============================================================================
// permission.ts - Permission window entry point
//
// Shown at launch (macOS) if the app lacks the screen recording authorization
// required by the color picker to read pixels.
// =============================================================================

import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import Alpine from 'alpinejs';
import { locale as getSystemLocale } from '@tauri-apps/plugin-os';
import { initLocale, t as i18nT } from './i18n';
import { initTheme } from './theme';
import './components/AppTitleBar';

// =============================================================================
// STORE ALPINE
// =============================================================================

Alpine.store('perm', {
  locale: 'en',

  // Reactive translation
  t(key: string): string {
    void (this as any).locale;
    return i18nT(key);
  },

  // Triggers the system request then opens the correct System Settings pane.
  async openSettings(): Promise<void> {
    try {
      await invoke('request_screen_recording_permission');
      await invoke('open_screen_recording_settings');
    } catch (err) {
      console.error('Error opening screen recording settings:', err);
    }
  },

  // Close the window
  close(): void {
    getCurrentWindow().close();
  },
});

// =============================================================================
// INITIALISATION
// =============================================================================

// Block the webview's native context menu in production builds.
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
  const store = Alpine.store('perm') as any;
  store.locale = detectedLocale;
})();
