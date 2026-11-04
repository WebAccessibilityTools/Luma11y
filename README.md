<div align="center">

# Luma11y

![](src-tauri/icons/64x64.png)

**Determine the legibility of text and the contrast of visual elements, such as graphical controls and visual indicators.**

[![GitHub (pre-)release](https://img.shields.io/github/release/WebAccessibilityTools/Luma11y/all.svg?style=flat-square)](https://github.com/WebAccessibilityTools/Luma11y/releases)
[![GPL-3.0 licence](https://img.shields.io/github/license/WebAccessibilityTools/Luma11y.svg?style=flat-square)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built_with-Tauri-ffc131?style=flat-square&logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

*Read this in other languages: [English](README.md) | [Français](README.fr.md) | [Slovenčina](README.sk.md)*

</div>

---

This repository contains the source code for Luma11y, a colour contrast analyser tool for Windows and MacOS.
Build with Rust and [Tauri](https://tauri.app/).

<div align="center">

![Luma11y Interface](en/luma11y-modern-en.png)

</div>

## ❤️ Support this project

This project is something I build entirely on my own free time, driven by
the belief that accessibility tooling should be open, and
available to everyone. To keep it trustworthy and easy to install, I need
code signing certificates for both Windows and macOS — which come with a
real annual cost. If this tool saves you time or helps your team ship more
accessible products, consider sponsoring this project. Even a small
contribution goes a long way toward keeping it signed, maintained, and free.

[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github)](https://github.com/sponsors/WebAccessibilityTools)

Or, if you prefer, buy me some coffe for further improve this tool even more.

<a href="https://www.buymeacoffee.com/ctrevisan" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height=48 ></a>

## Table of Contents

- [Features](#features)
- [Keyboard shortcuts](#keyboard-shortcuts)
- [Contributing](#contributing)
- [Contact](#contact)
- [License](#license)

## Features

- **WCAG contrast ratio**: pass/fail results for 1.4.3 Contrast (Minimum), 1.4.6 Contrast (Enhanced) and 1.4.11 Non-text Contrast, for regular and large text.
- **Screen colour picker**: a magnifier to pick any pixel on screen, with adjustable zoom, pixel-precise keyboard moves and a *continue mode* to pick both colours in a row.
- **Colour formats**: Hex, RGB, HSL, HSV, Lab, OKLCH and OKLab. Type a value or adjust each channel with a slider. Choose the displayed formats in Settings.
- **Alpha channel**: a transparent foreground is blended over the background before the contrast is measured.
- **Colour names**: the CSS colour name is shown when a colour exactly matches a named colour.
- **Copy templates**: copy the results to the clipboard in your own format, each template with its own shortcut.
- **Global picker shortcuts**: start the picker even when Luma11y is not focused.
- **Appearance**: Modern or Classic style, automatic/light/dark theme, window zoom, “Always on top”.
- **Colour profiles (macOS)**: switch the display ICC profile from the menu bar.
- **Restore last colours** on startup (optional).
- **Automatic updates** from the Updates tab in Settings.
- **Accessible and localised**: keyboard and screen reader friendly, available in English, French and Slovak.

> [!NOTE]
> On macOS, the picker needs the “Screen & System Audio Recording” permission to read pixels on screen. The app asks for it at first launch.

## Keyboard shortcuts

On macOS, <kbd>Cmd</kbd> replaces <kbd>Ctrl</kbd>.

### Main window

| Shortcut | Action |
| --- | --- |
| <kbd>F11</kbd> | Pick the foreground colour (global, customisable) |
| <kbd>F12</kbd> | Pick the background colour (global, customisable) |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Copy the results with the default template (customisable) |
| <kbd>Ctrl</kbd>+<kbd>,</kbd> | Open Settings |
| <kbd>Ctrl</kbd>+<kbd>+</kbd> / <kbd>Ctrl</kbd>+<kbd>-</kbd> | Zoom in / out the window |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Reset the window zoom |
| <kbd>Shift</kbd>+arrows | On a channel slider: change the value by 10 instead of 1 |
| <kbd>Esc</kbd> | Close the Settings window, or an open menu |

### Colour picker

| Shortcut | Action |
| --- | --- |
| Arrows | Move by 1 pixel |
| <kbd>Shift</kbd>+arrows | Move by 50 pixels |
| <kbd>Enter</kbd>, <kbd>Space</kbd> or click | Select the colour |
| <kbd>Esc</kbd> or right click | Cancel |
| <kbd>C</kbd> | Toggle continue mode (pick foreground and background in a row) |
| <kbd>I</kbd> / <kbd>O</kbd> or mouse wheel | Zoom in / out |
| <kbd>Shift</kbd>+<kbd>I</kbd> / <kbd>Shift</kbd>+<kbd>O</kbd> or <kbd>Shift</kbd>+wheel | Capture more / fewer pixels |

Picker and copy shortcuts can be changed in **Settings › Shortcuts** and **Settings › Copy templates**.

## Contributing

If you have an idea for a new feature or found a bug, please [open an issue](https://github.com/WebAccessibilityTools/Luma11y/issues). Search existing issues first to prevent duplicates.

Pull requests are welcome! Please follow the [Contribution Guidelines](CONTRIBUTING.md) before submitting.

## Contact

If you have any questions, feel free to [open an issue](https://github.com/WebAccessibilityTools/Luma11y/issues) here on GitHub.

## License

[![GNU GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Luma11y is Free Software: you can use, study, share and improve it at your will. Specifically you can redistribute and/or modify it under the terms of the [GNU General Public License](https://www.gnu.org/licenses/gpl.html) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

> This program is distributed in the hope that it will be useful, but **WITHOUT ANY WARRANTY**; without even the implied warranty of **MERCHANTABILITY** or **FITNESS FOR A PARTICULAR PURPOSE**. See the [GNU General Public License](https://www.gnu.org/licenses/gpl.html) for more details.
