## ratuin

A tiny implementation inspired by [atuin](https://github.com/atuinsh/atuin), focused on local shell history recording and search.

## Features

- Local history storage with SQLite
- Auto-record shell commands via `init` hooks (`bash`/`zsh`/`fish`)
- Command search by keyword
- Import existing shell history files
- Sensitive command filtering by default

## Code Organization

- `src/lib.rs` / `src/main.rs`: crate and binary entry
- `src/ratuin.rs`: feature module aggregator
- `src/ratuin/app/`: command dispatch and validation (`mod.rs`, `handlers.rs`, `validators.rs`)
- `src/ratuin/db.rs`: database access and history querying
- `src/ratuin/search.rs`: fuzzy subsequence matching logic
- `src/ratuin/importer.rs`: import workflow and transaction handling
- `src/ratuin/importer_parsers.rs`: shell history parsing by format
- `src/ratuin/tui/`: TUI-layer reserved module and request model

## Build

```bash
cargo build --release
```

## Usage

### 1) Enable shell hooks

```bash
eval "$(ratuin init bash)"
eval "$(ratuin init zsh)"
ratuin init fish | source
```

Hooks record:
- command text
- current working directory
- exit code
- duration in milliseconds

### 2) Record manually

```bash
ratuin record --command "cargo check" --cwd "$PWD" --exit-code 0 --duration-ms 120
```

If a command looks sensitive (`token`, `password`, `secret`, etc.), it is skipped by default.
Use `--allow-sensitive` to force recording:

```bash
ratuin record --command "export TOKEN=abc" --allow-sensitive
```

### 3) Search

```bash
ratuin search cargo --limit 20
```

Search uses fuzzy subsequence matching by default: characters in query only need to appear in order, not continuously.
For example, `ratuin search cgo` can match `cargo`.

Show only failed commands:

```bash
ratuin search cargo --failed-only
```

Filter by working directory:

```bash
ratuin search cargo --cwd /home/user/project
```

Notes:
- default search limit is `50`
- `--limit` must be greater than `0`

### 3.5) TUI (placeholder)

```bash
ratuin tui --keyword cgo --cwd /home/user/project --limit 50 --failed-only
```

Use forward mode:

```bash
ratuin tui --forward
```

Current status (MVP):
- interactive list rendering is enabled
- typing updates fuzzy search results in real time
- each result shows execution time and relative time
- reverse mode is default: query and list order are reversed
- `--forward` switches query and list order back to normal direction
- fuzzy-matched characters are highlighted in the command line
- `Up/Down` to navigate, `Enter/Tab` to print selected command
- `Esc` or `Ctrl-C` to quit, `Ctrl-U` to clear query

Shell integration:
- `Ctrl-R` opens `ratuin tui`
- selecting with `Tab` (or `Enter`) returns command to shell buffer for manual editing

### 4) Import existing history

Import from default history path:

```bash
ratuin import --shell bash
ratuin import --shell zsh
ratuin import --shell fish
```

Import from a custom file:

```bash
ratuin import --shell zsh --file ~/.zsh_history.backup
```

Sensitive entries are skipped by default during import as well.
Use `--allow-sensitive` to include them.

Duplicate imported records are also skipped (same `command + cwd + timestamp`).
Import runs in a single database transaction for better performance and safer rollback on errors.