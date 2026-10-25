//! Builds the tax packs into the binary, so a downloaded `investor-mcp`
//! knows them without the repository next to it.

use std::fmt::Write;
use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../jurisdictions");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut packs = Vec::new();
    for country in std::fs::read_dir(&root).expect("jurisdictions/ exists").flatten() {
        println!("cargo:rerun-if-changed={}", country.path().display());
        for file in std::fs::read_dir(country.path()).into_iter().flatten().flatten() {
            let path = file.path();
            if path.extension().is_some_and(|e| e == "toml") {
                let code = country.file_name().to_string_lossy().to_uppercase();
                let year = path.file_stem().unwrap().to_string_lossy().into_owned();
                packs.push((code, year, path.canonicalize().unwrap()));
            }
        }
    }
    packs.sort();
    let mut out = String::from("/// Tax packs as (country, year, TOML), built in from jurisdictions/.\npub const PACKS: &[(&str, i32, &str)] = &[\n");
    for (code, year, path) in packs {
        writeln!(out, "    ({code:?}, {year}, include_str!({:?})),", path.display().to_string()).unwrap();
    }
    out.push_str("];\n");
    std::fs::write(Path::new(&std::env::var("OUT_DIR").unwrap()).join("packs.rs"), out).unwrap();
}
