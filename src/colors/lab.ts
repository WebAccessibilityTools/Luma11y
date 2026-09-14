// =============================================================================
// colors/lab.ts - CIE L*a*b* format ("lab(l, a, b)")
// =============================================================================

import type { ColorFormat, ColorCommit } from './types';
import { ALPHA_TAIL, alphaArgs } from './alpha';

// Accepts "lab(l, a, b)" with an optional alpha
// L* lies in 0-100; a* and b* may be negative
const LAB_RE = new RegExp(
  `^lab\\(\\s*(\\d{1,3})\\s*[,\\s]\\s*(-?\\d{1,3})\\s*[,\\s]\\s*(-?\\d{1,3})${ALPHA_TAIL}\\s*\\)$`,
  'i',
);

// L* lies in 0-100; a* and b* in roughly -128..128 (sRGB gamut).
const inL = (n: number) => n >= 0 && n <= 100;
const inAB = (n: number) => n >= -128 && n <= 128;

// Lab [l, a, b] → CSS lab() string. Used for the slider gradients.
export function labToCss(v: number[]): string {
  return `lab(${v[0]} ${v[1]} ${v[2]})`;
}

// The frontend extracts and validates l, a, b
export const labFormat: ColorFormat = {
  id: 'lab',

  parse(input: string): ColorCommit | null {
    const match = input.trim().match(LAB_RE);
    if (!match) return null;

    const l = parseInt(match[1], 10);
    const a = parseInt(match[2], 10);
    const b = parseInt(match[3], 10);

    // Reject any component out of range (L 0-100, a/b -128..128).
    if (!inL(l) || !inAB(a) || !inAB(b)) return null;

    // Optional alpha (comma or slash)
    const alpha = alphaArgs(match[4]);
    if (alpha === null) return null;

    return { command: 'update_store_lab', args: { l, a, b, ...alpha } };
  },
};
