// Link macOS Application Services; compile Cocoa handler for open document (drop-on-icon).
fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("apple-darwin") {
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=Foundation");
        cc::Build::new()
            .file("macos/open_document_handler.m")
            .compile("open_document_handler");
    }
}
