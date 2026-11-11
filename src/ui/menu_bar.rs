use crate::core::presets::get_preset_cases;
use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::engine::slope::grid_search::run_slope_stability_analysis;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::locale::translations::{Language, Localizer};
use crate::ui::canvas_2d::CanvasState;
use crate::ui::dialogs::{AboutDialog, ManualDialog, SettingsDialog, SoilCatalogDialog};
use crate::ui::export_actions::{
    export_memorial_html_or_pdf_dialog, export_memorial_markdown_dialog,
    export_profile_svg_dialog, open_project_dialog, save_project_dialog,
};
use egui::Ui;

pub struct MenuBar;

impl MenuBar {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        project: &mut GeoProject,
        active_tab: &mut usize,
        slope_report: &mut Option<SlopeAnalysisReport>,
        slope_canvas_state: &mut CanvasState,
        wall_canvas_state: &mut CanvasState,
        about_dialog: &mut AboutDialog,
        manual_dialog: &mut ManualDialog,
        soil_catalog_dialog: &mut SoilCatalogDialog,
        settings_dialog: &mut SettingsDialog,
        localizer: &mut Localizer,
        dark_mode: &mut bool,
    ) {
        let lang = localizer.language;

        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(localizer.t("menu.file"), |ui| {
                if ui
                    .button(format!("📄 {}", localizer.t("menu.file.new")))
                    .clicked()
                {
                    let mut fresh = GeoProject::default();
                    fresh.translate_defaults(Language::PtBr, lang);
                    *project = fresh;
                    *slope_report = Some(run_slope_stability_analysis(&project.slope_model));
                    slope_canvas_state.reset_view();
                    wall_canvas_state.reset_view();
                    *active_tab = 1;
                    ui.close();
                }

                if ui
                    .button(format!("📂 {}", localizer.t("menu.file.open")))
                    .clicked()
                {
                    if let Ok(Some(loaded)) = open_project_dialog(localizer) {
                        *project = loaded;
                        *slope_report = Some(run_slope_stability_analysis(&project.slope_model));
                        slope_canvas_state.reset_view();
                        wall_canvas_state.reset_view();
                        *active_tab = match project.active_mode {
                            ActiveProjectMode::SlopeStability => 1,
                            ActiveProjectMode::RetainingWall => 2,
                        };
                    }
                    ui.close();
                }

                if ui
                    .button(format!("💾 {}", localizer.t("menu.file.save")))
                    .clicked()
                {
                    let _ = save_project_dialog(project, localizer, false);
                    ui.close();
                }

                if ui
                    .button(format!("💾 {}", localizer.t("menu.file.save_as")))
                    .clicked()
                {
                    let _ = save_project_dialog(project, localizer, true);
                    ui.close();
                }

                ui.separator();
                if ui
                    .button(format!("📄 {}", localizer.t("menu.file.export_html")))
                    .clicked()
                {
                    let _ =
                        export_memorial_html_or_pdf_dialog(project, slope_report.as_ref(), localizer);
                    ui.close();
                }
                if ui
                    .button(format!("📝 {}", localizer.t("menu.file.export_md")))
                    .clicked()
                {
                    let _ =
                        export_memorial_markdown_dialog(project, slope_report.as_ref(), localizer);
                    ui.close();
                }
                if ui
                    .button(format!("📐 {}", localizer.t("menu.file.export_svg")))
                    .clicked()
                {
                    let _ = export_profile_svg_dialog(project, slope_report.as_ref(), localizer);
                    ui.close();
                }

                ui.separator();
                if ui
                    .button(format!("🚪 {}", localizer.t("menu.file.exit")))
                    .clicked()
                {
                    std::process::exit(0);
                }
            });

            ui.menu_button(localizer.t("menu.presets"), |ui| {
                let presets = get_preset_cases();
                for preset in presets {
                    let preset_lbl = preset.localized_name(lang);
                    if ui.button(preset_lbl).clicked() {
                        let mode = preset.mode;
                        *project = preset.project_for_lang(lang);
                        match mode {
                            ActiveProjectMode::SlopeStability => {
                                *slope_report =
                                    Some(run_slope_stability_analysis(&project.slope_model));
                                slope_canvas_state.reset_view();
                                *active_tab = 1;
                            }
                            ActiveProjectMode::RetainingWall => {
                                *slope_report = None;
                                wall_canvas_state.reset_view();
                                *active_tab = 2;
                            }
                        }
                        ui.close();
                    }
                }
            });

            let tools_lbl = localizer.t("menu.tools").to_string();
            let soils_lbl = format!("🗄 {}", localizer.t("menu.tools.soil_catalog"));
            let settings_lbl = format!("⚙ {}", localizer.t("menu.tools.settings"));
            let lang_menu_lbl = format!("🌐 {}", localizer.t("menu.tools.lang"));

            ui.menu_button(tools_lbl, |ui| {
                if ui.button(soils_lbl).clicked() {
                    soil_catalog_dialog.open();
                    ui.close();
                }
                if ui.button(settings_lbl).clicked() {
                    settings_dialog.open();
                    ui.close();
                }
                ui.separator();
                ui.menu_button(lang_menu_lbl, |ui| {
                    if ui
                        .selectable_label(localizer.language == Language::PtBr, "🇧🇷 Português (BR)")
                        .clicked()
                    {
                        let prev = localizer.language;
                        localizer.set_language(Language::PtBr);
                        project.translate_defaults(prev, Language::PtBr);
                        ui.close();
                    }
                    if ui
                        .selectable_label(localizer.language == Language::EnUs, "🇺🇸 English (US)")
                        .clicked()
                    {
                        let prev = localizer.language;
                        localizer.set_language(Language::EnUs);
                        project.translate_defaults(prev, Language::EnUs);
                        ui.close();
                    }
                });
                ui.separator();
                let theme_label = if *dark_mode {
                    format!("☀ {}", localizer.t("menu.tools.to_light"))
                } else {
                    format!("🌙 {}", localizer.t("menu.tools.to_dark"))
                };
                if ui.button(theme_label).clicked() {
                    *dark_mode = !*dark_mode;
                    crate::ui::theme::GeoEstabilTheme::apply(ui.ctx(), *dark_mode);
                    ui.close();
                }
            });

            ui.menu_button(localizer.t("menu.help"), |ui| {
                if ui
                    .button(format!("📖 {}", localizer.t("menu.help.manual")))
                    .clicked()
                {
                    manual_dialog.open();
                    ui.close();
                }
                ui.separator();
                if ui
                    .button(format!("ℹ {}", localizer.t("menu.help.about")))
                    .clicked()
                {
                    about_dialog.open();
                    ui.close();
                }
            });
        });
    }
}
