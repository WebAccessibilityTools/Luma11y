import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { t } from '../i18n';

@customElement('progress-bar')
export class ProgressBar extends LitElement {
  @property({ type: Number }) position = 30;
  @property({ type: Number }) split1 = 10;
  // We accepte Number or undefined
  @property({ type: Number }) split2?: number;
  @property({ type: String }) labels: 'levels' | 'ratios' = 'levels';

  static aaColor: string = 'var(--progress-aa)';
  static aaaColor: string = 'var(--progress-aaa)';
  static failColor: string = 'var(--progress-fail)';

  static styles = css`
    :host {
      display: block;
      position: relative;
      width: 100%;
      min-width: 0;
      height: 3.4rem;
      overflow: visible;
      margin: 20px 0;
      font-family: system-ui, -apple-system, sans-serif;
    }

    .progress-bar {
      position: absolute;
      top: 50%;
      left: 0;
      right: 0;
      height: 0.5rem;
      border-radius: 0.3rem;
      overflow: visible;
      transform: translateY(-50%);
    }

    /* white separator */
    .divider {
      position: absolute;
      top: 0;
      width: 2px;
      height: 100%;
      background-color: var(--progress-divider);
      z-index: 1;
    }

    .position-value {
      position: absolute;
      bottom: 100%;
      margin-bottom: 0;
      transform: translateX(-50%);
      font-size: 13px;
      font-weight: bold;
      color: var(--text-color);
      white-space: nowrap;
      z-index: 3;
      transition: left 0.3s cubic-bezier(0.4, 0, 0.2, 1);
      display: flex;
      flex-direction: column;
      align-items: center;
    }

    .position-chevron {
      font-size: 10px;
      line-height: 1;
      color: var(--background-color-inverted);
    }

    .label-ratio {
      position: absolute;
      left: 0;
      transform: translateX(-50%);
      font-size: 13px;
      font-weight: bold;
      color: var(--text-color);
      letter-spacing: 0.5px;
      white-space: nowrap;
      &:focus-visible {
        outline-width: 3px;
        outline-offset: 4px;
        outline-style: solid;
        outline-color: var(--text-color);
        border-radius: 5px;
      }
      &.border {
        border: 1px solid var(--border-color-light);
        border-radius: 3px;
      }

      &.end {
        transform: translateX(-100%);
      }

      &.label-switch {
        cursor: pointer;
        user-select: none;
        &:hover {
          text-decoration: underline;
        }
        border: none;
        background: none;
      }

      &.above {
        bottom: 100%;
        margin-bottom: 4px;
      }

      &.below {
        top: 100%;
        margin-top: 4px;
      }
    }
  `;

  private _toggleLabels() {
    this.dispatchEvent(new CustomEvent('toggle-labels', { bubbles: true, composed: true }));
  }

  // Convert a value on the 1-21 scale to a percentage
  private toPercent(value: number): number {
    return ((value - 1) / 20) * 100;
  }

  render() {
    const posPercent = this.toPercent(this.position);
    const s1Percent = this.toPercent(this.split1);
    const s2Percent = this.split2 != null ? this.toPercent(this.split2) : 100;

    // Dynamic gradient: without split2, only go from red to yellow/green
    const gradient = this.split2 != null
      ? `linear-gradient(to right,
          var(--progress-fail) 0%, var(--progress-fail) ${s1Percent}%,
          var(--progress-aa) ${s1Percent}%, var(--progress-aa) ${s2Percent}%,
          var(--progress-aaa) ${s2Percent}%, var(--progress-aaa) 100%)`
      : `linear-gradient(to right,
          var(--progress-fail) 0%, var(--progress-fail) ${s1Percent}%,
          var(--progress-aaa) ${s1Percent}%, var(--progress-aaa) 100%)`;

    return html`
      <div class="progress-bar" style="background: ${gradient}" aria-hidden="true">
        <div class="divider" style="left: ${s1Percent}%"></div>

        ${this.split2 != null && s2Percent < 100
          ? html`<div class="divider" style="left: ${s2Percent}%"></div>`
          : ''}

        <span class="position-value" style="left: ${posPercent}%">
          <span>${this.position}</span>
          <span class="position-chevron">▼</span>
        </span>

        ${this.labels === 'ratios'
          ? html`
            <span class="label-ratio border below" style="left: ${s1Percent}%">${this.split1}</span>
            ${this.split2 != null
              ? html`<span class="label-ratio border below" style="left: ${s2Percent}%">${this.split2}</span>`
              : ''}
            <button class="label-ratio below end label-switch" style="left: 100%" @click=${this._toggleLabels} aria-label="${t('results.switch_to')} ${t('results.level')}">${t('results.ratio')}</button>
          `
          : ''}

        ${this.labels === 'levels'
          ? html`
            ${this.split2 != null
              ? html`
                  <span class="label-ratio below" style="left: ${(s1Percent + s2Percent) / 2}%">AA</span>
                  <span class="label-ratio below" style="left: ${(s2Percent + 100) / 2}%">AAA</span>
                `
              : html`
                  <span class="label-ratio below" style="left: ${(s1Percent + 100) / 2}%">AA</span>
                `
            }
            <button class="label-ratio below end label-switch" style="left: 100%" @click=${this._toggleLabels} aria-label="${t('results.switch_to')} ${t('results.ratio')}">${t('results.level')}</button>`
          : ''
        }

      </div>
    `;
  }
}

// Optional: type declaration for IntelliSense in JSX/TSX
declare global {
  interface HTMLElementTagNameMap {
    'progress-bar': ProgressBar;
  }
}