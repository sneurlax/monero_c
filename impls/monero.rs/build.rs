use bindgen::EnumVariation;
use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let header_path = manifest_dir.join("../../monero_libwallet2_api_c/src/main/cpp/wallet2_api_c.h");

    let lib_search_paths = [
        manifest_dir.join("../../release"),
        manifest_dir.join("lib"),
        manifest_dir.join("target/debug"),
        manifest_dir.join("target/release"),
    ];

    for path in &lib_search_paths {
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        }
    }

    println!("cargo:rerun-if-changed={}", header_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-lib=dylib=monero_libwallet2_api_c");

    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .allowlist_function("MONERO_.*")
        .allowlist_var("MONERO_.*")
        .allowlist_var("NetworkType_.*")
        .allowlist_var("PendingTransactionStatus_.*")
        .allowlist_var("Priority_.*")
        .allowlist_var("UnsignedTransactionStatus_.*")
        .allowlist_var("TransactionInfoDirection_.*")
        .allowlist_var("AddressBookErrorCode.*")
        .allowlist_var("WalletDevice_.*")
        .allowlist_var("WalletStatus_.*")
        .allowlist_var("WalletConnectionStatus_.*")
        .allowlist_var("WalletBackgroundSync_.*")
        .allowlist_var("BackgroundSync_.*")
        .allowlist_var("LogLevel_.*")
        .blocklist_type("__.*")
        .blocklist_type("_.*")
        .blocklist_function("__.*")
        .layout_tests(false)
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .derive_default(false)
        .conservative_inline_namespaces()
        .generate_comments(false)
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(dead_code)]")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = manifest_dir.join("src").join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
