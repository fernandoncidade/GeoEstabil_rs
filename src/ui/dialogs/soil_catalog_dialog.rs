use crate::core::project::GeoProject;
use crate::core::types::{Point2D, Polyline};
use crate::engine::soils::get_standard_nbr6122_soils;
use crate::locale::translations::Localizer;
use crate::ui::dialogs::doc_renderer::rich_math_label;
use crate::ui::dialogs::modeless::{show_modeless_dialog, ModelessState};
use eframe::egui;
use egui::ScrollArea;

#[derive(Default)]
pub struct SoilCatalogDialog {
    pub modeless: ModelessState,
}

impl SoilCatalogDialog {
    pub fn open(&mut self) {
        self.modeless.open();
    }

    pub fn close(&mut self) {
        self.modeless.close();
    }

    pub fn is_open(&self) -> bool {
        self.modeless.is_open
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        project: &mut GeoProject,
        localizer: &Localizer,
        dark_mode: bool,
    ) {
        let mut modeless_state = self.modeless.clone();
        let lang = localizer.language;
        let title = localizer.t("soils.title").to_string();

        show_modeless_dialog(
            ctx,
            "geoestabil_soil_catalog",
            &title,
            [760.0, 520.0],
            [540.0, 380.0],
            dark_mode,
            &mut modeless_state,
            |ui, close_clicked| {
                ui.label(localizer.t("soils.intro"));
                ui.separator();

                let scroll_h = (ui.available_height() - 42.0).max(200.0);
                ScrollArea::vertical()
                    .max_height(scroll_h)
                    .show(ui, |ui| {
                        let soils = get_standard_nbr6122_soils();
                        for soil in soils {
                            ui.group(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.strong(soil.localized_name(lang));
                                    ui.label(format!("({})", soil.localized_classification(lang)));
                                    rich_math_label(
                                        ui,
                                        &soil.localized_spt_range(lang),
                                        12.5,
                                        false,
                                        dark_mode,
                                        None,
                                    );

                                    if ui.button(localizer.t("soils.insert")).clicked() {
                                        let next_id = project.slope_model.layers.len() + 1;
                                        let mut layer = soil.to_soil_layer_lang(next_id, lang);
                                        layer.top_profile = Polyline::new(vec![
                                            Point2D::new(0.0, -2.5 * next_id as f64),
                                            Point2D::new(40.0, -2.5 * next_id as f64),
                                        ]);
                                        project.slope_model.layers.push(layer);
                                        project.mark_dirty();
                                    }
                                });
                                ui.horizontal_wrapped(|ui| {
                                    rich_math_label(ui, &format!("γ = {:.1} kN/m³", soil.gamma), 12.5, false, dark_mode, None);
                                    rich_math_label(ui, &format!("γ_{{sat}} = {:.1} kN/m³", soil.gamma_sat), 12.5, false, dark_mode, None);
                                    ui.label(format!("c' = {:.1} kPa", soil.cohesion));
                                    ui.label(format!("φ' = {:.1}°", soil.friction_angle_deg));
                                    rich_math_label(ui, &format!("σ_{{adm}} = {:.1} kPa", soil.sigma_adm), 12.5, false, dark_mode, None);
                                });
                            });
                            ui.add_space(4.0);
                        }
                    });

                ui.add_space(6.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(localizer.t("dialog.close")).clicked() {
                            *close_clicked = true;
                        }
                    });
                });
            },
        );

        self.modeless = modeless_state;
    }
}
