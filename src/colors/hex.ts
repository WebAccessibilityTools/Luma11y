// =============================================================================
// colors/hex.ts - Hexadecimal format (#abc / #abcd / #aabbcc / #aabbccdd)
// =============================================================================

import type { ColorFormat, ColorCommit } from './types';

// 3/6-digit (no alpha) and 4/8-digit (with alpha) notations, the # is optional.
// Any alpha is carried by the hex string itself (decoded on the Rust side).
const HEX = /^[0-9a-fA-F]{3,4}$|^[0-9a-fA-F]{6}$|^[0-9a-fA-F]{8}$/;

// The frontend validates the notation and forwards the hex string
export const hexFormat: ColorFormat = {
  id: 'hex',

  parse(input: string): ColorCommit | null {
    const cleaned = input.replace(/^#/, '').trim();

    if (HEX.test(cleaned)) {
      return { command: 'update_store_hex', args: { hex: cleaned } };
    }

    return null;
  },
};
