// =============================================================================
// colors/hsl.ts - HSL format ("hsl(h, s%, l%)")
// =============================================================================

import type { ColorFormat, ColorCommit } from './types';
import { ALPHA_TAIL, alphaArgs } from './alpha';

// Accepts "hsl(h, s%, l%)" and the "hsla(...)" alias (the % sign is optional),
// with an optional alpha
const HSL_RE = new RegExp(
  `^hsla?\\(\\s*(\\d{1,3})\\s*[,\\s]\\s*(\\d{1,3})%?\\s*[,\\s]\\s*(\\d{1,3})%?${ALPHA_TAIL}\\s*\\)$`,
  'i',
);

// Hue 0-360, saturation and lightness 0-100.
const inHue = (n: number) => n >= 0 && n <= 360;
const inPercent = (n: number) => n >= 0 && n <= 100;

// HSL [h, s, l] → CSS hsl() string. Return color in css value
export function hslToCss(v: number[]): string {
  return `hsl(${v[0]}, ${v[1]}%, ${v[2]}%)`;
}

// Extracts and validates h, s, l
export const hslFormat: ColorFormat = {
  id: 'hsl',

  parse(input: string): ColorCommit | null {
    const match = input.trim().match(HSL_RE);
    if (!match) return null;

    const h = parseInt(match[1], 10);
    const s = parseInt(match[2], 10);
    const l = parseInt(match[3], 10);

    // Reject any component out of range (hue 0-360, s/l 0-100).
    if (!inHue(h) || !inPercent(s) || !inPercent(l)) return null;

    // Optional alpha (comma or slash)
    const alpha = alphaArgs(match[4]);
    if (alpha === null) return null;

    return { command: 'update_store_hsl', args: { h, s, l, ...alpha } };
  },
};
