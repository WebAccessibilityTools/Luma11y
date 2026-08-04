// =============================================================================
// main.ts - Point d'entrée de l'application frontend
// main.ts - Frontend application entry point
// =============================================================================

// Import de la fonction invoke pour appeler les commandes Tauri depuis le frontend
// Import invoke function to call Tauri commands from the frontend
import { invoke } from "@tauri-apps/api/core";

// Import de la fonction listen pour écouter les événements émis par Tauri
// Import listen function to listen to events emitted by Tauri
import { listen } from "@tauri-apps/api/event";

// Import de la fenêtre courante pour le redimensionnement
// Import current window for resizing
import { getCurrentWindow, PhysicalSize } from "@tauri-apps/api/window";

// Import d'Alpine.js pour la réactivité de l'interface utilisateur
// Import Alpine.js for user interface reactivity
import Alpine from 'alpinejs';

// Import du store et des interfaces depuis store.ts
// Import store and interfaces from store.ts
import { UIStore, BackendStore } from './store';

// Import du module i18n
// Import i18n module
import { initLocale, onLocaleChange, setLocale, t as i18nT, isDefaultTemplateName } from './i18n';

// Import de la détection de locale système via Tauri plugin OS
// Import system locale detection via Tauri plugin OS
import { locale as getSystemLocale } from '@tauri-apps/plugin-os';

// Import du module thème / Import theme module
import { initTheme, applyTheme, getThemePreference, setThemePreference, getCurrentTheme, onThemeChange } from './theme';

// Import du module thème de style / Import style theme module
import { initStyleTheme, applyStyleTheme, getStyleTheme, setStyleTheme } from './styleTheme';

// Import pour la création de fenêtre / Import for window creation
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

// Import du helper de plateforme / Platform helper import
import { IS_MAC } from './lib/platform';

// Import pour les raccourcis globaux (système-wide) / Global (system-wide) shortcuts
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
// REDIMENSIONNEMENT AUTOMATIQUE DE LA FENÊTRE
// AUTOMATIC WINDOW RESIZING
// See https://github.com/tauri-apps/tauri/issues/12420
// =============================================================================

// Ajuste la taille de la fenêtre Tauri au contenu via PhysicalSize
// Adjusts Tauri window size to match content using PhysicalSize
let lastSetHeight = 0;

async function resizeWindow() {
  const currentWindow = getCurrentWindow();
  const factor = window.devicePixelRatio;
  const currentSize = await currentWindow.innerSize();
  const width = currentSize.width;

  // On observe body pour inclure header (titlebar) + main dans le calcul.
  // Observe body so header (titlebar) + main are both included.
  const bodyHeight = document.body.getBoundingClientRect().height;
  const bufferCssPx = 10; /* Add some space to the bottom of the app. (Fixes #11) */
  const newHeight = Math.ceil((bodyHeight + bufferCssPx) * factor);

  // Évite les boucles de redimensionnement sur Windows
  // Avoid resize loops on Windows
  if (Math.abs(newHeight - lastSetHeight) < 2) return;
  lastSetHeight = newHeight;

  await currentWindow.setSize(new PhysicalSize(width, newHeight));
}

// Initialise le ResizeObserver sur <body> après le chargement du DOM
// Initialize ResizeObserver on <body> after DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  const observer = new ResizeObserver(() => {
    resizeWindow();
  });
  observer.observe(document.body);
});

// Bloque le menu contextuel natif de la webview en build de production.
// En dev (vite dev), on le laisse pour pouvoir ouvrir l'inspecteur via clic droit.
// Block the webview's native context menu in production builds.
// In dev (vite dev), we keep it so the inspector stays reachable via right-click.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

// =============================================================================
// RACCOURCI CLAVIER POUR COPIER LES RÉSULTATS
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

  // Mode manuel (duration === 0) : ajoute un bouton fermer
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

// Enregistre les raccourcis pickers en hotkeys système (fonctionnent même
// quand l'app n'a pas le focus).
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
        // Déclenche au relâchement de la touche pour éviter de réagir à
        // l'auto-repeat (la touche restant enfoncée).
        // Trigger on key release to avoid reacting to auto-repeat events
        // when the key is held down.
        if (event.state !== 'Released') return;
        const store = Alpine.store('uiStore') as UIStore;
        store.pickColor(sc.id === 'pick_fg');
      });
    } catch (err) {
      // Combinaison déjà prise par une autre app, ou syntaxe invalide
      // Combination already taken by another app, or invalid syntax
      console.error(`Failed to register global shortcut "${sc.key}" for ${sc.id}:`, err);
    }
  }
}

// =============================================================================
// CONFIGURATION DU STORE ALPINE.JS
// ALPINE.JS STORE CONFIGURATION
// =============================================================================

// Enregistre le store dans Alpine.js avec le nom 'uiStore'
// Register the store in Alpine.js with the name 'uiStore'
Alpine.store('uiStore', UIStore);

// =============================================================================
// SYNCHRONISATION i18n BIDIRECTIONNELLE
// BIDIRECTIONAL i18n SYNCHRONIZATION
// =============================================================================

// Quand la locale change côté frontend (setLocale), on synchronise Alpine et Rust
// When locale changes on frontend (setLocale), sync Alpine and Rust
onLocaleChange((locale) => {
  const alpineStore = Alpine.store('uiStore') as UIStore;
  alpineStore.locale = locale;

  // Notifie le backend Rust pour reconstruire les menus
  // Notify Rust backend to rebuild menus
  invoke('set_locale', { locale }).catch((err) => {
    console.error('Error setting locale in backend:', err);
  });
});

// =============================================================================
// INITIALISATION
// INITIALIZATION
// =============================================================================

// Initialise Alpine.js et active la réactivité dans le DOM
// Initialize Alpine.js and activate reactivity in the DOM
Alpine.start();
initTheme();
initStyleTheme();

// Synchronise le thème courant dans le store Alpine pour les getters réactifs
// (ex. backgroundContrastWithSurrounding).
// Keep the current theme reactive in the Alpine store for reactive getters
// (e.g. backgroundContrastWithSurrounding).
(Alpine.store('uiStore') as UIStore).currentTheme = getCurrentTheme();
onThemeChange((theme) => {
  (Alpine.store('uiStore') as UIStore).currentTheme = theme;
});

// Fonction immédiatement invoquée asynchrone (IIFE) pour la synchronisation avec Tauri
// Immediately Invoked Async Function Expression (IIFE) for Tauri synchronization
(async () => {
  // Étape 0 : Détection de la locale système et initialisation i18n
  // Step 0: Detect system locale and initialize i18n
  let detectedLocale = 'en';
  try {
    const systemLocale = await getSystemLocale();
    detectedLocale = initLocale(systemLocale ?? undefined);
  } catch (error) {
    console.error('Error detecting system locale:', error);
    detectedLocale = initLocale();
  }

  // Synchronise la locale dans le store Alpine
  // Sync locale into Alpine store
  const alpineStore = Alpine.store('uiStore') as UIStore;
  alpineStore.locale = detectedLocale;

  // Envoie la locale initiale au backend
  // Send initial locale to backend
  try {
    await invoke('set_locale', { locale: detectedLocale });
  } catch (error) {
    console.error('Error setting initial locale:', error);
  }

  // Synchronise l'apparence (auto/light/dark) avec le menu natif
  // Sync appearance (auto/light/dark) with native menu
  try {
    await invoke('set_appearance', { appearance: getThemePreference() });
  } catch (error) {
    console.error('Error setting initial appearance:', error);
  }

  // Synchronise le thème de style (modern/classic) avec le menu natif
  // Sync style theme (modern/classic) with native menu
  try {
    await invoke('set_style_theme', { style: getStyleTheme() });
  } catch (error) {
    console.error('Error setting initial style theme:', error);
  }

  // Étape 0b : Ouvre le sélecteur de style au premier lancement
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

    // Attend que l'utilisateur choisisse un style avant de continuer
    // Wait for user to choose a style before continuing
    await new Promise<void>((resolve) => {
      listen<string>('style-chosen', (event) => {
        applyStyleTheme(event.payload as any);
        resolve();
      });
      // Si la fenêtre est fermée sans choix, applique le défaut
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

  // Étape 1 : Récupération de l'état initial du store Tauri au chargement de la page
  // Step 1: Fetch initial Tauri store state on page load
  try {
    // Appelle la commande get_store pour obtenir l'état actuel du backend
    // Call get_store command to get current backend state
    const initialStore = await invoke<BackendStore>('get_store');

    // Récupère la référence au store Alpine.js
    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Synchronise le store Alpine avec l'état initial de Tauri
    // Synchronize Alpine store with Tauri's initial state
    store.updateFromTauriStore(initialStore);
  } catch (error) {
    // Affiche l'erreur si le chargement initial échoue
    // Display error if initial load fails
    console.error('Error loading initial store:', error);
  }

  // Étape 2 : Écoute en continu des mises à jour du store Tauri
  // Step 2: Continuously listen for Tauri store updates
  await listen<BackendStore>('store-updated', (event) => {
    // Récupère la référence au store Alpine.js
    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Synchronise le store Alpine avec le nouveau payload reçu de Tauri
    // Ceci rend l'interface réactive aux changements du backend
    // Synchronize Alpine store with new payload received from Tauri
    // This makes the interface reactive to backend changes
    store.updateFromTauriStore(event.payload);
  });

  // Étape 3 : Écoute les changements de profil ICC depuis le menu
  // Step 3: Listen for ICC profile changes from the menu
  await listen<string>('icc-profile-changed', (event) => {
    // Récupère le nom du profil ICC sélectionné
    // Get the selected ICC profile name
    const profileName = event.payload;

    // Affiche le profil sélectionné dans la console (pour debug)
    // Display selected profile in console (for debug)
    console.log('ICC Profile changed to:', profileName);

    // Récupère la référence au store Alpine.js
    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Met à jour le profil ICC dans le store Alpine
    // Update ICC profile in Alpine store
    store.currentICCProfile = profileName;
  });

  // Étape 3b : Écoute les changements d'apparence depuis le menu natif
  // Step 3b: Listen for appearance changes from the native menu
  await listen<string>('appearance-changed', (event) => {
    setThemePreference(event.payload as 'auto' | 'light' | 'dark');
  });

  // Étape 3c : Écoute les changements de thème de style depuis le menu natif
  // Step 3c: Listen for style theme changes from the native menu
  await listen<string>('style-theme-changed', (event) => {
    setStyleTheme(event.payload as 'modern' | 'classic');
  });

  // Étape 3d : Enregistre les pickers en hotkeys système
  // et ré-enregistre quand Settings sauvegarde
  // Step 3d: Register pickers as system-wide hotkeys
  // and re-register when Settings saves
  await registerPickerShortcuts();
  await listen('shortcuts-changed', () => {
    registerPickerShortcuts();
  });

  // Étape 4 : Écoute les changements de locale depuis le menu natif Rust
  // Step 4: Listen for locale changes from native Rust menu
  await listen<string>('locale-changed', (event) => {
    const locale = event.payload;

    // Appelle setLocale qui mettra à jour le module i18n et déclenchera onLocaleChange
    // Calls setLocale which updates the i18n module and triggers onLocaleChange
    // Note: onLocaleChange invoquera invoke('set_locale') mais le backend est déjà à jour,
    // donc c'est un no-op côté Rust (la locale est déjà la bonne)
    // Note: onLocaleChange will invoke invoke('set_locale') but backend is already up to date,
    // so it's a no-op on Rust side (locale is already correct)
    setLocale(locale);
  });

  // Étape 5 : Écoute l'événement focus-main depuis la fenêtre settings
  // Step 5: Listen for focus-main event from settings window
  await listen('focus-main', () => {
    getCurrentWindow().setFocus();
    // Réapplique le thème en cas de changement dans les settings
    // Re-apply theme in case it changed in settings
    applyTheme();
    applyStyleTheme();
    // Re-charge les formats de couleur activés
    // Reload the enabled color formats
    (Alpine.store('uiStore') as UIStore).refreshEnabledFormats();
  });

  // Étape 5b : Envoie les modèles de copie au backend pour le menu Édition
  // Step 5b: Send copy templates to backend for Edit menu
  try {
    const templates = loadCopyTemplates();
    await invoke('set_copy_templates', { templates });
  } catch (error) {
    console.error('Error sending templates to backend:', error);
  }

  // Étape 5c : Écoute les clics sur les modèles de copie depuis le menu natif
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

  // Étape 6 : Récupère le profil ICC initial
  // Step 6: Get initial ICC profile
  try {
    // Appelle la commande pour obtenir le profil ICC actuellement sélectionné
    // Call command to get currently selected ICC profile
    const currentProfile = await invoke<string | null>('get_selected_icc_profile');

    // Récupère la référence au store Alpine.js
    // Get reference to Alpine.js store
    const store = Alpine.store('uiStore') as UIStore;

    // Met à jour le profil ICC dans le store (ou 'Auto' par défaut)
    // Update ICC profile in store (or 'Auto' as default)
    store.currentICCProfile = currentProfile || 'Auto';
  } catch (error) {
    // Affiche l'erreur si la récupération échoue
    // Display error if retrieval fails
    console.error('Error loading ICC profile:', error);
  }
})();
