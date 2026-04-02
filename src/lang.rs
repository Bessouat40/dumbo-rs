pub struct LangDef {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub ignored_dirs: &'static [&'static str],
}

pub const LANGUAGES: &[LangDef] = &[
    LangDef { name: "rs",   extensions: &["rs"],                           ignored_dirs: &["target", ".git"] },
    LangDef { name: "py",   extensions: &["py"],                           ignored_dirs: &["__pycache__", "venv", ".venv", ".git"] },
    LangDef { name: "js",   extensions: &["js", "jsx"],                    ignored_dirs: &["node_modules", "dist", ".git"] },
    LangDef { name: "ts",   extensions: &["ts", "tsx"],                    ignored_dirs: &["node_modules", "dist", ".git"] },
    LangDef { name: "go",   extensions: &["go"],                           ignored_dirs: &["vendor", ".git"] },
    LangDef { name: "java", extensions: &["java"],                         ignored_dirs: &["target", "build", ".gradle", ".git"] },
    LangDef { name: "c",    extensions: &["c", "h"],                       ignored_dirs: &["build", "cmake-build-debug", "cmake-build-release", ".git"] },
    LangDef { name: "cpp",  extensions: &["cpp", "cc", "cxx", "hpp", "h"], ignored_dirs: &["build", "cmake-build-debug", "cmake-build-release", ".git"] },
];

pub fn find_lang(name: &str) -> Option<&'static LangDef> {
    LANGUAGES.iter().find(|l| l.name == name)
}
