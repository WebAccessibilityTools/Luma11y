// =============================================================================
// colors/types.ts - Shared types for color formats
// =============================================================================

// Each format declares the command to call and the extracted components.
export interface ColorCommit {
  // Name of the Tauri command that converts then updates the store.
  command: string;

  // Extracted components, forwarded as-is to the command.
  args: Record<string, string | number>;
}

// Common structure for every color format (hex, rgb, hsl, ...).
export interface ColorFormat {
  // Unique identifier. Also used as the i18n key via `color.${id}`.
  id: string;

  // Recognizes and validates an input, and returns the backend path (command +
  // components) to invoke, or null if the input is invalid.
  parse(input: string): ColorCommit | null;
}
