use std::path::PathBuf;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dir_parts = out_dir.split(std::path::MAIN_SEPARATOR_STR).collect::<Vec<_>>();
    let out_dir = PathBuf::from(dir_parts[..dir_parts.len() - 3].join(std::path::MAIN_SEPARATOR_STR));
    let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    sqcrab_builder::domain::build_domain_initalization(root_dir.as_path()).unwrap();
    std::fs::copy(root_dir.join("scripts/unit.nut"), out_dir.join("unit.nut")).unwrap();
}