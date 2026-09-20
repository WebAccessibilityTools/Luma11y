// =============================================================================
// styleTheme.ts - Style theme management (modern / classic)
// =============================================================================

export type StyleTheme = 'modern' | 'classic';

const STORAGE_KEY = 'luma11y-style-theme';

const STYLE_ELEMENT_ID = 'luma11y-style-theme';

/** Returns saved preference */
export function getStyleTheme(): StyleTheme {
  return (localStorage.getItem(STORAGE_KEY) as StyleTheme) || 'modern';
}

/** Save preference and apply */
export function setStyleTheme(theme: StyleTheme): void {
  localStorage.setItem(STORAGE_KEY, theme);
  applyStyleTheme(theme);
}

/** Apply style theme */
export async function applyStyleTheme(theme?: StyleTheme): Promise<void> {
  const resolved = theme ?? getStyleTheme();

  // Remove previously injected style
  const existing = document.getElementById(STYLE_ELEMENT_ID);
  if (existing) existing.remove();

  let css = await import('./themes/modern.css?inline').then(m => m.default);
  if (resolved === 'classic')
    css = await import('./themes/classic.css?inline').then(m => m.default);
  const style = document.createElement('style');
  style.id = STYLE_ELEMENT_ID;
  style.textContent = css;
  document.head.appendChild(style);
}

/** Initialize style theme */
export function initStyleTheme(): void {
  applyStyleTheme();
}
