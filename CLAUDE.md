# Luma11y — Notes pour Claude / Notes for Claude

Application Tauri (frontend Lit + Alpine.js / TypeScript, backend Rust).
Tauri app (Lit + Alpine.js / TypeScript frontend, Rust backend).

## Principe : calculs et conversions au backend / Principle: compute on the backend

**Tous les calculs et conversions de couleur (et plus largement la logique
numérique) doivent se faire en priorité côté backend Rust**, via `palette`
(`src-tauri/src/color.rs`). Le RGB `u8` est la représentation **canonique** ;
les autres formats (hex, hsl, hsv, lab, …) en sont dérivés côté Rust et exposés
en chaînes prêtes à afficher dans le store (`src-tauri/src/store.rs`).

**All color calculations/conversions (and numeric logic in general) must be done
on the Rust backend first**, through `palette`. RGB `u8` is the canonical
representation; the other formats are derived on the Rust side and exposed as
ready-to-display strings in the store.

Le frontend se limite à :
The frontend is limited to:
- **parser** la saisie (regex + extraction + validation des composantes), sans
  math couleur — cf. `src/colors/*.ts` (chaque format renvoie `{ command, args }`) ;
  **parsing** input (regex + extraction + validation), no color math.
- **afficher** les valeurs renvoyées par le backend.
  **displaying** the values returned by the backend.

Conséquence : pour appliquer une couleur, le frontend invoque la commande Tauri
du format (`update_store_rgb` / `_hsl` / `_hsv` / `_lab` / `_hex`) avec les
composantes brutes ; le backend convertit, recalcule **tous** les formats dérivés
via la voie unique `apply_color`, et émet `store-updated`.
To apply a color, the frontend invokes the format's Tauri command with raw
components; the backend converts, recomputes **all** derived formats through the
single `apply_color` path, and emits `store-updated`.

Note : une conversion lossy (Lab/HSL/HSV → RGB 8 bits → format) ferait « sauter »
la valeur du format édité. `apply_color` reçoit donc la chaîne saisie *verbatim*
pour ce format et la conserve telle quelle (les autres sont recalculées du RGB).
Note: a lossy round-trip would make the edited format's value "jump". So
`apply_color` receives the entered string *verbatim* for that format and keeps it
as-is (the others are recomputed from RGB).

### Ajouter un format de couleur / Adding a color format
1. `src-tauri/src/color.rs` : `rgb_to_<fmt>_string` + `<fmt>_to_rgb` (via `palette`).
2. `src-tauri/src/store.rs` : champs `*_<fmt>`, commande `update_store_<fmt>`,
   verbatim dans `apply_color`. `src-tauri/src/lib.rs` : enregistrer la commande.
3. `src/colors/<fmt>.ts` : `parse` (→ `{ command: 'update_store_<fmt>', args }`)
   + `<fmt>ToCss` ; enregistrer dans `src/colors/index.ts`.
4. `src/store.ts` : champs backend/UI + `colorValues`. `src/components/ColorControls.ts` :
   entrée `FORMAT_CHANNELS`. `src/main.ts` + `settings.html` : balises de copie.
   `src/locales/{en,fr}.json` : libellés (`color.<fmt>`, canaux, balises).
5. Activation : le format est automatiquement *activable* dans Settings (liste dérivée
   de `colorFormats`, hex exclu). Pour qu'il soit **actif par défaut**, l'ajouter à
   `DEFAULT_ENABLED_FORMATS` (`src/colors/index.ts`). `hex` est toujours actif.

## Build

- Frontend : `npx tsc --noEmit` puis `npx vite build`.
- Rust : `cargo check` / `cargo build` dans `src-tauri/`. À lancer via
  `nix-shell ../shell.nix` ; selon l'environnement, la toolchain `clang` peut
  manquer dans le sandbox (deps natives `objc2`) — compiler hors sandbox.

## Conventions

- Commentaires en anglais, comme le code existant. Supprimer les autres versions.
- i18n : clés dans `src/locales/{en,fr}.json` ; un format `id` sert de clé
  `color.${id}`. Garder les deux langues synchronisées.
