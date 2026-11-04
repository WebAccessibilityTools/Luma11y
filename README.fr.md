<div align="center">

# Luma11y

![](src-tauri/icons/64x64.png)

**Déterminez la lisibilité du texte et le contraste des éléments visuels, tels que les contrôles graphiques et les indicateurs visuels.**

[![Version GitHub (pré-)release](https://img.shields.io/github/release/WebAccessibilityTools/Luma11y/all.svg?style=flat-square)](https://github.com/WebAccessibilityTools/Luma11y/releases)
[![Licence GPL-3.0](https://img.shields.io/github/license/WebAccessibilityTools/Luma11y.svg?style=flat-square)](LICENSE)
[![Construit avec Tauri](https://img.shields.io/badge/construit_avec-Tauri-ffc131?style=flat-square&logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

*Read this in other languages: [English](README.md) | [Français](README.fr.md) | [Slovenčina](README.sk.md)*

</div>

---

Ce dépôt contient le code source de Luma11y, un outil d'analyse de contraste des couleurs pour Windows et macOS.
Codé en Rust, basée sur [Tauri](https://tauri.app/).

<div align="center">

![Interface Luma11y](fr/luma11y-modern-fr.png)

</div>

## ❤️ Soutenir ce projet

Ce projet est développé principalement sur mon temps libre, porté par la
conviction que les outils d'accessibilité doivent être ouverts et
accessibles à tous. Pour qu'il reste fiable et facile à installer, j'ai
besoin de certificats de signature de code pour Windows et macOS — ce qui
représente un coût annuel réel. Si cet outil vous fait gagner du temps ou
aide votre équipe à livrer des produits plus accessibles, pensez à
sponsoriser ce projet. Même une petite contribution aide à le garder signé,
maintenu et gratuit.

[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github)](https://github.com/sponsors/WebAccessibilityTools)

Ou, si vous préférez, offrez-moi un café pour m'aider à améliorer cet outil encore davantage.

<a href="https://www.buymeacoffee.com/ctrevisan" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height=48 ></a>

## Table des matières

- [Fonctionnalités](#fonctionnalités)
- [Raccourcis clavier](#raccourcis-clavier)
- [Contribuer](#contribuer)
- [Contact](#contact)
- [Licence](#licence)

## Fonctionnalités

- **Ratio de contraste WCAG** : résultats réussite/échec pour les critères 1.4.3 Contraste (minimum), 1.4.6 Contraste (amélioré) et 1.4.11 Contraste du contenu non textuel, pour le texte normal et le grand texte.
- **Pipette à l'écran** : une loupe pour capturer n'importe quel pixel de l'écran, avec zoom réglable, déplacement au pixel près au clavier et un *mode continu* pour capturer les deux couleurs à la suite.
- **Formats de couleur** : Hex, RGB, HSL, HSV, Lab, OKLCH et OKLab. Saisissez une valeur ou ajustez chaque composante avec un curseur. Choisissez les formats affichés dans les Préférences.
- **Canal alpha** : une couleur de premier plan transparente est fusionnée avec l'arrière-plan avant le calcul du contraste.
- **Noms de couleur** : le nom de couleur CSS est affiché lorsqu'une couleur correspond exactement à une couleur nommée.
- **Modèles de copie** : copiez les résultats dans le presse-papiers au format de votre choix, chaque modèle avec son propre raccourci.
- **Raccourcis globaux de la pipette** : lancez la pipette même lorsque Luma11y n'a pas le focus.
- **Apparence** : style Moderne ou Classique, thème automatique/clair/sombre, zoom de la fenêtre, « Toujours au premier plan ».
- **Profils de couleur (macOS)** : changez le profil ICC de l'écran depuis la barre de menus.
- **Restauration des dernières couleurs** au démarrage (optionnel).
- **Mises à jour automatiques** depuis l'onglet Mises à jour des Préférences.
- **Accessible et traduit** : utilisable au clavier et avec un lecteur d'écran, disponible en anglais, français et slovaque.

> [!NOTE]
> Sur macOS, la pipette a besoin de l'autorisation « Enregistrement de l'écran et de l'audio du système » pour lire les pixels de l'écran. L'application la demande au premier lancement.

## Raccourcis clavier

Sur macOS, <kbd>Cmd</kbd> remplace <kbd>Ctrl</kbd>.

### Fenêtre principale

| Raccourci | Action |
| --- | --- |
| <kbd>F11</kbd> | Capturer la couleur de premier plan (global, personnalisable) |
| <kbd>F12</kbd> | Capturer la couleur d'arrière-plan (global, personnalisable) |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Copier les résultats avec le modèle par défaut (personnalisable) |
| <kbd>Ctrl</kbd>+<kbd>,</kbd> | Ouvrir les Préférences |
| <kbd>Ctrl</kbd>+<kbd>+</kbd> / <kbd>Ctrl</kbd>+<kbd>-</kbd> | Agrandir / réduire le zoom de la fenêtre |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Réinitialiser le zoom de la fenêtre |
| <kbd>Maj</kbd>+flèches | Sur un curseur de composante : modifier la valeur de 10 au lieu de 1 |
| <kbd>Échap</kbd> | Fermer la fenêtre des Préférences, ou un menu ouvert |

### Pipette

| Raccourci | Action |
| --- | --- |
| Flèches | Déplacer d'un pixel |
| <kbd>Maj</kbd>+flèches | Déplacer de 50 pixels |
| <kbd>Entrée</kbd>, <kbd>Espace</kbd> ou clic | Sélectionner la couleur |
| <kbd>Échap</kbd> ou clic droit | Annuler |
| <kbd>C</kbd> | Activer/désactiver le mode continu (capturer premier plan et arrière-plan à la suite) |
| <kbd>I</kbd> / <kbd>O</kbd> ou molette | Zoom avant / arrière |
| <kbd>Maj</kbd>+<kbd>I</kbd> / <kbd>Maj</kbd>+<kbd>O</kbd> ou <kbd>Maj</kbd>+molette | Capturer plus / moins de pixels |

Les raccourcis de la pipette et de copie se modifient dans **Préférences › Raccourcis** et **Préférences › Modèles de copie**.

## Contribuer

Si vous avez une idée de nouvelle fonctionnalité ou si vous avez trouvé un bug, veuillez [ouvrir un ticket](https://github.com/WebAccessibilityTools/Luma11y/issues). Recherchez d'abord les issues existantes pour éviter les doublons.

Les pull requests sont les bienvenues ! Veuillez suivre les [directives de contribution](CONTRIBUTING.md) avant de soumettre.

## Contact

Si vous avez des questions, n'hésitez pas à [ouvrir un ticket](https://github.com/WebAccessibilityTools/Luma11y/issues) ici sur GitHub.

## Licence

[![GNU GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.fr.html)

Luma11y est un Logiciel Libre : vous pouvez l'utiliser, l'étudier, le partager et l'améliorer à votre guise. Plus précisément, vous pouvez le redistribuer et/ou le modifier selon les termes de la [Licence Publique Générale GNU](https://www.gnu.org/licenses/gpl.html) telle que publiée par la Free Software Foundation, soit la version 3 de la Licence, soit (à votre choix) toute version ultérieure.

> Ce programme est distribué dans l'espoir qu'il sera utile, mais **SANS AUCUNE GARANTIE** ; sans même la garantie implicite de **QUALITÉ MARCHANDE** ou d'**ADÉQUATION À UN USAGE PARTICULIER**. Voir la [Licence Publique Générale GNU](https://www.gnu.org/licenses/gpl.html) pour plus de détails.
