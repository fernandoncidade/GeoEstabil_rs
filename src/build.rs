fn main() {
    println!("cargo:rerun-if-changed=src/assets/icons/geoestabil.ico");
    println!("cargo:rerun-if-changed=src/build.rs");

    #[cfg(windows)]
    {
        if std::path::Path::new("src/assets/icons/geoestabil.ico").exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("src/assets/icons/geoestabil.ico");
            res.set("FileDescription", "GeoEstabil — Estabilidade de Taludes e Muros de Arrimo (NBR 11682 / NBR 6122 / NBR 6118)");
            res.set("ProductName", "GeoEstabil");
            res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
            res.set("CompanyName", "Fernando Nillsson Cidade");
            res.set("LegalCopyright", "Copyright © 2026 Fernando Nillsson Cidade");
            let _ = res.compile();
        }
    }
}
