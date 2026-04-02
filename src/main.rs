use std::fs::{self, OpenOptions};
use std::env;
use std::io::{self, Write};
use std::path::Path;

fn generate_tree(dir: &Path, prefix: &str, ext_info: &Extensions, extra_ignored: &[String]) -> String {
    let mut tree = String::new();
    
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    entries.sort_by_key(|e| ( !e.path().is_dir(), e.file_name() ));

    let count = entries.len();
    for (i, entry) in entries.into_iter().enumerate() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };

        if path.is_dir() {
            if ext_info.ignored_dirs().contains(&name) || extra_ignored.contains(&name.to_string()) {
                continue;
            }
            tree.push_str(&format!("{}{}{}/\n", prefix, connector, name));
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            tree.push_str(&generate_tree(&path, &new_prefix, ext_info, extra_ignored));
        } else {
            tree.push_str(&format!("{}{}{}\n", prefix, connector, name));
        }
    }
    tree
}

fn write_text_from_file(input_file: &Path, output_file: &Path) -> std::io::Result<()> {
    let mut opened_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_file)?;
    let content = fs::read(input_file)?;
    let filename = input_file.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let header = format!("--- File : {} ---\n", filename);
    opened_file.write_all(header.as_bytes())?;
    opened_file.write_all(&content)?;
    opened_file.write_all(b"\n\n")?;
    Ok(())
}

fn process_directory(dir: &Path, output_file: &Path, ext_info: &Extensions, extra_ignored: &[String]) -> io::Result<()> {
    assert!(dir.is_dir(), "{} is not a directory", dir.to_str().unwrap());
    
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            
            // Si le dossier fait partie de la liste à ignorer, on le saute !
            if ext_info.ignored_dirs().contains(&dir_name) || extra_ignored.contains(&dir_name.to_string()) {
                continue; 
            }
            
            process_directory(&path, output_file, ext_info, extra_ignored)?;
        } else {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            let name_matches = Extensions::universal_files().contains(&filename);

            let ext_matches = ext_info.extensions().contains(&extension) 
                              || Extensions::universal_extensions().contains(&extension);

            if name_matches || ext_matches {
                write_text_from_file(&path, output_file)?;
            }
        }
    }
    Ok(())
}

enum Extensions {
    Rust,
    Python,
    Javascript,
    Typescript,
}

impl Extensions {
    fn from_str(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Extensions::Rust),
            "py" => Some(Extensions::Python),
            "js" | "jsx" => Some(Extensions::Javascript),
            "ts" | "tsx" => Some(Extensions::Typescript),
            _ => None,
        }
    }

    fn universal_files() -> &'static [&'static str] {
        &["Dockerfile", "docker-compose.yml", "docker-compose.yaml", "Makefile"]
    }

    fn universal_extensions() -> &'static [&'static str] {
        &["yaml", "yml", "md"]
    }

    fn ignored_dirs(&self) -> Vec<&'static str> {
        match self {
            Extensions::Rust => vec!["target", ".git"],
            Extensions::Python => vec!["__pycache__", "venv", ".venv", ".git"],
            Extensions::Javascript | Extensions::Typescript => vec!["node_modules", "dist", ".git"],
        }
    }

    fn extensions(&self) -> &[&'static str] {
        match self {
            Extensions::Rust => &["rs"],
            Extensions::Python => &["py"],
            Extensions::Javascript => &["js", "jsx"],
            Extensions::Typescript => &["ts", "tsx"],
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        panic!("Usage: cargo run <ext> <input_dir> <output_file>");
    }
    let ext_str = &args[1];
    let ext_info = Extensions::from_str(ext_str).expect("Extension is not yet supported.");

    let dir = Path::new(&args[2]);
    let output_file = Path::new(&args[3]);

    let extra_ignored: Vec<String> = if args.len() > 4 {
        args[4..].to_vec()
    } else {
        vec![]
    };

    let mut file = fs::File::create(output_file).expect("Error : impossible to create output file !");

    let tree_header = format!("PROJECT TREE\n============\n{}/\n{}", dir.to_str().unwrap(), generate_tree(dir, "", &ext_info, &extra_ignored));
    file.write_all(tree_header.as_bytes()).expect("Failed to write tree");
    file.write_all(b"\n\nFILE CONTENTS\n=============\n").expect("Failed to write separator");
    
    println!("Search for all {} files inside {} directory", ext_str, dir.to_str().unwrap());
    let result = process_directory(dir, output_file, &ext_info, &extra_ignored);

    if let Err(e) = result {
        panic!("An error occured during files processing : {}", e);
    }
    println!("Ingestion is done ✅");
}
