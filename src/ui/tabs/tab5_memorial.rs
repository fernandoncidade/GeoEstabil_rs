use crate::core::project::GeoProject;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::locale::translations::Localizer;
use crate::reports::memorial_generator::MemorialGenerator;
use crate::ui::dialogs::doc_renderer::{parse_doc_blocks, render_doc_blocks_localized};
use crate::ui::export_actions::{
    export_memorial_html_or_pdf_dialog, export_memorial_markdown_dialog, export_profile_svg_dialog,
};
use crate::ui::theme::GeoEstabilTheme;
use egui::{Color32, RichText, ScrollArea, Ui};

pub struct Tab5Memorial;

impl Tab5Memorial {
    pub fn show(
        ui: &mut Ui,
        project: &mut GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        localizer: &Localizer,
        dark_mode: bool,
    ) {
        let lang = localizer.language;

        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading(localizer.t("tab5.title"));
            ui.label(localizer.t("tab5.subtitle"));
            ui.add_space(10.0);

            // Dados Cadastrais da ART / Engenheiro
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.heading(localizer.t("tab5.metadata"));
                ui.horizontal_wrapped(|ui| {
                    ui.label(localizer.t("tab5.field_title"));
                    ui.add(egui::TextEdit::singleline(&mut project.title).desired_width(240.0));
                    ui.label(localizer.t("tab5.field_client"));
                    ui.add(egui::TextEdit::singleline(&mut project.client).desired_width(220.0));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(localizer.t("tab5.field_site"));
                    ui.add(
                        egui::TextEdit::singleline(&mut project.site_location).desired_width(240.0),
                    );
                    ui.label(localizer.t("tab5.field_date"));
                    ui.add(egui::TextEdit::singleline(&mut project.date).desired_width(120.0));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(localizer.t("tab5.field_engineer"));
                    ui.add(
                        egui::TextEdit::singleline(&mut project.engineer_name)
                            .hint_text("GeoEstabil")
                            .desired_width(220.0),
                    );
                    ui.label(localizer.t("tab5.field_crea"));
                    ui.add(egui::TextEdit::singleline(&mut project.crea_cau).desired_width(130.0));
                    ui.label(localizer.t("tab5.field_art"));
                    ui.add(egui::TextEdit::singleline(&mut project.art_rrt).desired_width(130.0));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(localizer.t("tab5.field_notes"));
                    ui.add(egui::TextEdit::singleline(&mut project.notes).desired_width(360.0));
                });
            });

            ui.add_space(10.0);

            // Botões de Exportação
            let status_id = ui.make_persistent_id("tab5_export_status");
            let mut export_feedback: Option<(bool, String)> = None;

            ui.horizontal_wrapped(|ui| {
                let html_btn = egui::Button::new(
                    RichText::new(localizer.t("tab5.export_html"))
                        .color(Color32::WHITE)
                        .strong(),
                )
                .fill(GeoEstabilTheme::primary_button_fill(dark_mode))
                .min_size(egui::Vec2::new(260.0, 32.0));

                if ui.add(html_btn).clicked() {
                    match export_memorial_html_or_pdf_dialog(project, slope_report, localizer) {
                        Ok(Some(path)) => {
                            export_feedback = Some((
                                true,
                                format!(
                                    "✔ {}: {}",
                                    localizer.pick("Exportado", "Exported"),
                                    path.display()
                                ),
                            ));
                        }
                        Ok(None) => {}
                        Err(err) => {
                            export_feedback = Some((
                                false,
                                format!("✖ {}: {}", localizer.pick("Erro", "Error"), err),
                            ));
                        }
                    }
                }

                if ui.button(localizer.t("tab5.export_md")).clicked() {
                    match export_memorial_markdown_dialog(project, slope_report, localizer) {
                        Ok(Some(path)) => {
                            export_feedback = Some((
                                true,
                                format!(
                                    "✔ {}: {}",
                                    localizer.pick("Exportado", "Exported"),
                                    path.display()
                                ),
                            ));
                        }
                        Ok(None) => {}
                        Err(err) => {
                            export_feedback = Some((
                                false,
                                format!("✖ {}: {}", localizer.pick("Erro", "Error"), err),
                            ));
                        }
                    }
                }

                if ui.button(localizer.t("tab5.export_svg")).clicked() {
                    match export_profile_svg_dialog(project, slope_report, localizer) {
                        Ok(Some(path)) => {
                            export_feedback = Some((
                                true,
                                format!(
                                    "✔ {}: {}",
                                    localizer.pick("Exportado", "Exported"),
                                    path.display()
                                ),
                            ));
                        }
                        Ok(None) => {}
                        Err(err) => {
                            export_feedback = Some((
                                false,
                                format!("✖ {}: {}", localizer.pick("Erro", "Error"), err),
                            ));
                        }
                    }
                }

                if ui.button(localizer.t("tab5.copy_text")).clicked() {
                    let md_content =
                        MemorialGenerator::generate_markdown_lang(project, slope_report, lang);
                    ui.ctx().copy_text(md_content);
                    export_feedback = Some((
                        true,
                        format!("✔ {}", localizer.t("dialog.copied")),
                    ));
                }
            });

            if let Some((ok, msg)) = export_feedback {
                ui.ctx().data_mut(|d| d.insert_temp(status_id, (ok, msg)));
            }
            if let Some((ok, msg)) = ui.ctx().data(|d| d.get_temp::<(bool, String)>(status_id)) {
                let col = if ok {
                    GeoEstabilTheme::success_green(dark_mode)
                } else {
                    GeoEstabilTheme::slip_circle_red(dark_mode)
                };
                ui.colored_label(col, msg);
            }

            ui.add_space(10.0);

            // Pré-visualização do Relatório Formatado
            ui.group(|ui| {
                ui.strong(localizer.t("tab5.preview"));
                let md_text =
                    MemorialGenerator::generate_markdown_lang(project, slope_report, lang);
                let blocks = parse_doc_blocks(&md_text);
                egui::Frame::NONE
                    .fill(GeoEstabilTheme::canvas_bg(dark_mode))
                    .stroke(egui::Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)))
                    .inner_margin(12.0)
                    .corner_radius(egui::CornerRadius::same(6))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .max_height(450.0)
                            .show(ui, |ui| {
                                let diagram_cb = |d_ui: &mut Ui, code: &str, dm: bool| {
                                    let mode_opt = match code {
                                        "SLOPE_TAB2" => Some(crate::core::project::ActiveProjectMode::SlopeStability),
                                        "WALL_TAB3" => Some(crate::core::project::ActiveProjectMode::RetainingWall),
                                        _ => None,
                                    };
                                    if let Some(m) = mode_opt {
                                        crate::ui::canvas_2d::Canvas2D::show_static_preview(
                                            d_ui,
                                            project,
                                            m,
                                            slope_report,
                                            localizer,
                                            dm,
                                            295.0,
                                        );
                                    }
                                };
                                render_doc_blocks_localized(
                                    ui,
                                    &blocks,
                                    "",
                                    0,
                                    false,
                                    dark_mode,
                                    lang,
                                    Some(&diagram_cb),
                                );
                            });
                    });
            });
        });
    }
}
