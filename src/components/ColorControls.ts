// =============================================================================
// ColorControls.ts - Color control component
//
// Displays three channels with a number input and a slider each.
//
// The displayed channels adapt to the selected format (RGB, HSL, HSV…);
// `hex` reuses the RGB channels.
//
// Events:
//   - color-change  : detail { command, args } — backend conversion command
//                     and components of the displayed format
//   - format-change : detail { format } — id of the selected format radio
//
// The slider display mode (standard / static / dynamic) is handled internally
// via a <select>, offered for formats that have a CSS representation
// (RGB, HSL)
// =============================================================================

import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { t } from '../i18n';
import { srOnly } from './shared-styles';
import { rgbToCss } from '../colors/rgb';
import { hslToCss } from '../colors/hsl';
import { hsvToCss } from '../colors/hsv';
import { labToCss } from '../colors/lab';
import { oklchToCss } from '../colors/oklch';

// Channel definitions per color format (order = display order).
//   - command / argKeys : backend command that converts + updates the store
//   - max               : slider and number-input bound
//   - toCss             : CSS color from the components [c0, c1, c2]. Present =
//                         dynamic mode available; absent (e.g. hsv, no CSS
//                         function) = standard sliders only.
//   - staticBase        : enables the "colored" (static) mode. Other channels'
//                         values, independent of the current color. Only meaningful
//                         for independent channels (RGB); absent for HSL (S/L have
//                         no meaningful fixed hue) → no "colored" mode.
// The `hex` format reuses the RGB channels (see channelFormat).
// min: lower bound (default 0; negative for a*/b* in Lab).
// step: slider/input step (default 1; decimal for OKLCH).
interface ChannelDef { letter: string; labelKey: string; max: number; min?: number; step?: number; }
interface FormatChannels {
  command: string;
  argKeys: string[];
  channels: ChannelDef[];
  toCss?: (v: number[]) => string;
  staticBase?: number[];
}

// Rounds `value` to the nearest multiple of `step` (handles steps < 1).
function roundToStep(value: number, step: number): number {
  const inv = 1 / step;
  return Math.round(value * inv) / inv;
}

const FORMAT_CHANNELS: Record<string, FormatChannels> = {
  rgb: {
    command: 'update_store_rgb',
    argKeys: ['r', 'g', 'b'],
    toCss: rgbToCss,
    staticBase: [0, 0, 0],
    channels: [
      { letter: 'R', labelKey: 'color.red', max: 255 },
      { letter: 'G', labelKey: 'color.green', max: 255 },
      { letter: 'B', labelKey: 'color.blue', max: 255 },
    ],
  },
  hsl: {
    command: 'update_store_hsl',
    argKeys: ['h', 's', 'l'],
    toCss: hslToCss,
    channels: [
      { letter: 'H', labelKey: 'color.hue', max: 360 },
      { letter: 'S', labelKey: 'color.saturation', max: 100 },
      { letter: 'L', labelKey: 'color.lightness', max: 100 },
    ],
  },
  hsv: {
    command: 'update_store_hsv',
    argKeys: ['h', 's', 'v'],
    toCss: hsvToCss,
    channels: [
      { letter: 'H', labelKey: 'color.hue', max: 360 },
      { letter: 'S', labelKey: 'color.saturation', max: 100 },
      { letter: 'V', labelKey: 'color.value_component', max: 100 },
    ],
  },
  lab: {
    command: 'update_store_lab',
    argKeys: ['l', 'a', 'b'],
    toCss: labToCss,
    channels: [
      { letter: 'L', labelKey: 'color.lightness', max: 100 },
      { letter: 'a', labelKey: 'color.lab_a', min: -128, max: 128 },
      { letter: 'b', labelKey: 'color.lab_b', min: -128, max: 128 },
    ],
  },
  oklch: {
    command: 'update_store_oklch',
    argKeys: ['l', 'c', 'h'],
    toCss: oklchToCss,
    channels: [
      { letter: 'L', labelKey: 'color.lightness', max: 1, step: 0.001 },
      { letter: 'C', labelKey: 'color.chroma', max: 0.4, step: 0.001 },
      { letter: 'H', labelKey: 'color.hue', max: 360 },
    ],
  },
};

// Alpha channel. Entered as a percentage (0-100)
const ALPHA_CHANNEL: ChannelDef = { letter: 'A', labelKey: 'color.alpha', max: 100, min: 0, step: 1 };

// Slider display modes and their i18n key.
type SliderMode = 'standard' | 'static' | 'dynamic';
const SLIDER_MODE_LABELS: Record<SliderMode, string> = {
  standard: 'color.slider_standard',
  static: 'color.slider_colored',
  dynamic: 'color.slider_dynamic',
};

@customElement('color-controls')
export class ColorControls extends LitElement {
  // RGB values as "r, g, b"
  @property({ type: String }) rgb = '0, 0, 0';

  // Current locale — creates a reactive dependency for translations
  @property({ type: String }) locale = 'en';

  // Section name displayed as sr-only for a11y context
  @property({ type: String }) label = '';

  // Available color formats (id + label + value)
  @property({ type: Array }) formats: { id: string; label: string; value: string }[] = [];

  // Selected format (id): drives the value shown in the color preview.
  @property({ type: String, attribute: 'selected-format' }) selectedFormat = 'hex';

  // Enables the alpha channel (foreground only).
  @property({ type: Boolean, attribute: 'allow-alpha' }) allowAlpha = false;

  // Current opacity ∈ [0,1], driven by the store. Source of the alpha channel (× 100).
  @property({ type: Number }) alpha = 1;

  // Colour this instance edits ("foreground" / "background"), used to persist its slider mode.
  @property({ type: String, attribute: 'color-key' }) colorKey = '';

  // Slider display mode (raw user choice; see effectiveMode for the fallback)
  @state() private sliderMode: SliderMode = 'standard';

  // localStorage key holding this instance's slider mode
  private get sliderModeKey(): string {
    return `luma11y-slider-mode-${this.colorKey}`;
  }

  connectedCallback() {
    super.connectedCallback();
    const saved = localStorage.getItem(this.sliderModeKey);
    if (saved && saved in SLIDER_MODE_LABELS) this.sliderMode = saved as SliderMode;
  }

  private setSliderMode(mode: SliderMode) {
    this.sliderMode = mode;
    localStorage.setItem(this.sliderModeKey, mode);
  }

  // While dragging a slider, the channel values are frozen locally: the sliders
  // follow this state instead of the backend value to prevent the dirft on the other channels.
  // When the drag ends, we resync on the canonical values (see channelValues).
  @state() private dragValues: number[] | null = null;

  // Parses the RGB string into an array of three numeric values
  private get rgbValues(): [number, number, number] {
    const parts = this.rgb.split(',').map(v => parseInt(v.trim()));
    return [parts[0] ?? 0, parts[1] ?? 0, parts[2] ?? 0];
  }

  // Channel format displayed: `hex` reuses the RGB channels.
  private get channelFormat(): string {
    return this.selectedFormat === 'hex' ? 'rgb' : this.selectedFormat;
  }

  private get channelConfig(): FormatChannels {
    const base = FORMAT_CHANNELS[this.channelFormat] ?? FORMAT_CHANNELS.rgb;
    // In alpha mode, add a 4th `alpha` channel onto the format.
    if (!this.allowAlpha) return base;
    return {
      ...base,
      argKeys: [...base.argKeys, 'alpha'],
      channels: [...base.channels, ALPHA_CHANNEL],
    };
  }

  // Slider display modes available for the current format:
  //   - standard : always
  //   - dynamic  : if the format has a CSS representation (toCss)
  //   - static   : only if staticBase is defined (independent channels, RGB)
  private get availableModes(): SliderMode[] {
    const cfg = this.channelConfig;
    if (!cfg.toCss) return ['standard'];
    return cfg.staticBase ? ['standard', 'static', 'dynamic'] : ['standard', 'dynamic'];
  }

  // Effective mode: the chosen mode if available for this format, otherwise standard
  private get effectiveMode(): SliderMode {
    return this.availableModes.includes(this.sliderMode) ? this.sliderMode : 'standard';
  }

  // Builds a CSS gradient for channel `index`
  private gradient(index: number, others: number[]): string {
    const cfg = this.channelConfig;
    // toCss is guaranteed: gradient() is only called in colored/dynamic mode, which
    // are available only when the format defines toCss (see availableModes).
    const toCss = cfg.toCss!;

    // Bounds of the swept channel (e.g. RGB 0..255, H 0..360, a*/b* -128..128).
    const { min = 0, max, step = 1 } = cfg.channels[index];

    // 7 stops spread over [min, max]. Multiple stops are required for non-monotonic
    // channels (hue: 0 and 360 = red → a 2-stop gradient would be flat; lightness:
    // black → color → white).
    const STOPS = 7;
    const stops: string[] = [];
    for (let i = 0; i < STOPS; i++) {
      // Copy the other components and vary only channel `index`.
      const vals = [...others];
      vals[index] = roundToStep(min + ((max - min) * i) / (STOPS - 1), step);
      stops.push(toCss(vals));
    }

    // Horizontal gradient: from the slider's minimum (left) to maximum (right).
    return `linear-gradient(to right, ${stops.join(', ')})`;
  }

  // Current numeric values of the displayed format's channels.
  private get channelValues(): number[] {
    let base: number[];
    if (this.channelFormat === 'rgb') {
      base = [...this.rgbValues];
    } else {
      const entry = this.formats.find((f) => f.id === this.channelFormat);
      // -?\d*\.?\d+: components possibly negative (Lab) or decimal (OKLCH). Keep only the
      // first 3 (a possible 4th number would be the suffix's alpha).
      const nums = (entry?.value.match(/-?\d*\.?\d+/g) ?? []).map(Number);
      base = [nums[0] ?? 0, nums[1] ?? 0, nums[2] ?? 0];
    }
    if (this.allowAlpha) base.push(Math.round(this.alpha * 100));
    return base;
  }

  // Values displayed by the sliders: the frozen state during a drag, otherwise the
  // canonical values from the backend.
  private get displayValues(): number[] {
    return this.dragValues ?? this.channelValues;
  }

  // Applies a new value to slider `index` and emits the change in the active
  // format. The backend command (rgb/hsl/hsv) is carried by the event.
  // If a drag is in progress, also updates the frozen state.
  private applyChannel(index: number, value: number) {
    const cfg = this.channelConfig;
    const ch = cfg.channels[index];
    const values = [...this.displayValues];
    values[index] = Math.min(ch.max, Math.max(ch.min ?? 0, roundToStep(value, ch.step ?? 1)));

    if (this.dragValues) this.dragValues = values;

    // Alpha is entered as a percentage (0-100) but sent to the backend as [0,1].
    const args: Record<string, number> = {};
    cfg.argKeys.forEach((k, i) => { args[k] = k === 'alpha' ? values[i] / 100 : values[i]; });

    this.dispatchEvent(new CustomEvent('color-change', {
      detail: { command: cfg.command, args },
      bubbles: true,
      composed: true,
    }));
  }

  // Emits a format-change event when a format radio is selected
  private onFormatChange(format: string) {
    this.dispatchEvent(new CustomEvent('format-change', {
      detail: { format },
      bubbles: true,
      composed: true,
    }));
  }

  // Slider drag: freeze the channel state on the first move so
  // the other sliders do not move during the drag.
  private onInput(index: number, value: number) {
    if (!this.dragValues) this.dragValues = [...this.channelValues];
    this.applyChannel(index, value);
  }

  // End of drag: release the frozen state to resync on the backend.
  private onSliderCommit() {
    this.dragValues = null;
  }

  // Number input commit.
  private onChange(index: number, value: number) {
    this.applyChannel(index, value);
  }

  // Shift + arrow keys on slider: increment/decrement by 10
  private onSliderKeydown(e: KeyboardEvent, index: number, current: number) {
    if (!e.shiftKey) return;
    const step = e.key === 'ArrowRight' || e.key === 'ArrowUp' ? 10
               : e.key === 'ArrowLeft' || e.key === 'ArrowDown' ? -10
               : 0;
    if (step === 0) return;
    e.preventDefault();
    this.applyChannel(index, current + step);
  }

  // ---------------------------------------------------------------------------
  // Styles
  // ---------------------------------------------------------------------------

  static styles = [srOnly, css`
    :host {
      display: block;
      padding: 0.5rem 1rem;
    }

    /* Format selector (radios) + values */
    fieldset.formats {
      margin: 0 0 0.5rem;
      padding: 0;
      border: none;
      display: grid;
      grid-template-columns: auto 1fr;
      gap: 0.1rem 0.5rem;
      font-size: 0.8rem;

      /* Radio label (radio + format name) in the 1st column */
      .format {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        cursor: pointer;
      }

      .format input[type="radio"] {
        margin: 0;
        accent-color: var(--text-color);
      }

      .format-name {
        font-weight: 600;
      }

      .format-value {
        padding: 0;
        margin: 0;
      }
    }

    /* Slider mode selector, right-aligned */
    select {
      display: block;
      margin-left: auto;
      background: none;
      border: none;
      color: var(--text-color-light);
      cursor: pointer;
      font-size: 0.75rem;
      font-family: inherit;
      option {
        background-color: var(--background-color);
      }
    }

    /* RGB channel row (label + input + slider) */
    .channel {
      margin: 0.3rem 0;
      display: flex;
      align-items: center;
      gap: 0.5rem;
      font-size: 0.8rem;
      font-weight: 600;

      span:first-child {
        width: 1ch;
      }
    }

    input[type="range"] {
      flex: 1;
      -webkit-appearance: none;
      appearance: none;
      height: 6px;
      border-radius: 3px;
      background: var(--progress-background);

      &::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 14px;
        height: 14px;
        border-radius: 50%;
        background: #fff;
        border: 1px solid var(--border-color);
        cursor: pointer;
      }
    }

    .rgb-input {
      width: 5.5ch;
      text-align: center;
      font-variant-numeric: tabular-nums;
      font-size: 0.8rem;
      font-weight: 600;
      border: 1px solid var(--border-color);
      color: var(--text-color);
      border-radius: 3px;
      padding: 0 0.2rem;
      background: transparent;
      -moz-appearance: textfield;

      &::-webkit-inner-spin-button,
      &::-webkit-outer-spin-button {
        -webkit-appearance: none;
        margin: 0;
      }
    }

  `];

  // Renders a slider
  private renderChannel(index: number) {
    const cfg = this.channelConfig;
    const ch = cfg.channels[index];
    const values = this.displayValues;
    const value = values[index];

    // Slider gradient depending on the mode: colored or dynamic. In standard: no inline background.
    let sliderStyle = '';
    const mode = this.effectiveMode;
    if (mode !== 'standard') {
      // 'static' is in effectiveMode only when staticBase exists; 'dynamic' uses the current values.
      const others = mode === 'static' ? cfg.staticBase! : values;
      sliderStyle = `background: ${this.gradient(index, others)}`;
    }

    const channelLabel = t(ch.labelKey);

    return html`
      <div class="channel" role="group" aria-label="${channelLabel}">
        <span aria-hidden="true">${ch.letter}</span>
        <input
          aria-label="${t('color.component_value')}"
          aria-describedby="section-label"
          type="number" min="${ch.min ?? 0}" max="${ch.max}" step="${ch.step ?? 1}" class="rgb-input"
          .value="${String(value)}"
          @change="${(e: Event) => this.onChange(index, +(e.target as HTMLInputElement).value)}"
        />
        <input
          aria-label="${t('color.component_slider')}"
          aria-describedby="section-label"
          type="range" min="${ch.min ?? 0}" max="${ch.max}" step="${ch.step ?? 1}"
          .value="${String(value)}"
          @input="${(e: Event) => this.onInput(index, +(e.target as HTMLInputElement).value)}"
          @change="${() => this.onSliderCommit()}"
          @keydown="${(e: KeyboardEvent) => this.onSliderKeydown(e, index, value)}"
          style="${sliderStyle}"
        />
      </div>
    `;
  }

  render() {
    // Read this.locale to force re-render when locale changes
    void this.locale;

    return html`
      <span id="section-label" class="sr-only">${this.label}</span>
      ${this.formats.length ? html`
        <fieldset class="formats">
          <legend class="sr-only">${t('color.display_format')}</legend>
          ${this.formats.map((f) => html`
            <label class="format">
              <input
                type="radio"
                name="format"
                value="${f.id}"
                .checked="${f.id === this.selectedFormat}"
                @change="${() => this.onFormatChange(f.id)}"
                aria-describedby="${f.id}-help"
              />
              <span class="format-name">${f.label}</span>
            </label>
            <p id="${f.id}-help" class="format-value">${f.value}</p>
          `)}
        </fieldset>
      ` : ''}
      ${this.availableModes.length > 1 ? html`
        <select
          aria-label="${t('color.slider_mode')}"
          @change="${(e: Event) => this.setSliderMode((e.target as HTMLSelectElement).value as SliderMode)}"
        >
          ${this.availableModes.map((m) => html`
            <option value="${m}" .selected="${m === this.effectiveMode}">${t(SLIDER_MODE_LABELS[m])}</option>
          `)}
        </select>
      ` : ''}
      ${this.channelConfig.channels.map((_, i) => this.renderChannel(i))}
    `;
  }
}

// Type declaration for IntelliSense
declare global {
  interface HTMLElementTagNameMap {
    'color-controls': ColorControls;
  }
}
