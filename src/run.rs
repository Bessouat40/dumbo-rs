use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process;
use crate::config::{Config, is_included_file, lang_hint, format_size};
use crate::clipboard::{copy_to_clipboard, CLIPBOARD_WARN_BYTES};

// ── .dumboignore ───────────────────────────────────────────────────────────────

pub fn load_dumboignore(dir: &Path) -> Vec<String> {
    let path = dir.join(".dumboignore");
    let Ok(content) = fs::read_to_string(&path) else { return vec![] };
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

// ── Dumbo.toml ─────────────────────────────────────────────────────────────────

pub fn load_dumbo_config(dir: &Path) -> Option<toml::Value> {
    let content = fs::read_to_string(dir.join("Dumbo.toml")).ok()?;
    content.parse().ok()
}

fn read_section_lang(section: &str) -> Option<String> {
    let value = load_dumbo_config(Path::new("."))?;
    value.get(section)?.get("lang")?.as_str().map(|s| s.to_string())
}

// ── File helpers ───────────────────────────────────────────────────────────────

fn write_text_from_file(input_file: &Path, output_file: &Path) -> io::Result<()> {
    let mut out = OpenOptions::new().create(true).append(true).open(output_file)?;
    let content = fs::read_to_string(input_file).unwrap_or_default();
    let lang = lang_hint(input_file);
    let block = format!(
        "### `{}`\n\n```{}\n{}\n```\n\n",
        input_file.to_str().unwrap_or(""),
        lang,
        content.trim_end()
    );
    out.write_all(block.as_bytes())
}

// ── Tree ───────────────────────────────────────────────────────────────────────

pub fn generate_tree(dir: &Path, prefix: &str, config: &Config, extra_ignored: &[String]) -> String {
    let mut tree = String::new();

    let Ok(read) = fs::read_dir(dir) else { return tree };
    let mut entries: Vec<_> = read
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                !config.ignored_dirs.contains(&name) && !extra_ignored.contains(&name.to_string())
            } else {
                is_included_file(&path, config) && !extra_ignored.contains(&name.to_string())
            }
        })
        .collect();

    entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

    let count = entries.len();
    for (i, entry) in entries.into_iter().enumerate() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };

        if path.is_dir() {
            tree.push_str(&format!("{}{}{}/\n", prefix, connector, name));
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            tree.push_str(&generate_tree(&path, &new_prefix, config, extra_ignored));
        } else {
            tree.push_str(&format!("{}{}{}\n", prefix, connector, name));
        }
    }
    tree
}

// ── Directory processing ───────────────────────────────────────────────────────

pub struct Stats {
    pub file_count: usize,
    pub total_bytes: u64,
}

pub fn process_directory(
    dir: &Path,
    output_file: &Path,
    config: &Config,
    extra_ignored: &[String],
    stats: &mut Stats,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if config.ignored_dirs.contains(&name) || extra_ignored.contains(&name.to_string()) {
                continue;
            }
            process_directory(&path, output_file, config, extra_ignored, stats)?;
        } else if is_included_file(&path, config) {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if extra_ignored.contains(&name.to_string()) { continue; }
            stats.file_count += 1;
            stats.total_bytes += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            write_text_from_file(&path, output_file)?;
        }
    }
    Ok(())
}

// ── Commands ───────────────────────────────────────────────────────────────────

pub fn cmd_run(dirs: &[&Path], use_clipboard: bool) {
    if !Path::new("Dumbo.toml").exists() {
        eprintln!("Error: no Dumbo.toml found in current directory.");
        eprintln!("Run `dumbo init` to set up your project, then try again.");
        process::exit(1);
    }

    // Pour chaque dir, dérive le nom de section et charge le lang.
    // file_name() retourne None pour "." → on mappe sur "root".
    let projects: Vec<(&Path, String, String)> = dirs.iter().map(|dir| {
        let section = dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string();
        let lang_str = match read_section_lang(&section) {
            Some(l) => l,
            None => {
                eprintln!("Error: no [{}] section in Dumbo.toml.", section);
                eprintln!("Run `dumbo init` or `dumbo update` to add it.");
                process::exit(1);
            }
        };
        (*dir, section, lang_str)
    }).collect();

    // Nom du fichier de sortie : dumbo_frontend_backend.md, ou dumbo_output.md pour root
    let output_name = {
        let names: Vec<&str> = projects.iter().map(|(_, section, _)| {
            if section == "root" { "output" } else { section.as_str() }
        }).collect();
        format!("dumbo_{}.md", names.join("_"))
    };
    let output_file = Path::new(&output_name);

    let mut file = match fs::File::create(output_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: cannot create output file '{}': {}", output_name, e);
            process::exit(1);
        }
    };

    let mut total_stats = Stats { file_count: 0, total_bytes: 0 };

    for (dir, section, lang_str) in &projects {
        let config = Config::from_ext_str(lang_str);
        let extra_ignored = load_dumboignore(dir);

        println!("Running [{}] on '{}' ({})", section, dir.to_str().unwrap_or("."), lang_str);

        let tree_header = format!(
            "# [{}] — {}\n\n```\n{}/\n{}```\n\n# File Contents\n\n",
            section,
            dir.to_str().unwrap_or("."),
            dir.to_str().unwrap_or("."),
            generate_tree(dir, "", &config, &extra_ignored)
        );
        file.write_all(tree_header.as_bytes()).expect("Failed to write tree");

        let mut stats = Stats { file_count: 0, total_bytes: 0 };
        if let Err(e) = process_directory(dir, output_file, &config, &extra_ignored, &mut stats) {
            eprintln!("Error during processing: {}", e);
            process::exit(1);
        }
        total_stats.file_count += stats.file_count;
        total_stats.total_bytes += stats.total_bytes;
    }

    println!("✅ Ingestion done → {}", output_name);
    println!("   {} files ingested ({} source)", total_stats.file_count, format_size(total_stats.total_bytes));

    if use_clipboard {
        copy_output_to_clipboard(output_file);
    }
}

fn copy_output_to_clipboard(output_file: &Path) {
    let content = fs::read_to_string(output_file).unwrap_or_default();
    let size = content.len() as u64;
    if size > CLIPBOARD_WARN_BYTES {
        eprintln!("   Warning: output is {} — clipboard may be slow.", format_size(size));
    }
    match copy_to_clipboard(&content) {
        Ok(_) => println!("   Copied to clipboard 📋"),
        Err(e) => eprintln!("   Warning: clipboard failed: {}", e),
    }
}
