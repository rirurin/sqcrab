fn main() {
    cc::Build::new()
        .cpp(true)
        .files([
            "squirrel/squirrel/sqapi.cpp",
            "squirrel/squirrel/sqbaselib.cpp",
            "squirrel/squirrel/sqfuncstate.cpp",
            "squirrel/squirrel/sqdebug.cpp",
            "squirrel/squirrel/sqlexer.cpp",
            "squirrel/squirrel/sqobject.cpp",
            "squirrel/squirrel/sqcompiler.cpp",
            "squirrel/squirrel/sqstate.cpp",
            "squirrel/squirrel/sqtable.cpp",
            "squirrel/squirrel/sqmem.cpp",
            "squirrel/squirrel/sqvm.cpp",
            "squirrel/squirrel/sqclass.cpp",
        ])
        .includes([
            "squirrel/include",
            "squirrel/squirrel"
        ])
        .define("SQ64", None)
        .compile("squirrel");
    let bindings = bindgen::builder().header("squirrel/include/squirrel.h")
        .enable_cxx_namespaces()
        .generate().unwrap();
    bindings.write_to_file("src/bindings.rs").unwrap();
}

