// =============================================================================
// colors/alpha.ts - Parsing of an optional alpha component (foreground)
//
// Alpha may follow a format's components with two separators:
//   - comma (legacy notation): rgba(r, g, b, 0.5) / hsl(h, s%, l%, 0.5)
//   - slash (modern notation): rgb(r g b / 50%)   / oklch(l c h / 50%)
// and two units: float or percentage (50%).
// =============================================================================

// Regex fragment for an optional trailing alpha, before the closing paren
export const ALPHA_TAIL = '(?:\\s*[,/]\\s*([0-9]*\\.?[0-9]+%?))?';

// Normalizes an alpha token ('0.5' | '50%') into a float
export function parseAlpha(token: string): number | null {
  const isPercent = token.endsWith('%');
  const raw = parseFloat(token);
  if (Number.isNaN(raw)) return null;
  const a = isPercent ? raw / 100 : raw;
  if (a < 0 || a > 1) return null;
  return a;
}

// Builds the alpha argument fragment:
//   - token absent   → {} (opaque color, no `alpha` arg sent)
//   - valid token    → { alpha } (∈ [0,1])
//   - invalid token  → null (signals the calling format to reject the parse)
export function alphaArgs(token: string | undefined): Record<string, number> | null {
  if (token === undefined) return {};
  const a = parseAlpha(token);
  if (a === null) return null;
  return { alpha: a };
}
