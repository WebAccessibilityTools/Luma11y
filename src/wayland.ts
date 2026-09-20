// =============================================================================
// wayland.ts - "Wayland not supported" window entry point
//
// Opened by the backend (Linux) at startup when the session is Wayland,
// instead of the main window. Closing it quits the application.
// =============================================================================

import { getCurrentWindow } from '@tauri-apps/api/window';
import { initTheme } from './theme';
import './components/AppTitleBar';

// Block the webview's native context menu in production builds.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

initTheme();

// Closing the window is enough: the backend exits once it is destroyed.
document.getElementById('quit')?.addEventListener('click', () => {
  getCurrentWindow().close();
});
