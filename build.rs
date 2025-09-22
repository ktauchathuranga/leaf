fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Static linking for musl targets
    let target = std::env::var("TARGET").unwrap();
    if target.contains("musl") {
        println!("cargo:rustc-link-arg=-static");
        println!("cargo:rustc-env=RUSTFLAGS=-C target-feature=+crt-static");
    }
}

