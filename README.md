# Mejlmannen

Rust replacement for Postman.

## Supports

- Collections of requests.
- Storage of key/value properties for use in requests.
- Replacing Headers, URIs, and body data with properties.
- Extracting JSON and headers from responses.
- Loading of files for use in request bodies.
- Allow for binary data to be loaded from disk and used in requests.
  - I think.

## TODO

- Better logging/verbosity for CLI.
- Possibly handle conditional execution of requests in the sequence.
- CLI/TUI.
- GUI.

## Usage

So far the usage is:

```cli
mejlman <collection_directory>
```

For testing, you can try:

```cli
mejlman <repo_root>\tests\httpbin\ip
```
