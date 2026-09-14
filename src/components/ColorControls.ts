// =============================================================================
// ColorControls.ts - Composant de contrôle des couleurs
// ColorControls.ts - Color control component
//
// Affiche trois canaux avec un champ numérique et un slider chacun.
// Displays three channels with a number input and a slider each.
//
// Les canaux affichés s'adaptent au format sélectionné (RGB, HSL, HSV…) ;
// `hex` réutilise les canaux RGB.
//
// Événements :
//   - color-change  : detail { command, args } — commande backend de conversion
//                     et composantes du format affiché
//   - format-change : detail { format } — id du format radio sélectionné
//
// Le mode d'affichage des sliders (standard / statique / dynamique) est géré
// en interne via un <select>, proposé pour les formats ayant une représentation
// CSS (RGB, HSL)
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

// Définition des canaux par format de couleur (ordre = ordre d'affichage).
//   - command / argKeys : commande backend qui convertit + met à jour le store
//   - max / min / step  : bornes et pas du slider et du champ numérique
//                         (step fractionnaire pour OKLCH : L/C en décimales).
//   - toCss             : couleur CSS à partir des composantes [c0, c1, c2].
//                         Présent = modes coloré/dynamique disponibles ; absent
//                         (ex. hsv, sans fonction CSS) = sliders standards seulement.
//   - staticBase        : active le mode « coloré » (statique). Valeurs des autres
//                         canaux, indépendantes de la couleur courante. N'a de sens
//                         que pour des canaux indépendants (RGB) ; absent pour HSL
//                         (S/L n'ont pas de teinte fixe parlante) → pas de « coloré ».
// Le format `hex` réutilise les canaux RGB (cf. channelFormat).
//
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
// min : borne inférieure (défaut 0 ; négative pour a*/b* en Lab).
// step : pas du slider/champ (défaut 1 ; decimal pour OKLCH).
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

// Arrondit `value` au multiple de `step` le plus proche (gère les pas < 1).
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

// Canal alpha, Saisi en pourcentage (0-100)
// Alpha channel. Entered as a percentage (0-100)
const ALPHA_CHANNEL: ChannelDef = { letter: 'A', labelKey: 'color.alpha', max: 100, min: 0, step: 1 };

// Modes d'affichage des sliders et leur clé i18n.
// Slider display modes and their i18n key.
type SliderMode = 'standard' | 'static' | 'dynamic';
const SLIDER_MODE_LABELS: Record<SliderMode, string> = {
  standard: 'color.slider_standard',
  static: 'color.slider_colored',
  dynamic: 'color.slider_dynamic',
};

@customElement('color-controls')
export class ColorControls extends LitElement {
  // Valeurs RGB sous forme "r, g, b" / RGB values as "r, g, b"
  @property({ type: String }) rgb = '0, 0, 0';

  // Locale courante — crée une dépendance réactive pour les traductions
  // Current locale — creates a reactive dependency for translations
  @property({ type: String }) locale = 'en';

  // Nom de la section affiché en sr-only pour le contexte a11y
  // Section name displayed as sr-only for a11y context
  @property({ type: String }) label = '';

  // Formats de couleur disponibles (id + label + valeur)
  // Available color formats (id + label + value)
  @property({ type: Array }) formats: { id: string; label: string; value: string }[] = [];

  // Format sélectionné (id) : pilote la valeur affichée dans le color preview.
  // Selected format (id): drives the value shown in the color preview.
  @property({ type: String, attribute: 'selected-format' }) selectedFormat = 'hex';

  // Active le canal alpha (premier plan uniquement).
  // Enables the alpha channel (foreground only).
  @property({ type: Boolean, attribute: 'allow-alpha' }) allowAlpha = false;

  // Opacité courante ∈ [0,1], pilotée par le store. Source du canal alpha (× 100).
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

  // Pendant le drag d'un slider, on fige les valeurs des canaux localement : les
  // sliders suivent cet état au lieu de la réponse du backend pour éviter le "drift" des autres canaux.
  // À la fin du drag, on resynchronise sur les valeurs canoniques (cf. channelValues).
  //
  // While dragging a slider, the channel values are frozen locally: the sliders
  // follow this state instead of the backend value to prevent the dirft on the other channels.
  // When the drag ends, we resync on the canonical values (see channelValues).
  @state() private dragValues: number[] | null = null;

  // Parse la chaîne RGB en tableau de trois valeurs numériques
  // Parses the RGB string into an array of three numeric values
  private get rgbValues(): [number, number, number] {
    const parts = this.rgb.split(',').map(v => parseInt(v.trim()));
    return [parts[0] ?? 0, parts[1] ?? 0, parts[2] ?? 0];
  }

  // Format des canaux affichés : `hex` réutilise les canaux RGB.
  // Channel format displayed: `hex` reuses the RGB channels.
  private get channelFormat(): string {
    return this.selectedFormat === 'hex' ? 'rgb' : this.selectedFormat;
  }

  private get channelConfig(): FormatChannels {
    const base = FORMAT_CHANNELS[this.channelFormat] ?? FORMAT_CHANNELS.rgb;
    // En mode alpha, on ajoute un 4e canal `alpha` au format courant.
    // In alpha mode, add a 4th `alpha` channel onto the format.
    if (!this.allowAlpha) return base;
    return {
      ...base,
      argKeys: [...base.argKeys, 'alpha'],
      channels: [...base.channels, ALPHA_CHANNEL],
    };
  }

  // Modes d'affichage des sliders disponibles pour le format courant :
  //   - standard : toujours
  //   - dynamic  : si le format a une représentation CSS (toCss)
  //   - static   : seulement si staticBase est défini (canaux indépendants, RGB)
  //
  // Slider display modes available for the current format:
  //   - standard : always
  //   - dynamic  : if the format has a CSS representation (toCss)
  //   - static   : only if staticBase is defined (independent channels, RGB)
  private get availableModes(): SliderMode[] {
    const cfg = this.channelConfig;
    if (!cfg.toCss) return ['standard'];
    return cfg.staticBase ? ['standard', 'static', 'dynamic'] : ['standard', 'dynamic'];
  }

  // Mode effectif : le mode choisi s'il est disponible pour ce format, sinon standard
  // Effective mode: the chosen mode if available for this format, otherwise standard
  private get effectiveMode(): SliderMode {
    return this.availableModes.includes(this.sliderMode) ? this.sliderMode : 'standard';
  }

  // Construit un dégradé CSS pour le slider `index`
  // Builds a CSS gradient for channel `index`
  private gradient(index: number, others: number[]): string {
    const cfg = this.channelConfig;
    // toCss est garanti présent : gradient() n'est appelé qu'en mode coloré/dynamique,
    // disponibles uniquement si le format définit toCss (cf. availableModes).
    // toCss is guaranteed: gradient() is only called in colored/dynamic mode, which
    // are available only when the format defines toCss (see availableModes).
    const toCss = cfg.toCss!;

    // Bornes du canal qu'on balaie (ex. RGB 0..255, H 0..360, a*/b* -128..128).
    // Bounds of the swept channel (e.g. RGB 0..255, H 0..360, a*/b* -128..128).
    const { min = 0, max, step = 1 } = cfg.channels[index];

    // 7 arrêts répartis sur [min, max]. Plusieurs arrêts sont nécessaires pour les
    // canaux non monotones (teinte : 0 et 360 = rouge → un dégradé 2 points serait
    // plat ; luminosité : noir → couleur → blanc).
    // 7 stops spread over [min, max]. Multiple stops are required for non-monotonic
    // channels (hue: 0 and 360 = red → a 2-stop gradient would be flat; lightness:
    // black → color → white).
    const STOPS = 7;
    const stops: string[] = [];
    for (let i = 0; i < STOPS; i++) {
      // On copie les autres composantes et on ne fait varier que le canal `index`.
      // Copy the other components and vary only channel `index`.
      const vals = [...others];
      vals[index] = roundToStep(min + ((max - min) * i) / (STOPS - 1), step);
      stops.push(toCss(vals));
    }

    // Dégradé horizontal : du minimum (gauche) au maximum (droite) du slider.
    // Horizontal gradient: from the slider's minimum (left) to maximum (right).
    return `linear-gradient(to right, ${stops.join(', ')})`;
  }

  // Valeurs numériques courantes des canaux du format affiché.
  // Current numeric values of the displayed format's channels.
  private get channelValues(): number[] {
    let base: number[];
    if (this.channelFormat === 'rgb') {
      base = [...this.rgbValues];
    } else {
      const entry = this.formats.find((f) => f.id === this.channelFormat);
      // -?\d*\.?\d+ : composantes éventuellement négatives (Lab) ou décimales (OKLCH).
      // On ne garde que les 3 premières (un éventuel 4e nombre serait l'alpha du suffixe).
      // -?\d*\.?\d+: components possibly negative (Lab) or decimal (OKLCH). Keep only the
      // first 3 (a possible 4th number would be the suffix's alpha).
      const nums = (entry?.value.match(/-?\d*\.?\d+/g) ?? []).map(Number);
      base = [nums[0] ?? 0, nums[1] ?? 0, nums[2] ?? 0];
    }
    if (this.allowAlpha) base.push(Math.round(this.alpha * 100));
    return base;
  }

  // Valeurs affichées par les sliders : l'état figé pendant un drag, sinon les
  // valeurs canoniques issues du backend.
  // Values displayed by the sliders: the frozen state during a drag, otherwise the
  // canonical values from the backend.
  private get displayValues(): number[] {
    return this.dragValues ?? this.channelValues;
  }

  // Applique une nouvelle valeur au slider `index` et émet le changement dans le
  // format actif. La commande backend (rgb/hsl/hsv) est portée par l'événement.
  // Si un drag est en cours, met aussi à jour l'état figé.
  //
  // Applies a new value to slider `index` and emits the change in the active
  // format. The backend command (rgb/hsl/hsv) is carried by the event.
  // If a drag is in progress, also updates the frozen state.
  private applyChannel(index: number, value: number) {
    const cfg = this.channelConfig;
    const ch = cfg.channels[index];
    const values = [...this.displayValues];
    values[index] = Math.min(ch.max, Math.max(ch.min ?? 0, roundToStep(value, ch.step ?? 1)));

    if (this.dragValues) this.dragValues = values;

    // L'alpha est saisi en pourcentage (0-100) mais transmis en [0,1] au backend.
    // Alpha is entered as a percentage (0-100) but sent to the backend as [0,1].
    const args: Record<string, number> = {};
    cfg.argKeys.forEach((k, i) => { args[k] = k === 'alpha' ? values[i] / 100 : values[i]; });

    this.dispatchEvent(new CustomEvent('color-change', {
      detail: { command: cfg.command, args },
      bubbles: true,
      composed: true,
    }));
  }

  // Émet un événement format-change quand un radio de format est sélectionné
  // Emits a format-change event when a format radio is selected
  private onFormatChange(format: string) {
    this.dispatchEvent(new CustomEvent('format-change', {
      detail: { format },
      bubbles: true,
      composed: true,
    }));
  }

  // Déplacement du slider: fige l'état des canaux au premier
  // mouvement pour que les autres sliders ne bougent pas pendant le drag.
  // Slider drag: freeze the channel state on the first move so
  // the other sliders do not move during the drag.
  private onInput(index: number, value: number) {
    if (!this.dragValues) this.dragValues = [...this.channelValues];
    this.applyChannel(index, value);
  }

  // Fin du drag : on relâche l'état figé pour resynchroniser sur le backend.
  // End of drag: release the frozen state to resync on the backend.
  private onSliderCommit() {
    this.dragValues = null;
  }

  // Validation du champ numérique.
  // Number input commit.
  private onChange(index: number, value: number) {
    this.applyChannel(index, value);
  }

  // Shift + flèches sur le slider : incrémente/décrémente de 10
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

    /* Sélecteur de format (radios) + valeurs */
    /* Format selector (radios) + values */
    fieldset.formats {
      margin: 0 0 0.5rem;
      padding: 0;
      border: none;
      display: grid;
      grid-template-columns: auto 1fr;
      gap: 0.1rem 0.5rem;
      font-size: 0.8rem;

      /* Label radio (radio + nom du format) en 1re colonne */
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

    /* Sélecteur du mode slider, aligné à droite */
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

    /* Ligne d'un canal RGB (label + input + slider) */
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

  // Rendu d'un slider.
  // Renders a slider
  private renderChannel(index: number) {
    const cfg = this.channelConfig;
    const ch = cfg.channels[index];
    const values = this.displayValues;
    const value = values[index];

    // Dégradé du slider selon le mode : coloré ou dynamique. En standard : pas de fond inline.
    // Slider gradient depending on the mode: colored or dynamic. In standard: no inline background.
    let sliderStyle = '';
    const mode = this.effectiveMode;
    if (mode !== 'standard') {
      // 'static' n'est dans effectiveMode que si staticBase existe ; 'dynamic' utilise les valeurs courantes.
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
    // Lire this.locale pour forcer le re-render quand la locale change
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

// Déclaration du type pour l'IntelliSense
// Type declaration for IntelliSense
declare global {
  interface HTMLElementTagNameMap {
    'color-controls': ColorControls;
  }
}
