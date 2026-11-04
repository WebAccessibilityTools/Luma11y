// =============================================================================
// colors/oklab.ts - OKLab format ("oklab(l a b)")
// =============================================================================

import type { ColorFormat, ColorCommit } from './types';
import { ALPHA_TAIL, alphaArgs } from './alpha';

// Accepts "oklab(l a b)" (space or comma separators) with an optional alpha
// Decimal components, a and b may be negative
const OKLAB_RE = new RegExp(
  `^oklab\\(\\s*([\\d.]+)\\s*[,\\s]\\s*(-?[\\d.]+)\\s*[,\\s]\\s*(-?[\\d.]+)${ALPHA_TAIL}\\s*\\)$`,
  'i',
);

const inRange = (n: number, min: number, max: number) => n >= min && n <= max;

// OKLab [l, a, b] → CSS oklab() string. Used for the slider gradients.
export function oklabToCss(v: number[]): string {
  return `oklab(${v[0]} ${v[1]} ${v[2]})`;
}

// The frontend extracts and validates l, a, b
export const oklabFormat: ColorFormat = {
  id: 'oklab',

  parse(input: string): ColorCommit | null {
    const match = input.trim().match(OKLAB_RE);
    if (!match) return null;

    const l = parseFloat(match[1]);
    const a = parseFloat(match[2]);
    const b = parseFloat(match[3]);

    // Reject any component out of range (L 0-1, a/b -0.5..0.5).
    if (!inRange(l, 0, 1) || !inRange(a, -0.5, 0.5) || !inRange(b, -0.5, 0.5)) return null;

    // Optional alpha (comma or slash)
    const alpha = alphaArgs(match[4]);
    if (alpha === null) return null;

    return { command: 'update_store_oklab', args: { l, a, b, ...alpha } };
  },
};
