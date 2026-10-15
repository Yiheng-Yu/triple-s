fn main() {
    // Let Rust test binaries locate the interpreter selected by PyO3.
    #[cfg(feature = "pyo3")]
    pyo3_build_config::add_libpython_rpath_link_args();
}
