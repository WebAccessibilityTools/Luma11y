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
- [Installation (macOS)](#installation-macos)
- [Contribuer](#contribuer)
- [Contact](#contact)
- [Licence](#licence)

## Fonctionnalités

### Pour la Bêta 2

| Fonctionnalité | Statut |
|---|:---:|
| Accessibilité | En cours |
| Composante alpha | Prévu |
| Valeurs de couleur HCL | Prévu |
| Valeurs de couleur HSV | Prévu |
| Valeurs de couleur LAB | Prévu |
| Valeurs de couleur LCHab | Prévu |
| Valeurs de couleur CMJN | Prévu |
| Saisie libre | Fait |
| Installeur Windows/macOS | Prévu |
| Mise à jour automatique | Prévu |
| Certificats signés | Prévu |

### Futur

| Fonctionnalité | Statut |
|---|:---:|
| Version Linux | Prévu |
| Mode réduit/barre de menus | Prévu |
| Simulateur de daltonisme | Prévu |

## Installation (macOS)

> [!NOTE]
> L'application n'est pas encore signée avec un certificat Apple Developer. Vous devrez contourner Gatekeeper en utilisant l'une des options ci-dessous.

### Option 1 &mdash; Désactiver Gatekeeper (recommandé)

1. Téléchargez la dernière version depuis la page [Releases](https://github.com/WebAccessibilityTools/Luma11y/releases).
2. Déplacez le fichier `Luma11y.app` décompressé dans votre dossier **Applications**. **Ne double-cliquez pas encore.**
3. Ouvrez le **Terminal** et exécutez :
   ```shell
   sudo spctl --master-disable
   ```
4. Allez dans **Réglages du système** > **Confidentialité et sécurité** > **Sécurité** et choisissez **N'importe où**.
5. Double-cliquez sur `Luma11y.app` pour le lancer. Acceptez l'invite si demandé.

### Option 2 &mdash; Sans désactiver Gatekeeper

1. Téléchargez la dernière version depuis la page [Releases](https://github.com/WebAccessibilityTools/Luma11y/releases).
2. Activez **Réglages du système** > **Confidentialité et sécurité** > **Sécurité** > **App Store et développeurs identifiés**.
3. Double-cliquez sur `Luma11y.app`. Il sera bloqué. 
4. Allez dans **Réglages du système** > **Confidentialité et sécurité** > **Sécurité** et cliquez sur **Ouvrir quand même**.
5. Si demandé, autorisez l'exécution de l'application.

### Option 3 &mdash; Supprimer la quarantaine pour Luma11y uniquement

1. Téléchargez la dernière version depuis la page [Releases](https://github.com/WebAccessibilityTools/Luma11y/releases).
2. Déplacez le fichier `Luma11y.app` décompressé dans votre dossier **Applications**. **Ne double-cliquez pas encore.**
3. Ouvrez le **Terminal** et exécutez :
   ```shell
   sudo xattr -cr ~/Applications/Luma11y.app
   ```

## Contribuer

Si vous avez une idée de nouvelle fonctionnalité ou si vous avez trouvé un bug, veuillez [ouvrir un ticket](https://github.com/WebAccessibilityTools/Luma11y/issues). Recherchez d'abord les issues existantes pour éviter les doublons.

Les pull requests sont les bienvenues ! Veuillez suivre les [directives de contribution](CONTRIBUTING.md) avant de soumettre.

## Contact

Si vous avez des questions, n'hésitez pas à [ouvrir un ticket](https://github.com/WebAccessibilityTools/Luma11y/issues) ici sur GitHub.

## Licence

[![GNU GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.fr.html)

Luma11y est un Logiciel Libre : vous pouvez l'utiliser, l'étudier, le partager et l'améliorer à votre guise. Plus précisément, vous pouvez le redistribuer et/ou le modifier selon les termes de la [Licence Publique Générale GNU](https://www.gnu.org/licenses/gpl.html) telle que publiée par la Free Software Foundation, soit la version 3 de la Licence, soit (à votre choix) toute version ultérieure.

> Ce programme est distribué dans l'espoir qu'il sera utile, mais **SANS AUCUNE GARANTIE** ; sans même la garantie implicite de **QUALITÉ MARCHANDE** ou d'**ADÉQUATION À UN USAGE PARTICULIER**. Voir la [Licence Publique Générale GNU](https://www.gnu.org/licenses/gpl.html) pour plus de détails.
