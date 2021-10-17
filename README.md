# gdrive_search

**Very WIP. I also suck at Rust.**

Fifty percent learning Rust, fifty percent building a CLI to v. quickly
search & access files in your Google Drive.

On my Mac I had the Google Drive desktop app, and an Alfred workflow to use
that synchronised filesystem to quickly search & open Drive files. But I
haven't found an equivalent super-fast autocompleting search & open for Linux
so I decided to build one.

My basic idea is:

 - Sync files regularly 
    - do a quick sync when using the CLI tool if the sync is out of date,
    - though I might prefer to do this as some background task or something,
    - or maybe Google Drive have a streaming API I can use?)
 - Do a fairly straightforward substring search (always seemed to work OK for
   me)
 - Some kind of interface where you can do these searches and choose a file
   and open it. I'm wondering if I can just talk 
   [dmenu](https://tools.suckless.org/dmenu/).
    - By interface I don't necessarily mean graphical. I'm hoping that this 
      is the lower level command that can be piped into something else.

# Google Drive API setup notes.

 - Create a new project
 - Enable the Google Drive API for your project
 - Setup oauth consent screen
 - Create a new oauth client ID
 - Download the JSON provided and put it in <TBD>
