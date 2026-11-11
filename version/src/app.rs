use std::path::PathBuf;
use chrono::{Datelike, Local, NaiveDate};
use egui::{Color32, RichText, ScrollArea, Vec2};
use crate::engine::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    General,
    PrivacyPolicy,
    Eula,
    Clc,
    Copyright,
    About,
    Notices,
    Docs,
    Root,
    SrcRust,
    Manifests,
    Repo,
}

pub struct VersionEditorApp {
    pub base_dir: PathBuf,
    pub base_dir_str: String,
    pub pt: LangInput,
    pub en: LangInput,

    pub statuses: Vec<FileCheckStatus>,
    pub update_results: Vec<UpdateResult>,
    pub active_tab: ActiveTab,
    pub status_message: Option<(String, bool)>, // (message, is_success)
    pub show_save_modal: bool,
    pub save_modal_summary: String,
}

impl Default for VersionEditorApp {
    fn default() -> Self {
        let base_dir = project_root_dir();
        let base_dir_str = base_dir.to_string_lossy().to_string();

        let mut app = Self {
            base_dir,
            base_dir_str,
            pt: LangInput::default(),
            en: LangInput::default(),
            statuses: Vec::new(),
            update_results: Vec::new(),
            active_tab: ActiveTab::General,
            status_message: None,
            show_save_modal: false,
            save_modal_summary: String::new(),
        };

        app.set_today_all();
        app.check_registered_files();
        app.load_versions_from_disk();
        app
    }
}

impl VersionEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn check_registered_files(&mut self) {
        self.statuses = check_all_files(&self.base_dir);
    }

    pub fn set_today_all(&mut self) {
        let today = Local::now().date_naive();
        let day = today.day();
        let month_index = today.month() - 1;
        let year = today.year();

        let set_input = |inp: &mut LangInput| {
            inp.day = day;
            inp.month_index = month_index;
            inp.year = year;
            inp.version = format_version_from_date(year, month_index + 1, day, &inp.version);
        };

        set_input(&mut self.pt);
        set_input(&mut self.en);
    }

    pub fn sync_pt_version(&mut self) {
        let pt_ver = plain_version(&self.pt.version);
        let d = self.pt.day;
        let m = self.pt.month_index;
        let y = self.pt.year;

        let sync = |target: &mut LangInput| {
            target.version = pt_ver.clone();
            target.day = d;
            target.month_index = m;
            target.year = y;
        };

        sync(&mut self.en);

        self.status_message = Some(("Versão e data PT sincronizadas para Inglês (en-US).".to_string(), true));
    }

    pub fn check_files(&mut self) {
        self.statuses = check_all_files(&self.base_dir);
    }

    pub fn load_versions_from_disk(&mut self) {
        let statuses = check_all_files(&self.base_dir);

        let readme_path = self.base_dir.join("README.md");
        let mut readme_loaded: std::collections::HashMap<&'static str, (Option<String>, Option<(u32, u32, i32)>)> = std::collections::HashMap::new();
        if let Ok(readme_txt) = read_file_robust(&readme_path) {
            let clean_readme = readme_txt.replace('\r', "");
            let lang_map = [
                ("pt", "ptbr"),
                ("en", "enus"),
            ];
            for (lang, section_id) in lang_map {
                let (ver, date_opt) = extract_readme_section_version_and_date(&clean_readme, section_id);
                if ver.is_some() || date_opt.is_some() {
                    readme_loaded.insert(lang, (ver, date_opt));
                }
            }
        }

        let try_populate = |list: &[FileCheckStatus], lang: &str, inp: &mut LangInput| {
            if let Some((ver_opt, date_opt)) = readme_loaded.get(lang) {
                if let Some(ref v) = ver_opt {
                    inp.version = plain_version(v);
                }
                if let Some((d, m_idx, y)) = *date_opt {
                    inp.day = d;
                    inp.month_index = m_idx;
                    inp.year = y;
                } else if let Some(ref v) = ver_opt {
                    if let Some((vd, vm, vy, _)) = parse_version_to_date(v) {
                        inp.day = vd;
                        inp.month_index = vm;
                        inp.year = vy;
                    }
                }
                if ver_opt.is_some() && date_opt.is_some() {
                    return;
                }
            }

            for st in list {
                if !st.is_updatable {
                    continue;
                }
                if (st.lang == lang || (lang == "pt" && st.lang == "all")) && st.exists {
                    let mut date_ok = false;
                    if let Some(ref d_str) = st.detected_date {
                        if let Some((d, m_idx, y)) = parse_date_string(d_str) {
                            inp.day = d;
                            inp.month_index = m_idx;
                            inp.year = y;
                            date_ok = true;
                        }
                    }

                    if let Some(ref v_str) = st.detected_version {
                        inp.version = plain_version(v_str);
                        if !date_ok {
                            if let Some((vd, vm, vy, _)) = parse_version_to_date(v_str) {
                                inp.day = vd;
                                inp.month_index = vm;
                                inp.year = vy;
                            }
                        }
                    }

                    if st.detected_version.is_some() || date_ok {
                        break;
                    }
                }
            }
        };

        try_populate(&statuses, "pt", &mut self.pt);
        try_populate(&statuses, "en", &mut self.en);
    }

    pub fn save_all(&mut self) {
        self.update_results = apply_updates(
            &self.pt,
            &self.en,
            &self.base_dir,
        );

        let success_count = self.update_results.iter().filter(|r| r.ok).count();
        let total = self.update_results.len();

        self.save_modal_summary = format!(
            "Atualização concluída com sucesso!\n\n\
            • {} de {} arquivos processados com sucesso.\n\
            • Versão PT gravada: {}\n\
            • Data base: {}/{}/{}\n\
            • Os metadados em código fonte (.rs), manifestos (.toml, .lock), instaladores (.iss) e documentações (.md, .txt) foram atualizados.",
            success_count, total, self.pt.version, self.pt.day, self.pt.month_index + 1, self.pt.year
        );
        self.show_save_modal = true;

        // Reload data from disk
        self.check_files();
        self.status_message = Some((
            format!("{} de {} arquivos gravados e atualizados com sucesso.", success_count, total),
            true,
        ));
    }

    fn render_lang_card(ui: &mut egui::Ui, title: &str, months: &[&str; 12], inp: &mut LangInput) {
        ui.group(|ui| {
            ui.set_min_width(460.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong().size(14.0));
            });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Versão:");
                let ver_resp = ui.add(egui::TextEdit::singleline(&mut inp.version).desired_width(120.0));
                if ver_resp.lost_focus() || ver_resp.changed() {
                    if let Some((d, m_idx, y, _)) = parse_version_to_date(&inp.version) {
                        inp.day = d;
                        inp.month_index = m_idx;
                        inp.year = y;
                    }
                }

                ui.add_space(8.0);
                ui.label("Dia:");
                let max_days = NaiveDate::from_ymd_opt(inp.year, inp.month_index + 1, 1)
                    .map(|d| {
                        let next_month = if inp.month_index == 11 {
                            NaiveDate::from_ymd_opt(inp.year + 1, 1, 1)
                        } else {
                            NaiveDate::from_ymd_opt(inp.year, inp.month_index + 2, 1)
                        };
                        next_month.map(|nm| nm.signed_duration_since(d).num_days() as u32).unwrap_or(31)
                    })
                    .unwrap_or(31);

                let mut day_changed = false;
                let day_widget = egui::DragValue::new(&mut inp.day).range(1..=max_days);
                if ui.add(day_widget).changed() {
                    day_changed = true;
                }

                ui.label("Mês:");
                let mut month_changed = false;
                let selected_text = months.get(inp.month_index as usize).copied().unwrap_or("Janeiro");
                egui::ComboBox::from_id_salt(title)
                    .selected_text(selected_text)
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        for (idx, &m_name) in months.iter().enumerate() {
                            if ui.selectable_value(&mut inp.month_index, idx as u32, m_name).clicked() {
                                month_changed = true;
                            }
                        }
                    });

                ui.label("Ano:");
                let mut year_changed = false;
                let year_widget = egui::DragValue::new(&mut inp.year).range(1900..=3000);
                if ui.add(year_widget).changed() {
                    year_changed = true;
                }

                if day_changed || month_changed || year_changed {
                    inp.version = format_version_from_date(inp.year, inp.month_index + 1, inp.day, &inp.version);
                }
            });
        });
    }

    fn render_status_text(ui: &mut egui::Ui, text: &str) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut text.to_string())
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(24)
                    .interactive(true),
            );
        });
    }

    fn render_file_table(ui: &mut egui::Ui, files: &[FileCheckStatus]) {
        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("file_status_grid")
                .striped(true)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Status").strong());
                    ui.label(RichText::new("Arquivo").strong());
                    ui.label(RichText::new("Grupo").strong());
                    ui.label(RichText::new("Versão Detectada").strong());
                    ui.label(RichText::new("Data Detectada").strong());
                    ui.label(RichText::new("Gravação").strong());
                    ui.end_row();

                    for st in files {
                        let (icon, color) = if !st.exists {
                            ("❌ Não Encontrado", Color32::from_rgb(220, 80, 80))
                        } else if !st.missing_patterns.is_empty() {
                            ("⚠️ Pendente", Color32::from_rgb(230, 180, 40))
                        } else {
                            ("✅ OK", Color32::from_rgb(80, 200, 100))
                        };

                        ui.label(RichText::new(icon).color(color));
                        ui.label(RichText::new(&st.rel_path).monospace());
                        ui.label(&st.group);
                        ui.label(st.detected_version.as_deref().unwrap_or("N/D"));
                        ui.label(st.detected_date.as_deref().unwrap_or("N/D"));
                        if st.is_updatable {
                            ui.label("Automática");
                        } else {
                            ui.label(RichText::new("Somente Leitura").color(Color32::GRAY));
                        }
                        ui.end_row();
                    }
                });
        });
    }

    pub fn render_editor(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("🦀 Editor de Versões e Datas Multi-idioma");
            ui.add_space(8.0);
            ui.label(RichText::new("GeoEstabil (Rust Edition)").color(Color32::LIGHT_BLUE));
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Pasta Raiz do Projeto:").strong());
            ui.add(egui::TextEdit::singleline(&mut self.base_dir_str).desired_width(550.0));

            if ui.button("📁 Selecionar...").clicked() {
                if let Some(folder) = rfd::FileDialog::new().set_directory(&self.base_dir).pick_folder() {
                    self.base_dir = folder;
                    self.base_dir_str = self.base_dir.to_string_lossy().to_string();
                    self.check_files();
                    self.load_versions_from_disk();
                }
            }

            if ui.button("🔍 Atualizar Raiz").clicked() {
                self.base_dir = PathBuf::from(&self.base_dir_str);
                self.check_files();
            }
        });
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // 1. Language inputs (PT-BR & EN-US)
        ui.horizontal_wrapped(|ui| {
            Self::render_lang_card(ui, "🇧🇷 Português (pt-BR)", &PT_MONTHS, &mut self.pt);
            ui.add_space(8.0);
            Self::render_lang_card(ui, "🇺🇸 English (en-US)", &EN_MONTHS, &mut self.en);
        });

        ui.add_space(8.0);

        // 2. Action buttons
        ui.horizontal(|ui| {
            if ui.button(RichText::new("🔍 Verificar Status dos Arquivos").strong()).clicked() {
                self.check_files();
                self.status_message = Some(("Verificação de status concluída com sucesso.".to_string(), true));
            }

            if ui.button(RichText::new("🔄 Recarregar Dados").strong()).clicked() {
                self.load_versions_from_disk();
                self.check_files();
                self.status_message = Some(("Dados recarregados a partir dos arquivos em disco.".to_string(), true));
            }

            if ui.button(RichText::new("💾 Salvar Alterações em Todos").strong().color(Color32::from_rgb(255, 220, 100))).clicked() {
                self.save_all();
            }

            if ui.button("🔗 Sincronizar Versão PT").clicked() {
                self.sync_pt_version();
            }

            if ui.button("📅 Hoje para Todos").clicked() {
                self.set_today_all();
                self.status_message = Some(("Data presente do sistema configurada em todos os idiomas.".to_string(), true));
            }
        });

        if let Some((ref msg, is_ok)) = self.status_message {
            ui.add_space(4.0);
            let color = if is_ok { Color32::from_rgb(100, 220, 130) } else { Color32::from_rgb(230, 80, 80) };
            ui.label(RichText::new(msg).color(color));
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        // 3. Tab Bar
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.active_tab, ActiveTab::General, "📊 Geral");
            ui.selectable_value(&mut self.active_tab, ActiveTab::PrivacyPolicy, "🔒 PRIVACY_POLICY");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Eula, "📜 EULA");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Clc, "💼 CLC");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Copyright, "©️ COPYRIGHT");
            ui.selectable_value(&mut self.active_tab, ActiveTab::About, "ℹ️ ABOUT");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Notices, "📢 NOTICES");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Docs, "📚 DOCS");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Root, "📁 RAIZ");
            ui.selectable_value(&mut self.active_tab, ActiveTab::SrcRust, "🦀 SRC_RUST & TESTS");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Manifests, "⚙️ MANIFESTS");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Repo, "📦 REPO");
        });

        ui.add_space(6.0);

        // 4. Tab contents
        match self.active_tab {
            ActiveTab::General => {
                let today = Local::now().date_naive();
                let today_ver = format_version_from_date(today.year(), today.month(), today.day(), "");
                let total_files = self.statuses.len();
                let present_files = self.statuses.iter().filter(|s| s.exists).count();
                let pending_files = self.statuses.iter().filter(|s| s.exists && !s.missing_patterns.is_empty()).count();

                let mut general_lines = Vec::new();
                general_lines.push("=== Resumo Geral da Verificação ===".to_string());
                general_lines.push(format!("Data Presente do Sistema: {} (Dia: {}, Mês: {}, Ano: {})",
                    today.format("%d/%m/%Y"), today.day(), today.month(), today.year()));
                general_lines.push(format!("Versão Atual Composta: {}", today_ver));
                general_lines.push(format!("Diretório Base: {}", self.base_dir.display()));
                general_lines.push(format!("Total de Arquivos Monitorados: {}", total_files));
                general_lines.push(format!("Arquivos Presentes: {} / {}", present_files, total_files));
                general_lines.push(format!("Arquivos com Padrões Pendentes: {}", pending_files));
                general_lines.push("".to_string());
                general_lines.push("=== Configuração Atual de Versão e Data nos Idiomas ===".to_string());
                general_lines.push(format!("  • PT: Versão: {} | Data: {}/{}/{}", self.pt.version, self.pt.day, self.pt.month_index + 1, self.pt.year));
                general_lines.push(format!("  • EN: Versão: {} | Data: {}/{}/{}", self.en.version, self.en.day, self.en.month_index + 1, self.en.year));

                if !self.update_results.is_empty() {
                    general_lines.push("".to_string());
                    general_lines.push("=== Último Relatório de Gravação ===".to_string());
                    for r in &self.update_results {
                        general_lines.push(format!("[{}] {} ({}) -> {}", if r.ok { "OK" } else { "FALHA" }, r.rel_path, r.group, r.msg));
                    }
                }

                general_lines.push("".to_string());
                general_lines.push("Status por Arquivo:".to_string());
                for st in &self.statuses {
                    let tag = if !st.exists {
                        "[NÃO ENCONTRADO]"
                    } else if !st.missing_patterns.is_empty() {
                        "[PENDENTE]"
                    } else {
                        "[OK]"
                    };
                    let ro_tag = if st.is_updatable { "" } else { " [SOMENTE LEITURA]" };
                    general_lines.push(format!("{:<16} {} ({}){} - Versão: {}, Data: {}",
                        tag, st.rel_path, st.group, ro_tag,
                        st.detected_version.as_deref().unwrap_or("N/D"),
                        st.detected_date.as_deref().unwrap_or("N/D")));
                }

                Self::render_status_text(ui, &general_lines.join("\n"));
            }
            ActiveTab::PrivacyPolicy => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "PRIVACY_POLICY").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Eula => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "EULA").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Clc => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "CLC").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Copyright => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "COPYRIGHT").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::About => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "ABOUT").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Notices => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "NOTICES").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Docs => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "DOCS").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Root => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| matches!(s.group.as_str(), "RAIZ" | "GITHUB")).cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::SrcRust => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| matches!(s.group.as_str(), "SRC_RUST" | "TESTS")).cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Manifests => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "MANIFESTS").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
            ActiveTab::Repo => {
                let group_files: Vec<FileCheckStatus> = self.statuses.iter().filter(|s| s.group == "REPO").cloned().collect();
                Self::render_file_table(ui, &group_files);
            }
        }

        // 5. Save summary modal dialog
        if self.show_save_modal {
            egui::Window::new("💾 Atualização Concluída")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(&self.save_modal_summary).size(13.0));
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(150.0);
                        if ui.button(RichText::new("OK").strong().size(14.0)).clicked() {
                            self.show_save_modal = false;
                        }
                    });
                    ui.add_space(8.0);
                });
        }
    }
}

impl eframe::App for VersionEditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !ui.visuals().dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        }

        self.render_editor(ui, &ctx);
    }
}
