# Dumbo-RS

<p align="center">
  <img src="https://raw.githubusercontent.com/Bessouat40/dumbo-rs/master/media/dumbo-rs.png" alt="Dumbo-RS Logo" width="200">
</p>

Dumbo-RS is an ultra-fast Command Line Interface (CLI) tool written in Rust, designed to condense an entire codebase into a single, well-structured text file.

It is the perfect companion for providing comprehensive context to LLMs (ChatGPT, Claude, etc.) by including a visual project tree and all relevant source files while intelligently skipping "junk" directories.

## Features

- 📁 Project Tree Generation: Automatically builds a visual directory structure at the top of the output file.

- 🔍 Language-Specific Filtering: Native support for Rust, Python, JavaScript, and TypeScript.

- 📦 Universal Files: Automatically includes Dockerfile, docker-compose, Makefile, and .yml / .toml files regardless of the chosen language.

- 🚫 Smart Ignore: Default exclusions for heavy or irrelevant directories (target, node_modules, venv, .git, etc.).

- 🛠️ On-the-fly Customization: Add extra directories to ignore directly from the command line.

- 🚀 Blazing Fast: Built with Rust for near-instant execution, even on large repositories.

## Installation With Cargo

If you have Rust installed, you can install the tool directly from the source:

```bash
cargo install dumbo-rs
```

## 📖 Usage

The basic syntax is as follows:

```bash
dumbo-rs <extension> <input_dir> <output_file> [extra_ignore_dirs...]
```

### Examples

For a Rust project:

```bash
dumbo-rs rs ./my_project context_rust.txt
```

For a TypeScript project (includes .ts and .tsx) while ignoring custom test data folders:

```bash
dumbo-rs ts ./web_app output.txt tests_data logs temp
```

📂 Supported Languages

Argument

Extensions Included

Default Ignored Directories

- rs
- py
- js (both js, jsx)
- ts (both ts, tsx)

Infrastructure files like Dockerfile, docker-compose.yml, Makefile, and config files (.yaml, .toml) are included by default.
