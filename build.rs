#[cfg(windows)]
fn main() {
    let mut res = winresource::WindowsResource::new();
    res.set("ProductName", "UniRust");
    res.set("FileDescription", "UniRust - Pure-Rust Vietnamese IME (Telex)");
    res.set("CompanyName", "Phạm Ngọc Trí (NPTBuy)");
    res.set("LegalCopyright", "Copyright (c) 2026 Phạm Ngọc Trí (NPTBuy). Licensed under GNU GPL v3.0.");
    res.set("OriginalFilename", "unirust.exe");
    res.set("InternalName", "UniRust");
    let version = env!("CARGO_PKG_VERSION");
    res.set("ProductVersion", version);
    res.set("FileVersion", version);

    // If rc.exe is not found in the default Windows SDK location, locate it via PATH
    if std::env::var_os("RC_PATH").is_none() {
        if let Ok(path) = std::env::var("PATH") {
            for p in std::env::split_paths(&path) {
                let rc = p.join("rc.exe");
                if rc.exists() {
                    res.set_toolkit_path(p.to_str().unwrap());
                    break;
                }
            }
        }
    }

    if let Err(e) = res.compile() {
        eprintln!("cargo:warning=Failed to compile Windows resource: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
