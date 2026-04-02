use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use crate::config::{lang_hint, format_size};
use crate::clipboard::{copy_to_clipboard, CLIPBOARD_WARN_BYTES};

// ── Git helpers ────────────────────────────────────────────────────────────────

fn check_commit_exists(commit: &str) {
    let ok = Command::new("git")
        .args(["cat-file", "-e", &format!("{}^{{commit}}", commit)])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        eprintln!("Error: '{}' is not a valid commit reference.", commit);
        process::exit(1);
    }
}

// commit = None → --staged
fn git_diff_stat(commit: Option<&str>, dir: &Path) -> String {
    let mut args = vec!["diff", "--stat"];
    if let Some(c) = commit { args.push(c); } else { args.push("--staged"); }
    args.extend_from_slice(&["--", dir.to_str().unwrap_or(".")]);

    let output = Command::new("git").args(&args).output();
    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            if text.trim().is_empty() { String::new() } else { text }
        }
        Err(e) => { eprintln!("Error: could not run git: {}", e); process::exit(1); }
    }
}

fn git_diff(commit: Option<&str>, dir: &Path) -> String {
    let mut args = vec!["diff"];
    if let Some(c) = commit { args.push(c); } else { args.push("--staged"); }
    args.extend_from_slice(&["--", dir.to_str().unwrap_or(".")]);

    let output = Command::new("git").args(&args).output();
    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            if text.trim().is_empty() {
                match commit {
                    Some(c) => format!("No differences found between {} and current state.\n", c),
                    None    => "No staged changes.\n".to_string(),
                }
            } else {
                text
            }
        }
        Err(e) => { eprintln!("Error: could not run git: {}", e); process::exit(1); }
    }
}

fn git_changed_files(commit: Option<&str>, dir: &Path) -> Vec<PathBuf> {
    let mut args = vec!["diff", "--name-only"];
    if let Some(c) = commit { args.push(c); } else { args.push("--staged"); }
    args.extend_from_slice(&["--", dir.to_str().unwrap_or(".")]);

    let output = Command::new("git").args(&args).output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| PathBuf::from(l.trim()))
            .filter(|p| p.is_file())
            .collect(),
        Err(e) => { eprintln!("Error: could not run git: {}", e); process::exit(1); }
    }
}

// ── Format ─────────────────────────────────────────────────────────────────────

fn with_line_numbers(content: &str) -> String {
    content.lines()
        .enumerate()
        .map(|(i, line)| format!("{:4} | {}", i + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Command ────────────────────────────────────────────────────────────────────

// commit = None → mode --staged
pub fn cmd_diff(commit: Option<&str>, dirs: &[&Path], use_clipboard: bool) {
    if !Path::new("Dumbo.toml").exists() {
        eprintln!("Error: no Dumbo.toml found. Run `dumbo init` first.");
        process::exit(1);
    }

    if let Some(c) = commit {
        check_commit_exists(c);
    }

    let label = commit.unwrap_or("staged");
    let short = &label[..label.len().min(8)];
    let output_name = format!("dumbo_diff_{}.md", short);
    let output_file = Path::new(&output_name);

    let mut file = match fs::File::create(output_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: cannot create output file '{}': {}", output_name, e);
            process::exit(1);
        }
    };

    // ── En-tête ───────────────────────────────────────────────────────────────

    let title = match commit {
        Some(c) => format!("# Diff since `{}`\n\n", c),
        None    => "# Staged changes\n\n".to_string(),
    };
    file.write_all(title.as_bytes()).expect("Failed to write");

    // ── Stat ──────────────────────────────────────────────────────────────────

    file.write_all(b"## Summary\n\n```\n").expect("Failed to write");
    for dir in dirs {
        let stat = git_diff_stat(commit, dir);
        if !stat.is_empty() {
            file.write_all(stat.as_bytes()).expect("Failed to write stat");
        }
    }
    file.write_all(b"```\n\n").expect("Failed to write");

    // ── Diff complet ──────────────────────────────────────────────────────────

    file.write_all(b"## Changes\n\n```diff\n").expect("Failed to write");
    for dir in dirs {
        file.write_all(git_diff(commit, dir).as_bytes()).expect("Failed to write diff");
    }
    file.write_all(b"```\n\n").expect("Failed to write");

    // ── Contenu actuel des fichiers modifiés (avec numéros de ligne) ──────────

    file.write_all(b"## Current state of changed files\n\n").expect("Failed to write");

    let mut total_bytes: u64 = 0;
    let mut file_count = 0;

    for dir in dirs {
        let changed = git_changed_files(commit, dir);

        if changed.is_empty() {
            file.write_all(b"_No changed files._\n\n").expect("Failed to write");
            continue;
        }

        for path in &changed {
            let content = fs::read_to_string(path).unwrap_or_default();
            let lang = lang_hint(path);
            let numbered = with_line_numbers(content.trim_end());
            let block = format!(
                "### `{}`\n\n```{}\n{}\n```\n\n",
                path.to_str().unwrap_or(""),
                lang,
                numbered
            );
            file.write_all(block.as_bytes()).expect("Failed to write file content");
            total_bytes += fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            file_count += 1;
        }
    }

    println!("✅ Diff done → {}", output_name);
    println!("   {} changed file(s) included ({})", file_count, format_size(total_bytes));

    if use_clipboard {
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
}
