// =============================================================================
// i18n.ts - Internationalization module
// =============================================================================

import en from './locales/en.json';
import fr from './locales/fr.json';
import sk from './locales/sk.json';

// Available translations
const translations: Record<string, Record<string, unknown>> = { en, fr, sk };

// Supported languages
const SUPPORTED_LOCALES = ['en', 'fr', 'sk'];
const DEFAULT_LOCALE = 'en';
const STORAGE_KEY = 'luma11y-locale';
const PREFERENCE_KEY = 'luma11y-locale-preference';

// Preference type
export type LocalePreference = 'auto' | 'en' | 'fr' | 'sk';

// Current locale
let currentLocale = DEFAULT_LOCALE;

// Change callbacks
type LocaleChangeCallback = (locale: string) => void;
const callbacks: LocaleChangeCallback[] = [];

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Normalizes a BCP-47 tag to a supported locale
function normalizeLocale(tag: string): string {
  // Extract language code (before first dash)
  const lang = tag.split('-')[0].toLowerCase();
  return SUPPORTED_LOCALES.includes(lang) ? lang : DEFAULT_LOCALE;
}

/// Resolves a nested key in an object (e.g., "app.title")
function resolve(obj: Record<string, unknown>, key: string): string | undefined {
  const parts = key.split('.');
  let current: unknown = obj;
  for (const part of parts) {
    if (current === null || current === undefined || typeof current !== 'object') {
      return undefined;
    }
    current = (current as Record<string, unknown>)[part];
  }
  return typeof current === 'string' ? current : undefined;
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Returns the translation for a key, with EN fallback then raw key
export function t(key: string, ...args: (string | number)[]): string {
  const raw = resolve(translations[currentLocale], key)
    ?? resolve(translations[DEFAULT_LOCALE], key)
    ?? key;
  if (args.length === 0) return raw;
  return raw.replace(/\{(\d+)\}/g, (_, i) => String(args[Number(i)] ?? _));
}

/// Returns the current locale
export function getLocale(): string {
  return currentLocale;
}

/// Returns the stored locale preference ('auto', 'en', 'fr', or 'sk')
export function getLocalePreference(): LocalePreference {
  const stored = localStorage.getItem(PREFERENCE_KEY);
  if (stored === 'auto' || stored === 'en' || stored === 'fr' || stored === 'sk') {
    return stored;
  }
  // If no preference but an explicit locale stored, it's an explicit choice
  const legacyLocale = localStorage.getItem(STORAGE_KEY);
  if (legacyLocale && SUPPORTED_LOCALES.includes(legacyLocale)) {
    return legacyLocale as LocalePreference;
  }
  return 'auto';
}

/// Applies an effective locale without persisting (preview)
///
/// @param pref - 'auto', 'en', 'fr', or 'sk'
/// @param systemLocale - system locale (required when pref === 'auto')
export function previewLocalePreference(pref: LocalePreference, systemLocale?: string): void {
  const effective = pref === 'auto'
    ? (systemLocale ? normalizeLocale(systemLocale) : DEFAULT_LOCALE)
    : pref;

  if (effective !== currentLocale) {
    currentLocale = effective;
    for (const cb of callbacks) {
      cb(effective);
    }
  }
}

/// Changes the locale preference and applies the effective locale
///
/// @param pref - 'auto', 'en', 'fr', or 'sk'
/// @param systemLocale - system locale (required when pref === 'auto')
export function setLocalePreference(pref: LocalePreference, systemLocale?: string): void {
  localStorage.setItem(PREFERENCE_KEY, pref);

  // Resolve effective locale
  const effective = pref === 'auto'
    ? (systemLocale ? normalizeLocale(systemLocale) : DEFAULT_LOCALE)
    : pref;

  // Apply if changed
  if (effective !== currentLocale) {
    currentLocale = effective;
    localStorage.setItem(STORAGE_KEY, effective);
    for (const cb of callbacks) {
      cb(effective);
    }
  }
}

/// Changes the locale and persists in localStorage
export function setLocale(locale: string): void {
  const normalized = normalizeLocale(locale);
  if (normalized === currentLocale) return;

  currentLocale = normalized;
  localStorage.setItem(STORAGE_KEY, normalized);

  // Also update preference to explicit value
  localStorage.setItem(PREFERENCE_KEY, normalized);

  // Notify callbacks
  for (const cb of callbacks) {
    cb(normalized);
  }
}

/// Initializes the locale: preference > localStorage > system > EN
export function initLocale(systemLocale?: string): string {
  // Check preference first
  const pref = localStorage.getItem(PREFERENCE_KEY);
  if (pref === 'auto') {
    currentLocale = systemLocale ? normalizeLocale(systemLocale) : DEFAULT_LOCALE;
  } else if (pref && SUPPORTED_LOCALES.includes(pref)) {
    currentLocale = pref;
  } else {
    // Legacy fallback: localStorage > system > EN
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && SUPPORTED_LOCALES.includes(stored)) {
      currentLocale = stored;
    } else if (systemLocale) {
      currentLocale = normalizeLocale(systemLocale);
    } else {
      currentLocale = DEFAULT_LOCALE;
    }
  }
  return currentLocale;
}

/// Registers a callback called on each locale change
export function onLocaleChange(callback: LocaleChangeCallback): void {
  callbacks.push(callback);
}
