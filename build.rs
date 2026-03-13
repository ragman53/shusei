// Build script for PDFium dynamic linking
// Copies PDFium DLL to output directory for runtime loading

use std::fs;
use std::path::PathBuf;

fn main() {
    // Only process PDFium when the pdf feature is enabled
    if cfg!(feature = "pdf") {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let manifest_path = PathBuf::from(manifest_dir);

        // Check if pdfium.dll exists in the project root
        let pdfium_dll = manifest_path.join("pdfium.dll");

        if pdfium_dll.exists() {
            // Copy DLL to output directory for runtime loading
            let out_dir = std::env::var("OUT_DIR").unwrap();
            let target_dll = PathBuf::from(&out_dir).join("../../../../pdfium.dll");

            if let Err(e) = fs::copy(&pdfium_dll, &target_dll) {
                println!("cargo:warning=Failed to copy pdfium.dll to target: {}", e);
            } else {
                println!("cargo:warning=PDFium DLL copied to target directory");
            }
        } else {
            println!("cargo:warning=PDFium DLL not found. Download from https://github.com/bblanchon/pdfium-binaries/releases");
        }

        // Fix CRT conflict: tell linker to ignore duplicate symbols from LIBCMT vs MSVCRT
        // This is safe because we're using dynamic loading, not static linking
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:LIBCMT");
    }
}
