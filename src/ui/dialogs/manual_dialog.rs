use crate::locale::manual::{get_manual_sections, ManualSection};
use crate::locale::translations::Localizer;
use crate::ui::dialogs::doc_renderer::{
    count_matches_ci, parse_doc_blocks, render_doc_blocks_localized,
};
use crate::ui::dialogs::modeless::{show_modeless_dialog, ModelessState};
use crate::ui::theme::GeoEstabilTheme;
use eframe::egui;

#[derive(Default)]
pub struct ManualDialog {
    pub modeless: ModelessState,
    pub selected_section: usize,
    pub search_query: String,
    pub active_match_index: usize,
    pub copied_feedback_timer: f64,
    scroll_to_match_requested: bool,
}

impl ManualDialog {
    pub fn open(&mut self) {
        self.modeless.open();
    }

    pub fn show(&mut self, ctx: &egui::Context, localizer: &Localizer, dark_mode: bool) {
        let mut modeless_state = self.modeless.clone();
        let title = localizer.t("manual.title").to_string();

        show_modeless_dialog(
            ctx,
            "geoestabil_user_manual",
            &title,
            [1120.0, 740.0],
            [780.0, 520.0],
            dark_mode,
            &mut modeless_state,
            |ui, close_clicked| {
                self.render_body(ui, localizer, dark_mode, close_clicked);
            },
        );

        self.modeless = modeless_state;
    }

    fn render_body(
        &mut self,
        ui: &mut egui::Ui,
        localizer: &Localizer,
        dark_mode: bool,
        close_clicked: &mut bool,
    ) {
        let sections: Vec<ManualSection> = get_manual_sections(localizer.language);
        if self.selected_section >= sections.len() {
            self.selected_section = 0;
        }

        let current = &sections[self.selected_section];
        let section_hits = count_matches_ci(current.content, &self.search_query);

        // Barra superior responsiva: quebra em duas linhas compactas quando a janela é estreitada
        let search_w = (ui.available_width() * 0.28).clamp(120.0, 260.0);
        let single_row_top = ui.available_width() >= 840.0;
        let show_search = |ui: &mut egui::Ui, dlg: &mut Self| {
            ui.label(localizer.t("dialog.search"));
            let edit = ui.add_sized(
                [search_w, 24.0],
                egui::TextEdit::singleline(&mut dlg.search_query)
                    .hint_text(localizer.t("manual.search_hint")),
            );
            if edit.changed() {
                dlg.active_match_index = 0;
                dlg.scroll_to_match_requested = true;
            }
            if edit.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && section_hits > 0
            {
                if ui.input(|i| i.modifiers.shift) {
                    dlg.active_match_index =
                        (dlg.active_match_index + section_hits - 1) % section_hits;
                } else {
                    dlg.active_match_index = (dlg.active_match_index + 1) % section_hits;
                }
                dlg.scroll_to_match_requested = true;
            }

            if !dlg.search_query.trim().is_empty() {
                if ui.button("◀").clicked() && section_hits > 0 {
                    dlg.active_match_index =
                        (dlg.active_match_index + section_hits - 1) % section_hits;
                    dlg.scroll_to_match_requested = true;
                }
                if ui.button("▶").clicked() && section_hits > 0 {
                    dlg.active_match_index = (dlg.active_match_index + 1) % section_hits;
                    dlg.scroll_to_match_requested = true;
                }
                let ratio = if section_hits == 0 {
                    "0/0".to_string()
                } else {
                    format!(
                        "{}/{}",
                        (dlg.active_match_index % section_hits) + 1,
                        section_hits
                    )
                };
                ui.colored_label(GeoEstabilTheme::warning_amber(dark_mode), ratio);
                if ui.button("✖").clicked() {
                    dlg.search_query.clear();
                    dlg.active_match_index = 0;
                }
            }
        };
        let show_actions = |ui: &mut egui::Ui, dlg: &mut Self, close_clicked: &mut bool| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(format!("❌ {}", localizer.t("dialog.close")))
                    .clicked()
                {
                    *close_clicked = true;
                }
                if ui
                    .button(format!("💾 {}", localizer.t("dialog.export_md")))
                    .clicked()
                {
                    if let Some(mut path) = rfd::FileDialog::new()
                        .set_file_name(if localizer.is_pt() {
                            "Manual_GeoEstabil.md"
                        } else {
                            "GeoEstabil_User_Manual.md"
                        })
                        .add_filter("Markdown", &["md"])
                        .save_file()
                    {
                        if path.extension().is_none() {
                            path.set_extension("md");
                        }
                        let _ = std::fs::write(
                            path,
                            format!("# {}\n\n{}", current.title, current.content),
                        );
                    }
                }
                if ui
                    .button(format!("📋 {}", localizer.t("dialog.copy")))
                    .clicked()
                {
                    ui.ctx()
                        .copy_text(format!("{}\n\n{}", current.title, current.content));
                    dlg.copied_feedback_timer = ui.input(|i| i.time) + 2.5;
                }
                let now = ui.input(|i| i.time);
                if now < dlg.copied_feedback_timer {
                    ui.colored_label(
                        GeoEstabilTheme::success_green(dark_mode),
                        localizer.t("dialog.copied"),
                    );
                }
            });
        };

        if single_row_top {
            ui.horizontal(|ui| {
                show_search(ui, self);
                show_actions(ui, self, close_clicked);
            });
        } else {
            ui.horizontal(|ui| show_search(ui, self));
            ui.horizontal(|ui| show_actions(ui, self, close_clicked));
        }

        ui.separator();

        // Corpo em 2 colunas com molduras individuais e ajuste dinâmico de largura ao redimensionar
        let avail_rect = ui.available_rect_before_wrap();
        let total_w = avail_rect.width().max(320.0);
        let total_h = avail_rect.height().max(240.0);
        let col_gap = 8.0;

        let max_chars = sections
            .iter()
            .map(|s| s.title.chars().count())
            .max()
            .unwrap_or(28);
        let ideal_toc_w = ((max_chars as f32) * 5.9 + 52.0).clamp(220.0, 320.0);
        let col1_outer_w = ideal_toc_w.min((total_w * 0.34).max(160.0)).max(150.0);
        let col2_outer_w = (total_w - col1_outer_w - col_gap).max(150.0);
        let frame_pad = 10.0_f32;

        ui.horizontal_top(|ui| {
            // Coluna 1 (Esquerda): Moldura do Sumário de Capítulos
            ui.allocate_ui_with_layout(
                egui::vec2(col1_outer_w, total_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_size(egui::vec2(col1_outer_w, total_h));
                    ui.set_max_size(egui::vec2(col1_outer_w, total_h));
                    ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));

                    Self::column_frame(dark_mode, frame_pad).show(ui, |ui| {
                        let inner_w = (col1_outer_w - frame_pad * 2.0).max(80.0);
                        let inner_h = (total_h - frame_pad * 2.0).max(100.0);
                        ui.set_width(inner_w);
                        ui.set_max_width(inner_w);
                        ui.set_min_height(inner_h);
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

                        ui.add(egui::Label::new(egui::RichText::new(localizer.t("manual.toc")).strong()).wrap());
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        let toc_scroll_h = ui.available_height().max(80.0);
                        let toc_item_w = (inner_w - 10.0).max(60.0);
                        egui::ScrollArea::vertical()
                            .id_salt("manual_toc_scroll")
                            .max_height(toc_scroll_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(toc_item_w);
                                ui.set_max_width(toc_item_w);
                                for (idx, sec) in sections.iter().enumerate() {
                                    let hits = count_matches_ci(sec.content, &self.search_query);
                                    let lbl = if hits > 0 {
                                        format!("{} [{}]", sec.title, hits)
                                    } else {
                                        sec.title.to_string()
                                    };
                                    let is_sel = idx == self.selected_section;
                                    let resp = ui.add(
                                        egui::Button::selectable(
                                            is_sel,
                                            egui::RichText::new(lbl).size(12.5),
                                        )
                                        .wrap(),
                                    );
                                    if resp.clicked() {
                                        self.selected_section = idx;
                                        self.active_match_index = 0;
                                    }
                                    ui.add_space(2.0);
                                }
                            });
                    });
                },
            );

            ui.add_space(col_gap);

            // Coluna 2 (Direita): Moldura do Conteúdo Formatado com quebra automática dinâmica
            ui.allocate_ui_with_layout(
                egui::vec2(col2_outer_w, total_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_size(egui::vec2(col2_outer_w, total_h));
                    ui.set_max_size(egui::vec2(col2_outer_w, total_h));
                    ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));

                    Self::column_frame(dark_mode, frame_pad).show(ui, |ui| {
                        let inner_w = (col2_outer_w - frame_pad * 2.0).max(80.0);
                        let inner_h = (total_h - frame_pad * 2.0).max(100.0);
                        ui.set_width(inner_w);
                        ui.set_max_width(inner_w);
                        ui.set_min_height(inner_h);
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(current.title)
                                    .size(16.0)
                                    .strong()
                                    .color(GeoEstabilTheme::accent_blue(dark_mode)),
                            )
                            .wrap(),
                        );
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(current.subtitle)
                                    .size(12.0)
                                    .color(GeoEstabilTheme::text_secondary(dark_mode)),
                            )
                            .wrap(),
                        );
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        let blocks = parse_doc_blocks(current.content);
                        let active_idx = if section_hits > 0 {
                            self.active_match_index % section_hits
                        } else {
                            0
                        };
                        let scroll_req = self.scroll_to_match_requested;
                        self.scroll_to_match_requested = false;

                        let doc_scroll_h = ui.available_height().max(80.0);
                        let doc_inner_w = (inner_w - 14.0).max(60.0);

                        egui::ScrollArea::vertical()
                            .id_salt(("manual_doc_scroll", self.selected_section))
                            .max_height(doc_scroll_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(doc_inner_w);
                                ui.set_max_width(doc_inner_w);
                                render_doc_blocks_localized(
                                    ui,
                                    &blocks,
                                    &self.search_query,
                                    active_idx,
                                    scroll_req,
                                    dark_mode,
                                    localizer.language,
                                    None,
                                );
                            });
                    });
                },
            );
        });
    }

    fn column_frame(dark_mode: bool, inner_margin: f32) -> egui::Frame {
        let fill = if dark_mode {
            GeoEstabilTheme::canvas_bg(true)
        } else {
            egui::Color32::from_rgb(252, 253, 255)
        };
        let margin_px = inner_margin.round() as i8;
        egui::Frame::NONE
            .fill(fill)
            .stroke(egui::Stroke::new(
                1.0,
                GeoEstabilTheme::panel_border(dark_mode),
            ))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(margin_px))
    }
}
