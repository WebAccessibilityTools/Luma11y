# Accessibility Statement
(Thanks to https://github.com/ericwbailey/a11y-webring.club/blob/main/.github/ACCESSIBILITY.md)

Luma11y strives to be AA [WCAG 2.2 compliant](https://www.w3.org/WAI/standards-guidelines/wcag/), and is committed to creating and maintaining an accessible, inclusive environment. It is a tool built for accessibility work, and it is intended to be able to be used by everyone.

## What we are doing

The following initiatives are how this application attempts to be accessible. We are:

- Using time-tested, stable and interoperable technology based on open standards (HTML, CSS and ARIA rendered by the native system WebView) so the interface is exposed to the platform accessibility APIs and assistive technologies.
- Making every feature operable with the keyboard alone, including the screen colour picker (pixel-precise arrow moves, zoom, selection and cancellation), with documented and customisable [keyboard shortcuts](README.md#keyboard-shortcuts).
- Testing with screen readers (VoiceOver on macOS and NVDA on Windows), in addition to automated checks.
- Hosting our code on [a public repository](https://github.com/WebAccessibilityTools/Luma11y/), allowing anyone with the interest and capability to inspect and modify it.
- Striving to keep our interactions and user interface unambiguous and easy to understand, with labels and results available in English, French and other to come.
- Striving to keep our download size small and memory footprint light (thanks to Tauri).
- Supporting window zoom, and following the system light or dark theme (or a theme chosen in Settings).
- Maintaining a color palette with high contrast ratio values to keep content legible.
- Using distinct, obvious focus states for all interactive elements.
- Providing open, structured guidance on [how to contribute](CONTRIBUTING.md), with provisions for individuals who may be unwilling or unable to use our preferred workflows.

## Technical specifications

We rely on the following technologies to work with the particular combination of operating system and any assistive technologies or plugins installed on your computer:

* [HTML](https://html.spec.whatwg.org/multipage/),
* [ARIA](https://www.w3.org/WAI/standards-guidelines/aria/),
* [CSS](https://www.w3.org/Style/CSS/),
* [JavaScript](https://www.w3.org/standards/webdesign/script.html),
* the system WebView provided by [Tauri](https://tauri.app/) (WKWebView on macOS, WebView2 on Windows), and
* native menus provided by the operating system.

These technologies are relied upon for conformance with the accessibility standards used.

## Known issues

The [issue tracker](https://github.com/WebAccessibilityTools/Luma11y/issues?q=is%3Aissue+label%3Aa11y) keeps a list of all known open and addressed accessibility issues.

## Reporting issues

I ask that you please let me know if you discover an accessibility-related issue. I take these issues extremely seriously and will fix them as fast as possible. I will also include you in the process, if desired.

You can submit an accessibility issue by [opening an issue on the issue tracker](https://github.com/WebAccessibilityTools/Luma11y/issues/new).

