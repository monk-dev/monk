# Monk

`monk` is a cli and daemon combo that manages articles that you want store and save for a later date.

Currently `monk` can store, list, and download webpages. Planned future features include: full-text search and indexing on page metadata and content, easy and automatic git integration, general date-types (not just html).

## Installation

### Build from Source:

To build from source, first clone the repo and make sure you have a nightly rust compiler. To install rust, visit: https://rustup.rs/

```
$ cargo install --path monkd
$ cargo install --path monk-cli
$ monk --version
monk-cli 0.1.0
```

In the future, pre-built binaries will be available!

## Usage

For any subcommand, simply use `--help` to view all of the options.

### Adding, downloading, and opening an article:
```
$ monk add "AF_XDP" -u https://lwn.net/Articles/750845/ -c "Cool article about fast packet capturing, pretty pictures\!"
╭─────────┬───────────────────────────────────┬────────────────────────────────────────┬───────────────┬─────────────╮
│   name  │                url                │                 comment                │      date     │      id     │
├─────────┼───────────────────────────────────┼────────────────────────────────────────┼───────────────┼─────────────┤
│ AF_XDP  │ https://lwn.net/Articles/750845/  │ Cool article about fast packet capturi │  May 29, 2020 │  njrjdlbd19 │
│         │                                   │ ng, pretty pictures!                   │               │             │
╰─────────┴───────────────────────────────────┴────────────────────────────────────────┴───────────────┴─────────────╯
# ids only need to uniquely identify an item, in this case,
# a single `n` would work.
$ monk download njrj
$ monk open njrj
`njrj` cannot be opened, status: Downloading
$ monk open njrj
```
As you can see above, open will fail until the document is fully downloaded. Downloading a document embeds as many of the assets as possible into a single html file. In the future, there will be options to disable things like css, js, iframes, etc.

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
The daemon will automatically stop after the not receiving a command within `timout` time. This can be set in the config file.

## Configuration

Configuration and data is stored in the preferred system folders. For example, on linux it will use the `XDG_*` environment variables to locate and create monk directors. On linux, the config file is located at `~/.config/monk/monkd.yaml`. Data, logs, and documents are stored under `~/.local/share/monk`.

To view the configuration or the default configuration for the system:

```
$ monk config [file]  # Actual running config
$ monk default-config # System default config
```

Monk will automatically create an missing folders and config files that it needs.

## Backing up the Document Store

The only file `monk` needs for recreating its internal state is the `store.json`, located on linux under: `~/.local/share/monk/store.json`. I currently have a git repo in the folder that commits and pushes that `store.json`.