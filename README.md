# Monk

`monk` is a cli and daemon combo that manages articles that you want store and save for a later date.

Currently `monk` can store, list, and download webpages. Planned future features include: easy and automatic git integration, general date-types (not just html), and more data adapters.

## Installation

### Build from Source:

To build from source, first clone the repo (recursively!) and install Rust. To install rust, visit: https://rustup.rs/

```sh
$ cargo install --path monkd
$ cargo install --path monk-cli
$ monk --version
monk-cli 0.1.0
```

In the future, pre-built binaries will be available!

## Usage

For any subcommand, simply use `--help` to view all available options and descriptions.

### Adding, downloading, and opening an article:
```sh
$ monk add "AF_XDP" -u https://lwn.net/Articles/750845/ -c "Cool article about fast packet capturing, pretty pictures\!"
╭─────────┬───────────────────────────────────┬────────────────────────────────────────┬───────────────┬─────────────╮
│   name  │                url                │                 comment                │      date     │      id     │
├─────────┼───────────────────────────────────┼────────────────────────────────────────┼───────────────┼─────────────┤
│ AF_XDP  │ https://lwn.net/Articles/750845/  │ Cool article about fast packet capturi │  Jun 25, 2020 │  ls7d2ypeks │
│         │                                   │ ng, pretty pictures!                   │               │             │
╰─────────┴───────────────────────────────────┴────────────────────────────────────────┴───────────────┴─────────────╯
$ monk open ls7d2
status for [ls7d2]: Downloading
$ monk open njrj
```
IDs only need to uniquely identify a single item. In this case, a single `n` will work.

As you can see above, `monk open` will fail until the document is fully downloaded. Downloading a document embeds as many of the assets as possible into a single html file, usually around `5MB`. In the future, there will be options to disable downloading css, js, iframes, etc.

### Searching for articles

Searching for documents in `monk` is relatively straight forward. Once a document is added, its metadata is immediately searchable.

Since the term "packet" is contained within the comment section of the article, that article is returned:
```sh
$ monk add "AF_XDP" -u https://lwn.net/Articles/750845/ -c "Cool article about fast packet capturing, pretty pictures\!"
$ monk search packet
╭─────────┬───────────────────────────────────┬────────────────────────────────────────┬───────────────┬─────────────╮
│   name  │                url                │                 comment                │      date     │      id     │
├─────────┼───────────────────────────────────┼────────────────────────────────────────┼───────────────┼─────────────┤
│ AF_XDP  │ https://lwn.net/Articles/750845/  │ Cool article about fast packet capturi │  Jun 25, 2020 │  ls7d2ypeks │
│         │                                   │ ng, pretty pictures!                   │               │             │
╰─────────┴───────────────────────────────────┴────────────────────────────────────────┴───────────────┴─────────────╯
```

To search across the article's _contents_, the article must first be downloaded and then manually indexed.
```sh
$ monk download ls7d
$ monk status ls7d
[ls7d2ypeks]:
size:     234 B
index:    not indexed
offline:  Ready
$ monk index ls7d    # begin processing the contents
$ monk index status ls7d
[njrjdlbd19] Indexing
$ monk index status ls7d
[njrjdlbd19] Indexed # at this point contents can now be searched
$ monk search processing bpf interface
╭─────────┬───────────────────────────────────┬────────────────────────────────────────┬───────────────┬─────────────╮
│   name  │                url                │                 comment                │      date     │      id     │
├─────────┼───────────────────────────────────┼────────────────────────────────────────┼───────────────┼─────────────┤
│ AF_XDP  │ https://lwn.net/Articles/750845/  │ Cool article about fast packet capturi │  Jun 25, 2020 │  ls7d2ypeks │
│         │                                   │ ng, pretty pictures!                   │               │             │
╰─────────┴───────────────────────────────────┴────────────────────────────────────────┴───────────────┴─────────────╯
```

To simply index everything that's capable of being indexed:
```sh
$ monk index all
[all] Indexing
```

`monk` uses [tantivy](https://github.com/tantivy-search/tantivy) for its full text search needs. The [query grammar](https://docs.rs/tantivy/0.12.0/tantivy/query/struct.QueryParser.html) supports boolean logic, lexical ranges, phrases, etc. Most queries will feel a lot like dumb Google though, and words must be spelled correctly (fuzzy search soon!).

### Removing an article
```sh
$ monk delete <id>
```
This removes metadata, offline data, and any indexed data.

### Listing all saved articles
```sh
$ monk list
```

### Status

Get the status of and disk space of the different parts of `monkd`:
```sh
$ monk status all
File Store [0.1.0]:
  586 B
  2 item(s)

Offline Store:
  1.07 KB
  2 item(s)

Search Index:
  16.75 KB
  2 item(s)
```
Status of an individual item:
```sh
$ monk status ls7d
[ls7d2ypeks]:
size:     239 B
index:    Indexed
offline:  Ready
```

### Stopping the daemon
```sh
$ monk stop
```

The daemon will automatically stop after the not receiving a command within `timeout` time (default: 10 seconds). This can be set in the config file.

## Configuration

Configuration and data is stored in the preferred system folders. For example, on linux it will use the `XDG_*` environment variables to locate and create monk directories. On linux, the config file is located at `~/.config/monk/monkd.yaml`. Data, logs, and documents are stored under `~/.local/share/monk`.

To view the configuration or the default configuration for the system:

```sh
$ monk config [file]  # Actual running config
$ monk default-config # Default system config
```

Monk will automatically create any missing folders and config files that it needs.

## Backing up the Document Store

The only file `monk` needs for recreating its internal state is the `store.json` file, located on linux under: `~/.local/share/monk/store.json`. I recommend creating a git repo in wherever the `store.json` is and commiting and pushing it.

## Thank You!

Huge thanks to the [monolith](https://github.com/Y2Z/monolith) project for their amazing web archiving library. It's works incredibly well.

Thank you to `Mr. Monk`, the protagonist of the 2002 TV Series [Monk](https://en.wikipedia.org/wiki/Monk_(TV_series)) and one of my favorite fictional characters of all-time. Monk is a quirky detective afflicted with OCD, that can remember and recall even the tiniest of details. The goal of this project is to reach his level of attention to detail and capability to learn everything about an anything.
