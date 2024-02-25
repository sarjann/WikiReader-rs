# WikiTerm-rs

This project's goal is to create a terminal app (using mainly vim motions) to
allow you to quickly search through an offline copy of wikipedia.

## Installation / Setup

This can be installed through crates.io

```bash
cargo install wiki_reader
# To start (Please set the config as below first)
wiki_reader
```

This app expects a config file to be present at
`~/.config/wikiterm/config.json`

With the following format:
```json
{
    "wiki_bzip_path": "~/Documents/wiki/simple/base.bz2",
    "meta_directory": "~/Documents/wiki/simple/meta",
}
```


The meta_directory will be created from the above config if it does not
already exist.

The `wiki_bzip_path` is the path to the bzip2 (xml) archive file that is
downloaded. It's expected that this is a multistream version.

https://meta.wikimedia.org/wiki/Data_dump_torrents

Note that larger dumps will take a lot longer to index (but this only needs
to be done once).

The meta_directory is the place you want any indexing / file produced by this
project to go.

## Usage

You should be able to navigate with (currently a subset of vim bindings)

```
j, k
up, down

:q to quit or Ctrl-c

Esc to exit back to normal mode
/ to search
```
These are some basic bindings, in order to get the rest, use ?.


## Limitations / Room for improvement
This is currently a work in progress so many features will be missing / not
perfect, e.g.

* Search isn't super great right now, will need to be improved
* Not currently parsing wiki text so no formatting on the output
* Support for images locally, and possibly allowing you to set a flag for remote.

As this is still fairly new there will probably be quite a few breaking changes
along the way, e.g. changes the config path or the saved file names (which
will require reindexing).

## Contributing

If you want to make significant changes please first open an issue to avoid
any wasted time. This is my first OS project and one of my first proper rust
projects which I'm using to learn rust so this might not be properly idiomatic
rust and there may be some shortcuts here.

