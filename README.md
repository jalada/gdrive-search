# gdrive-search

![Crates.io](https://img.shields.io/crates/v/gdrive-search)

**Warning: I used this to learn Rust. It might be the worst Rust ever.
Feedback to help me learn is very welcome!**

Fifty percent learning Rust, fifty percent building a CLI to very quickly
search & access files in your Google Drive.

On my Mac I had the Google Drive desktop app, and an Alfred workflow to use
that synchronised filesystem to quickly search & open Drive files. But I
haven't found an equivalent super-fast autocompleting search & open for Linux
so I decided to build one.

## What does it look like?

![demo](docs/gdrive_search_demo.gif)

_Hitting enter would then open in your browser_

## What works?

 - Sign in with Google Drive (if you follow setup below).
 - Sync files & folders (currently fetches incrementally, every 5 minutes) and
   save to file.
 - Interface for searching for file/folders.
 - Open chosen file/folder in your web browser.

## How do I get it?

### Binaries

Download [a release from GitHub](https://github.com/jalada/gdrive-search/releases)
if there is one for your platform.

### Arch Linux (including Manjaro etc.)

gdrive-search can be built from source via the
[AUR](https://aur.archlinux.org/packages/gdrive-search/).

### From source

If you have Rust installed you can download and compile gdrive-search from
Cargo:

```
$ cargo install gdrive-search
```

### Windows support

Windows is currently not supported because gdrive-search depends on
[skim](https://github.com/lotabout/skim) which depends on
[tuikit](https://github.com/lotabout/tuikit) which [doesn't support
Windows](https://github.com/lotabout/tuikit/issues?q=windows).

## Google Drive API application setup

This dance will be familiar if you've used any other open source tools that
use the Google Drive API.

 - Create a new project
 - Enable the Google Drive API for your project
 - Setup oauth consent screen
 - Create a new oauth client ID
 - Download the JSON provided and put it in the configuration directory. This
   varies depending on operating system:
   
   |Platform | Value                                                             | Example                                                      |
   |---------|-----------------------------------                                |--------------------------------------------------------------|
   |Linux    | $XDG_CONFIG_HOME or $HOME/.config/gdrive-search/clientsecret.json | /home/alice/.config/gdrive-search/clientsecret.json              |
   |macOS    | $HOME/Library/Preferences/gdrive-search/clientsecret.json         | /Users/Alice/Library/Preferences/gdrive-search/clientsecret.json |

## Releasing a new version

1. Tag the release on Github.
2. Wait for Github Actions to cross compile everything.
3. Update the release notes.
4. Update the [AUR repository](https://aur.archlinux.org/packages/gdrive-search/)
   PKGBUILD.
5. Test the PKGBUILD (ideally in a clean `archlinux:base-devel` docker
   container).
6. Follow the [AUR release instructions](https://wiki.archlinux.org/title/AUR_submission_guidelines#Publishing_new_package_content).

## Roadmap

 - Some indication of what type of file/folder each entry is.
 - Can we fetch in the background whilst skim loads to make it even faster?
 - A way of resetting the configuration.
 - Better error handling if config files are in an invalid state.
 - Multi-account support.
 - A way to force a fetch from scratch.
