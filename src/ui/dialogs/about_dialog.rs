use crate::locale::translations::Localizer;
use crate::ui::dialogs::doc_renderer::{build_highlighted_job, count_matches_ci};
use crate::ui::dialogs::modeless::{show_modeless_dialog, ModelessState};
use crate::ui::theme::GeoEstabilTheme;
use crate::utils::paths::{load_asset_bytes_or, load_asset_text_or};
use eframe::egui;

struct LicenseEntry {
    label_pt: &'static str,
    label_en: &'static str,
    rel_path: &'static str,
    fallback: &'static str,
}

impl LicenseEntry {
    fn label(&self, pt: bool) -> &'static str {
        if pt { self.label_pt } else { self.label_en }
    }
}

const LICENSE_ENTRIES: &[LicenseEntry] = &[
    LicenseEntry {
        label_pt: "GNU AGPLv3 (Repositório GitHub — Licença Principal do Projeto)",
        label_en: "GNU AGPLv3 (GitHub Repository — Primary Project License)",
        rel_path: "LICENSES/GNU AFFERO GENERAL PUBLIC LICENSE Version3.txt",
        fallback: include_str!("../../assets/LICENSES/GNU AFFERO GENERAL PUBLIC LICENSE Version3.txt"),
    },
    LicenseEntry {
        label_pt: "Apache License 2.0 (wgpu, winit, egui, self_cell)",
        label_en: "Apache License 2.0 (wgpu, winit, egui, self_cell)",
        rel_path: "LICENSES/Apache License Version 2.0.txt",
        fallback: include_str!("../../assets/LICENSES/Apache License Version 2.0.txt"),
    },
    LicenseEntry {
        label_pt: "MIT License (egui, rusqlite, rfd, serde, rayon)",
        label_en: "MIT License (egui, rusqlite, rfd, serde, rayon)",
        rel_path: "LICENSES/MIT License.txt",
        fallback: include_str!("../../assets/LICENSES/MIT License.txt"),
    },
    LicenseEntry {
        label_pt: "BSD 3-Clause License (moxcms, pxfm)",
        label_en: "BSD 3-Clause License (moxcms, pxfm)",
        rel_path: "LICENSES/BSD License Clause-3.txt",
        fallback: include_str!("../../assets/LICENSES/BSD License Clause-3.txt"),
    },
    LicenseEntry {
        label_pt: "BSD 2-Clause License (zerocopy)",
        label_en: "BSD 2-Clause License (zerocopy)",
        rel_path: "LICENSES/BSD License Clause-2.txt",
        fallback: include_str!("../../assets/LICENSES/BSD License Clause-2.txt"),
    },
    LicenseEntry {
        label_pt: "Boost Software License 1.0 (clipboard-win)",
        label_en: "Boost Software License 1.0 (clipboard-win)",
        rel_path: "LICENSES/Boost Software License 1.0.txt",
        fallback: include_str!("../../assets/LICENSES/Boost Software License 1.0.txt"),
    },
    LicenseEntry {
        label_pt: "ISC License (libloading)",
        label_en: "ISC License (libloading)",
        rel_path: "LICENSES/ISC LICENSE.txt",
        fallback: include_str!("../../assets/LICENSES/ISC LICENSE.txt"),
    },
    LicenseEntry {
        label_pt: "Zlib License (foldhash / miniz)",
        label_en: "Zlib License (foldhash / miniz)",
        rel_path: "LICENSES/Zlib License.txt",
        fallback: include_str!("../../assets/LICENSES/Zlib License.txt"),
    },
    LicenseEntry {
        label_pt: "Unicode License v3 (icu4x crates)",
        label_en: "Unicode License v3 (icu4x crates)",
        rel_path: "LICENSES/Unicode License v3.txt",
        fallback: include_str!("../../assets/LICENSES/Unicode License v3.txt"),
    },
    LicenseEntry {
        label_pt: "SIL Open Font License 1.1 (epaint fonts)",
        label_en: "SIL Open Font License 1.1 (epaint fonts)",
        rel_path: "LICENSES/SIL Open Font License 1.1.txt",
        fallback: include_str!("../../assets/LICENSES/SIL Open Font License 1.1.txt"),
    },
    LicenseEntry {
        label_pt: "Ubuntu Font Licence 1.0 (epaint default font)",
        label_en: "Ubuntu Font Licence 1.0 (epaint default font)",
        rel_path: "LICENSES/Ubuntu Font Licence 1.0.txt",
        fallback: include_str!("../../assets/LICENSES/Ubuntu Font Licence 1.0.txt"),
    },
    LicenseEntry {
        label_pt: "SQLite Public Domain Dedication",
        label_en: "SQLite Public Domain Dedication",
        rel_path: "LICENSES/SQLite Is Public Domain.txt",
        fallback: include_str!("../../assets/LICENSES/SQLite Is Public Domain.txt"),
    },
    LicenseEntry {
        label_pt: "GNU GPLv3 (Referência de Compatibilidade)",
        label_en: "GNU GPLv3 (Compatibility Reference)",
        rel_path: "LICENSES/GNU GENERAL PUBLIC LICENSE Version 3.txt",
        fallback: include_str!("../../assets/LICENSES/GNU GENERAL PUBLIC LICENSE Version 3.txt"),
    },
    LicenseEntry {
        label_pt: "GNU LGPLv3 (Referência)",
        label_en: "GNU LGPLv3 (Reference)",
        rel_path: "LICENSES/GNU LESSER GENERAL PUBLIC LICENSE Version 3.txt",
        fallback: include_str!("../../assets/LICENSES/GNU LESSER GENERAL PUBLIC LICENSE Version 3.txt"),
    },
    LicenseEntry {
        label_pt: "GNU LGPLv2.1 (Referência)",
        label_en: "GNU LGPLv2.1 (Reference)",
        rel_path: "LICENSES/GNU LESSER GENERAL PUBLIC LICENSE Version 2.1.txt",
        fallback: include_str!("../../assets/LICENSES/GNU LESSER GENERAL PUBLIC LICENSE Version 2.1.txt"),
    },
    LicenseEntry {
        label_pt: "Python Software Foundation License (Referência)",
        label_en: "Python Software Foundation License (Reference)",
        rel_path: "LICENSES/PYTHON SOFTWARE FOUNDATION LICENSE.txt",
        fallback: include_str!("../../assets/LICENSES/PYTHON SOFTWARE FOUNDATION LICENSE.txt"),
    },
];

#[derive(Default)]
pub struct AboutDialog {
    pub modeless: ModelessState,
    pub selected_tab: usize,
    pub selected_license_idx: usize,
    pub selected_history_sub: usize,
    pub search_query: String,
    pub active_match_index: usize,
    pub copied_feedback_timer: f64,
    scroll_to_match_requested: bool,
    logo_texture: Option<egui::TextureHandle>,
}

impl AboutDialog {
    pub fn open(&mut self) {
        self.modeless.open();
    }

    pub fn show(&mut self, ctx: &egui::Context, localizer: &Localizer, dark_mode: bool) {
        self.ensure_logo(ctx);
        let mut modeless_state = self.modeless.clone();
        let title = localizer.t("about.title").to_string();

        show_modeless_dialog(
            ctx,
            "geoestabil_about_dialog",
            &title,
            [1060.0, 720.0],
            [740.0, 520.0],
            dark_mode,
            &mut modeless_state,
            |ui, close_clicked| {
                self.render_body(ui, localizer, dark_mode, close_clicked);
            },
        );

        self.modeless = modeless_state;
    }

    fn ensure_logo(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() {
            return;
        }
        let embedded = include_bytes!("../../assets/icons/geoestabil.png");
        let bytes = load_asset_bytes_or("icons/geoestabil.png", embedded);
        let decoded = image::load_from_memory(bytes.as_ref())
            .or_else(|_| image::load_from_memory(embedded));
        if let Ok(img) = decoded {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let color_img = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            let tex = ctx.load_texture(
                "geoestabil_about_logo",
                color_img,
                egui::TextureOptions::LINEAR,
            );
            self.logo_texture = Some(tex);
        }
    }

    fn load_asset_text(rel_path: &str, fallback: &'static str) -> String {
        load_asset_text_or(rel_path, fallback)
    }

    fn active_tab_text(&self, localizer: &Localizer) -> String {
        let pt = localizer.is_pt();
        match self.selected_tab {
            0 => Self::load_asset_text(
                if pt {
                    "ABOUT/ABOUT_pt_BR.txt"
                } else {
                    "ABOUT/ABOUT_en_US.txt"
                },
                if pt {
                    include_str!("../../assets/ABOUT/ABOUT_pt_BR.txt")
                } else {
                    include_str!("../../assets/ABOUT/ABOUT_en_US.txt")
                },
            ),
            1 => {
                if pt {
                    "AUTORIA, LICENCIAMENTO E CANAL OFICIAL DE SUPORTE\n\n\
                     • Desenvolvedor e Titular Exclusivo: Fernando Nillsson Cidade\n\
                     • E-mail Oficial de Suporte: linceu_lighthouse@outlook.com\n\
                     • Código-Fonte Oficial (GNU AGPLv3): https://github.com/fernandoncidade\n\
                     • AppUserModelID (Windows 11): fernandoncidade.GeoEstabil\n\n\
                     POLÍTICA DE SUPORTE E PRIVACIDADE LOCAL:\n\
                     • Canal Único de Atendimento: O suporte ocorre exclusivamente via e-mail (linceu_lighthouse@outlook.com), em regime de melhor esforço conforme a disponibilidade do autor e sem obrigação de urgência, prazo mínimo ou SLA (sem atendimento telefônico, acesso remoto ou consultoria).\n\
                     • Privacidade Local (100% Offline): O GeoEstabil processa tudo localmente no dispositivo do usuário, sem telemetria e sem conexão externa; aquisições na Microsoft Store seguem as regras de faturamento e privacidade da própria plataforma Microsoft.\n\
                     • Uso Exclusivamente Acadêmico (AS IS): Ferramenta auxiliar voltada a estudo acadêmico, ensino e experimentação computacional, podendo conter erros numéricos. Não substitui conferência manual, ferramentas consolidadas de mercado ou profissional habilitado (ART/RRT), ficando o autor isento por qualquer uso indevido ou aplicação definitiva em produção.\n\n\
                     MODELO SIMPLES DE DUPLO LICENCIAMENTO:\n\
                     1) Código-Fonte Público (GitHub) -> GNU AGPLv3 (AGPL-3.0-only), permitindo uso, estudo, modificação e compartilhamento sob a mesma licença, com proteção nominal de nome e marca (Seção 7e).\n\
                     2) Binários Microsoft Store -> EULA Comercial (src/assets/EULA/) e CLC (src/assets/CLC/), preservando direitos legais irrenunciáveis aplicáveis."
                        .to_string()
                } else {
                    "AUTHORSHIP, LICENSING & OFFICIAL SUPPORT CHANNEL\n\n\
                     • Developer & Exclusive Copyright Holder: Fernando Nillsson Cidade\n\
                     • Official Support Email: linceu_lighthouse@outlook.com\n\
                     • Open-Source Repository (GNU AGPLv3): https://github.com/fernandoncidade\n\
                     • Windows AppUserModelID: fernandoncidade.GeoEstabil\n\n\
                     SUPPORT POLICY & LOCAL PRIVACY:\n\
                     • Single Support Channel: Technical support is provided exclusively via email (linceu_lighthouse@outlook.com), on a best-effort basis according to the author's availability and with no obligation of urgency, timeline, or SLA (no phone support, remote access, or engineering consulting).\n\
                     • Local Privacy (100% Offline): GeoEstabil runs strictly locally on the user's computer with zero telemetry or network traffic; Microsoft Store purchases follow Microsoft's own billing and privacy policies.\n\
                     • Academic Study Use Only (AS IS): Auxiliary tool intended for academic study, teaching, and computational experimentation, which may contain bugs or numerical limitations. It does not replace manual checks, established market tools, or licensed professionals (ART/RRT), and the author is exempt from any improper or production use.\n\n\
                     SIMPLE DUAL-LICENSING FRAMEWORK:\n\
                     1) Public Source Code (GitHub) -> GNU AGPLv3 (AGPL-3.0-only), permitting free use, study, modification, and sharing under the same license, with Section 7(e) name/trademark protection.\n\
                     2) Commercial Microsoft Store Builds -> Commercial EULA (src/assets/EULA/) and CLC (src/assets/CLC/), preserving non-waivable statutory rights where applicable."
                        .to_string()
                }
            }
            2 => {
                if self.selected_history_sub == 0 {
                    Self::load_asset_text(
                        if pt {
                            "ABOUT/History_APP_pt_BR.txt"
                        } else {
                            "ABOUT/History_APP_en_US.txt"
                        },
                        if pt {
                            include_str!("../../assets/ABOUT/History_APP_pt_BR.txt")
                        } else {
                            include_str!("../../assets/ABOUT/History_APP_en_US.txt")
                        },
                    )
                } else {
                    Self::load_asset_text(
                        if pt {
                            "RELEASE/RELEASE NOTES_pt_BR.txt"
                        } else {
                            "RELEASE/RELEASE NOTES_en_US.txt"
                        },
                        if pt {
                            include_str!("../../assets/RELEASE/RELEASE NOTES_pt_BR.txt")
                        } else {
                            include_str!("../../assets/RELEASE/RELEASE NOTES_en_US.txt")
                        },
                    )
                }
            }
            3 => Self::load_asset_text(
                if pt {
                    "EULA/EULA_pt_BR - GeoEstabil.txt"
                } else {
                    "EULA/EULA_en_US - GeoEstabil.txt"
                },
                if pt {
                    include_str!("../../assets/EULA/EULA_pt_BR - GeoEstabil.txt")
                } else {
                    include_str!("../../assets/EULA/EULA_en_US - GeoEstabil.txt")
                },
            ),
            4 => Self::load_asset_text(
                if pt {
                    "CLC/CLC_pt_BR - GeoEstabil.txt"
                } else {
                    "CLC/CLC_en_US - GeoEstabil.txt"
                },
                if pt {
                    include_str!("../../assets/CLC/CLC_pt_BR - GeoEstabil.txt")
                } else {
                    include_str!("../../assets/CLC/CLC_en_US - GeoEstabil.txt")
                },
            ),
            5 => Self::load_asset_text(
                if pt {
                    "COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt"
                } else {
                    "COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt"
                },
                if pt {
                    include_str!("../../assets/COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt")
                } else {
                    include_str!("../../assets/COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt")
                },
            ),
            6 => Self::load_asset_text(
                if pt {
                    "PRIVACY_POLICY/Privacy_Policy_pt_BR.txt"
                } else {
                    "PRIVACY_POLICY/Privacy_Policy_en_US.txt"
                },
                if pt {
                    include_str!("../../assets/PRIVACY_POLICY/Privacy_Policy_pt_BR.txt")
                } else {
                    include_str!("../../assets/PRIVACY_POLICY/Privacy_Policy_en_US.txt")
                },
            ),
            7 => Self::load_asset_text(
                if pt {
                    "NOTICES/NOTICE_pt_BR.txt"
                } else {
                    "NOTICES/NOTICE_en_US.txt"
                },
                if pt {
                    include_str!("../../assets/NOTICES/NOTICE_pt_BR.txt")
                } else {
                    include_str!("../../assets/NOTICES/NOTICE_en_US.txt")
                },
            ),
            _ => {
                let entry = &LICENSE_ENTRIES[self.selected_license_idx.min(LICENSE_ENTRIES.len() - 1)];
                Self::load_asset_text(entry.rel_path, entry.fallback)
            }
        }
    }

    fn render_body(
        &mut self,
        ui: &mut egui::Ui,
        localizer: &Localizer,
        dark_mode: bool,
        close_clicked: &mut bool,
    ) {
        let pt = localizer.is_pt();
        let accent = GeoEstabilTheme::accent_blue(dark_mode);
        let text_sec = GeoEstabilTheme::text_secondary(dark_mode);

        // Cabeçalho com Logo + Identidade Institucional
        egui::Frame::new()
            .fill(GeoEstabilTheme::card_bg(dark_mode))
            .stroke(egui::Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(tex) = &self.logo_texture {
                        ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(64.0, 64.0)));
                        ui.add_space(8.0);
                    }
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(localizer.t("about.header"))
                                .size(16.5)
                                .strong()
                                .color(accent),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{} v{}.0 | Rust 2021 • egui 0.37 / wgpu • Rayon",
                                if pt { "Versão" } else { "Release Version" },
                                env!("CARGO_PKG_VERSION"),
                            ))
                            .size(12.5)
                            .strong(),
                        );
                        ui.label(
                            egui::RichText::new(if pt {
                                "Autor: Fernando Nillsson Cidade | Duplo Licenciamento: GNU AGPLv3 (GitHub) & EULA Comercial (Microsoft Store)"
                            } else {
                                "Author: Fernando Nillsson Cidade | Dual-Licensed: GNU AGPLv3 (GitHub) & Commercial EULA (Microsoft Store)"
                            })
                            .size(11.8)
                            .color(text_sec),
                        );
                        ui.label(
                            egui::RichText::new(
                                "ABNT NBR 11682:2009 • ABNT NBR 6122:2022 • ABNT NBR 6118:2023 • ABNT NBR 6484:2020",
                            )
                            .size(11.5)
                            .color(GeoEstabilTheme::soil_brown(dark_mode)),
                        );
                    });
                });
            });

        ui.add_space(6.0);

        // Barra das 9 Abas
        let tabs = [
            localizer.t("about.tab.overview"),
            localizer.t("about.tab.author"),
            localizer.t("about.tab.history"),
            localizer.t("about.tab.eula"),
            localizer.t("about.tab.clc"),
            localizer.t("about.tab.copyright"),
            localizer.t("about.tab.privacy"),
            localizer.t("about.tab.notices"),
            localizer.t("about.tab.licenses"),
        ];

        ui.horizontal_wrapped(|ui| {
            for (idx, lbl) in tabs.iter().enumerate() {
                if ui.selectable_label(self.selected_tab == idx, *lbl).clicked() {
                    self.selected_tab = idx;
                    self.active_match_index = 0;
                }
            }
        });

        ui.add_space(4.0);

        // Controles específicos das abas Histórico (Histórico vs Release Notes) e Licenças
        if self.selected_tab == 2 {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        self.selected_history_sub == 0,
                        if pt { "📜 Histórico Técnico" } else { "📜 Architecture History" },
                    )
                    .clicked()
                {
                    self.selected_history_sub = 0;
                    self.active_match_index = 0;
                }
                if ui
                    .selectable_label(
                        self.selected_history_sub == 1,
                        if pt { "🚀 Notas de Lançamento (2026.11.11.0)" } else { "🚀 Release Notes (2026.11.11.0)" },
                    )
                    .clicked()
                {
                    self.selected_history_sub = 1;
                    self.active_match_index = 0;
                }
            });
        } else if self.selected_tab == 8 {
            ui.horizontal(|ui| {
                ui.label(if pt { "Documento de Licença:" } else { "License Document:" });
                let cur_lbl = LICENSE_ENTRIES[self.selected_license_idx.min(LICENSE_ENTRIES.len() - 1)].label(pt);
                egui::ComboBox::from_id_salt("about_license_combo")
                    .selected_text(cur_lbl)
                    .width(480.0)
                    .show_ui(ui, |ui| {
                        for (idx, item) in LICENSE_ENTRIES.iter().enumerate() {
                            if ui
                                .selectable_value(&mut self.selected_license_idx, idx, item.label(pt))
                                .clicked()
                            {
                                self.active_match_index = 0;
                            }
                        }
                    });
            });
        }

        let content = self.active_tab_text(localizer);
        let total_hits = count_matches_ci(&content, &self.search_query);

        // Barra de busca + Copiar + Fechar
        ui.horizontal(|ui| {
            ui.label(localizer.t("dialog.search"));
            let search_edit = ui.add_sized(
                [240.0, 23.0],
                egui::TextEdit::singleline(&mut self.search_query).hint_text(if pt {
                    "Pesquisar neste documento..."
                } else {
                    "Search in document..."
                }),
            );
            if search_edit.changed() {
                self.active_match_index = 0;
                self.scroll_to_match_requested = true;
            }
            if search_edit.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && total_hits > 0
            {
                if ui.input(|i| i.modifiers.shift) {
                    self.active_match_index =
                        (self.active_match_index + total_hits - 1) % total_hits;
                } else {
                    self.active_match_index = (self.active_match_index + 1) % total_hits;
                }
                self.scroll_to_match_requested = true;
            }

            if !self.search_query.trim().is_empty() {
                if ui.button("◀").clicked() && total_hits > 0 {
                    self.active_match_index =
                        (self.active_match_index + total_hits - 1) % total_hits;
                    self.scroll_to_match_requested = true;
                }
                if ui.button("▶").clicked() && total_hits > 0 {
                    self.active_match_index = (self.active_match_index + 1) % total_hits;
                    self.scroll_to_match_requested = true;
                }
                let disp = if total_hits == 0 {
                    "0/0".to_string()
                } else {
                    format!(
                        "{}/{}",
                        (self.active_match_index % total_hits) + 1,
                        total_hits
                    )
                };
                ui.colored_label(GeoEstabilTheme::warning_amber(dark_mode), disp);
                if ui.button("✖").clicked() {
                    self.search_query.clear();
                    self.active_match_index = 0;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(format!("❌ {}", localizer.t("dialog.close")))
                    .clicked()
                {
                    *close_clicked = true;
                }
                if ui
                    .button(format!("📋 {}", localizer.t("dialog.copy")))
                    .clicked()
                {
                    ui.ctx().copy_text(content.clone());
                    self.copied_feedback_timer = ui.input(|i| i.time) + 2.5;
                }
                if ui.input(|i| i.time) < self.copied_feedback_timer {
                    ui.colored_label(
                        GeoEstabilTheme::success_green(dark_mode),
                        localizer.t("dialog.copied"),
                    );
                }
            });
        });

        ui.separator();

        // Área de texto rolável com destaque de busca linha por linha
        let avail_h = ui.available_height().max(240.0);
        let text_pri = GeoEstabilTheme::text_primary(dark_mode);
        let active_idx = if total_hits > 0 {
            self.active_match_index % total_hits
        } else {
            0
        };
        let scroll_req = self.scroll_to_match_requested;
        self.scroll_to_match_requested = false;

        egui::Frame::new()
            .fill(if dark_mode {
                GeoEstabilTheme::canvas_bg(true)
            } else {
                egui::Color32::from_rgb(252, 253, 255)
            })
            .stroke(egui::Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(("about_doc_scroll", self.selected_tab, self.selected_license_idx, self.selected_history_sub))
                    .max_height((avail_h - 14.0).max(180.0))
                    .show(ui, |ui| {
                        let mut match_counter = 0usize;
                        for line in content.lines() {
                            if line.is_empty() {
                                ui.add_space(4.0);
                                continue;
                            }
                            let (mut job, is_active) = build_highlighted_job(
                                line,
                                egui::FontId::monospace(12.2),
                                text_pri,
                                false,
                                &self.search_query,
                                &mut match_counter,
                                active_idx,
                                dark_mode,
                            );
                            job.wrap.max_width = ui.available_width();
                            let resp = ui.label(job);
                            if scroll_req && is_active {
                                resp.scroll_to_me(Some(egui::Align::Center));
                            }
                        }
                    });
            });
    }
}
