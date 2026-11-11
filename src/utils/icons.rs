use crate::utils::paths::{load_asset_bytes_or, resolver_caminho_asset};
use std::path::PathBuf;
use std::sync::OnceLock;

static ICON_DATA_CACHE: OnceLock<Option<egui::IconData>> = OnceLock::new();

pub fn obter_caminho_icone(nome: &str) -> Option<PathBuf> {
    resolver_caminho_asset(&format!("icons/{}", nome))
}

pub fn carregar_icone_janela() -> Option<egui::IconData> {
    ICON_DATA_CACHE
        .get_or_init(|| {
            let embedded = include_bytes!("../assets/icons/geoestabil.ico");
            let bytes = load_asset_bytes_or("icons/geoestabil.ico", embedded);
            let img = image::load_from_memory(bytes.as_ref())
                .or_else(|_| image::load_from_memory(embedded))
                .ok()?;
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            })
        })
        .clone()
}
