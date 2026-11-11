use crate::core::project::GeoProject;
use crate::core::types::{Point2D, Polyline, SoilLayer};
use crate::engine::soils::get_standard_nbr6122_soils;
use crate::locale::translations::Localizer;
use crate::ui::dialogs::doc_renderer::rich_math_label;
use egui::{ScrollArea, Ui};

pub struct Tab1Geotechnical;

impl Tab1Geotechnical {
    pub fn show(ui: &mut Ui, project: &mut GeoProject, localizer: &Localizer, dark_mode: bool) {
        let lang = localizer.language;

        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading(localizer.t("tab1.title"));
            ui.label(localizer.t("tab1.subtitle"));
            ui.add_space(10.0);

            // Seção de Solos da ABNT NBR 6122
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.strong(localizer.t("tab1.soil_bank"));
                    let soils = get_standard_nbr6122_soils();
                    for soil in soils.iter().take(5) {
                        if ui.button(format!("+ {}", soil.localized_name(lang))).clicked() {
                            let next_id = project.slope_model.layers.len() + 1;
                            let mut layer = soil.to_soil_layer_lang(next_id, lang);
                            layer.top_profile = Polyline::new(vec![
                                Point2D::new(0.0, -2.0 * next_id as f64),
                                Point2D::new(40.0, -2.0 * next_id as f64),
                            ]);
                            project.slope_model.layers.push(layer);
                            project.mark_dirty();
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // Tabela de Camadas
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.heading(localizer.t("tab1.layers_title"));
                    if ui.button(localizer.t("tab1.add_layer")).clicked() {
                        let next_id = project.slope_model.layers.len() + 1;
                        project.slope_model.layers.push(SoilLayer {
                            id: next_id,
                            name: format!(
                                "{} {}",
                                localizer.pick("Camada de Solo", "Soil Layer"),
                                next_id
                            ),
                            description: localizer
                                .pick("Solo residual ou coluvial", "Residual or colluvial soil")
                                .to_string(),
                            gamma: 18.0,
                            gamma_sat: 19.5,
                            cohesion: 10.0,
                            friction_angle_deg: 28.0,
                            sigma_adm: 200.0,
                            top_profile: Polyline::new(vec![
                                Point2D::new(0.0, -3.0),
                                Point2D::new(40.0, -3.0),
                            ]),
                            color_rgba: [205, 133, 63, 200],
                        });
                        project.mark_dirty();
                    }
                });

                let num_layers = project.slope_model.layers.len();
                let mut to_remove = None;
                for (idx, layer) in project.slope_model.layers.iter_mut().enumerate() {
                    ui.push_id(idx, |ui| {
                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("#{}", idx + 1));
                            ui.add(egui::TextEdit::singleline(&mut layer.name).desired_width(180.0));
                            rich_math_label(ui, "γ [kN/m³]:", 12.5, false, dark_mode, None);
                            ui.add(egui::DragValue::new(&mut layer.gamma).speed(0.1).range(10.0..=25.0));
                            rich_math_label(ui, "γ_{sat} [kN/m³]:", 12.5, false, dark_mode, None);
                            ui.add(egui::DragValue::new(&mut layer.gamma_sat).speed(0.1).range(12.0..=26.0));
                            ui.label("c' [kPa]:");
                            ui.add(egui::DragValue::new(&mut layer.cohesion).speed(0.5).range(0.0..=200.0));
                            ui.label("φ' [°]:");
                            ui.add(egui::DragValue::new(&mut layer.friction_angle_deg).speed(0.5).range(0.0..=50.0));
                            rich_math_label(ui, "σ_{adm} [kPa]:", 12.5, false, dark_mode, None);
                            ui.add(egui::DragValue::new(&mut layer.sigma_adm).speed(5.0).range(30.0..=1000.0));

                            if num_layers > 1 && ui.button("🗑").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    });
                }

                if let Some(rem_idx) = to_remove {
                    project.slope_model.layers.remove(rem_idx);
                    project.mark_dirty();
                }
            });

            ui.add_space(10.0);

            // Nível Freático e Sobrecarga
            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    ui.heading(localizer.t("tab1.water_title"));
                    ui.checkbox(
                        &mut project.slope_model.water_table.enabled,
                        localizer.t("tab1.water_enable"),
                    );
                    if project.slope_model.water_table.enabled {
                        ui.horizontal(|ui| {
                            ui.label(localizer.t("tab1.water_gamma"));
                            ui.add(
                                egui::DragValue::new(&mut project.slope_model.water_table.gamma_w)
                                    .speed(0.01)
                                    .range(9.8..=10.0),
                            );
                        });
                        ui.label(localizer.t("tab1.water_points"));
                        for (idx, pt) in project
                            .slope_model
                            .water_table
                            .phreatic_line
                            .points
                            .iter_mut()
                            .enumerate()
                        {
                            ui.horizontal(|ui| {
                                ui.label(format!("P{}: x=", idx + 1));
                                ui.add(egui::DragValue::new(&mut pt.x).speed(0.5));
                                ui.label("y=");
                                ui.add(egui::DragValue::new(&mut pt.y).speed(0.5));
                            });
                        }
                    }
                });

                cols[1].group(|ui| {
                    ui.heading(localizer.t("tab1.surcharge_title"));
                    ui.checkbox(
                        &mut project.slope_model.surcharge.enabled,
                        localizer.t("tab1.surcharge_enable"),
                    );
                    if project.slope_model.surcharge.enabled {
                        ui.text_edit_singleline(&mut project.slope_model.surcharge.name);
                        ui.horizontal(|ui| {
                            ui.label(localizer.t("tab1.surcharge_q"));
                            ui.add(
                                egui::DragValue::new(&mut project.slope_model.surcharge.q_kpa)
                                    .speed(1.0)
                                    .range(0.0..=500.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(localizer.t("tab1.surcharge_start"));
                            ui.add(
                                egui::DragValue::new(&mut project.slope_model.surcharge.x_start)
                                    .speed(0.5),
                            );
                            ui.label(localizer.t("tab1.surcharge_end"));
                            ui.add(
                                egui::DragValue::new(&mut project.slope_model.surcharge.x_end)
                                    .speed(0.5),
                            );
                        });
                    }
                });
            });

            ui.add_space(10.0);
            ui.group(|ui| {
                ui.heading(localizer.t("tab1.safety_title"));
                ui.horizontal(|ui| {
                    ui.label(localizer.t("tab1.safety_label"));
                    ui.selectable_value(
                        &mut project.slope_model.safety_level,
                        crate::core::types::SafetyLevelNBR11682::High,
                        localizer.t("tab1.safety_high"),
                    );
                    ui.selectable_value(
                        &mut project.slope_model.safety_level,
                        crate::core::types::SafetyLevelNBR11682::Medium,
                        localizer.t("tab1.safety_medium"),
                    );
                    ui.selectable_value(
                        &mut project.slope_model.safety_level,
                        crate::core::types::SafetyLevelNBR11682::Low,
                        localizer.t("tab1.safety_low"),
                    );
                });
                ui.label(project.slope_model.safety_level.description(lang));
            });
        });
    }
}
