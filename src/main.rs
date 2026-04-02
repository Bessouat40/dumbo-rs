mod clipboard;
mod config;
mod diff;
mod init;
mod lang;
mod run;

use std::env;
use std::path::Path;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!("dumbo v{}", VERSION);
    println!();
    println!("COMMANDS:");
    println!("  init [dir]                 Detect languages, create Dumbo.toml + .dumboignore");
    println!("  init --recursive|-r [dir]  Init all sub-projects in a monorepo");
    println!("  update [dir]               Add new sub-projects to an existing Dumbo.toml");
    println!("  list [dir]                 List all sections in Dumbo.toml");
    println!("  diff <commit> [dir...]     Diff since a commit + current state of changed files");
    println!("  diff --staged [dir...]     Same but for staged changes");
    println!("  run [dir...]               Generate context file(s) from Dumbo.toml");
    println!();
    println!("OPTIONS:");
    println!("  -c, --clipboard    Copy output to clipboard");
    println!("  -h, --help         Print this help message");
    println!("  -v, --version      Print version");
    println!();
    println!("EXAMPLES:");
    println!("  dumbo init                     # single project");
    println!("  dumbo init -r                  # monorepo");
    println!("  dumbo update                   # add a new service");
    println!("  dumbo run                      # → dumbo_output.md");
    println!("  dumbo run frontend             # → dumbo_frontend.md");
    println!("  dumbo run frontend backend -c  # → dumbo_frontend_backend.md + clipboard");
    println!("  dumbo diff abc1234             # → dumbo_diff_abc1234.md");
    println!("  dumbo diff main~3 backend      # → diff du backend depuis 3 commits");
}

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

    // dumbo diff <commit>|--staged [dir...] [-c]
    if args.get(1).map(|s| s.as_str()) == Some("diff") {
        let use_clipboard = args.iter().any(|a| a == "--clipboard" || a == "-c");
        let staged = args.iter().any(|a| a == "--staged");
        let (commit, dir_skip) = if staged {
            (None, 2)
        } else {
            match args.get(2) {
                Some(c) if !c.starts_with('-') => (Some(c.as_str()), 3),
                _ => {
                    eprintln!("Error: missing commit reference.");
                    eprintln!("Usage: dumbo diff <commit> [dir...]  or  dumbo diff --staged [dir...]");
                    process::exit(1);
                }
            }
        };
        let dir_args: Vec<&str> = args.iter().skip(dir_skip)
            .filter(|a| *a != "--clipboard" && *a != "-c" && *a != "--staged")
            .map(|s| s.trim_end_matches('/'))
            .collect();
        let dir_args = if dir_args.is_empty() { vec!["."] } else { dir_args };
        let dirs: Vec<&Path> = dir_args.iter().map(|s| {
            let p = Path::new(s);
            if !p.is_dir() {
                eprintln!("Error: '{}' is not a directory.", s);
                process::exit(1);
            }
            p
        }).collect();
        diff::cmd_diff(commit, &dirs, use_clipboard);
        return;
    }

    // dumbo list [dir]
    if args.get(1).map(|s| s.as_str()) == Some("list") {
        let dir_arg = args.get(2).map(|s| s.as_str()).unwrap_or(".");
        let dir = Path::new(dir_arg);
        if !dir.is_dir() {
            eprintln!("Error: '{}' is not a directory.", dir_arg);
            process::exit(1);
        }
        init::cmd_list(dir);
        return;
    }

    // dumbo update [dir]
    if args.get(1).map(|s| s.as_str()) == Some("update") {
        let dir_arg = args.get(2).map(|s| s.as_str()).unwrap_or(".");
        let dir = Path::new(dir_arg);
        if !dir.is_dir() {
            eprintln!("Error: '{}' is not a directory.", dir_arg);
            process::exit(1);
        }
        init::cmd_update(dir);
        return;
    }

    // dumbo init [--recursive|-r] [dir]
    if args.get(1).map(|s| s.as_str()) == Some("init") {
        let recursive = args.iter().any(|a| a == "--recursive" || a == "-r");
        let dir_arg = args.iter().skip(2).find(|a| *a != "--recursive" && *a != "-r");
        let dir = dir_arg.map(|s| Path::new(s.as_str())).unwrap_or(Path::new("."));
        if !dir.is_dir() {
            eprintln!("Error: '{}' is not a directory.", dir.to_str().unwrap_or("?"));
            process::exit(1);
        }
        init::cmd_init(dir, recursive);
        return;
    }

    // dumbo run [dir...] [-c]
    if args.get(1).map(|s| s.as_str()) == Some("run") {
        let use_clipboard = args.iter().any(|a| a == "--clipboard" || a == "-c");
        let dir_args: Vec<&str> = args.iter().skip(2)
            .filter(|a| *a != "--clipboard" && *a != "-c")
            .map(|s| s.trim_end_matches('/'))
            .collect();
        let dir_args = if dir_args.is_empty() { vec!["."] } else { dir_args };
        let dirs: Vec<&Path> = dir_args.iter().map(|s| {
            let p = Path::new(s);
            if !p.is_dir() {
                eprintln!("Error: '{}' is not a directory.", s);
                process::exit(1);
            }
            p
        }).collect();
        run::cmd_run(&dirs, use_clipboard);
        return;
    }

    eprintln!("Error: unknown command '{}'.", args.get(1).map(|s| s.as_str()).unwrap_or(""));
    eprintln!("Run `dumbo --help` for usage.");
    process::exit(1);
}
