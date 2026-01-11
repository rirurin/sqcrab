use std::error::Error;

fn main() {
    build_cpp_wrapper().unwrap();
}

fn build_cpp_wrapper() -> Result<(), Box<dyn Error>> {
    let path = std::env::current_dir()?.join("cpp/format_print.cpp");
    cc::Build::new()
        .cpp(true)
        .file(path)
        .compile("squirrel_print_format");
    Ok(())
}