use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::core::types::{Point2D, SlopeAnalysisMethod};
use crate::engine::slope::grid_search::run_slope_stability_analysis;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::locale::translations::Localizer;
use crate::ui::canvas_2d::{Canvas2D, CanvasState};
use crate::ui::theme::GeoEstabilTheme;
use egui::{Color32, CornerRadius, Frame, Margin, Panel, RichText, ScrollArea, Stroke, Ui, Vec2};

pub struct Tab2SlopeStability;

impl Tab2SlopeStability {
    pub fn show(
        ui: &mut Ui,
        project: &mut GeoProject,
        slope_report: &mut Option<SlopeAnalysisReport>,
        canvas_state: &mut CanvasState,
        localizer: &Localizer,
        dark_mode: bool,
    ) {
        // 1. Painel Lateral Esquerdo (Largura redimensionável e altura total dinâmica)
        Panel::left("tab2_slope_left_panel")
            .resizable(true)
            .default_size(350.0)
            .size_range(260.0..=520.0)
            .frame(
                Frame::NONE
                    .fill(GeoEstabilTheme::card_bg(dark_mode))
                    .stroke(Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)))
                    .inner_margin(Margin::same(10)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.heading(localizer.t("tab2.title"));
                        ui.label(
                            RichText::new(localizer.t("tab2.subtitle"))
                                .color(GeoEstabilTheme::text_secondary(dark_mode)),
                        );
                        ui.separator();

                        // Botão Principal de Cálculo
                        let btn_w = ui.available_width().max(200.0);
                        let calc_btn = egui::Button::new(
                            RichText::new(localizer.t("tab2.compute"))
                                .size(13.5)
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(GeoEstabilTheme::primary_button_fill(dark_mode))
                        .min_size(Vec2::new(btn_w, 36.0));

                        if ui.add(calc_btn).clicked() {
                            let report = run_slope_stability_analysis(&project.slope_model);
                            *slope_report = Some(report);
                            project.mark_dirty();
                        }

                        ui.add_space(8.0);

                        // Card de Resultados do Círculo Crítico
                        if let Some(ref report) = slope_report {
                            let bg_color = if report.is_stable {
                                Color32::from_rgba_unmultiplied(34, 197, 94, 32)
                            } else {
                                Color32::from_rgba_unmultiplied(239, 68, 68, 32)
                            };

                            Frame::NONE
                                .fill(bg_color)
                                .stroke(Stroke::new(
                                    1.5,
                                    if report.is_stable {
                                        GeoEstabilTheme::success_green(dark_mode)
                                    } else {
                                        GeoEstabilTheme::slip_circle_red(dark_mode)
                                    },
                                ))
                                .inner_margin(10.0)
                                .corner_radius(CornerRadius::same(6))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.strong(if report.is_stable {
                                        localizer.t("tab2.stable")
                                    } else {
                                        localizer.t("tab2.unstable")
                                    });
                                    ui.separator();
                                    ui.label(format!(
                                        "{}: FS = {:.3}",
                                        localizer.pick("Bishop Simplificado", "Simplified Bishop"),
                                        report.critical_circle.fs_bishop
                                    ));
                                    ui.label(format!(
                                        "{}: FS = {:.3}",
                                        localizer.pick("Fellenius (Ordinário)", "Fellenius (Ordinary)"),
                                        report.critical_circle.fs_fellenius
                                    ));
                                    ui.label(format!(
                                        "{}: FS = {:.3}",
                                        localizer.pick("Janbu (com fator f0)", "Janbu (with f0 factor)"),
                                        report.critical_circle.fs_janbu
                                    ));
                                    ui.label(format!(
                                        "{}: ≥ {:.2}",
                                        localizer.pick("FS Mínimo Exigido", "Minimum Required FS"),
                                        report.required_min_fs
                                    ));
                                    ui.separator();
                                    ui.label(format!(
                                        "{}: ({:.2}; {:.2}) m",
                                        localizer.pick("Centro Crítico", "Critical Center"),
                                        report.critical_circle.center_x,
                                        report.critical_circle.center_y
                                    ));
                                    ui.label(format!(
                                        "{}: R = {:.2} m",
                                        localizer.pick("Raio Crítico", "Critical Radius"),
                                        report.critical_circle.radius
                                    ));
                                    ui.label(format!(
                                        "{}: x={:.2}m {} x={:.2}m",
                                        localizer.pick("Extensão", "Span"),
                                        report.critical_circle.x_entry,
                                        localizer.pick("até", "to"),
                                        report.critical_circle.x_exit
                                    ));
                                    ui.label(format!(
                                        "CPU: {} {} {:.2} ms",
                                        report.total_circles_evaluated,
                                        localizer.pick("círculos em", "circles in"),
                                        report.elapsed_ms
                                    ));
                                });
                        }

                        ui.add_space(8.0);

                        // Método Primário
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong(localizer.t("tab2.method_title"));
                            ui.radio_value(
                                &mut project.slope_model.method,
                                SlopeAnalysisMethod::BishopSimplified,
                                localizer.t("tab2.method_bishop"),
                            );
                            ui.radio_value(
                                &mut project.slope_model.method,
                                SlopeAnalysisMethod::FelleniusOrdinary,
                                localizer.t("tab2.method_fellenius"),
                            );
                            ui.radio_value(
                                &mut project.slope_model.method,
                                SlopeAnalysisMethod::JanbuSimplified,
                                localizer.t("tab2.method_janbu"),
                            );
                        });

                        ui.add_space(8.0);

                        // Parâmetros da Malha de Busca (Grid Search)
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong(localizer.t("tab2.grid_title"));
                            egui::Grid::new("slope_grid_search_inputs")
                                .num_columns(2)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(localizer.t("tab2.xc_minmax"));
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut project.slope_model.grid_x_min)
                                                .speed(0.5),
                                        );
                                        ui.add(
                                            egui::DragValue::new(&mut project.slope_model.grid_x_max)
                                                .speed(0.5),
                                        );
                                    });
                                    ui.end_row();

                                    ui.label(localizer.t("tab2.xc_steps"));
                                    ui.add(
                                        egui::DragValue::new(&mut project.slope_model.grid_x_steps)
                                            .range(5..=60),
                                    );
                                    ui.end_row();

                                    ui.label(localizer.t("tab2.yc_minmax"));
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut project.slope_model.grid_y_min)
                                                .speed(0.5),
                                        );
                                        ui.add(
                                            egui::DragValue::new(&mut project.slope_model.grid_y_max)
                                                .speed(0.5),
                                        );
                                    });
                                    ui.end_row();

                                    ui.label(localizer.t("tab2.yc_steps"));
                                    ui.add(
                                        egui::DragValue::new(&mut project.slope_model.grid_y_steps)
                                            .range(5..=60),
                                    );
                                    ui.end_row();

                                    ui.label(localizer.t("tab2.r_minmax"));
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut project.slope_model.radius_min)
                                                .speed(0.5),
                                        );
                                        ui.add(
                                            egui::DragValue::new(&mut project.slope_model.radius_max)
                                                .speed(0.5),
                                        );
                                    });
                                    ui.end_row();

                                    ui.label(localizer.t("tab2.r_steps"));
                                    ui.add(
                                        egui::DragValue::new(&mut project.slope_model.radius_steps)
                                            .range(5..=60),
                                    );
                                    ui.end_row();

                                    ui.label(localizer.t("tab2.num_slices"));
                                    ui.add(
                                        egui::DragValue::new(&mut project.slope_model.num_slices)
                                            .range(15..=80),
                                    );
                                    ui.end_row();
                                });
                        });

                        ui.add_space(8.0);

                        // Geometria do Terreno (Pontos)
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal_wrapped(|ui| {
                                ui.strong(localizer.t("tab2.terrain_title"));
                                if ui.button(localizer.t("tab2.add_point")).clicked() {
                                    if let Some(last) =
                                        project.slope_model.terrain_surface.points.last()
                                    {
                                        let new_p = Point2D::new(last.x + 5.0, last.y);
                                        project.slope_model.terrain_surface.points.push(new_p);
                                        project.mark_dirty();
                                    }
                                }
                            });

                            let num_points = project.slope_model.terrain_surface.points.len();
                            let mut rem_p = None;
                            for (idx, pt) in project
                                .slope_model
                                .terrain_surface
                                .points
                                .iter_mut()
                                .enumerate()
                            {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(format!("P{}:", idx + 1));
                                    ui.label("x:");
                                    ui.add(egui::DragValue::new(&mut pt.x).speed(0.5));
                                    ui.label("y:");
                                    ui.add(egui::DragValue::new(&mut pt.y).speed(0.5));
                                    if num_points > 2 && ui.button("🗑").clicked() {
                                        rem_p = Some(idx);
                                    }
                                });
                            }
                            if let Some(idx) = rem_p {
                                project.slope_model.terrain_surface.points.remove(idx);
                                project.mark_dirty();
                            }
                        });
                    });
            });

        // 2. Painel Central/Direito: Controles Superiores + Visualizador Gráfico 2D
        egui::CentralPanel::default()
            .frame(Frame::NONE.inner_margin(Margin::symmetric(8, 4)))
            .show(ui, |ui| {
                Frame::NONE
                    .fill(GeoEstabilTheme::card_bg(dark_mode))
                    .stroke(Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(8, 5))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            ui.checkbox(
                                &mut canvas_state.show_heatmap,
                                localizer.t("tab2.show_heatmap"),
                            );
                            ui.checkbox(
                                &mut canvas_state.show_slices,
                                localizer.t("tab2.show_slices"),
                            );
                            ui.checkbox(
                                &mut canvas_state.show_grid,
                                localizer.t("tab2.show_grid"),
                            );
                            if ui.button(localizer.t("tab2.fit_view")).clicked() {
                                canvas_state.auto_fit = true;
                            }
                            if ui.button(localizer.t("tab2.reset_view")).clicked() {
                                *canvas_state = CanvasState::default();
                            }
                            ui.separator();
                            ui.label(
                                RichText::new(localizer.t("tab2.zoom_pan"))
                                    .size(11.5)
                                    .color(GeoEstabilTheme::text_secondary(dark_mode)),
                            );
                        });
                    });

                ui.add_space(6.0);

                Canvas2D::show(
                    ui,
                    project,
                    ActiveProjectMode::SlopeStability,
                    slope_report.as_ref(),
                    canvas_state,
                    localizer,
                    dark_mode,
                );
            });
    }
}
