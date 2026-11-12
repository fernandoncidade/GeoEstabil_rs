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

    // Seleção robusta do backend de renderização gráfico:
    // O backend Glow (OpenGL 3.3 / WGL) consome uma fração mínima da VRAM (~25-40 MB contra >600 MB a 1.5 GB do DX12 no wgpu)
    // e impede deadlocks/TDR de swapchains do DWM quando outros aplicativos com aceleração gráfica são abertos.
    // Pode ser sobrescrito definindo a variável de ambiente `GEOESTABIL_RENDERER=wgpu`.
    let force_wgpu = std::env::var("GEOESTABIL_RENDERER")
        .map(|v| v.eq_ignore_ascii_case("wgpu"))
        .unwrap_or(false);

    if force_wgpu {
        // Se wgpu for explicitamente solicitado, otimiza para baixo consumo e evita saturação do adaptador dedicado
        if std::env::var("WGPU_POWER_PREF").is_err() {
            std::env::set_var("WGPU_POWER_PREF", "low");
        }
        native_options.renderer = eframe::Renderer::Wgpu;
    } else {
        native_options.renderer = eframe::Renderer::Glow;
    }

    // Economia de VRAM: desativa buffers 3D (depth/stencil) e MSAA desnecessários para interface 2D vetorial
    native_options.depth_buffer = 0;
    native_options.stencil_buffer = 0;
    native_options.multisampling = 0;
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
