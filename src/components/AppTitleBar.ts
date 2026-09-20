import { LitElement, html, css, nothing } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { IS_MAC } from '../lib/platform';
import appIconUrl from '../../app-icon.png';

const NO_DRAG_SELECTOR = 'button, a, input, select, textarea, [data-no-drag]';

@customElement('app-titlebar')
export class AppTitleBar extends LitElement {
  @property({ type: String, attribute: 'app-title' }) appTitle = '';

  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: var(--titlebar-height, 32px);
      flex-shrink: 0;
      user-select: none;
      -webkit-user-select: none;
    }

    .bar {
      display: flex;
      align-items: stretch;
      width: 100%;
      height: 100%;
      box-sizing: border-box;
      padding-left: 0.5rem;
    }

    .bar.is-mac {
      padding-left: 76px;
    }

    .menu {
      display: flex;
      align-items: center;
      flex: 0 0 auto;
    }

    .spacer {
      flex: 1 1 auto;
      min-width: 8px;
      display: flex;
      align-items: center;
      justify-content: left;
      gap: 0.375rem;
    }

    /* Add the app icon into the bar */
    .app-icon {
      height: 1rem;
      width: 1rem;
      flex-shrink: 0;
      pointer-events: none;
      -webkit-user-drag: none;
    }

    .title {
      font-size: 0.78rem;
      font-weight: 500;
      color: var(--text-color-light, #6b7280);
      pointer-events: none;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      margin: 0;
    }

    .controls {
      display: flex;
      flex: 0 0 auto;
      align-items: stretch;

      button {
        width: 46px;
        height: 100%;
        border: 0;
        background: transparent;
        color: var(--text-color-strong, #111);
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        font: inherit;
      }

      button:hover {
        background: color-mix(in srgb, var(--text-color-strong, #111) 10%, transparent);
      }

      button.close:hover {
        background: #e81123;
        color: white;
      }

      button:focus-visible {
        outline: 2px solid var(--text-color-strong, currentColor);
        outline-offset: -3px;
      }

      svg {
        width: 1rem;
        height: 1rem;
      }
    }
  `;

  private isInteractive(e: MouseEvent): boolean {
    // Use composedPath() to crosses Shadow DOM
    return e.composedPath().some(
      (el) => el instanceof Element && el.matches(NO_DRAG_SELECTOR)
    );
  }

  private async handleDragStart(e: MouseEvent) {
    if (e.button !== 0) return;
    if (this.isInteractive(e)) return;
    e.preventDefault();
    await getCurrentWindow().startDragging();
  }

  private async handleMinimize() {
    await getCurrentWindow().minimize();
  }

  private async handleClose() {
    await getCurrentWindow().close();
  }

  render() {
    return html`
      <div
        class="bar ${IS_MAC ? 'is-mac' : ''}"
        @mousedown=${this.handleDragStart}
      >
        <div class="spacer">
          <img class="app-icon" src=${appIconUrl} alt="" />
          ${this.appTitle
            ? html`<h1 class="title">${this.appTitle}</h1>`
            : nothing}
        </div>
        <div class="menu">
          <slot name="menu"></slot>
        </div>
        ${!IS_MAC
          ? html`
              <div class="controls">
                <button
                  type="button"
                  @click=${this.handleMinimize}
                  aria-label="Minimize"
                >
                  <svg viewBox="0 0 10 10" aria-hidden="true">
                    <path d="M0 5 H10" stroke="currentColor" stroke-width="1" fill="none" />
                  </svg>
                </button>
                <button
                  type="button"
                  class="close"
                  @click=${this.handleClose}
                  aria-label="Close"
                >
                  <svg viewBox="0 0 10 10" aria-hidden="true">
                    <path d="M0 0 L10 10 M10 0 L0 10" stroke="currentColor" stroke-width="1" fill="none" />
                  </svg>
                </button>
              </div>
            `
          : nothing}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'app-titlebar': AppTitleBar;
  }
}
