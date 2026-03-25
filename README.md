## ratuin

A tiny implementation inspired by [atuin](https://github.com/atuinsh/atuin), focused on local shell history recording and search.

## Features

- Local history storage with SQLite
- Auto-record shell commands via `init` hooks (`bash`/`zsh`/`fish`)
- Command search by keyword
- Import existing shell history files
- Sensitive command filtering by default

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