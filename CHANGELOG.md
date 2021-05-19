# `monk` Change Log
-----

## 0.2.0 - 2020-16-29

### Added

* Added a [LICENSE](monk-cli/LICENSE) (Apache 2) to `monk-cli`
* Added a [LICENSE](monkd/LICENSE) (AGPLv3) to `monkd`
* Added descriptions to both `Cargo.toml`s
* Added a [CHANGELOG.md](CHANGELOG.md)

## 0.3.0

### Added
* Monk now uses UUIDs
* Added Search Snippets
* Added the ability to open the (non-local) online version of an article
* Monk will use the `<title>` tag of a web page as a name if none are provided
* Monk now allows you to tag articles
* Monk now allows for most commands to be sorted by tags. `monk list dogos`
will only bring up articles with the `dogos` tag
* You can now edit monk articles
* Added Import/Export for monk stores
* Youtube links will use `youtube-dl` (if installed with `ffmpeg`) to download videos

### How to upgrade from 0.2.0
1. delete your old monk config file. This is need to get `youtube-dl` functionality. You can find it by running `monk config`:
```
"/home/liamwarfield/.config/monk/monkd.yaml"    <--- This is the path to your monk config
---
daemon:
  address: 127.0.0.1
  port: 41562
  timeout: 10000
  download_after_add: true
  download_on_open: true
store:
  path: /home/liamwarfield/.local/share/monk/store.json
offline:
  data_folder: /home/liamwarfield/.local/share/monk/offline
  store_file: /home/liamwarfield/.local/share/monk/offline.json
index:
  path: /home/liamwarfield/.local/share/monk/index
log_dir: /home/liamwarfield/.local/share/monk/logs
adapters:
  - Http
  - Youtube
```

2. Delete your search index. **This will not delete any of your articles:**
```
"/home/liamwarfield/.config/monk/monkd.yaml" 
---
daemon:
  address: 127.0.0.1
  port: 41562
  timeout: 10000
  download_after_add: true
  download_on_open: true
store:
  path: /home/liamwarfield/.local/share/monk/store.json
offline:
  data_folder: /home/liamwarfield/.local/share/monk/offline
  store_file: /home/liamwarfield/.local/share/monk/offline.json
index:
  path: /home/liamwarfield/.local/share/monk/index   <---- Delete this whole directory
log_dir: /home/liamwarfield/.local/share/monk/logs
adapters:
  - Http
  - Youtube
```
3. (Optional) Delete stored articles. If you want old youtube videos to be downloaded with `youtube-dl`,
you need to do this step.  **This will not delete any of your articles, but you will have to run "monk download" after this step:**
```
"/home/liamwarfield/.config/monk/monkd.yaml" 
---
daemon:
  address: 127.0.0.1
  port: 41562
  timeout: 10000
  download_after_add: true
  download_on_open: true
store:
  path: /home/liamwarfield/.local/share/monk/store.json
offline:
  data_folder: /home/liamwarfield/.local/share/monk/offline      <---- Delete this direcory
  store_file: /home/liamwarfield/.local/share/monk/offline.json  <---- Delete this File
index:
  path: /home/liamwarfield/.local/share/monk/index
log_dir: /home/liamwarfield/.local/share/monk/logs
adapters:
  - Http
  - Youtube
```

4. Sit back and enjoy a drink of your choice!