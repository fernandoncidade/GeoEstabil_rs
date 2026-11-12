use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use chrono::{Datelike, Local, NaiveDate};
use regex::{Captures, Regex};

pub const PT_MONTHS: [&str; 12] = [
    "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
    "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
];
pub const EN_MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangInput {
    pub version: String,
    pub day: u32,
    pub month_index: u32, // 0..11
    pub year: i32,
}

impl Default for LangInput {
    fn default() -> Self {
        let today = Local::now().date_naive();
        let day = today.day();
        let month_index = today.month() - 1;
        let year = today.year();
        let version = format_version_from_date(year, month_index + 1, day, "");
        Self {
            version,
            day,
            month_index,
            year,
        }
    }
}

impl LangInput {
    pub fn new(ver: &str, d: u32, m: u32, y: i32) -> Self {
        Self {
            version: ver.to_string(),
            day: d,
            month_index: m,
            year: y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub key: String,
    pub rel_path: String,
    pub default_group: String,
    pub lang: String,
    pub is_updatable: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileCheckStatus {
    pub key: String,
    pub rel_path: String,
    pub group: String,
    pub lang: String,
    pub absolute_path: PathBuf,
    pub exists: bool,
    pub detected_version: Option<String>,
    pub detected_date: Option<String>,
    pub found_patterns: Vec<String>,
    pub missing_patterns: Vec<String>,
    pub is_updatable: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UpdateResult {
    pub key: String,
    pub rel_path: String,
    pub group: String,
    pub ok: bool,
    pub msg: String,
}

pub fn project_root_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        for _ in 0..8 {
            if let Some(dir) = cur {
                if let Some(name) = dir.file_name() {
                    if name == "version" || name == "mocks" {
                        cur = dir.parent();
                        continue;
                    }
                }
                if dir.join("Cargo.toml").exists() && dir.join("src").exists() {
                    return dir.to_path_buf();
                }
                cur = dir.parent();
            } else {
                break;
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = Some(cwd.as_path());
        for _ in 0..8 {
            if let Some(dir) = cur {
                if let Some(name) = dir.file_name() {
                    if name == "version" || name == "mocks" {
                        cur = dir.parent();
                        continue;
                    }
                }
                if dir.join("Cargo.toml").exists() && dir.join("src").exists() {
                    return dir.to_path_buf();
                }
                cur = dir.parent();
            } else {
                break;
            }
        }
    }

    PathBuf::from(".")
}

pub fn plain_version(version: &str) -> String {
    let mut v = version.trim();
    if v.starts_with('v') || v.starts_with('V') {
        v = &v[1..];
    }
    v.to_string()
}

pub fn readme_version(version: &str) -> String {
    format!("v{}", plain_version(version))
}

pub fn cargo_version(version: &str) -> String {
    let plain = plain_version(version);
    let parts: Vec<&str> = plain.split('.').collect();
    if parts.len() >= 3 {
        format!("{}.{}.{}", parts[0], parts[1], parts[2])
    } else {
        plain
    }
}

pub fn format_pt_date(day: u32, month_index: u32, year: i32) -> String {
    let idx = (month_index as usize).min(11);
    format!("{} de {} de {}", day, PT_MONTHS[idx].to_lowercase(), year)
}

pub fn format_en_date(day: u32, month_index: u32, year: i32) -> String {
    let idx = (month_index as usize).min(11);
    format!("{} {}, {}", EN_MONTHS[idx], day, year)
}

pub fn format_version_from_date(
    year: i32,
    month: u32,
    day: u32,
    existing_version: &str,
) -> String {
    let mut revision = "0".to_string();
    if !existing_version.is_empty() {
        let plain = plain_version(existing_version);
        let parts: Vec<&str> = plain.split('.').collect();
        if parts.len() >= 4 {
            if let Ok(rev) = parts[3].parse::<u32>() {
                revision = rev.to_string();
            }
        }
    }
    format!("{}.{}.{}.{}", year, month, day, revision)
}

pub fn parse_month_name_to_index(month_name: &str) -> Option<u32> {
    let mut norm = month_name.trim().to_lowercase();
    if let Ok(num) = norm.parse::<u32>() {
        if (1..=12).contains(&num) {
            return Some(num - 1);
        }
    }

    for (from, to) in [
        ("á", "a"),
        ("ã", "a"),
        ("ç", "c"),
        ("é", "e"),
        ("í", "i"),
        ("ó", "o"),
        ("õ", "o"),
        ("ú", "u"),
    ] {
        norm = norm.replace(from, to);
    }

    if norm.starts_with("jan") {
        Some(0)
    } else if norm.starts_with("fev") || norm.starts_with("feb") {
        Some(1)
    } else if norm.starts_with("mar") {
        Some(2)
    } else if norm.starts_with("abr") || norm.starts_with("apr") {
        Some(3)
    } else if norm.starts_with("mai") || norm.starts_with("may") {
        Some(4)
    } else if norm.starts_with("jun") {
        Some(5)
    } else if norm.starts_with("jul") {
        Some(6)
    } else if norm.starts_with("ago") || norm.starts_with("aug") {
        Some(7)
    } else if norm.starts_with("set") || norm.starts_with("sep") {
        Some(8)
    } else if norm.starts_with("out") || norm.starts_with("oct") {
        Some(9)
    } else if norm.starts_with("nov") {
        Some(10)
    } else if norm.starts_with("dez") || norm.starts_with("dec") {
        Some(11)
    } else {
        None
    }
}

static RE_VERSION_TO_DATE: OnceLock<Regex> = OnceLock::new();
static RE_DATE_TEXTUAL: OnceLock<Regex> = OnceLock::new();
static RE_DATE_MONTH_FIRST: OnceLock<Regex> = OnceLock::new();
static RE_DATE_ISO: OnceLock<Regex> = OnceLock::new();
static RE_DATE_DMY: OnceLock<Regex> = OnceLock::new();
static RE_VERSION_CALVER: OnceLock<Regex> = OnceLock::new();
static RE_VERSION_LABEL: OnceLock<Regex> = OnceLock::new();
static RE_VERSION_V_PREFIX: OnceLock<Regex> = OnceLock::new();

pub fn parse_version_to_date(version_str: &str) -> Option<(u32, u32, i32, Option<u32>)> {
    let re = RE_VERSION_TO_DATE
        .get_or_init(|| Regex::new(r"(?i)^v?(\d{4})\.(\d{1,2})\.(\d{1,2})(?:\.(\d+))?$").unwrap());
    let caps = re.captures(version_str.trim())?;
    let y = caps[1].parse::<i32>().ok()?;
    let mo = caps[2].parse::<u32>().ok()?;
    let d = caps[3].parse::<u32>().ok()?;
    if (1..=12).contains(&mo)
        && (1..=31).contains(&d)
        && (1900..=3000).contains(&y)
        && NaiveDate::from_ymd_opt(y, mo, d).is_some()
    {
        let rev = caps.get(4).and_then(|m| m.as_str().parse::<u32>().ok());
        return Some((d, mo - 1, y, rev));
    }
    None
}

pub fn parse_date_string(raw_text: &str) -> Option<(u32, u32, i32)> {
    let mut text = raw_text.trim();
    if text.starts_with('`') && text.ends_with('`') && text.len() >= 2 {
        text = text[1..text.len() - 1].trim();
    }

    let re_textual = RE_DATE_TEXTUAL.get_or_init(|| {
        Regex::new(r"(?i)\b(\d{1,2})(?:st|nd|rd|th|\.)?\s+(?:de\s+)?([A-Za-zÀ-ÿ]+)(?:\s+de)?\s+(20[2-9]\d)\b").unwrap()
    });
    if let Some(m) = re_textual.captures(text) {
        if let (Ok(d), Ok(y)) = (m[1].parse::<u32>(), m[3].parse::<i32>()) {
            if let Some(m_idx) = parse_month_name_to_index(&m[2]) {
                if (1..=31).contains(&d) {
                    return Some((d, m_idx, y));
                }
            }
        }
    }

    let re_month_first = RE_DATE_MONTH_FIRST.get_or_init(|| {
        Regex::new(r"(?i)\b([A-Za-zÀ-ÿ]+)\s+(\d{1,2})(?:st|nd|rd|th)?(?:,?\s+)(20[2-9]\d)\b").unwrap()
    });
    if let Some(m) = re_month_first.captures(text) {
        if let Some(m_idx) = parse_month_name_to_index(&m[1]) {
            if let (Ok(d), Ok(y)) = (m[2].parse::<u32>(), m[3].parse::<i32>()) {
                if (1..=31).contains(&d) {
                    return Some((d, m_idx, y));
                }
            }
        }
    }

    let re_iso = RE_DATE_ISO.get_or_init(|| {
        Regex::new(r"\b(20[2-9]\d)[-/](\d{1,2})[-/](\d{1,2})\b").unwrap()
    });
    if let Some(m) = re_iso.captures(text) {
        if let (Ok(y), Ok(mo), Ok(d)) = (
            m[1].parse::<i32>(),
            m[2].parse::<u32>(),
            m[3].parse::<u32>(),
        ) {
            if (1..=12).contains(&mo) && (1..=31).contains(&d) {
                return Some((d, mo - 1, y));
            }
        }
    }

    let re_dmy = RE_DATE_DMY.get_or_init(|| {
        Regex::new(r"\b(\d{1,2})[-/.](\d{1,2})[-/.](20[2-9]\d)\b").unwrap()
    });
    if let Some(m) = re_dmy.captures(text) {
        if let (Ok(n1), Ok(n2), Ok(y)) = (
            m[1].parse::<u32>(),
            m[2].parse::<u32>(),
            m[3].parse::<i32>(),
        ) {
            if (1..=31).contains(&n1) && (1..=12).contains(&n2) {
                return Some((n1, n2 - 1, y));
            } else if (1..=12).contains(&n1) && (1..=31).contains(&n2) {
                return Some((n2, n1 - 1, y));
            }
        }
    }

    None
}

pub fn parse_version_string(raw_text: &str) -> Option<String> {
    let re_calver = RE_VERSION_CALVER.get_or_init(|| {
        Regex::new(r"(?i)\b((?:v|V)?20[2-9]\d\.(?:0?[1-9]|1[0-2])(?:\.(?:0?[1-9]|[12]\d|3[01]))?(?:\.\d+)?)\b").unwrap()
    });
    if let Some(caps) = re_calver.captures(raw_text) {
        return Some(caps[1].to_string());
    }

    let re_label = RE_VERSION_LABEL.get_or_init(|| {
        Regex::new(r#"(?i)\b(?:vers[aã]o|version|app_version)\b\s*[:=\-]?\s*["'`]?\s*((?:v|V)?[0-9]+(?:\.[0-9]+){1,3})\b"#).unwrap()
    });
    if let Some(caps) = re_label.captures(raw_text) {
        return Some(caps[1].to_string());
    }

    let re_v = RE_VERSION_V_PREFIX
        .get_or_init(|| Regex::new(r"\b([vV][0-9]+(?:\.[0-9]+){1,3})\b").unwrap());
    if let Some(caps) = re_v.captures(raw_text) {
        return Some(caps[1].to_string());
    }

    None
}

pub fn resolve_file_path(base_dir: &Path, rel_path: &str) -> PathBuf {
    base_dir.join(rel_path)
}

pub fn find_release_doc_rel_path(base_dir: &Path) -> String {
    let docs_dir = base_dir.join("docs");
    if let Ok(entries) = fs::read_dir(&docs_dir) {
        let mut names: Vec<String> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("release-v") && name.ends_with(".md") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        names.sort();
        if let Some(name) = names.pop() {
            return format!("docs/{}", name);
        }
    }
    "docs/release-v2026.11.10.0.md".to_string()
}

pub fn get_all_file_entries(base_dir: &Path) -> Vec<FileEntry> {
    let rel_doc_path = find_release_doc_rel_path(base_dir);
    let asset_prefix = "src/assets";

    let mut entries = vec![
        // PRIVACY_POLICY
        FileEntry { key: "privacy_pt".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_pt_BR.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "privacy_en".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_en_US.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "en".into(), is_updatable: true },

        // EULA
        FileEntry { key: "eula_pt".into(), rel_path: format!("{}/EULA/EULA_pt_BR - GeoEstabil.txt", asset_prefix), default_group: "EULA".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "eula_en".into(), rel_path: format!("{}/EULA/EULA_en_US - GeoEstabil.txt", asset_prefix), default_group: "EULA".into(), lang: "en".into(), is_updatable: true },

        // CLC
        FileEntry { key: "clc_pt".into(), rel_path: format!("{}/CLC/CLC_pt_BR - GeoEstabil.txt", asset_prefix), default_group: "CLC".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "clc_en".into(), rel_path: format!("{}/CLC/CLC_en_US - GeoEstabil.txt", asset_prefix), default_group: "CLC".into(), lang: "en".into(), is_updatable: true },

        // COPYRIGHT
        FileEntry { key: "copyright_pt".into(), rel_path: format!("{}/COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt", asset_prefix), default_group: "COPYRIGHT".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "copyright_en".into(), rel_path: format!("{}/COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt", asset_prefix), default_group: "COPYRIGHT".into(), lang: "en".into(), is_updatable: true },

        // ABOUT
        FileEntry { key: "about_pt".into(), rel_path: format!("{}/ABOUT/ABOUT_pt_BR.txt", asset_prefix), default_group: "ABOUT".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "about_en".into(), rel_path: format!("{}/ABOUT/ABOUT_en_US.txt", asset_prefix), default_group: "ABOUT".into(), lang: "en".into(), is_updatable: true },
        FileEntry { key: "history_pt".into(), rel_path: format!("{}/ABOUT/History_APP_pt_BR.txt", asset_prefix), default_group: "ABOUT".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "history_en".into(), rel_path: format!("{}/ABOUT/History_APP_en_US.txt", asset_prefix), default_group: "ABOUT".into(), lang: "en".into(), is_updatable: true },

        // NOTICES
        FileEntry { key: "notice_pt".into(), rel_path: format!("{}/NOTICES/NOTICE_pt_BR.txt", asset_prefix), default_group: "NOTICES".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "notice_en".into(), rel_path: format!("{}/NOTICES/NOTICE_en_US.txt", asset_prefix), default_group: "NOTICES".into(), lang: "en".into(), is_updatable: true },

        // DOCS
        FileEntry { key: "doc_architecture".into(), rel_path: "docs/architecture.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_cli_reference".into(), rel_path: "docs/cli-reference.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_compliance".into(), rel_path: "docs/compliance.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_licensing".into(), rel_path: "docs/licensing.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_migration_report".into(), rel_path: "docs/MIGRATION_REPORT.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_release".into(), rel_path: rel_doc_path, default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_security_model".into(), rel_path: "docs/security-model.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_user_guide".into(), rel_path: "docs/user-guide.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },

        // RAIZ & GITHUB
        FileEntry { key: "readme".into(), rel_path: "README.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "security_md".into(), rel_path: "SECURITY.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "code_of_conduct_md".into(), rel_path: "CODE_OF_CONDUCT.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "contributing_md".into(), rel_path: "CONTRIBUTING.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "manual".into(), rel_path: "MANUAL.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "observacao".into(), rel_path: "OBSERVACAO.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "bug_report_md".into(), rel_path: ".github/ISSUE_TEMPLATE/bug_report.md".into(), default_group: "GITHUB".into(), lang: "all".into(), is_updatable: true },

        // SRC_RUST & TESTS
        FileEntry { key: "about_dialog_rs".into(), rel_path: "src/ui/dialogs/about_dialog.rs".into(), default_group: "SRC_RUST".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "dialog_render_test_rs".into(), rel_path: "tests/dialog_render_test.rs".into(), default_group: "TESTS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "main_cli_rs".into(), rel_path: "src/bin/main_cli.rs".into(), default_group: "SRC_RUST".into(), lang: "all".into(), is_updatable: false },
        FileEntry { key: "project_rs".into(), rel_path: "src/core/project.rs".into(), default_group: "SRC_RUST".into(), lang: "all".into(), is_updatable: false },
        FileEntry { key: "app_rs".into(), rel_path: "src/ui/app.rs".into(), default_group: "SRC_RUST".into(), lang: "all".into(), is_updatable: false },

        // MANIFESTS
        FileEntry { key: "cargo_toml".into(), rel_path: "Cargo.toml".into(), default_group: "MANIFESTS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "cargo_lock".into(), rel_path: "Cargo.lock".into(), default_group: "MANIFESTS".into(), lang: "all".into(), is_updatable: true },
    ];

    // REPO (Inno Setup Installers - scan dynamically all .iss files in repo/)
    let repo_dir = base_dir.join("repo");
    let mut iss_entries = Vec::new();
    if repo_dir.is_dir() {
        if let Ok(dir_entries) = fs::read_dir(&repo_dir) {
            let mut paths: Vec<_> = dir_entries.filter_map(|e| e.ok().map(|de| de.path())).collect();
            paths.sort();
            for path in paths {
                if path.is_file()
                    && path
                        .extension()
                        .map(|e| e.to_string_lossy().eq_ignore_ascii_case("iss"))
                        .unwrap_or(false)
                {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        let key = if stem == "geoestabil" {
                            "installer_default".to_string()
                        } else {
                            format!("installer_{}", stem)
                        };
                        iss_entries.push(FileEntry {
                            key,
                            rel_path: format!("repo/{}", file_name),
                            default_group: "REPO".into(),
                            lang: "all".into(),
                            is_updatable: true,
                        });
                    }
                }
            }
        }
    }
    if iss_entries.is_empty() {
        iss_entries.push(FileEntry {
            key: "installer_default".into(),
            rel_path: "repo/GeoEstabil.iss".into(),
            default_group: "REPO".into(),
            lang: "all".into(),
            is_updatable: true,
        });
    }
    entries.extend(iss_entries);

    // REPO - AppxManifest.xml & GeoEstabil_rs.txt
    entries.push(FileEntry {
        key: "repo_appx_manifest".into(),
        rel_path: "repo/AppxManifest.xml".into(),
        default_group: "REPO".into(),
        lang: "all".into(),
        is_updatable: true,
    });
    entries.push(FileEntry {
        key: "repo_geoestabil_txt".into(),
        rel_path: "repo/GeoEstabil_rs.txt".into(),
        default_group: "REPO".into(),
        lang: "all".into(),
        is_updatable: true,
    });

    entries
}

pub fn build_patterns() -> HashMap<String, Vec<&'static str>> {
    let mut map = HashMap::new();

    // Assets .txt
    map.insert("privacy_pt".into(), vec![
        r"(?m)^Última atualização:\s*.*$",
        r"(?m)^Versão:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)",
    ]);
    map.insert("privacy_en".into(), vec![
        r"(?m)^Last updated:\s*.*$",
        r"(?m)^Version:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)",
    ]);
    map.insert("eula_pt".into(), vec![
        r"(?m)^Versão\s+[0-9]+(?:\.[0-9]+){1,3},\s*.*$",
    ]);
    map.insert("eula_en".into(), vec![
        r"(?m)^Version\s+[0-9]+(?:\.[0-9]+){1,3},\s*.*$",
    ]);
    map.insert("clc_pt".into(), vec![
        r"(?m)^Versão\s+[0-9]+(?:\.[0-9]+){1,3},\s*.*$",
    ]);
    map.insert("clc_en".into(), vec![
        r"(?m)^Version\s+[0-9]+(?:\.[0-9]+){1,3},\s*.*$",
    ]);
    map.insert("copyright_pt".into(), vec![
        r"(?m)^Versão\s+[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)\s*\|\s*Data:\s*.*$",
    ]);
    map.insert("copyright_en".into(), vec![
        r"(?m)^Version\s+[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)\s*\|\s*Date:\s*.*$",
    ]);
    map.insert("about_pt".into(), vec![
        r"(?m)^Versão:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)\s*\|\s*Data:\s*.*$",
    ]);
    map.insert("about_en".into(), vec![
        r"(?m)^Version:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)\s*\|\s*Date:\s*.*$",
    ]);
    map.insert("history_pt".into(), vec![
        r"(?m)^Versão:\s*[0-9]+(?:\.[0-9]+){1,3}\s*—",
    ]);
    map.insert("history_en".into(), vec![
        r"(?m)^Version:\s*[0-9]+(?:\.[0-9]+){1,3}\s*—",
    ]);
    map.insert("notice_pt".into(), vec![
        r"(?m)^Versão:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\|\s*GEOESTABIL",
    ]);
    map.insert("notice_en".into(), vec![
        r"(?m)^Version:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\|\s*GEOESTABIL",
    ]);

    // Docs modular (.md)
    map.insert("doc_architecture".into(), vec![r"GeoEstabil\s*\(`v[0-9]+(?:\.[0-9]+){1,3}`\)"]);
    map.insert("doc_user_guide".into(), vec![r"GeoEstabil\s*\(`v[0-9]+(?:\.[0-9]+){1,3}`\)"]);
    map.insert("doc_cli_reference".into(), vec![
        r"versão semântica do pacote\s*\(`[0-9]+(?:\.[0-9]+){1,2}`\)",
        r"semantic package version\s*\(`[0-9]+(?:\.[0-9]+){1,2}`\)",
    ]);
    map.insert("doc_compliance".into(), vec![
        r"GeoEstabil\s*\(`v[0-9]+(?:\.[0-9]+){1,3}`\)",
    ]);
    map.insert("doc_licensing".into(), vec![
        r"(?m)^\*\*Versão:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`\s*\(Cargo:\s*`[0-9]+(?:\.[0-9]+){1,2}`\)",
        r"(?m)^\*\*Data:\*\*\s*`[^`\r\n]+`",
        r"(?m)^\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`\s*\(Cargo:\s*`[0-9]+(?:\.[0-9]+){1,2}`\)",
        r"(?m)^\*\*Date:\*\*\s*`[^`\r\n]+`",
    ]);
    map.insert("doc_migration_report".into(), vec![
        r"\(`v[0-9]+(?:\.[0-9]+){1,3}`\)",
    ]);
    map.insert("doc_release".into(), vec![
        r"(?m)^# Notas de Lançamento / Release Notes — `v[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^\*\*Data de Lançamento:\*\*\s*.*$",
        r"(?m)^\*\*Versão Semântica \(Cargo\):\*\*\s*`[0-9]+(?:\.[0-9]+){1,2}`\s*\|\s*\*\*Versão Empacotada:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^\*\*Release Date:\*\*\s*.*$",
        r"(?m)^\*\*Semantic Version \(Cargo\):\*\*\s*`[0-9]+(?:\.[0-9]+){1,2}`\s*\|\s*\*\*Packaged Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
    ]);
    map.insert("doc_security_model".into(), vec![
        r"(?m)^\*\*Versão:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`\s*\(Cargo:\s*`[0-9]+(?:\.[0-9]+){1,2}`\)\s*\|\s*\*\*Data:\*\*\s*`[^`\r\n]+`",
        r"(?m)^\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`\s*\(Cargo:\s*`[0-9]+(?:\.[0-9]+){1,2}`\)\s*\|\s*\*\*Date:\*\*\s*`[^`\r\n]+`",
    ]);

    // Root & GitHub
    map.insert("readme".into(), vec![
        r"Release-v?[0-9]+(?:\.[0-9]+){1,3}-blue\.svg",
        r"docs/release-v[0-9]+(?:\.[0-9]+){1,3}\.md",
        r"(?im)^\s*-\s*\*\*Vers[aã]o:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`\s*\(\s*Data\s*:\s*[^)]+\)",
        r"(?im)^\s*-\s*\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`\s*\(\s*Date\s*:\s*[^)]+\)",
    ]);
    map.insert("security_md".into(), vec![
        r"(?m)^\*\*Versão:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"\|\s*`v?[0-9]+\.[0-9]+\.x`\s*\|\s*:white_check_mark:\s*\|",
        r"\|\s*`< v?[0-9]+\.[0-9]+`\s*\|\s*:x:\s*\|",
    ]);
    map.insert("code_of_conduct_md".into(), vec![
        r"(?m)^\*\*Versão:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
    ]);
    map.insert("contributing_md".into(), vec![
        r"(?m)^\*\*Versão:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
    ]);
    map.insert("manual".into(), vec![
        r"(?m)^\*\*Versão:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^Versão documentada\s*:\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^Documented version\s*:\s*`[0-9]+(?:\.[0-9]+){1,3}`",
    ]);
    map.insert("observacao".into(), vec![
        r"(?m)^Versão documentada\s*:\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^Documented version\s*:\s*`[0-9]+(?:\.[0-9]+){1,3}`",
        r"(?m)^\*\*Data:\*\*\s*`[^`\r\n]+`",
        r"(?m)^\*\*Date:\*\*\s*`[^`\r\n]+`",
    ]);
    map.insert("bug_report_md".into(), vec![
        r"\*\*Versão do GeoEstabil:\*\*\s*\(ex:\s*`[0-9]+(?:\.[0-9]+){1,3}`\)",
        r"\*\*GeoEstabil Version:\*\*\s*\(e\.g\.,\s*`[0-9]+(?:\.[0-9]+){1,3}`\)",
    ]);

    // Source & Tests
    map.insert("about_dialog_rs".into(), vec![
        r"Notas de Lançamento \([0-9]+(?:\.[0-9]+){1,3}\)",
        r"Release Notes \([0-9]+(?:\.[0-9]+){1,3}\)",
    ]);
    map.insert("dialog_render_test_rs".into(), vec![
        r#""docs/release-v[0-9]+(?:\.[0-9]+){1,3}\.md""#,
    ]);
    map.insert("main_cli_rs".into(), vec![r#"env!\("CARGO_PKG_VERSION"\)"#]);
    map.insert("project_rs".into(), vec![
        r#"env!\("CARGO_PKG_VERSION"\)"#,
        r"chrono::Local::now\(\)",
    ]);
    map.insert("app_rs".into(), vec![r#"env!\("CARGO_PKG_VERSION"\)"#]);

    // Manifests & Installers
    map.insert("cargo_toml".into(), vec![
        r#"(?m)^version\s*=\s*"[0-9]+(?:\.[0-9]+){1,2}""#,
    ]);
    map.insert("cargo_lock".into(), vec![
        r#"name = "geoestabil_rs""#,
    ]);
    map.insert("installer_default".into(), vec![
        r#"(?m)^(#define\s+MyAppVersion\s+"[0-9]+(?:\.[0-9]+){1,3}".*)$"#,
        r#"(?m)^(OutputBaseFilename=GeoEstabil_v(?:\{#MyAppVersion\}|[0-9]+(?:\.[0-9]+){1,3}).*)$"#,
    ]);

    // REPO (AppxManifest.xml & GeoEstabil_rs.txt)
    map.insert("repo_appx_manifest".into(), vec![
        r#"<Identity\b[^>]*?\bVersion="[0-9]+(?:\.[0-9]+){3}""#,
    ]);
    map.insert("repo_geoestabil_txt".into(), vec![
        r#"(?i)docs[/\\]release-v[0-9]+(?:\.[0-9]+){1,3}\.md"#,
        r#"(?m)^Versão:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)\s*\|\s*Data:\s*.*$"#,
        r#"(?m)^DESTAQUES DA VERSÃO\s+[0-9]+(?:\.[0-9]+){1,3}\s+\(LANÇAMENTO INICIAL COMERCIAL E OPEN-SOURCE\)\r?$"#,
        r#"(?m)^Version:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\(Cargo:\s*[0-9]+(?:\.[0-9]+){1,2}\)\s*\|\s*Date:\s*.*$"#,
        r#"(?m)^HIGHLIGHTS IN VERSION\s+[0-9]+(?:\.[0-9]+){1,3}\s+\(INITIAL COMMERCIAL & OPEN-SOURCE RELEASE\)\r?$"#,
    ]);

    map
}

pub fn replace_line_by_regex(text: &str, pattern: &str, replacement_full_line: &str) -> String {
    if let Ok(re) = Regex::new(pattern) {
        re.replace_all(text, replacement_full_line).into_owned()
    } else {
        text.to_string()
    }
}

pub fn replace_regex_capture(
    text: &str,
    pattern: &str,
    capture_index: usize,
    replacement: &str,
) -> String {
    replace_regex_capture_with(text, pattern, capture_index, |_| replacement.to_string())
}

pub fn replace_regex_capture_with<F>(
    text: &str,
    pattern: &str,
    capture_index: usize,
    mut replacer: F,
) -> String
where
    F: FnMut(&str) -> String,
{
    if let Ok(re) = Regex::new(pattern) {
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;
        for caps in re.captures_iter(text) {
            if let Some(m) = caps.get(capture_index) {
                result.push_str(&text[last_end..m.start()]);
                result.push_str(&replacer(m.as_str()));
                last_end = m.end();
            }
        }
        result.push_str(&text[last_end..]);
        result
    } else {
        text.to_string()
    }
}

pub fn replace_with_caps<F>(text: &str, pattern: &str, mut replacer: F) -> String
where
    F: FnMut(&Captures) -> String,
{
    if let Ok(re) = Regex::new(pattern) {
        re.replace_all(text, |caps: &Captures| replacer(caps)).into_owned()
    } else {
        text.to_string()
    }
}

pub fn read_file_robust(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    if let Ok(mut s) = String::from_utf8(bytes.clone()) {
        if s.starts_with('\u{feff}') {
            s.remove(0);
        }
        return Ok(s);
    }
    let s: String = bytes.into_iter().map(|b| b as char).collect();
    Ok(s)
}

pub fn write_file_utf8(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content.as_bytes())
}

fn find_section_bounds(text: &str, ids: &[&str]) -> Option<(usize, usize)> {
    let mut start_idx: Option<usize> = None;
    for id in ids {
        for pat in [
            format!("<a id=\"{id}\">"),
            format!("<h2 id=\"{id}\">"),
            format!("<strong id=\"{id}\">"),
        ] {
            if let Some(idx) = text.find(&pat) {
                start_idx = Some(idx);
                break;
            }
        }
        if start_idx.is_some() {
            break;
        }
    }
    let start = start_idx?;
    let search_start = start + 12;
    let rest = &text[search_start.min(text.len())..];

    let mut end = text.len();
    for marker in ["<a id=\"", "<h2 id=\"", "<strong id=\""] {
        if let Some(rel_idx) = rest.find(marker) {
            end = end.min(search_start + rel_idx);
        }
    }
    Some((start, end))
}

fn read_cargo_pkg_version(base_dir: &Path) -> Option<String> {
    let content = read_file_robust(&base_dir.join("Cargo.toml")).ok()?;
    let re = Regex::new(r#"(?m)^\[package\][\s\S]*?^version\s*=\s*"([^"]+)""#).ok()?;
    re.captures(&content.replace('\r', "")).map(|c| c[1].to_string())
}

pub fn check_all_files(base_dir: &Path) -> Vec<FileCheckStatus> {
    let mut statuses = Vec::new();
    let entries = get_all_file_entries(base_dir);
    let patterns_map = build_patterns();
    let pkg_ver = read_cargo_pkg_version(base_dir);
    let re_lock = Regex::new(
        r#"(?m)\[\[package\]\]\s*\r?\nname\s*=\s*"geoestabil_rs"\s*\r?\nversion\s*=\s*"([^"]+)""#,
    )
    .ok();
    let re_pkg = Regex::new(r#"(?m)\[package\][\s\S]*?^version\s*=\s*"([^"]+)""#).ok();

    for entry in &entries {
        let abs_path = resolve_file_path(base_dir, &entry.rel_path);
        let exists = abs_path.exists();
        let mut detected_version = None;
        let mut detected_date = None;
        let mut found_patterns = Vec::new();
        let mut missing_patterns = Vec::new();

        if exists {
            if let Ok(raw_txt) = read_file_robust(&abs_path) {
                let clean_txt = raw_txt.replace('\r', "");

                let default_iss_patterns = [
                    r#"(?m)^(#define\s+MyAppVersion\s+"[0-9]+(?:\.[0-9]+){1,3}".*)$"#,
                ];
                let pats: &[&str] = if let Some(p) = patterns_map.get(&entry.key) {
                    p.as_slice()
                } else if entry.rel_path.to_lowercase().ends_with(".iss")
                    || entry.key.starts_with("installer_")
                {
                    &default_iss_patterns
                } else {
                    &[]
                };

                for &pat in pats {
                    if let Ok(re) = Regex::new(pat) {
                        if let Some(caps) = re.captures(&clean_txt) {
                            found_patterns.push(caps[0].trim().to_string());
                        } else {
                            missing_patterns.push(pat.to_string());
                        }
                    }
                }

                if entry.key == "readme" {
                    let (ver_opt, date_opt) =
                        extract_readme_section_version_and_date(&clean_txt, "ptbr");
                    if ver_opt.is_some() {
                        detected_version = ver_opt;
                    }
                    if let Some((d, mo, y)) = date_opt {
                        detected_date = Some(format!("{}/{}/{}", d, mo + 1, y));
                    }
                } else if entry.key == "cargo_lock" {
                    if let Some(ref re) = re_lock {
                        if let Some(caps) = re.captures(&clean_txt) {
                            detected_version = Some(caps[1].to_string());
                        }
                    }
                } else if entry.key == "cargo_toml" {
                    if let Some(ref re) = re_pkg {
                        if let Some(caps) = re.captures(&clean_txt) {
                            detected_version = Some(caps[1].to_string());
                        }
                    }
                } else if matches!(
                    entry.key.as_str(),
                    "main_cli_rs" | "project_rs" | "app_rs"
                ) {
                    detected_version = pkg_ver.clone();
                    if entry.key == "project_rs" {
                        let today = Local::now().date_naive();
                        detected_date = Some(format!(
                            "{}/{}/{} (Runtime)",
                            today.day(),
                            today.month(),
                            today.year()
                        ));
                    }
                }

                if detected_version.is_none() {
                    for f in &found_patterns {
                        if let Some(v) = parse_version_string(f) {
                            detected_version = Some(v);
                            break;
                        }
                    }
                }
                if detected_version.is_none() {
                    detected_version = parse_version_string(&clean_txt);
                }

                for f in &found_patterns {
                    if let Some((d, m, y)) = parse_date_string(f) {
                        detected_date = Some(format!("{}/{}/{}", d, m + 1, y));
                        break;
                    }
                }

                if detected_date.is_none() {
                    for line in clean_txt.lines() {
                        let lower = line.to_lowercase();
                        if lower.contains("atualiza")
                            || lower.contains("updated")
                            || lower.contains("data")
                            || lower.contains("date")
                            || lower.contains("vigência")
                            || lower.contains("a partir de")
                            || lower.contains("starting")
                            || lower.contains("a contar de")
                            || lower.contains("validity from")
                        {
                            if let Some((d, m, y)) = parse_date_string(line) {
                                detected_date = Some(format!("{}/{}/{}", d, m + 1, y));
                                break;
                            }
                        }
                    }
                }

                if detected_date.is_none() {
                    if let Some(ref ver) = detected_version {
                        if let Some((vd, vm, vy, _)) = parse_version_to_date(ver) {
                            detected_date = Some(format!("{}/{}/{}", vd, vm + 1, vy));
                        }
                    }
                }
            }
        }

        statuses.push(FileCheckStatus {
            key: entry.key.clone(),
            rel_path: entry.rel_path.clone(),
            group: entry.default_group.clone(),
            lang: entry.lang.clone(),
            absolute_path: abs_path,
            exists,
            detected_version,
            detected_date,
            found_patterns,
            missing_patterns,
            is_updatable: entry.is_updatable,
        });
    }

    statuses
}

pub fn format_updated_version(existing_str: &str, target_version: &str) -> String {
    let clean = existing_str.trim().trim_matches('`').trim();
    let has_v = clean.starts_with('v') || clean.starts_with('V');
    let plain_clean = plain_version(clean);
    let plain_target = plain_version(target_version);
    let dot_parts = plain_clean.split('.').count();

    let final_plain = if dot_parts == 2 {
        let parts: Vec<&str> = plain_target.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            plain_target
        }
    } else if dot_parts == 3 {
        cargo_version(&plain_target)
    } else {
        plain_target
    };

    if has_v {
        format!("v{}", final_plain)
    } else {
        final_plain
    }
}

pub fn format_updated_date(
    existing_str: &str,
    day: u32,
    month_idx: u32,
    year: i32,
    lang: &str,
) -> String {
    let clean = existing_str.trim().trim_matches('`').trim();
    if clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() == 3 {
            let pad = parts[0].len() == 2 || parts[1].len() == 2;
            return if pad {
                format!("{:02}/{:02}/{}", day, month_idx + 1, year)
            } else {
                format!("{}/{}/{}", day, month_idx + 1, year)
            };
        }
    }
    if clean.contains('-') && clean.len() == 10 {
        return format!("{:04}-{:02}-{:02}", year, month_idx + 1, day);
    }
    match lang {
        "en" => format_en_date(day, month_idx, year),
        _ => format_pt_date(day, month_idx, year),
    }
}

pub fn update_markdown_version_and_date(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let mut updated = text.to_string();

    updated = replace_regex_capture_with(
        &updated,
        r#"(?im)((?:^\s*(?:>\s*)?|\|\s*)\*?\*?Vers(?:ã|a)o\s*:\*?\*?\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?)"#,
        2,
        |old| format_updated_version(old, &pt.version),
    );
    updated = replace_regex_capture_with(
        &updated,
        r#"(?im)((?:^\s*(?:>\s*)?|\|\s*)\*?\*?Version\s*:\*?\*?\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?)"#,
        2,
        |old| format_updated_version(old, &en.version),
    );
    updated = replace_regex_capture_with(
        &updated,
        r#"(?im)((?:^\s*(?:>\s*)?|\|\s*)\*?\*?Data\s*:\*?\*?\s*[`"]?)([^`"\r\n]+)([`"]?)"#,
        2,
        |old| format_updated_date(old, pt.day, pt.month_index, pt.year, "pt"),
    );
    updated = replace_regex_capture_with(
        &updated,
        r#"(?im)((?:^\s*(?:>\s*)?|\|\s*)\*?\*?Date\s*:\*?\*?\s*[`"]?)([^`"\r\n]+)([`"]?)"#,
        2,
        |old| format_updated_date(old, en.day, en.month_index, en.year, "en"),
    );

    // Inline Cargo version tags "(Cargo: `2026.11.10`)" or "(Cargo: 2026.11.10)"
    let pt_cargo = cargo_version(&pt.version);
    updated = replace_regex_capture(
        &updated,
        r#"(\(Cargo:\s*`?)([0-9]+(?:\.[0-9]+){1,2})(`?\))"#,
        2,
        &pt_cargo,
    );

    updated
}

pub fn update_standard_doc_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let pt_ver = plain_version(&pt.version);
    let pt_cargo = cargo_version(&pt.version);
    let en_cargo = cargo_version(&en.version);

    let mut updated = update_markdown_version_and_date(text, pt, en);

    // Header & table tags: "GeoEstabil (`v2026.11.10.0`)" / "Rust (`v2026.11.10.0`)"
    updated = replace_regex_capture(
        &updated,
        r"((?:GeoEstabil|Rust)\s*\(`v?)([0-9]+(?:\.[0-9]+){1,3})(`\))",
        2,
        &pt_ver,
    );

    // docs/cli-reference.md: versão semântica do pacote (`C`) / semantic package version (`C`)
    updated = replace_regex_capture(
        &updated,
        r"(versão semântica do pacote\s*\(`)([0-9]+(?:\.[0-9]+){1,2})(`\))",
        2,
        &pt_cargo,
    );
    updated = replace_regex_capture(
        &updated,
        r"(semantic package version\s*\(`)([0-9]+(?:\.[0-9]+){1,2})(`\))",
        2,
        &en_cargo,
    );

    updated
}

pub fn update_release_doc_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let pt_ver = plain_version(&pt.version);
    let en_ver = plain_version(&en.version);
    let pt_cargo = cargo_version(&pt.version);
    let en_cargo = cargo_version(&en.version);
    let pt_date = format_pt_date(pt.day, pt.month_index, pt.year);
    let en_date = format_en_date(en.day, en.month_index, en.year);

    let mut updated = text.to_string();
    updated = replace_regex_capture(
        &updated,
        r"(# Notas de Lançamento / Release Notes — `v)([0-9]+(?:\.[0-9]+){1,3})(`)",
        2,
        &pt_ver,
    );
    updated = replace_regex_capture(
        &updated,
        r"(Marco Inicial `v)([0-9]+(?:\.[0-9]+){1,3})(`)",
        2,
        &pt_ver,
    );
    updated = replace_regex_capture(
        &updated,
        r"(Initial Release `v)([0-9]+(?:\.[0-9]+){1,3})(`)",
        2,
        &en_ver,
    );
    updated = replace_regex_capture(
        &updated,
        r"(?m)^(\*\*Data de Lançamento:\*\*\s*)([^\r\n]+)",
        2,
        &format!("{pt_date}  "),
    );
    // Preserve exact trailing spaces only if original didn't have two spaces
    updated = updated.replace(&format!("{pt_date}    "), &format!("{pt_date}  "));
    updated = replace_regex_capture(
        &updated,
        r"(?m)^(\*\*Release Date:\*\*\s*)([^\r\n]+)",
        2,
        &format!("{en_date}  "),
    );
    updated = updated.replace(&format!("{en_date}    "), &format!("{en_date}  "));
    updated = replace_regex_capture(
        &updated,
        r"(\*\*Versão Semântica \(Cargo\):\*\*\s*`)([0-9]+(?:\.[0-9]+){1,2})(`)",
        2,
        &pt_cargo,
    );
    updated = replace_regex_capture(
        &updated,
        r"(\*\*Versão Empacotada:\*\*\s*`)([0-9]+(?:\.[0-9]+){1,3})(`)",
        2,
        &pt_ver,
    );
    updated = replace_regex_capture(
        &updated,
        r"(\*\*Semantic Version \(Cargo\):\*\*\s*`)([0-9]+(?:\.[0-9]+){1,2})(`)",
        2,
        &en_cargo,
    );
    updated = replace_regex_capture(
        &updated,
        r"(\*\*Packaged Version:\*\*\s*`)([0-9]+(?:\.[0-9]+){1,3})(`)",
        2,
        &en_ver,
    );
    updated
}

pub fn update_security_md_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let (year, month_idx) = if let Some((_, m, y, _)) = parse_version_to_date(&pt.version) {
        (y, m)
    } else {
        (pt.year, pt.month_index)
    };
    let ym = format!("{}.{}", year, month_idx + 1);

    let mut updated = replace_regex_capture(
        text,
        r"(\|\s*`v?)(\d+\.\d+)(\.x`\s*\|\s*:white_check_mark:\s*\|)",
        2,
        &ym,
    );
    updated = replace_regex_capture(&updated, r"(\|\s*`< v?)(\d+\.\d+)(`\s*\|\s*:x:\s*\|)", 2, &ym);
    update_markdown_version_and_date(&updated, pt, en)
}

pub fn update_code_of_conduct_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    update_markdown_version_and_date(text, pt, en)
}

pub fn update_contributing_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    update_markdown_version_and_date(text, pt, en)
}

pub fn update_readme_section(text: &str, section_id: &str, version: &str, date_str: &str) -> String {
    let Some((start, end)) = find_section_bounds(text, &[section_id]) else {
        return text.to_string();
    };

    let mut section = text[start..end].to_string();
    let target_plain_ver = plain_version(version);
    let target_v_ver = readme_version(version);

    // 1. Callout note: > **Observação:** ... **`2026.11.10.0`**
    section = replace_regex_capture(
        &section,
        r"(?m)(^\s*(?:>\s*)?\*\*(?:Observação|Observacao|Note|Nota)\s*:\*\*.*?\*\*`)(v?[0-9]+(?:\.[0-9]+){1,3})(`\*\*.*$)",
        2,
        &target_plain_ver,
    );
    section = replace_regex_capture(
        &section,
        r"(?m)(^\s*(?:>\s*)?\*\*(?:Observação|Observacao|Note|Nota)\s*:\*\*.*?\*\*)(v?[0-9]+(?:\.[0-9]+){1,3})(\*\*.*$)",
        2,
        &target_v_ver,
    );

    // 2. Compound Version + Date bullet: - **Versão:** `2026.11.10.0` (Data: 10 de novembro de 2026)
    section = replace_with_caps(
        &section,
        r#"(?im)(^\s*-\s*\*\*(?:Versão|Versao|Version)\s*:\s*\*\*\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?\s*\(\s*(?:Data|Date)\s*:\s*)([^)]+)(\)\s*(?:<br>)?\s*$)"#,
        |caps| format!("{}{}{}{}{}", &caps[1], target_plain_ver, &caps[3], date_str, &caps[5]),
    );

    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..start]);
    out.push_str(&section);
    out.push_str(&text[end..]);
    out
}

pub fn update_readme_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let mut updated = text.to_string();
    let target_plain_ver = plain_version(&pt.version);

    updated = replace_regex_capture(&updated, r"(Release-v?)([\d\.]+)(-blue\.svg)", 2, &target_plain_ver);
    updated = replace_regex_capture(
        &updated,
        r"(docs/release-v)([0-9]+(?:\.[0-9]+){1,3})(\.md)",
        2,
        &target_plain_ver,
    );
    updated = replace_regex_capture(
        &updated,
        r"(\[\s*(?:Release Notes|Notas de Lançamento)\s+v?)([\d\.]+)(\])",
        2,
        &target_plain_ver,
    );

    let pt_date = format_pt_date(pt.day, pt.month_index, pt.year);
    let en_date = format_en_date(en.day, en.month_index, en.year);

    updated = update_readme_section(&updated, "ptbr", &pt.version, &pt_date);
    updated = update_readme_section(&updated, "enus", &en.version, &en_date);
    updated
}

pub fn extract_readme_section_version_and_date(
    text: &str,
    section_id: &str,
) -> (Option<String>, Option<(u32, u32, i32)>) {
    let Some((start, end)) = find_section_bounds(text, &[section_id]) else {
        return (None, None);
    };
    let section = &text[start..end];
    let mut detected_version = None;
    let mut detected_date = None;

    if let Ok(re_compound) = Regex::new(
        r#"(?im)^\s*-\s*\*\*(?:Versão|Versao|Version)\s*:\s*\*\*\s*[`"]?((?:v|V)?[0-9]+(?:\.[0-9]+){1,3})[`"]?\s*\(\s*(?:Data|Date)\s*:\s*([^)]+)\)"#,
    ) {
        if let Some(caps) = re_compound.captures(section) {
            detected_version = Some(caps[1].to_string());
            detected_date = parse_date_string(&caps[2]);
        }
    }

    if detected_version.is_none() {
        detected_version = parse_version_string(section);
    }
    if detected_date.is_none() {
        detected_date = parse_date_string(section);
    }

    (detected_version, detected_date)
}

pub fn update_manual_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let mut updated = update_markdown_version_and_date(text, pt, en);
    updated = replace_regex_capture_with(
        &updated,
        r#"(?im)(^\s*Vers(?:ã|a)o documentada\s*:\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?)"#,
        2,
        |old| format_updated_version(old, &pt.version),
    );
    updated = replace_regex_capture_with(
        &updated,
        r#"(?im)(^\s*Documented version\s*:\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?)"#,
        2,
        |old| format_updated_version(old, &en.version),
    );
    let pt_cargo = cargo_version(&pt.version);
    let en_cargo = cargo_version(&en.version);
    updated = replace_regex_capture(
        &updated,
        r"(semântica\s*\(`v?)([0-9]+(?:\.[0-9]+){1,2})(`\))",
        2,
        &pt_cargo,
    );
    updated = replace_regex_capture(
        &updated,
        r"(semantic package version\s*\(`v?)([0-9]+(?:\.[0-9]+){1,2})(`\))",
        2,
        &en_cargo,
    );
    updated
}

pub fn update_observacao_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let mut updated = update_manual_versions(text, pt, en);
    updated = replace_regex_capture(
        &updated,
        r"(release-v)([0-9]+(?:\.[0-9]+){1,3})(\.md)",
        2,
        &plain_version(&pt.version),
    );
    updated
}

pub fn update_bug_report_md(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let mut updated = replace_regex_capture(
        text,
        r"(\*\*Versão do GeoEstabil:\*\*\s*\(ex:\s*`)([0-9]+(?:\.[0-9]+){1,3})(`\))",
        2,
        &plain_version(&pt.version),
    );
    updated = replace_regex_capture(
        &updated,
        r"(\*\*GeoEstabil Version:\*\*\s*\(e\.g\.,\s*`)([0-9]+(?:\.[0-9]+){1,3})(`\))",
        2,
        &plain_version(&en.version),
    );
    updated
}

pub fn update_asset_txt(key: &str, text: &str, pt: &LangInput, en: &LangInput) -> String {
    let pt_ver = plain_version(&pt.version);
    let en_ver = plain_version(&en.version);
    let pt_cargo = cargo_version(&pt.version);
    let en_cargo = cargo_version(&en.version);
    let pt_date = format_pt_date(pt.day, pt.month_index, pt.year);
    let en_date = format_en_date(en.day, en.month_index, en.year);

    match key {
        "privacy_pt" => {
            let mut s = replace_line_by_regex(text, r"(?m)^Última atualização:.*$", &format!("Última atualização: {pt_date}"));
            s = replace_line_by_regex(&s, r"(?m)^Versão:.*$", &format!("Versão: {pt_ver} (Cargo: {pt_cargo})"));
            s
        }
        "privacy_en" => {
            let mut s = replace_line_by_regex(text, r"(?m)^Last updated:.*$", &format!("Last updated: {en_date}"));
            s = replace_line_by_regex(&s, r"(?m)^Version:.*$", &format!("Version: {en_ver} (Cargo: {en_cargo})"));
            s
        }
        "eula_pt" => replace_line_by_regex(text, r"(?m)^Versão\s+[0-9].*$", &format!("Versão {pt_ver}, {pt_date}")),
        "eula_en" => replace_line_by_regex(text, r"(?m)^Version\s+[0-9].*$", &format!("Version {en_ver}, {en_date}")),
        "clc_pt" => replace_line_by_regex(text, r"(?m)^Versão\s+[0-9].*$", &format!("Versão {pt_ver}, {pt_date}")),
        "clc_en" => replace_line_by_regex(text, r"(?m)^Version\s+[0-9].*$", &format!("Version {en_ver}, {en_date}")),
        "copyright_pt" => replace_line_by_regex(
            text,
            r"(?m)^Versão\s+[0-9].*$",
            &format!("Versão {pt_ver} (Cargo: {pt_cargo}) | Data: {pt_date}"),
        ),
        "copyright_en" => replace_line_by_regex(
            text,
            r"(?m)^Version\s+[0-9].*$",
            &format!("Version {en_ver} (Cargo: {en_cargo}) | Date: {en_date}"),
        ),
        "about_pt" => replace_line_by_regex(
            text,
            r"(?m)^Versão:.*$",
            &format!("Versão: {pt_ver} (Cargo: {pt_cargo}) | Data: {pt_date}"),
        ),
        "about_en" => replace_line_by_regex(
            text,
            r"(?m)^Version:.*$",
            &format!("Version: {en_ver} (Cargo: {en_cargo}) | Date: {en_date}"),
        ),
        "history_pt" => replace_regex_capture(
            text,
            r"(?m)^(Versão:\s*)([0-9]+(?:\.[0-9]+){1,3})(\s*—)",
            2,
            &pt_ver,
        ),
        "history_en" => replace_regex_capture(
            text,
            r"(?m)^(Version:\s*)([0-9]+(?:\.[0-9]+){1,3})(\s*—)",
            2,
            &en_ver,
        ),
        "notice_pt" => replace_regex_capture(
            text,
            r"(?m)^(Versão:\s*)([0-9]+(?:\.[0-9]+){1,3})(\s*\|\s*GEOESTABIL)",
            2,
            &pt_ver,
        ),
        "notice_en" => replace_regex_capture(
            text,
            r"(?m)^(Version:\s*)([0-9]+(?:\.[0-9]+){1,3})(\s*\|\s*GEOESTABIL)",
            2,
            &en_ver,
        ),
        _ => text.to_string(),
    }
}

pub fn update_about_dialog_rs(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let pt_ver = plain_version(&pt.version);
    let en_ver = plain_version(&en.version);

    let mut updated = replace_regex_capture(
        text,
        r"(Notas de Lançamento \()([0-9]+(?:\.[0-9]+){1,3})(\))",
        2,
        &pt_ver,
    );
    updated = replace_regex_capture(
        &updated,
        r"(Release Notes \()([0-9]+(?:\.[0-9]+){1,3})(\))",
        2,
        &en_ver,
    );
    updated
}

pub fn update_dialog_render_test_rs(text: &str, version: &str) -> String {
    let v = plain_version(version);
    replace_regex_capture(
        text,
        r#"("docs/release-v)([0-9]+(?:\.[0-9]+){1,3})(\.md")"#,
        2,
        &v,
    )
}

pub fn update_cargo_toml(text: &str, version: &str) -> String {
    let cv = cargo_version(version);
    replace_regex_capture(text, r#"(?m)(^\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")"#, 2, &cv)
}

pub fn update_cargo_lock(text: &str, version: &str) -> String {
    let cv = cargo_version(version);
    replace_regex_capture(
        text,
        r#"(?m)(\[\[package\]\]\s*\r?\nname\s*=\s*"geoestabil_rs"\s*\r?\nversion\s*=\s*")([^"]+)(")"#,
        2,
        &cv,
    )
}

pub fn update_installer_version(text: &str, version: &str) -> String {
    let normalized = plain_version(version);
    let mut updated = replace_regex_capture(
        text,
        r#"(?m)(^\s*#define\s+MyAppVersion\s+")([0-9]+(?:\.[0-9]+){1,3})(")"#,
        2,
        &normalized,
    );
    updated = replace_regex_capture(
        &updated,
        r#"(?m)(^\s*OutputBaseFilename=GeoEstabil_v)([0-9]+(?:\.[0-9]+){1,3})(\r?$)"#,
        2,
        &normalized,
    );
    updated
}

pub fn appx_version(version: &str) -> String {
    let plain = plain_version(version);
    let parts: Vec<&str> = plain.split('.').collect();
    if parts.len() == 3 {
        format!("{}.0", plain)
    } else if parts.len() >= 4 {
        format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3])
    } else {
        plain
    }
}

pub fn update_appx_manifest(text: &str, version: &str) -> String {
    let appx_ver = appx_version(version);
    replace_regex_capture(
        text,
        r#"(?m)(<Identity\b[^>]*?\bVersion=")([^"]+)(")"#,
        2,
        &appx_ver,
    )
}

pub fn update_geoestabil_txt(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let pt_ver = plain_version(&pt.version);
    let en_ver = plain_version(&en.version);
    let pt_cargo = cargo_version(&pt.version);
    let en_cargo = cargo_version(&en.version);
    let pt_date = format_pt_date(pt.day, pt.month_index, pt.year);
    let en_date = format_en_date(en.day, en.month_index, en.year);

    let mut updated = text.to_string();

    // 1. docs\release-v<ver>.md or docs/release-v<ver>.md
    updated = replace_regex_capture(
        &updated,
        r#"(docs[/\\]release-v)([0-9]+(?:\.[0-9]+){1,3})(\.md)"#,
        2,
        &pt_ver,
    );

    // 2. Versão: <ver> (Cargo: <cargo>) | Data: <date>
    updated = replace_with_caps(
        &updated,
        r#"(?m)^(Versão:\s*)([0-9]+(?:\.[0-9]+){1,3})(\s*\(Cargo:\s*)([0-9]+(?:\.[0-9]+){1,2})(\)\s*\|\s*Data:\s*)([^\r\n]+)(\r?$)"#,
        |caps| format!("{}{}{}{}{}{}{}", &caps[1], pt_ver, &caps[3], pt_cargo, &caps[5], pt_date, &caps[7]),
    );

    // 3. DESTAQUES DA VERSÃO <ver> (LANÇAMENTO INICIAL COMERCIAL E OPEN-SOURCE)
    updated = replace_with_caps(
        &updated,
        r#"(?m)^(DESTAQUES DA VERSÃO\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+\(LANÇAMENTO INICIAL COMERCIAL E OPEN-SOURCE\)\r?$)"#,
        |caps| format!("{}{}{}", &caps[1], pt_ver, &caps[3]),
    );

    // 4. Version: <ver> (Cargo: <cargo>) | Date: <date>
    updated = replace_with_caps(
        &updated,
        r#"(?m)^(Version:\s*)([0-9]+(?:\.[0-9]+){1,3})(\s*\(Cargo:\s*)([0-9]+(?:\.[0-9]+){1,2})(\)\s*\|\s*Date:\s*)([^\r\n]+)(\r?$)"#,
        |caps| format!("{}{}{}{}{}{}{}", &caps[1], en_ver, &caps[3], en_cargo, &caps[5], en_date, &caps[7]),
    );

    // 5. HIGHLIGHTS IN VERSION <ver> (INITIAL COMMERCIAL & OPEN-SOURCE RELEASE)
    updated = replace_with_caps(
        &updated,
        r#"(?m)^(HIGHLIGHTS IN VERSION\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+\(INITIAL COMMERCIAL & OPEN-SOURCE RELEASE\)\r?$)"#,
        |caps| format!("{}{}{}", &caps[1], en_ver, &caps[3]),
    );

    updated
}

pub fn apply_updates(
    pt: &LangInput,
    en: &LangInput,
    base_dir: &Path,
) -> Vec<UpdateResult> {
    let mut results = Vec::new();
    let entries = get_all_file_entries(base_dir);

    for entry in &entries {
        if !entry.is_updatable {
            continue;
        }

        let mut abs_path = resolve_file_path(base_dir, &entry.rel_path);
        let group = entry.default_group.clone();

        let original = match read_file_robust(&abs_path) {
            Ok(content) => content,
            Err(_) => {
                results.push(UpdateResult {
                    key: entry.key.clone(),
                    rel_path: entry.rel_path.clone(),
                    group,
                    ok: false,
                    msg: "Arquivo não encontrado ou erro de leitura".to_string(),
                });
                continue;
            }
        };

        let mut new_txt = original.clone();
        let key = entry.key.as_str();

        match key {
            "privacy_pt" | "privacy_en" | "eula_pt" | "eula_en" | "clc_pt" | "clc_en"
            | "copyright_pt" | "copyright_en" | "about_pt" | "about_en" | "history_pt"
            | "history_en" | "notice_pt" | "notice_en" => {
                new_txt = update_asset_txt(key, &new_txt, pt, en);
            }
            "doc_architecture" | "doc_cli_reference" | "doc_compliance"
            | "doc_licensing" | "doc_migration_report"
            | "doc_security_model" | "doc_user_guide" => {
                new_txt = update_standard_doc_versions(&new_txt, pt, en);
            }
            "doc_release" => {
                new_txt = update_release_doc_versions(&new_txt, pt, en);
                let target_ver = plain_version(&pt.version);
                let new_rel_path = format!("docs/release-v{}.md", target_ver);
                if entry.rel_path != new_rel_path && abs_path.exists() {
                    let new_abs = resolve_file_path(base_dir, &new_rel_path);
                    if new_abs != abs_path {
                        let _ = fs::remove_file(&new_abs);
                        if fs::rename(&abs_path, &new_abs).is_ok() {
                            abs_path = new_abs;
                        }
                    }
                }
            }
            "readme" => new_txt = update_readme_versions(&new_txt, pt, en),
            "security_md" => new_txt = update_security_md_versions(&new_txt, pt, en),
            "code_of_conduct_md" => new_txt = update_code_of_conduct_versions(&new_txt, pt, en),
            "contributing_md" => new_txt = update_contributing_versions(&new_txt, pt, en),
            "manual" => new_txt = update_manual_versions(&new_txt, pt, en),
            "observacao" => new_txt = update_observacao_versions(&new_txt, pt, en),
            "bug_report_md" => new_txt = update_bug_report_md(&new_txt, pt, en),
            "about_dialog_rs" => new_txt = update_about_dialog_rs(&new_txt, pt, en),
            "dialog_render_test_rs" => {
                new_txt = update_dialog_render_test_rs(&new_txt, &pt.version);
            }
            "cargo_toml" => new_txt = update_cargo_toml(&new_txt, &pt.version),
            "cargo_lock" => new_txt = update_cargo_lock(&new_txt, &pt.version),
            "repo_appx_manifest" => {
                new_txt = update_appx_manifest(&new_txt, &pt.version);
            }
            "repo_geoestabil_txt" => {
                new_txt = update_geoestabil_txt(&new_txt, pt, en);
            }
            k if k.starts_with("installer_")
                || entry.rel_path.to_lowercase().ends_with(".iss") =>
            {
                new_txt = update_installer_version(&new_txt, &pt.version);
            }
            _ => {}
        }

        if new_txt != original {
            match write_file_utf8(&abs_path, &new_txt) {
                Ok(_) => results.push(UpdateResult {
                    key: entry.key.clone(),
                    rel_path: entry.rel_path.clone(),
                    group,
                    ok: true,
                    msg: "Atualizado com sucesso".to_string(),
                }),
                Err(e) => results.push(UpdateResult {
                    key: entry.key.clone(),
                    rel_path: entry.rel_path.clone(),
                    group,
                    ok: false,
                    msg: format!("Erro ao gravar: {}", e),
                }),
            }
        } else {
            results.push(UpdateResult {
                key: entry.key.clone(),
                rel_path: entry.rel_path.clone(),
                group,
                ok: true,
                msg: "Sem alterações necessárias (já atualizado)".to_string(),
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_geoestabil_repo_files_exist_and_match_patterns() {
        let root = project_root_dir();
        assert!(root.join("Cargo.toml").exists());

        let statuses = check_all_files(&root);
        assert!(
            statuses.len() >= 37,
            "Expected full registry of GeoEstabil files, got {}",
            statuses.len()
        );
        assert!(
            statuses.iter().all(|s| s.group != "RELEASE" && !s.rel_path.contains("RELEASE NOTES")),
            "Historical RELEASE NOTES files must not be tracked or updated by version-editor"
        );

        for st in &statuses {
            assert!(
                st.exists,
                "Monitored file {} ({}) does not exist at {:?}",
                st.rel_path,
                st.group,
                st.absolute_path
            );
            assert!(
                st.missing_patterns.is_empty(),
                "Monitored file {} ({}) failed patterns: {:?}",
                st.rel_path,
                st.group,
                st.missing_patterns
            );
            assert!(
                st.detected_version.is_some() || st.detected_date.is_some(),
                "Monitored file {} ({}) had neither detected version nor detected date",
                st.rel_path,
                st.group
            );
        }
    }

    #[test]
    fn test_update_assets_txt_preserves_structure_and_bilingual_fields() {
        let pt = LangInput::new("2027.1.15.2", 15, 0, 2027);
        let en = LangInput::new("2027.1.15.2", 15, 0, 2027);

        let eula_pt = "EULA\nVersão 2026.11.10.0, 10 de novembro de 2026\n3.1. SUPORTE: via e-mail.";
        let up_eula = update_asset_txt("eula_pt", eula_pt, &pt, &en);
        assert!(up_eula.contains("Versão 2027.1.15.2, 15 de janeiro de 2027"));

        let clc_en = "CLC\nVersion 2026.11.10.0, November 10, 2026\n4.1. Support via email.";
        let up_clc = update_asset_txt("clc_en", clc_en, &pt, &en);
        assert!(up_clc.contains("Version 2027.1.15.2, January 15, 2027"));

        let about_pt = "GEOESTABIL\nVersão: 2026.11.10.0 (Cargo: 2026.11.10) | Data: 10 de novembro de 2026\nSOBRE";
        let up_about = update_asset_txt("about_pt", about_pt, &pt, &en);
        assert!(up_about.contains("Versão: 2027.1.15.2 (Cargo: 2027.1.15) | Data: 15 de janeiro de 2027"));

        let notice_pt = "AVISO\nVersão: 2026.11.10.0 | GEOESTABIL";
        assert!(update_asset_txt("notice_pt", notice_pt, &pt, &en).contains("Versão: 2027.1.15.2 | GEOESTABIL"));
    }

    #[test]
    fn test_update_readme_docs_src_tests_and_installer_for_geoestabil() {
        let pt = LangInput::new("2027.2.20.0", 20, 1, 2027);
        let en = LangInput::new("2027.2.20.0", 20, 1, 2027);

        let readme = r#"[![Latest Release](https://img.shields.io/badge/Release-v2026.11.10.0-blue.svg)](docs/release-v2026.11.10.0.md)
## <a id="ptbr"></a>🇧🇷 Português (BR)
> **Observação:** Este repositório refere-se à versão **`2026.11.10.0`** do **GeoEstabil**
- **Versão:** `2026.11.10.0` (Data: 10 de novembro de 2026)
## <a id="enus"></a>🇺🇸 English (US)
> **Note:** This repository corresponds to release **`2026.11.10.0`** of **GeoEstabil**
- **Version:** `2026.11.10.0` (Date: November 10, 2026)"#;

        let up_readme = update_readme_versions(readme, &pt, &en);
        assert!(up_readme.contains("Release-v2027.2.20.0-blue.svg"));
        assert!(up_readme.contains("docs/release-v2027.2.20.0.md"));
        assert!(up_readme.contains("**`2027.2.20.0`** do **GeoEstabil**"));
        assert!(up_readme.contains("- **Versão:** `2027.2.20.0` (Data: 20 de fevereiro de 2027)"));
        assert!(up_readme.contains("- **Version:** `2027.2.20.0` (Date: February 20, 2027)"));
        assert_eq!(readme.lines().count(), up_readme.lines().count());

        let about_rs = r#"if pt { "🚀 Notas de Lançamento (2026.11.10.0)" } else { "🚀 Release Notes (2026.11.10.0)" },"#;
        let up_about_rs = update_about_dialog_rs(about_rs, &pt, &en);
        assert!(up_about_rs.contains("Notas de Lançamento (2027.2.20.0)"));
        assert!(up_about_rs.contains("Release Notes (2027.2.20.0)"));

        let test_rs = r#""docs/release-v2026.11.10.0.md","#;
        assert!(update_dialog_render_test_rs(test_rs, &pt.version).contains("\"docs/release-v2027.2.20.0.md\""));

        let iss = "#define MyAppVersion \"2026.11.10.0\"\nOutputBaseFilename=GeoEstabil_v{#MyAppVersion}\n";
        assert!(update_installer_version(iss, "2027.2.20.0").contains("#define MyAppVersion \"2027.2.20.0\""));

        let lock = "[[package]]\nname = \"geoestabil_rs\"\nversion = \"2026.11.10\"\n";
        assert!(update_cargo_lock(lock, "2027.2.20.0").contains("version = \"2027.2.20\""));
    }

    #[test]
    fn test_update_repo_appx_manifest_and_geoestabil_txt() {
        let pt = LangInput::new("2027.5.10.0", 10, 4, 2027);
        let en = LangInput::new("2027.5.10.0", 10, 4, 2027);

        let appx = r#"<Package><Identity Name="fernandoncidade.GeoEstabil" Publisher="CN=test" Version="2026.11.11.0" ProcessorArchitecture="x64" /></Package>"#;
        let up_appx = update_appx_manifest(appx, &pt.version);
        assert!(up_appx.contains(r#"Version="2027.5.10.0""#));

        let txt = r#"
"C:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\docs\release-v2026.11.11.0.md";
Versão: 2026.11.11.0 (Cargo: 2026.11.11) | Data: 11 de novembro de 2026
DESTAQUES DA VERSÃO 2026.11.11.0 (LANÇAMENTO INICIAL COMERCIAL E OPEN-SOURCE)
Version: 2026.11.11.0 (Cargo: 2026.11.11) | Date: November 11, 2026
HIGHLIGHTS IN VERSION 2026.11.11.0 (INITIAL COMMERCIAL & OPEN-SOURCE RELEASE)
"#;
        let up_txt = update_geoestabil_txt(txt, &pt, &en);
        assert!(up_txt.contains(r#"docs\release-v2027.5.10.0.md"#));
        assert!(up_txt.contains("Versão: 2027.5.10.0 (Cargo: 2027.5.10) | Data: 10 de maio de 2027"));
        assert!(up_txt.contains("DESTAQUES DA VERSÃO 2027.5.10.0 (LANÇAMENTO INICIAL COMERCIAL E OPEN-SOURCE)"));
        assert!(up_txt.contains("Version: 2027.5.10.0 (Cargo: 2027.5.10) | Date: May 10, 2027"));
        assert!(up_txt.contains("HIGHLIGHTS IN VERSION 2027.5.10.0 (INITIAL COMMERCIAL & OPEN-SOURCE RELEASE)"));
    }

    #[test]
    fn test_simulated_full_apply_preserves_parity_and_rechecks_clean() {
        let root = project_root_dir();
        let tmp = std::env::temp_dir().join(format!("geoestabil_version_sim_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);

        // Copy only versioned files from repo to isolated temp dir and record line counts
        let mut orig_line_counts = std::collections::HashMap::new();
        let all_entries = get_all_file_entries(&root);
        assert!(
            all_entries.iter().all(|e| !e.rel_path.contains("RELEASE NOTES")),
            "RELEASE NOTES files must NOT be registered or updated by version-editor"
        );
        for entry in all_entries {
            let src = resolve_file_path(&root, &entry.rel_path);
            let dst = tmp.join(&entry.rel_path);
            if let Some(p) = dst.parent() {
                fs::create_dir_all(p).unwrap();
            }
            if entry.rel_path.ends_with(".md") || entry.rel_path.ends_with(".txt") || entry.rel_path.ends_with(".xml") {
                let raw = fs::read_to_string(&src).unwrap();
                orig_line_counts.insert(entry.key.clone(), raw.lines().count());
            }
            fs::copy(&src, &dst).unwrap();
        }
        // Also copy historical RELEASE NOTES into simulated tree to guarantee apply_updates never touches them
        let mut orig_release_notes = std::collections::HashMap::new();
        for rel_rn in [
            "src/assets/RELEASE/RELEASE NOTES_pt_BR.txt",
            "src/assets/RELEASE/RELEASE NOTES_en_US.txt",
        ] {
            let src = root.join(rel_rn);
            let dst = tmp.join(rel_rn);
            if let Some(p) = dst.parent() {
                fs::create_dir_all(p).unwrap();
            }
            orig_release_notes.insert(rel_rn, fs::read_to_string(&src).unwrap());
            fs::copy(&src, &dst).unwrap();
        }

        let pt = LangInput::new("2027.3.5.1", 5, 2, 2027);
        let en = LangInput::new("2027.3.5.1", 5, 2, 2027);
        let results = apply_updates(&pt, &en, &tmp);
        assert!(results.iter().all(|r| r.ok), "All updates must succeed: {:?}", results);

        for (rel_rn, orig_txt) in &orig_release_notes {
            let after_txt = fs::read_to_string(tmp.join(rel_rn)).unwrap();
            assert_eq!(
                after_txt, *orig_txt,
                "Historical release file {} must remain untouched by version-editor",
                rel_rn
            );
        }

        // Re-check all files in simulated tree: must report new version 2027.3.5(.1) & 5/3/2027!
        let post_statuses = check_all_files(&tmp);
        for st in &post_statuses {
            assert!(st.exists, "Missing post-update file: {}", st.rel_path);
            assert!(
                st.missing_patterns.is_empty(),
                "Post-update pattern mismatch in {}: {:?}",
                st.rel_path,
                st.missing_patterns
            );
            let ver = st.detected_version.as_deref().unwrap_or("");
            assert!(
                ver.contains("2027.3.5"),
                "File {} did not update version cleanly (got {:?})",
                st.rel_path,
                st.detected_version
            );

            if let Some(&expected_lines) = orig_line_counts.get(&st.key) {
                let actual_lines = read_file_robust(&st.absolute_path).unwrap().lines().count();
                assert_eq!(
                    actual_lines, expected_lines,
                    "Line count changed after update in {}",
                    st.rel_path
                );
            }

            // Confirm zero stale active release markers remain in updatable files
            if st.is_updatable {
                let updated_content = read_file_robust(&st.absolute_path).unwrap();
                for stale in [
                    "10/11/2026",
                    "10 de novembro de 2026",
                    "November 10, 2026",
                    "Nov 10, 2026",
                    "11/11/2026",
                    "11 de novembro de 2026",
                    "November 11, 2026",
                    "Nov 11, 2026",
                ] {
                    assert!(
                        !updated_content.contains(stale),
                        "Stale date token `{}` still present in {}",
                        stale,
                        st.rel_path
                    );
                }
                if !st.key.starts_with("history_") {
                    for stale_v in ["2026.11.10", "2026.11.11"] {
                        assert!(
                            !updated_content.contains(stale_v),
                            "Stale version token `{}` still present in {}",
                            stale_v,
                            st.rel_path
                        );
                    }
                }
            }
        }

        // Verify renamed release doc and updated test reference match
        assert!(tmp.join("docs/release-v2027.3.5.1.md").is_file());
        let orig_release = find_release_doc_rel_path(&root);
        assert!(!tmp.join(orig_release).exists());

        let _ = fs::remove_dir_all(&tmp);
    }
}
