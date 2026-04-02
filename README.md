# Dumbo-RS

<p align="center">
  <img src="https://raw.githubusercontent.com/Bessouat40/dumbo-rs/master/media/dumbo-rs.png" alt="Dumbo-RS Logo" width="200">
</p>

Dumbo-RS is an ultra-fast CLI tool written in Rust that condenses an entire codebase into a single, well-structured Markdown file — the perfect companion for providing comprehensive context to LLMs (ChatGPT, Claude, etc.).

## Features

- **Project Tree** — Builds a visual directory structure at the top of the output, showing only the files that are actually included.
- **Markdown Output** — Files are wrapped in fenced code blocks with language hints for better LLM interpretation.
- **Multi-Language** — Combine languages in a single run with comma-separated extensions (`rs,ts`).
- **Universal Files** — Always includes `Dockerfile`, `docker-compose`, `Makefile`, `.yml`, `.toml`, and `.md` files regardless of language.
- **Smart Ignore** — Default exclusions per language (`target`, `node_modules`, `venv`, `.git`, etc.), plus CLI overrides.
- **`.dumboignore`** — Drop a `.dumboignore` file at the root of your project to permanently ignore specific directories or files (supports `#` comments).
- **Clipboard** — Copy the output directly to your clipboard with `-c`.
- **Blazing Fast** — Built with Rust for near-instant execution, even on large repositories.

## Installation

```bash
cargo install dumbo-rs
```

The installed binary is named `dumbo`.

## Usage

```bash
dumbo <ext> <input_dir> <output_file> [extra_ignore_dirs...] [options]
```

| Argument | Description |
|---|---|
| `<ext>` | Language(s): `rs`, `py`, `js`, `ts` — comma-separated for multiple |
| `<input_dir>` | Path to the project directory |
| `<output_file>` | Output file path (e.g. `context.md`) |
| `[extra_ignore_dirs]` | Additional directories or files to ignore (optional) |

| Option | Description |
|---|---|
| `-c`, `--clipboard` | Also copy the output to the clipboard |
| `-h`, `--help` | Print help |
| `-v`, `--version` | Print version |

## Examples

Rust project:
```bash
dumbo rs ./my_project context.md
```

Full-stack project (Rust backend + TypeScript frontend):
```bash
dumbo rs,ts ./my_app context.md
```

TypeScript project, ignoring test folders, copied to clipboard:
```bash
dumbo ts ./web_app output.md tests_data logs -c
```

## Supported Languages

| Argument | Extensions | Default Ignored Directories |
|---|---|---|
| `rs` | `.rs` | `target`, `.git` |
| `py` | `.py` | `__pycache__`, `venv`, `.venv`, `.git` |
| `js` | `.js`, `.jsx` | `node_modules`, `dist`, `.git` |
| `ts` | `.ts`, `.tsx` | `node_modules`, `dist`, `.git` |

The following are always included regardless of language: `Dockerfile`, `docker-compose.yml`, `Makefile`, `.yaml`, `.yml`, `.toml`, `.md`.

## .dumboignore

Place a `.dumboignore` file at the root of your project to define permanent ignore rules:

```
# .dumboignore

# directories
tests_data
fixtures

# files
Cargo.lock
secrets.toml
```

Entries are matched by name and apply to the entire directory tree.
