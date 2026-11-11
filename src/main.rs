#![windows_subsystem = "windows"]

use eframe::NativeOptions;
use geoestabil_rs::ui::GeoEstabilApp;
use geoestabil_rs::utils::icons::carregar_icone_janela;
use geoestabil_rs::utils::paths::resolve_startup_project_path;

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        // Desvincula o console caso o executável seja acionado por terminal ou duplo clique
        windows_sys::Win32::System::Console::FreeConsole();

        // Vincula o AppUserModelID explícito para a barra de tarefas do Windows 11
        let app_id: Vec<u16> = "fernandoncidade.GeoEstabil\0"
            .encode_utf16()
            .collect();
        windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr());
    }

    let mut native_options = NativeOptions::default();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("GeoEstabil — Estabilidade de Taludes e Muros de Arrimo (NBR 11682 / NBR 6122 / NBR 6118)")
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([980.0, 620.0])
        .with_active(true);

    if let Some(icon) = carregar_icone_janela() {
        viewport = viewport.with_icon(icon);
    }

    #[cfg(target_os = "windows")]
    {
        unsafe {
            let cx = windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows_sys::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
            );
            let cy = windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows_sys::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
            );
            if cx > 0 && cy > 0 {
                let x = ((cx as f32 - 1280.0) * 0.5).max(0.0);
                let y = ((cy as f32 - 820.0) * 0.5).max(0.0);
                viewport = viewport.with_position([x, y]);
            }
        }
    }

    native_options.viewport = viewport;

    let startup_path = resolve_startup_project_path(std::env::args_os());

    eframe::run_native(
        "GeoEstabil",
        native_options,
        Box::new(move |_cc| Ok(Box::new(GeoEstabilApp::with_startup_file(startup_path.as_deref())))),
    )
}
