# gdrive-search

**Warning: I used this to learn Rust. It might be the worst Rust ever.**

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

# Google Drive API setup notes.

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
   |Windows  | {FOLDERID_RoamingAppData}\gdrive-search\clientsecret.json         | C:\Users\Alice\AppData\Roaming\gdrive-search\clientsecret.json   |

# Roadmap

 - Some indication of what type of file/folder each entry is.
 - Can we fetch in the background whilst skim loads to make it even faster?
 - A way of resetting the configuration.
 - Better error handling if config files are in an invalid state.
 - Multi-account support.
 - A way to force a fetch from scratch.
