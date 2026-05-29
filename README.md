# forge-cli

CLI tool for the **ForgeFlux** ecosystem. Ties all `forge-*` crates together into a single command-line interface.

## Install

```bash
cargo install --path .
```

## Commands

### `forge detect <file>`

Detect the input format of a file.

```bash
$ forge detect data.csv
{"file":"data.csv","detected_format":"csv"}
```

### `forge decompose <file> [--format FMT]`

Decompose a file into tiles (printed as JSON).

```bash
$ forge decompose input.txt
$ forge decompose data.csv --format csv
```

### `forge compose <tiles.json>`

Reassemble tiles back into content.

```bash
$ forge compose tiles.json
```

### `forge stats <file>`

Show tile statistics for a file.

```bash
$ forge stats input.txt
```

### `forge pipeline <config.json>`

Run a pipeline defined by a JSON config file.

```json
{
  "input": "data.txt",
  "format": "text"
}
```

```bash
$ forge pipeline config.json
```

### `forge list`

List all `forge-*` crates in the ecosystem.

### `forge info <crate-name>`

Show details about a specific crate.

```bash
$ forge info forge-core
```

## Supported Formats

| Format | Detection | Decomposition |
|--------|-----------|---------------|
| Text | Default | Split by lines |
| CSV | Comma-separated with consistent columns | Rows with header-keyed metadata |
| JSON Object | `{...}` parseable | Key-value pairs |
| JSON Array | `[...]` parseable | Array elements |

## License

MIT
