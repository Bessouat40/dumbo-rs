use std::path::Path;
use std::process;
use crate::lang::{LANGUAGES, find_lang};

pub struct Config {
    pub extensions: Vec<&'static str>,
    pub ignored_dirs: Vec<&'static str>,
}

impl Config {
    pub fn from_ext_str(ext_str: &str) -> Self {
        let mut extensions: Vec<&'static str> = Vec::new();
        let mut ignored_dirs: Vec<&'static str> = Vec::new();

        for part in ext_str.split(',') {
            let lang = match find_lang(part.trim()) {
                Some(l) => l,
                None => {
                    eprintln!("Error: unsupported language '{}'.", part.trim());
                    eprintln!("Supported: {}", LANGUAGES.iter().map(|l| l.name).collect::<Vec<_>>().join(", "));
                    process::exit(1);
                }
            };
            for e in lang.extensions { if !extensions.contains(e) { extensions.push(e); } }
            for d in lang.ignored_dirs { if !ignored_dirs.contains(d) { ignored_dirs.push(d); } }
        }

        Config { extensions, ignored_dirs }
    }

    pub fn universal_files() -> &'static [&'static str] {
        &["Dockerfile", "docker-compose.yml", "docker-compose.yaml", "Makefile"]
    }

    pub fn universal_extensions() -> &'static [&'static str] {
        &["yaml", "yml", "md", "toml"]
    }
}

pub fn is_included_file(path: &Path, config: &Config) -> bool {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    Config::universal_files().contains(&filename)
        || config.extensions.contains(&extension)
        || Config::universal_extensions().contains(&extension)
}

pub fn lang_hint(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs"                          => "rust",
        "py"                          => "python",
        "js" | "jsx"                  => "javascript",
        "ts" | "tsx"                  => "typescript",
        "go"                          => "go",
        "java"                        => "java",
        "c" | "h"                     => "c",
        "cpp" | "cc" | "cxx" | "hpp"  => "cpp",
        "toml"                        => "toml",
        "yaml" | "yml"                => "yaml",
        "md"                          => "markdown",
        _                             => "",
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
