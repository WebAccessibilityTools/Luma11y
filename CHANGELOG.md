## [0.2.0] - 2026-09-11

### 🚀 Features

- *(colors)*: Added HSL, HSV, Lab, Oklch formats
- *(alpha)* Add alpha channel support for foreground
- *(MacOS)* Added permission dialog
- *(settings)* Redesign dialog with vertical tabs (Closes #43)
- Re-apply last used color on re-open (Closes #45)
- *(i18n)* Adding Slovak language support to Luma11y (#54)

### 🐛 Bug Fixes

- *(a11y)* Color contrast on shortcut selector (Fixes #47)
- *(a11y)*: Button focus ring contrast (Fixes #44)
- *(color)* Place color css name above value
- Implement full wcag algo using the **rounded** luminance coefficients (closes #51)
- Background section text color contrast (closes #52)

### 📚 Documentation

- Add translator list

## [0.1.9] - 2026-06-02

### Features

- Added menubar (Fixes #24)
- Added skip link
- Added free-input value (hex only for now) (Fixes #19)

### Fixes

- Style - fix disabled button style, in classic style, dark mode (fixes #25)
- Macos - Eye dropper with full screen app (fixes #25)
- Macos - Eye dropper with full screen app (fixes #26)
- Design - hide list bullet in MacOS15.x (Fixes #29)
- Replaced is_dark helper with real contrast ratio mesurment (Fixes #35)
- Re-introduce copy/paste menuitem to re-enable associated shortcuts (Fixes #18 and part of #36)
- Design - disable context menus (Closes #23)
- Fixes template double label (Fixes #38)
- Switch from BigColor to Palette + fix contrast ratio rounding (Fixes #37)
- Various design fixes (re Fixes #9)
- Use ESC to dismiss the settings window (Closes #41)
- Prevent multiple eyedropper opening (Fixes #39)

### 📚 Documentation

- Update with new design, and latest changes
