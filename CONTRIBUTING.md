# Code Contributions

Please follow these guidlines before sending your pull request and making contributions.

* When you submit a pull request, you agree that your code is published under the [GNU General Public License](https://www.gnu.org/licenses/gpl.html)
* Do not include non-free software or modules with your code.
* Make sure your pull request is setup to merge your branch to Luma11y's development branch.
* Make sure your branch is up to date with the development branch before submitting your pull request.
* Stick to a similar style of code already in the project. Please look at current code to get an idea on how to do this.
* Comment your code when necessary.
* Please test your code.  Make sure new features work as well as core features.
* Please limit the amount of Node Modules that you introduce into the project.  Only include them when absolutely necessary for your code to work or if a module provides similar functionality to what you are trying to achieve.

# Setting up Your Environment

Here's how to get your environment setup.  You will need Git and NPM installed on your system.

Clone down the repository:

```
git clone https://github.com/WebAccessibilityTools/Luma11y.git
```

Install Dependencies:

```
npm install
```

Install Dev Dependencies:

```
npm install --only=dev
```

Run the application:

```
npm run start
```

To build a new version:

```shell
  # Because some libraries need to be compiled natively, you can build only for your current OS.
  # i.e. you can't build a Windows version if you are under MacOS

  # build a version for the current platform
  npm run dist
```

The CHANGELOG.md is generated from the git history with
[git-cliff](https://git-cliff.org), configured in `cliff.toml`. Commits are
grouped following the [Conventional Commits](https://www.conventionalcommits.org)
convention (e.g. `feat:`, `fix:`, `doc:`), but non-conventional messages are
still included (in the "Other" group), so a strict format is not required.

The script only generates the *unreleased* section (commits since the latest git
tag) and prepends it to CHANGELOG.md, so previously released sections are never
rewritten and can be hand-edited safely. Run it once when cutting a release:

```shell
pnpm changelog
```

Tip: pass a tag to label the new section with the version instead of
`[unreleased]`:

```shell
pnpm changelog --tag v0.2.0
```

Other commands are available in the `package.json` file.

## Development
> pnpm install
> pnpm tauri dev

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Test for ICC profiles
https://www.color-hex.com/

## i18n
https://github.com/razein97/tauri-plugin-i18n