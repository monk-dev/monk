


## Developing Monk

You'll need to install a couple of tools:

- docker and docker-compose
- yarn and node
- psql

## Running the Server

To run the server in debug mode:
```
RUST_LOG=info cargo run -p monk-server
```

In debug mode, the server will wipe _and_ seed the DB with some inital articles and tags. Add `--release` to run in release mode.

The default port is `5555`

## Running the frontend

To run the frontend:
```
cd monk-frontend && yarn start
```