<div align="center">

# Luma11y

![](src-tauri/icons/64x64.png)

**Zistite čitateľnosť textu a kontrast vizuálnych prvkov, ako sú grafické ovládacie prvky a vizuálne indikátory.**

[![GitHub (pred-)vydanie](https://img.shields.io/github/release/WebAccessibilityTools/Luma11y/all.svg?style=flat-square)](https://github.com/WebAccessibilityTools/Luma11y/releases)
[![Licencia GPL-3.0](https://img.shields.io/github/license/WebAccessibilityTools/Luma11y.svg?style=flat-square)](LICENSE)
[![Postavené na Tauri](https://img.shields.io/badge/postaven%C3%A9_na-Tauri-ffc131?style=flat-square&logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

*Read this in other languages: [English](README.md) | [Français](README.fr.md) | [Slovenčina](README.sk.md)*

Slovenský preklad: **Radoslav Ďurač** ([@rraddatch](https://github.com/rraddatch))

</div>

---

Tento repozitár obsahuje zdrojový kód aplikácie Luma11y, nástroja na analýzu farebného kontrastu pre Windows a macOS.
Vytvorené v Rust a [Tauri](https://tauri.app/).

<div align="center">

![Rozhranie Luma11y](en/luma11y-modern-en.png)

</div>

## ❤️ Podporte tento projekt

Tento projekt vytváram výhradne vo svojom voľnom čase, poháňaný
presvedčením, že nástroje pre prístupnosť by mali byť otvorené a
dostupné pre všetkých. Aby zostal dôveryhodný a jednoducho inštalovateľný,
potrebujem certifikáty na podpisovanie kódu pre Windows aj macOS — čo so
sebou nesie reálne ročné náklady. Ak vám tento nástroj šetrí čas alebo
pomáha vášmu tímu vytvárať prístupnejšie produkty, zvážte sponzorovanie
tohto projektu. Aj malý príspevok výrazne pomôže udržať ho podpísaný,
udržiavaný a bezplatný.

[![Sponzor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github)](https://github.com/sponsors/WebAccessibilityTools)

Alebo, ak preferujete, kúpte mi kávu na ďalšie vylepšovanie tohto nástroja.

<a href="https://www.buymeacoffee.com/ctrevisan" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height=48 ></a>

## Obsah

- [Features](#features)
- [Keyboard shortcuts](#keyboard-shortcuts)
- [Prispievanie](#prispievanie)
- [Kontakt](#kontakt)
- [Licencia](#licencia)

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

## Prispievanie

Ak máte nápad na novú funkciu alebo ste našli chybu, [otvorte issue](https://github.com/WebAccessibilityTools/Luma11y/issues). Najprv prehľadajte existujúce issues, aby ste predišli duplicitám.

Pull requesty sú vítané! Pred odoslaním si prosím prečítajte [Pravidlá pre prispievateľov](CONTRIBUTING.md).

## Kontakt

Ak máte akékoľvek otázky, neváhajte [otvoriť issue](https://github.com/WebAccessibilityTools/Luma11y/issues) tu na GitHube.

## Licencia

[![GNU GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Luma11y je slobodný softvér: môžete ho podľa vlastného uváženia používať, skúmať, zdieľať a vylepšovať. Konkrétne ho môžete ďalej šíriť a/alebo upravovať za podmienok [GNU General Public License](https://www.gnu.org/licenses/gpl.html) tak, ako ju zverejnila Free Software Foundation, buď vo verzii 3 tejto licencie, alebo (podľa vášho výberu) v ktorejkoľvek neskoršej verzii.

> Tento program je šírený v nádeji, že bude užitočný, avšak **BEZ AKEJKOĽVEK ZÁRUKY**; dokonca aj bez implicitnej záruky **PREDAJNOSTI** alebo **VHODNOSTI NA KONKRÉTNY ÚČEL**. Podrobnosti nájdete v [GNU General Public License](https://www.gnu.org/licenses/gpl.html).
