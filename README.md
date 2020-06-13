# Monk

`monk` is a cli and daemon combo that manages articles that you want store and save for a later date.

Currently `monk` can store, list, and download webpages. Planned future features include: full-text search and indexing on page metadata and content, easy and automatic git integration, general date-types (not just html).

## Installation

### Build from Source:

To build from source, first clone the repo (recursively!) and make sure you have a nightly rust compiler. To install rust, visit: https://rustup.rs/

```
$ cargo install --path monkd
$ cargo install --path monk-cli
$ monk --version
monk-cli 0.1.0
```

In the future, pre-built binaries will be available!

## Usage

For any subcommand, simply use `--help` to view all available options and descriptions.

### Adding, downloading, and opening an article:
```
$ monk add "AF_XDP" -u https://lwn.net/Articles/750845/ -c "Cool article about fast packet capturing, pretty pictures\!"
╭─────────┬───────────────────────────────────┬────────────────────────────────────────┬───────────────┬─────────────╮
│   name  │                url                │                 comment                │      date     │      id     │
├─────────┼───────────────────────────────────┼────────────────────────────────────────┼───────────────┼─────────────┤
│ AF_XDP  │ https://lwn.net/Articles/750845/  │ Cool article about fast packet capturi │  May 29, 2020 │  njrjdlbd19 │
│         │                                   │ ng, pretty pictures!                   │               │             │
╰─────────┴───────────────────────────────────┴────────────────────────────────────────┴───────────────┴─────────────╯
# ids only need to uniquely identify an item. 
# In this case, a single `n` will work.
$ monk download njrj
$ monk open njrj
`njrj` cannot be opened, status: Downloading
$ monk open njrj
```

As you can see above, `monk open` will fail until the document is fully downloaded. Downloading a document embeds as many of the assets as possible into a single html file, usually around ~5MB. In the future, there will be options to disable downloading css, js, iframes, etc.

### Removing an article
```
$ monk delete <id>
```

### Listing all saved articles
```
$ monk list
```

### Stopping the daemon
```
$ monk stop
```

The daemon will automatically stop after the not receiving a command within `timeout` time (default: 10 seconds). This can be set in the config file.

## Configuration

Configuration and data is stored in the preferred system folders. For example, on linux it will use the `XDG_*` environment variables to locate and create monk directories. On linux, the config file is located at `~/.config/monk/monkd.yaml`. Data, logs, and documents are stored under `~/.local/share/monk`.

To view the configuration or the default configuration for the system:

```
$ monk config [file]  # Actual running config
$ monk default-config # Default system config
```

Monk will automatically create any missing folders and config files that it needs.

## Backing up the Document Store

The only file `monk` needs for recreating its internal state is the `store.json` file, located on linux under: `~/.local/share/monk/store.json`. I I recommend creating a git repo in the folder and commiting and pushing `store.json`.

## Thank You!

Huge thanks to the [monolith](https://github.com/Y2Z/monolith) project for their amazing web archiving library. It's simply amazing how well it works.

Thank you to `Mr. Monk`, the protagonist of the 2002 TV Series [Monk](https://en.wikipedia.org/wiki/Monk_(TV_series)) and one of my favorite characters of all-time. Monk is a quirky detective afflicted with OCD, that can remember and recall even the tiniest of details. The goal of this project is to reach his level of attention to detail and capability to learn everything about an article.