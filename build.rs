use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=PYO3_PYTHON");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let python = std::env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_owned());
    let output = Command::new(&python)
        .args(["-c", "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')"])
        .output()
        .unwrap_or_else(|error| panic!("failed to query Python library directory: {error}"));

    if !output.status.success() {
        panic!("Python failed while reporting its library directory");
    }

    let libdir = String::from_utf8(output.stdout)
        .expect("Python library directory was not valid UTF-8")
        .trim()
        .to_owned();

    if !libdir.is_empty() {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{libdir}");
    }
}
