fn main() {
    println!("cargo::rustc-check-cfg=cfg(nightly)");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let output = std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .expect("failed to run rustc");
    let version = String::from_utf8(output.stdout).unwrap();
    if version.contains("nightly") {
        println!("cargo:rustc-cfg=nightly");
    }
}
