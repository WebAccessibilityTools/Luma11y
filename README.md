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
- [Installation (macOS)](#installation-macos)
- [Contributing](#contributing)
- [Contact](#contact)
- [License](#license)

## Features

### For Beta 2

| Feature | Status |
|---|:---:|
| Accessibility | In progress |
| Alpha colour component | Planned |
| New colour formats | done |
| Free text entry | Done |
| Windows/macOS installer | Planned |
| Auto-update | Planned |
| Signed certificates | Planned |

### Future

| Feature | Status |
|---|:---:|
| Linux version | Planned |
| Reduced/menubar mode | Planned |
| Colour blindness simulator | Planned |

## Installation (macOS)

> [!NOTE]
> The app is not yet signed with an Apple Developer certificate. You will need to bypass Gatekeeper using one of the options below.

### Option 1 &mdash; Disable Gatekeeper (recommended)

1. Download the latest release from the [Releases](https://github.com/WebAccessibilityTools/Luma11y/releases) page.
2. Move the unzipped `Luma11y.app` to your **Applications** folder. **Do not double-click yet.**
3. Open **Terminal** and run:
   ```shell
   sudo spctl --master-disable
   ```
4. Go to **System Settings** > **Privacy & Security** > **Security** and choose **Anywhere**.
5. Double-click `Luma11y.app` to launch it. Accept the prompt if asked.

### Option 2 &mdash; Without disabling Gatekeeper

1. Download the latest release from the [Releases](https://github.com/WebAccessibilityTools/Luma11y/releases) page.
2. Enable **System Settings** > **Privacy & Security** > **Security** > **App Store and identified developers**.
3. Double-click `Luma11y.app`. It will be blocked.
4. Go to **System Settings** > **Privacy & Security** > **Security** and click **Open Anyway**.
5. If prompted, allow the application to run.

### Option 3 &mdash; Remove quarantine for Luma11y only

1. Download the latest release from the [Releases](https://github.com/WebAccessibilityTools/Luma11y/releases) page.
2. Move the unzipped `Luma11y.app` to your **Applications** folder. **Do not double-click yet.**
3. Open **Terminal** and run:
   ```shell
   sudo xattr -cr ~/Applications/Luma11y.app
   ```

## Contributing

If you have an idea for a new feature or found a bug, please [open an issue](https://github.com/WebAccessibilityTools/Luma11y/issues). Search existing issues first to prevent duplicates.

Pull requests are welcome! Please follow the [Contribution Guidelines](CONTRIBUTING.md) before submitting.

## Contact

If you have any questions, feel free to [open an issue](https://github.com/WebAccessibilityTools/Luma11y/issues) here on GitHub.

## License

[![GNU GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Luma11y is Free Software: you can use, study, share and improve it at your will. Specifically you can redistribute and/or modify it under the terms of the [GNU General Public License](https://www.gnu.org/licenses/gpl.html) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

> This program is distributed in the hope that it will be useful, but **WITHOUT ANY WARRANTY**; without even the implied warranty of **MERCHANTABILITY** or **FITNESS FOR A PARTICULAR PURPOSE**. See the [GNU General Public License](https://www.gnu.org/licenses/gpl.html) for more details.
