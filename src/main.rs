use std::fs::{self, OpenOptions};
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;

// ── Language config ────────────────────────────────────────────────────────────

struct Config {
    extensions: Vec<&'static str>,
    ignored_dirs: Vec<&'static str>,
}

impl Config {
    fn from_ext_str(ext_str: &str) -> Option<Self> {
        let mut extensions: Vec<&'static str> = Vec::new();
        let mut ignored_dirs: Vec<&'static str> = Vec::new();

        for part in ext_str.split(',') {
            let (exts, dirs) = match part.trim() {
                "rs"  => (&["rs"][..],              &["target", ".git"][..]),
                "py"  => (&["py"][..],              &["__pycache__", "venv", ".venv", ".git"][..]),
                "js"  => (&["js", "jsx"][..],       &["node_modules", "dist", ".git"][..]),
                "ts"  => (&["ts", "tsx"][..],       &["node_modules", "dist", ".git"][..]),
                other => {
                    eprintln!("Error: unsupported language '{}'.", other);
                    eprintln!("Supported: rs, py, js, ts");
                    process::exit(1);
                }
            };
            for e in exts { if !extensions.contains(e) { extensions.push(e); } }
            for d in dirs { if !ignored_dirs.contains(d) { ignored_dirs.push(d); } }
        }

        Some(Config { extensions, ignored_dirs })
    }

    fn universal_files() -> &'static [&'static str] {
        &["Dockerfile", "docker-compose.yml", "docker-compose.yaml", "Makefile"]
    }

    fn universal_extensions() -> &'static [&'static str] {
        &["yaml", "yml", "md", "toml"]
    }
}

// ── File helpers ───────────────────────────────────────────────────────────────

fn is_included_file(path: &Path, config: &Config) -> bool {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    Config::universal_files().contains(&filename)
        || config.extensions.contains(&extension)
        || Config::universal_extensions().contains(&extension)
}

fn lang_hint(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs"           => "rust",
        "py"           => "python",
        "js" | "jsx"   => "javascript",
        "ts" | "tsx"   => "typescript",
        "toml"         => "toml",
        "yaml" | "yml" => "yaml",
        "md"           => "markdown",
        _              => "",
    }
}

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

// ── .dumboignore ───────────────────────────────────────────────────────────────

fn load_dumboignore(dir: &Path) -> Vec<String> {
    let path = dir.join(".dumboignore");
    let Ok(content) = fs::read_to_string(&path) else { return vec![] };
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

// ── Tree ───────────────────────────────────────────────────────────────────────

fn generate_tree(dir: &Path, prefix: &str, config: &Config, extra_ignored: &[String]) -> String {
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

struct Stats {
    file_count: usize,
    total_bytes: u64,
}

fn process_directory(
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
        } else if is_included_file(&path, config) && !extra_ignored.contains(&path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string()) {
            stats.file_count += 1;
            stats.total_bytes += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            write_text_from_file(&path, output_file)?;
        }
    }
    Ok(())
}

// ── Clipboard ─────────────────────────────────────────────────────────────────

const CLIPBOARD_WARN_BYTES: u64 = 500 * 1024; // 500 KB

fn copy_to_clipboard(content: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let cmd = "pbcopy";
    #[cfg(target_os = "linux")]
    let cmd = "xclip -selection clipboard";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err("Clipboard not supported on this platform.".to_string());

    use std::process::{Command, Stdio};
    let mut child = Command::new("sh")
        .args(["-c", cmd])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch clipboard command: {}", e))?;
    if let Some(mut stdin) = child.stdin.take() {
        io::Write::write_all(&mut stdin, content.as_bytes())
            .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
    }
    child.wait().map_err(|e| format!("Clipboard command failed: {}", e))?;
    Ok(())
}

// ── Misc helpers ───────────────────────────────────────────────────────────────

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!("dumbo-rs v{}", VERSION);
    println!();
    println!("USAGE:");
    println!("  dumbo-rs <ext> <input_dir> <output_file> [extra_ignore_dirs...]");
    println!();
    println!("ARGS:");
    println!("  <ext>              Language(s): rs, py, js, ts — comma-separated for multiple");
    println!("  <input_dir>        Path to the project directory");
    println!("  <output_file>      Output file path (e.g. context.md)");
    println!("  [extra_ignore_dirs]  Additional directories to ignore (optional)");
    println!();
    println!("OPTIONS:");
    println!("  -c, --clipboard    Also copy the output to the clipboard");
    println!("  -h, --help         Print this help message");
    println!("  -v, --version      Print version");
    println!();
    println!("  A .dumboignore file at the root of <input_dir> can list directories");
    println!("  to ignore (one per line, # for comments).");
    println!();
    println!("EXAMPLES:");
    println!("  dumbo-rs rs ./my_project context.md");
    println!("  dumbo-rs rs,ts ./fullstack context.md -c");
    println!("  dumbo-rs ts ./web_app output.md tests_data logs");
}

// ── Main ───────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("{}", VERSION);
        return;
    }
    if args.len() < 4 {
        eprintln!("Error: missing arguments.");
        eprintln!("Run `dumbo-rs --help` for usage.");
        process::exit(1);
    }

    let ext_str = &args[1];
    let config = Config::from_ext_str(ext_str).unwrap();

    let dir = Path::new(&args[2]);
    if !dir.is_dir() {
        eprintln!("Error: '{}' is not a directory.", args[2]);
        process::exit(1);
    }

    let use_clipboard = args.iter().any(|a| a == "--clipboard" || a == "-c");
    let output_file = Path::new(&args[3]);

    // Merge: CLI extra dirs + .dumboignore
    let mut extra_ignored: Vec<String> = args[4..]
        .iter()
        .filter(|a| *a != "--clipboard" && *a != "-c")
        .cloned()
        .collect();
    let dumboignore = load_dumboignore(dir);
    if !dumboignore.is_empty() {
        println!("   .dumboignore: ignoring {:?}", dumboignore);
        for d in dumboignore { if !extra_ignored.contains(&d) { extra_ignored.push(d); } }
    }

    let mut file = match fs::File::create(output_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: cannot create output file '{}': {}", args[3], e);
            process::exit(1);
        }
    };

    let tree_header = format!(
        "# Project Tree\n\n```\n{}/\n{}```\n\n# File Contents\n\n",
        dir.to_str().unwrap(),
        generate_tree(dir, "", &config, &extra_ignored)
    );
    file.write_all(tree_header.as_bytes()).expect("Failed to write tree");

    let mut stats = Stats { file_count: 0, total_bytes: 0 };
    if let Err(e) = process_directory(dir, output_file, &config, &extra_ignored, &mut stats) {
        eprintln!("Error during processing: {}", e);
        process::exit(1);
    }

    println!("✅ Ingestion done");
    println!("   {} files ingested ({} source)", stats.file_count, format_size(stats.total_bytes));

    if use_clipboard {
        let content = fs::read_to_string(output_file).unwrap_or_default();
        let size = content.len() as u64;
        if size > CLIPBOARD_WARN_BYTES {
            eprintln!(
                "   Warning: output is {} — clipboard may be slow or truncated by your OS.",
                format_size(size)
            );
        }
        match copy_to_clipboard(&content) {
            Ok(_) => println!("   Copied to clipboard 📋"),
            Err(e) => eprintln!("   Warning: clipboard failed: {}", e),
        }
    }
}
