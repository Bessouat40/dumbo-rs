# Dumbo-RS

<p align="center">
  <img src="https://raw.githubusercontent.com/Bessouat40/dumbo-rs/master/media/dumbo-rs.png" alt="Dumbo-RS Logo" width="200">
</p>

Dumbo-RS is an ultra-fast CLI tool written in Rust that condenses an entire codebase into a single, well-structured Markdown file — the perfect companion for providing comprehensive context to LLMs (ChatGPT, Claude, etc.).

## Installation

```bash
cargo install dumbo-rs
```

The installed binary is named `dumbo`.

## Commands

### `dumbo init`

Detects languages in the current directory and creates a `Dumbo.toml` + `.dumboignore`.

```bash
dumbo init           # single project
dumbo init -r        # monorepo — scans all sub-folders
```

### `dumbo update`

Adds new sub-projects to an existing `Dumbo.toml` (e.g. after adding a new service to a monorepo).

```bash
dumbo update
```

### `dumbo list`

Lists all sections in the current `Dumbo.toml`.

```bash
dumbo list
```

### `dumbo run`

Generates a Markdown context file using the config from `Dumbo.toml`.

```bash
dumbo run                        # → dumbo_output.md  (root project)
dumbo run frontend               # → dumbo_frontend.md
dumbo run frontend backend       # → dumbo_frontend_backend.md
dumbo run frontend -c            # → also copies to clipboard
```

### `dumbo diff`

Generates a Markdown file with three sections: a stat summary, the full git diff, and the current content of every changed file (with line numbers). Useful for debugging a regression — paste the output into an LLM and it has exactly what it needs without flooding the context window.

```bash
dumbo diff abc1234               # → dumbo_diff_abc1234.md
dumbo diff main~3 backend        # → diff of backend since 3 commits ago
dumbo diff v1.0.0 -c             # → diff since a tag, copied to clipboard
dumbo diff --staged              # → diff of staged changes
```

## Config files

**`Dumbo.toml`** — one file at the project root, one section per sub-project:

```toml
[root]
lang = "rs"

[frontend]
lang = "ts"

[backend]
lang = "py"
```

**`.dumboignore`** — one per directory, lists files/folders to exclude:

```
# .dumboignore
secrets.toml
fixtures/
```

## Options

| Option | Description |
|---|---|
| `-c`, `--clipboard` | Copy output to clipboard |
| `-h`, `--help` | Print help |
| `-v`, `--version` | Print version |

## Supported languages

`rs`, `py`, `js`, `ts`, `go`, `java`, `c`, `cpp`

The following are always included regardless of language: `Dockerfile`, `docker-compose.yml`, `Makefile`, `.yaml`, `.yml`, `.toml`, `.md`.
