fn main() {
    // Tell cargo to link against libduckdb
    // The library path can be configured via DUCKDB_LIB_DIR environment variable
    if let Ok(lib_dir) = std::env::var("DUCKDB_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }

    println!("cargo:rustc-link-lib=duckdb");

    // Re-run build script if environment variables change
    println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
}
