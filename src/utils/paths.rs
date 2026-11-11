use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub fn get_app_data_dir() -> PathBuf {
    if let Some(mut dir) = dirs_next::data_dir() {
        dir.push("GeoEstabil");
        let _ = std::fs::create_dir_all(&dir);
        dir
    } else {
        PathBuf::from(".")
    }
}

pub fn get_database_path() -> PathBuf {
    let mut p = get_app_data_dir();
    p.push("geoestabil.db");
    p
}

/// Retorna o caminho do arquivo JSON de preferência de idioma compartilhado com o instalador (`%APPDATA%\GeoEstabil\language_config.json`).
pub fn get_language_config_path() -> PathBuf {
    let mut p = get_app_data_dir();
    p.push("language_config.json");
    p
}

/// Lê o código de idioma (`"pt_BR"` ou `"en_US"`) de um arquivo JSON no formato `{ "idioma": "..." }`.
pub fn load_language_code_from_file(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let code = json
        .get("idioma")
        .or_else(|| json.get("language"))
        .and_then(|v| v.as_str())?
        .trim();
    if code.is_empty() {
        None
    } else {
        Some(code.to_string())
    }
}

/// Carrega a preferência de idioma persistida em `%APPDATA%\GeoEstabil\language_config.json`.
pub fn load_saved_language_code() -> Option<String> {
    load_language_code_from_file(&get_language_config_path())
}

/// Persiste o código de idioma em um arquivo JSON específico (`{ "idioma": "..." }`).
pub fn save_language_code_to_file(path: &Path, code: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let payload = serde_json::json!({ "idioma": code });
    let formatted = serde_json::to_string_pretty(&payload)
        .unwrap_or_else(|_| format!("{{\"idioma\":\"{}\"}}", code));
    std::fs::write(path, format!("{formatted}\n"))
}

/// Persiste a preferência de idioma em `%APPDATA%\GeoEstabil\language_config.json`.
pub fn save_language_code(code: &str) {
    let _ = save_language_code_to_file(&get_language_config_path(), code);
}

/// Extrai o caminho de um arquivo de projeto `.geoestabil` passado via linha de comando (ex.: associação no Windows Explorer `"%1"`).
pub fn resolve_startup_project_path<I, S>(args: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString>,
{
    for arg in args.into_iter().skip(1) {
        let os: std::ffi::OsString = arg.into();
        if let Some(s) = os.to_str() {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed.starts_with('-') {
                continue;
            }
        }
        let path = PathBuf::from(&os);
        let is_geoestabil_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("geoestabil"));
        if is_geoestabil_ext || path.is_file() {
            return Some(path);
        }
    }
    None
}

fn normalize_asset_rel_path(relativo: &str) -> String {
    let forward = relativo.replace('\\', "/");
    let mut s = forward.trim_start_matches('/');
    for prefix in ["src/assets/", "assets/"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest;
            break;
        }
    }
    s.trim_start_matches('/').to_string()
}

fn push_asset_dir_variants(candidatos: &mut Vec<PathBuf>, base: &Path, rel_norm: &str) {
    candidatos.push(base.join("src").join("assets").join(rel_norm));
    candidatos.push(base.join("assets").join(rel_norm));
    candidatos.push(base.join(rel_norm));
}

/// Resolve o caminho absoluto de um asset a partir de bases explícitas (portável entre instalações Inno Setup,
/// subpastas `bin/`, execução por atalho com CWD arbitrário, modo portátil, MSIX e ambiente de desenvolvimento).
pub fn resolver_caminho_asset_em_bases(
    relativo: &str,
    exe_dir: Option<&Path>,
    cwd: Option<&Path>,
    manifest_dir: Option<&Path>,
    env_assets_dir: Option<&Path>,
    system_data_dirs: &[PathBuf],
) -> Option<PathBuf> {
    let rel_norm = normalize_asset_rel_path(relativo);
    if rel_norm.is_empty() {
        return None;
    }

    let mut candidatos: Vec<PathBuf> = Vec::new();

    // 1. Override explícito por variável de ambiente (ex.: lançador/instalador customizado)
    if let Some(env_dir) = env_assets_dir {
        candidatos.push(env_dir.join(&rel_norm));
        push_asset_dir_variants(&mut candidatos, env_dir, &rel_norm);
    }

    // 2. Diretório do executável instalado pelo Inno Setup e até 3 ancestrais (suporta `{app}\src\assets`,
    // `{app}\assets`, `{app}\bin\geoestabil.exe` ou `builds\geoestabil\release\...`)
    if let Some(dir) = exe_dir {
        let mut cursor = Some(dir);
        for _ in 0..=3 {
            if let Some(d) = cursor {
                push_asset_dir_variants(&mut candidatos, d, &rel_norm);
                cursor = d.parent();
            } else {
                break;
            }
        }
    }

    // 3. Pastas padrão de dados de aplicação no Windows (`%LOCALAPPDATA%\GeoEstabil`, `%APPDATA%\GeoEstabil`, `%ProgramData%\GeoEstabil`)
    for data_root in system_data_dirs {
        let app_sub = data_root.join("GeoEstabil");
        push_asset_dir_variants(&mut candidatos, &app_sub, &rel_norm);
    }

    // 4. Diretório de trabalho atual (desenvolvimento / scripts / invocação manual)
    if let Some(dir) = cwd {
        push_asset_dir_variants(&mut candidatos, dir, &rel_norm);
    }

    // 5. Fallback para diretório do manifesto em tempo de compilação (cargo run / cargo test)
    if let Some(dir) = manifest_dir {
        push_asset_dir_variants(&mut candidatos, dir, &rel_norm);
    }

    candidatos.into_iter().find(|p| p.is_file())
}

/// Resolve o caminho absoluto de um arquivo de asset portátil (`src/assets/...`),
/// funcionando independente de onde for instalado pelo Inno Setup, por atalhos com qualquer
/// diretório de trabalho, em pacotes MSIX ou em desenvolvimento (`cargo run` / `cargo test`).
pub fn resolver_caminho_asset(relativo: &str) -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_deref().and_then(Path::parent);
    let cwd = std::env::current_dir().ok();
    let env_assets = std::env::var_os("GEOESTABIL_ASSETS_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);

    let mut sys_dirs = Vec::new();
    for var in ["LOCALAPPDATA", "APPDATA", "ProgramData"] {
        if let Some(v) = std::env::var_os(var) {
            if !v.is_empty() {
                sys_dirs.push(PathBuf::from(v));
            }
        }
    }

    let manifest = option_env!("CARGO_MANIFEST_DIR").map(Path::new);
    resolver_caminho_asset_em_bases(
        relativo,
        exe_dir,
        cwd.as_deref(),
        manifest,
        env_assets.as_deref(),
        &sys_dirs,
    )
}

/// Carrega bytes do asset instalado em disco (`src/assets/...`) independente do local de instalação,
/// recorrendo ao `fallback` embutido somente se o arquivo externo estiver ausente ou ilegível.
pub fn load_asset_bytes_or<'a>(relativo: &str, fallback: &'a [u8]) -> Cow<'a, [u8]> {
    if let Some(p) = resolver_caminho_asset(relativo) {
        if let Ok(bytes) = std::fs::read(&p) {
            if !bytes.is_empty() {
                return Cow::Owned(bytes);
            }
        }
    }
    Cow::Borrowed(fallback)
}

/// Carrega texto UTF-8 do asset instalado em disco (`src/assets/...`) independente do local de instalação,
/// recorrendo ao `fallback` embutido somente se o arquivo externo estiver ausente ou vazio.
pub fn load_asset_text_or(relativo: &str, fallback: &str) -> String {
    if let Some(p) = resolver_caminho_asset(relativo) {
        if let Ok(text) = std::fs::read_to_string(&p) {
            if !text.trim().is_empty() {
                return text;
            }
        }
    }
    fallback.to_string()
}

mod dirs_next {
    use std::path::PathBuf;
    pub fn data_dir() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }
}
