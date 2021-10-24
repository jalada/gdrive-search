# gdrive_search

**Very WIP. I also suck at Rust.**

Fifty percent learning Rust, fifty percent building a CLI to v. quickly
search & access files in your Google Drive.

On my Mac I had the Google Drive desktop app, and an Alfred workflow to use
that synchronised filesystem to quickly search & open Drive files. But I
haven't found an equivalent super-fast autocompleting search & open for Linux
so I decided to build one.

## What works?

 - Sign in with Google Drive (if you follow setup below).
 - Sync files & folders (currently fetches incrementally, every 5 minutes) and
   save to file.
 - Interface for searching for file/folders.
 - Open chosen file/folder in your web browser.

# Google Drive API setup notes.

 - Create a new project
 - Enable the Google Drive API for your project
 - Setup oauth consent screen
 - Create a new oauth client ID
 - Download the JSON provided and put it in <TBD>

# TODO

Many things, including:

- [ ] Opening files seems to pick the right Google account, but opening folders...doesn't?!
- [ ] Not properly storing files.json and LAST_FETCHED in a good place.
- [ ] No good place for oauth JSON file.
