use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::engine::slope::grid_search::run_slope_stability_analysis;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::stability::check_wall_external_stability;
use crate::locale::translations::{Language, Localizer};
use crate::ui::canvas_2d::CanvasState;
use crate::ui::theme::GeoEstabilTheme;
use egui::{CornerRadius, RichText, Stroke, Ui};

pub struct ToolBar;

impl ToolBar {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        project: &mut GeoProject,
        active_tab: &mut usize,
        slope_report: &mut Option<SlopeAnalysisReport>,
        slope_canvas_state: &mut CanvasState,
        wall_canvas_state: &mut CanvasState,
        localizer: &Localizer,
        dark_mode: &mut bool,
    ) {
        let lang = localizer.language;

        egui::Frame::NONE
            .fill(GeoEstabilTheme::card_bg(*dark_mode))
            .stroke(Stroke::new(1.0, GeoEstabilTheme::panel_border(*dark_mode)))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(format!("📄 {}", localizer.t("toolbar.new")))
                        .clicked()
                    {
                        let mut fresh = GeoProject::default();
                        fresh.translate_defaults(Language::PtBr, lang);
                        *project = fresh;
                        *slope_report = Some(run_slope_stability_analysis(&project.slope_model));
                        slope_canvas_state.reset_view();
                        wall_canvas_state.reset_view();
                        *active_tab = 1;
                    }
                    if ui
                        .button(format!("📂 {}", localizer.t("toolbar.open")))
                        .clicked()
                    {
                        if let Ok(Some(loaded)) =
                            crate::ui::export_actions::open_project_dialog(localizer)
                        {
                            *project = loaded;
                            *slope_report =
                                Some(run_slope_stability_analysis(&project.slope_model));
                            slope_canvas_state.reset_view();
                            wall_canvas_state.reset_view();
                            *active_tab = match project.active_mode {
                                ActiveProjectMode::SlopeStability => 1,
                                ActiveProjectMode::RetainingWall => 2,
                            };
                        }
                    }
                    if ui
                        .button(format!("💾 {}", localizer.t("toolbar.save")))
                        .clicked()
                    {
                        let _ = crate::ui::export_actions::save_project_dialog(
                            project, localizer, false,
                        );
                    }

                    ui.separator();

                    ui.label(localizer.t("toolbar.active_mode"));
                    if ui
                        .selectable_label(
                            project.active_mode == ActiveProjectMode::SlopeStability,
                            format!("⛰ {}", localizer.t("toolbar.mode.slope")),
                        )
                        .clicked()
                    {
                        project.active_mode = ActiveProjectMode::SlopeStability;
                        *active_tab = 1;
                    }
                    if ui
                        .selectable_label(
                            project.active_mode == ActiveProjectMode::RetainingWall,
                            format!("🧱 {}", localizer.t("toolbar.mode.wall")),
                        )
                        .clicked()
                    {
                        project.active_mode = ActiveProjectMode::RetainingWall;
                        *active_tab = 2;
                    }

                    ui.separator();

                    // Telemetria rápida por modo
                    match project.active_mode {
                        ActiveProjectMode::SlopeStability => {
                            if let Some(report) = slope_report.as_ref() {
                                let (txt, color) = if report.is_stable {
                                    (
                                        format!(
                                            "✔ FS Bishop = {:.3} ({})",
                                            report.critical_circle.fs_bishop,
                                            localizer.pick("Estável", "Stable")
                                        ),
                                        GeoEstabilTheme::success_green(*dark_mode),
                                    )
                                } else {
                                    (
                                        format!(
                                            "✖ FS Bishop = {:.3} ({})",
                                            report.critical_circle.fs_bishop,
                                            localizer.pick("Instável", "Unstable")
                                        ),
                                        GeoEstabilTheme::slip_circle_red(*dark_mode),
                                    )
                                };
                                ui.label(RichText::new(txt).color(color).strong());
                            } else {
                                ui.label(
                                    RichText::new(localizer.t("toolbar.awaiting_calc"))
                                        .color(GeoEstabilTheme::text_secondary(*dark_mode)),
                                );
                            }
                        }
                        ActiveProjectMode::RetainingWall => {
                            let p = calculate_earth_pressures(&project.wall_model);
                            let st = check_wall_external_stability(&project.wall_model);
                            let ok =
                                st.is_overturning_ok && st.is_sliding_ok && st.is_bearing_stress_ok;
                            let color = if ok {
                                GeoEstabilTheme::success_green(*dark_mode)
                            } else {
                                GeoEstabilTheme::slip_circle_red(*dark_mode)
                            };
                            ui.label(
                                RichText::new(format!(
                                    "{} Ea={:.1}kN/m | {}={:.2} | {}={:.2}",
                                    if ok { "✔" } else { "✖" },
                                    p.ea_horizontal,
                                    localizer.pick("Tomb", "Ovt"),
                                    st.fs_overturning,
                                    localizer.pick("Desl", "Sld"),
                                    st.fs_sliding
                                ))
                                .color(color)
                                .strong(),
                            );
                        }
                    }

                    ui.separator();

                    let theme_icon = if *dark_mode {
                        format!("☀ {}", localizer.t("toolbar.theme_light"))
                    } else {
                        format!("🌙 {}", localizer.t("toolbar.theme_dark"))
                    };
                    if ui.button(theme_icon).clicked() {
                        *dark_mode = !*dark_mode;
                        crate::ui::theme::GeoEstabilTheme::apply(ui.ctx(), *dark_mode);
                    }

                    ui.label(
                        RichText::new("ABNT NBR 11682 / 6122 / 6118")
                            .size(11.0)
                            .color(GeoEstabilTheme::accent_blue(*dark_mode)),
                    );
                });
            });
    }
}
