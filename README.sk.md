<div align="center">

# Luma11y

![](src-tauri/icons/64x64.png)

**Zistite čitateľnosť textu a kontrast vizuálnych prvkov, ako sú grafické ovládacie prvky a vizuálne indikátory.**

[![GitHub (pred-)vydanie](https://img.shields.io/github/release/WebAccessibilityTools/Luma11y/all.svg?style=flat-square)](https://github.com/WebAccessibilityTools/Luma11y/releases)
[![Licencia GPL-3.0](https://img.shields.io/github/license/WebAccessibilityTools/Luma11y.svg?style=flat-square)](LICENSE)
[![Postavené na Tauri](https://img.shields.io/badge/postaven%C3%A9_na-Tauri-ffc131?style=flat-square&logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

*Prečítajte si v iných jazykoch: [English](README.md) | [Français](README.fr.md) | [Slovenčina](README.sk.md)*

Slovenský preklad: **Radoslav Ďurač** ([@rraddatch](https://github.com/rraddatch))

</div>

---

Tento repozitár obsahuje zdrojový kód aplikácie Luma11y, nástroja na analýzu farebného kontrastu pre Windows a macOS.
Vytvorené v jazyku Rust a [Tauri](https://tauri.app/).

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

- [Funkcie](#funkcie)
- [Klávesové skratky](#klávesové-skratky)
- [Prispievanie](#prispievanie)
- [Kontakt](#kontakt)
- [Licencia](#licencia)

## Funkcie

- **Kontrastný pomer podľa WCAG**: výsledky vyhovuje/nevyhovuje pre kritériá 1.4.3 Kontrast (minimálny), 1.4.6 Kontrast (zvýšený) a 1.4.11 Kontrast netextových prvkov, pre bežný aj veľký text.
- **Výber farby z obrazovky**: lupa na výber ľubovoľného pixelu na obrazovke s nastaviteľným priblížením, presným posunom po pixeloch pomocou klávesnice a *súvislým režimom* na výber oboch farieb za sebou.
- **Formáty farieb**: Hex, RGB, HSL, HSV, Lab, OKLCH a OKLab. Zadajte hodnotu alebo upravte každú zložku posuvníkom. Zobrazované formáty si zvoľte v Nastaveniach.
- **Alfa kanál**: priehľadná farba popredia sa pred meraním kontrastu zmieša s pozadím.
- **Názvy farieb**: ak sa farba presne zhoduje s pomenovanou farbou, zobrazí sa jej názov podľa CSS.
- **Šablóny kopírovania**: skopírujte výsledky do schránky vo vlastnom formáte, pričom každá šablóna má vlastnú skratku.
- **Globálne skratky na výber farby**: spustite výber farby, aj keď Luma11y nie je aktívne okno.
- **Vzhľad**: Moderný alebo Klasický štýl, automatický/svetlý/tmavý motív, priblíženie okna, „Vždy navrchu“.
- **Farebné profily (macOS)**: prepnite ICC profil displeja z panela menu.
- **Obnovenie posledných farieb** pri spustení (voliteľné).
- **Automatické aktualizácie** na karte Aktualizácie v Nastaveniach.
- **Prístupná a lokalizovaná**: dá sa ovládať klávesnicou aj čítačkou obrazovky, k dispozícii v angličtine, francúzštine a slovenčine.

> [!NOTE]
> Na macOS potrebuje výber farby povolenie „Nahrávanie obrazovky a systémového zvuku“, aby mohol čítať pixely na obrazovke. Aplikácia oň požiada pri prvom spustení.

## Klávesové skratky

Na macOS sa namiesto <kbd>Ctrl</kbd> používa <kbd>Cmd</kbd>.

### Hlavné okno

| Skratka | Akcia |
| --- | --- |
| <kbd>F11</kbd> | Vybrať farbu popredia (globálna, prispôsobiteľná) |
| <kbd>F12</kbd> | Vybrať farbu pozadia (globálna, prispôsobiteľná) |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Skopírovať výsledky pomocou predvolenej šablóny (prispôsobiteľná) |
| <kbd>Ctrl</kbd>+<kbd>,</kbd> | Otvoriť Nastavenia |
| <kbd>Ctrl</kbd>+<kbd>+</kbd> / <kbd>Ctrl</kbd>+<kbd>-</kbd> | Priblížiť / oddialiť okno |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Obnoviť priblíženie okna |
| <kbd>Shift</kbd>+šípky | Na posuvníku zložky: zmeniť hodnotu o 10 namiesto 1 |
| <kbd>Esc</kbd> | Zavrieť okno Nastavení alebo otvorené menu |

### Výber farby

| Skratka | Akcia |
| --- | --- |
| Šípky | Posunúť o 1 pixel |
| <kbd>Shift</kbd>+šípky | Posunúť o 50 pixelov |
| <kbd>Enter</kbd>, <kbd>Medzerník</kbd> alebo kliknutie | Vybrať farbu |
| <kbd>Esc</kbd> alebo kliknutie pravým tlačidlom | Zrušiť |
| <kbd>C</kbd> | Zapnúť/vypnúť súvislý režim (výber popredia a pozadia za sebou) |
| <kbd>I</kbd> / <kbd>O</kbd> alebo koliesko myši | Priblížiť / oddialiť |
| <kbd>Shift</kbd>+<kbd>I</kbd> / <kbd>Shift</kbd>+<kbd>O</kbd> alebo <kbd>Shift</kbd>+koliesko | Zachytiť viac / menej pixelov |

Skratky na výber farby a kopírovanie môžete zmeniť v **Nastavenia › Skratky** a **Nastavenia › Šablóny kopírovania**.

## Prispievanie

Ak máte nápad na novú funkciu alebo ste našli chybu, [otvorte issue](https://github.com/WebAccessibilityTools/Luma11y/issues). Najprv prehľadajte existujúce issues, aby ste predišli duplicitám.

Pull requesty sú vítané! Pred odoslaním si prosím prečítajte [Pravidlá pre prispievateľov](CONTRIBUTING.md).

## Kontakt

Ak máte akékoľvek otázky, neváhajte [otvoriť issue](https://github.com/WebAccessibilityTools/Luma11y/issues) tu na GitHube.

## Licencia

[![GNU GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Luma11y je slobodný softvér: môžete ho podľa vlastného uváženia používať, skúmať, zdieľať a vylepšovať. Konkrétne ho môžete ďalej šíriť a/alebo upravovať za podmienok [GNU General Public License](https://www.gnu.org/licenses/gpl.html) tak, ako ju zverejnila Free Software Foundation, buď vo verzii 3 tejto licencie, alebo (podľa vášho výberu) v ktorejkoľvek neskoršej verzii.

> Tento program je šírený v nádeji, že bude užitočný, avšak **BEZ AKEJKOĽVEK ZÁRUKY**; dokonca aj bez implicitnej záruky **PREDAJNOSTI** alebo **VHODNOSTI NA KONKRÉTNY ÚČEL**. Podrobnosti nájdete v [GNU General Public License](https://www.gnu.org/licenses/gpl.html).
