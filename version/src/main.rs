#![windows_subsystem = "windows"]

mod app;
mod engine;

use std::path::{Path, PathBuf};
use chrono::{Datelike, Local};
use eframe::NativeOptions;

use app::VersionEditorApp;
use engine::*;

fn carregar_icone_app(base_dir: &Path) -> Option<egui::IconData> {
    let candidate_paths = [
        base_dir.join("src/assets/icons/geoestabil_150-150.png"),
        base_dir.join("src/assets/icons/geoestabil.png"),
        base_dir.join("src/assets/icons/geoestabil_300-300.png"),
        base_dir.join("assets/icons/geoestabil_150-150.png"),
        base_dir.join("assets/icons/geoestabil.png"),
    ];

    for path in &candidate_paths {
        if path.exists() {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    return Some(egui::IconData {
                        rgba: rgba.into_raw(),
                        width: w,
                        height: h,
                    });
                }
            }
        }
    }
    None
}

fn print_help() {
    println!("version-editor - Editor de Versoes e Datas Multi-idioma (GeoEstabil)");
    println!("Uso:");
    println!("  version-editor.exe                   (abre a interface grafica egui)");
    println!("  version-editor.exe --check [dir]     (verifica status e padroes dos arquivos)");
    println!("  version-editor.exe --apply [dir] [v] (aplica versao e data atual a todos os arquivos)");
    println!("Opcoes:");
    println!("  --help, -h               Exibe esta mensagem de ajuda");
    println!("  --check, -c              Verifica arquivos e padroes de versao/data");
    println!("  --apply, --save, --update Aplica atualizacoes de versao e data em lote");
}

fn run_cli_check(base_dir: PathBuf) -> i32 {
    let today = Local::now().date_naive();
    let composed_ver = format_version_from_date(today.year(), today.month(), today.day(), "");

    println!("======================================================================");
    println!("Data Presente: {} (Dia: {}, Mes: {}, Ano: {})",
        today.format("%d/%m/%Y"), today.day(), today.month(), today.year());
    println!("Versao Atual Composta: {}", composed_ver);
    println!("Diretorio Base: {}", base_dir.display());
    println!("======================================================================");

    let statuses = check_all_files(&base_dir);
    let mut all_ok = true;

    for st in &statuses {
        if !st.exists {
            all_ok = false;
            println!("[AUSENTE]   {}", st.rel_path);
        } else if !st.missing_patterns.is_empty() {
            all_ok = false;
            println!("[PENDENTE]  {} ({} padroes ausentes)", st.rel_path, st.missing_patterns.len());
        } else {
            let ver_tag = st.detected_version.as_deref().map(|v| format!(" (v{})", plain_version(v))).unwrap_or_default();
            let date_tag = st.detected_date.as_deref().map(|d| format!(" [Data: {}]", d)).unwrap_or_default();
            let ro_tag = if st.is_updatable { "" } else { " [Somente leitura]" };
            println!("[OK]        {}{}{}{}", st.rel_path, ver_tag, date_tag, ro_tag);
        }
    }

    println!("======================================================================");
    if all_ok {
        println!("TODOS OS ARQUIVOS VERIFICADOS COM SUCESSO.");
        0
    } else {
        println!("FALHAS NA VERIFICACAO DE ARQUIVOS.");
        1
    }
}

fn run_cli_apply(base_dir: PathBuf, version_arg: Option<String>) -> i32 {
    let today = Local::now().date_naive();
    let version = version_arg.unwrap_or_else(|| {
        format_version_from_date(today.year(), today.month(), today.day(), "")
    });

    let mut day = today.day();
    let mut month_idx = today.month() - 1;
    let mut year = today.year();

    if let Some((d, m, y, _)) = parse_version_to_date(&version) {
        day = d;
        month_idx = m;
        year = y;
    }

    println!("======================================================================");
    println!("Data Presente: {} (Dia: {}, Mes: {}, Ano: {})",
        today.format("%d/%m/%Y"), today.day(), today.month(), today.year());
    println!("Versao Atual Composta Aplicada: {}", version);
    println!("Diretorio Base: {}", base_dir.display());
    println!("======================================================================");

    let pt = LangInput::new(&version, day, month_idx, year);
    let en = LangInput::new(&version, day, month_idx, year);

    let results = apply_updates(&pt, &en, &base_dir);
    let mut all_ok = true;

    for r in &results {
        if !r.ok {
            all_ok = false;
            println!("[FALHA]     {} - {}", r.rel_path, r.msg);
        } else {
            println!("[ATUALIZADO] {}", r.rel_path);
        }
    }

    println!("======================================================================");
    if all_ok {
        println!("TODOS OS ARQUIVOS FORAM PROCESSADOS COM SUCESSO.");
        0
    } else {
        println!("FALHAS NA ATUALIZACAO DE ARQUIVOS.");
        1
    }
}

#[cfg(windows)]
fn attach_console_if_cli() {
    use std::fs::OpenOptions;
    use std::os::windows::io::IntoRawHandle;
    unsafe {
        use windows_sys::Win32::System::Console::*;
        let current_out = GetStdHandle(STD_OUTPUT_HANDLE);
        if current_out.is_null() || current_out == (-1isize as *mut std::ffi::c_void) {
            if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
                if let Ok(conout) = OpenOptions::new().write(true).open("CONOUT$") {
                    let raw = conout.into_raw_handle();
                    SetStdHandle(STD_OUTPUT_HANDLE, raw as _);
                }
                if let Ok(conerr) = OpenOptions::new().write(true).open("CONOUT$") {
                    let raw = conerr.into_raw_handle();
                    SetStdHandle(STD_ERROR_HANDLE, raw as _);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        #[cfg(windows)]
        attach_console_if_cli();

        let arg1 = args[1].trim().to_lowercase();
        match arg1.as_str() {
            "--help" | "-h" | "help" => {
                print_help();
                return Ok(());
            }
            "--check" | "-c" => {
                let base_dir = if args.len() > 2 {
                    PathBuf::from(&args[2])
                } else {
                    project_root_dir()
                };
                let code = run_cli_check(base_dir);
                std::process::exit(code);
            }
            "--apply" | "--save" | "--update" => {
                let base_dir = if args.len() > 2 {
                    PathBuf::from(&args[2])
                } else {
                    project_root_dir()
                };
                let ver = if args.len() > 3 {
                    Some(args[3].clone())
                } else {
                    None
                };
                let code = run_cli_apply(base_dir, ver);
                std::process::exit(code);
            }
            _ => {}
        }
    }

    // Launch GUI mode
    let base_dir = project_root_dir();
    let mut native_options = NativeOptions::default();
    native_options.viewport = egui::ViewportBuilder::default()
        .with_title("Editor de Versões e Datas Multi-idioma — GeoEstabil")
        .with_inner_size([1040.0, 760.0])
        .with_min_inner_size([880.0, 600.0])
        .with_active(true)
        .with_visible(true);

    if let Some(icon) = carregar_icone_app(&base_dir) {
        native_options.viewport = native_options.viewport.with_icon(icon);
    }

    eframe::run_native(
        "VersionEditor",
        native_options,
        Box::new(|cc| Ok(Box::new(VersionEditorApp::new(cc)))),
    )?;

    Ok(())
}
