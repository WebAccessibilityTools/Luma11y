// =============================================================================
// permission.ts - Point d'entrée de la fenêtre de permission
// permission.ts - Permission window entry point
//
// Affichée au lancement (macOS) si l'app n'a pas l'autorisation d'enregistrement
// de l'écran, nécessaire à la pipette pour lire les pixels.
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

  // Traduction réactive / Reactive translation
  t(key: string): string {
    void (this as any).locale;
    return i18nT(key);
  },

  // Déclenche la demande système puis ouvre le bon volet des Réglages Système.
  // Triggers the system request then opens the correct System Settings pane.
  async openSettings(): Promise<void> {
    try {
      await invoke('request_screen_recording_permission');
      await invoke('open_screen_recording_settings');
    } catch (err) {
      console.error('Error opening screen recording settings:', err);
    }
  },

  // Ferme la fenêtre / Close the window
  close(): void {
    getCurrentWindow().close();
  },
});

// =============================================================================
// INITIALISATION
// =============================================================================

// Bloque le menu contextuel natif de la webview en build de production.
// Block the webview's native context menu in production builds.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

Alpine.start();
initTheme();

(async () => {
  // Détecte la locale système / Detect system locale
  let systemLocale: string | undefined;
  try {
    systemLocale = (await getSystemLocale()) ?? undefined;
  } catch {}

  const detectedLocale = initLocale(systemLocale);
  const store = Alpine.store('perm') as any;
  store.locale = detectedLocale;
})();
