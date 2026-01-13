fn main() {
    // Put the linker script beside the manifest
    println!("cargo:rustc-link-search=.");
}
