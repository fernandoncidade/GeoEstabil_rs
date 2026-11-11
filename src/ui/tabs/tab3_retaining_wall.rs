use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::core::types::{EarthPressureTheory, RetainingWallType};
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::stability::check_wall_external_stability;
use crate::locale::translations::Localizer;
use crate::ui::canvas_2d::{Canvas2D, CanvasState};
use crate::ui::dialogs::doc_renderer::rich_math_label;
use crate::ui::theme::GeoEstabilTheme;
use egui::{CornerRadius, Frame, Margin, Panel, RichText, ScrollArea, Stroke, Ui};

pub struct Tab3RetainingWall;

impl Tab3RetainingWall {
    pub fn show(
        ui: &mut Ui,
        project: &mut GeoProject,
        canvas_state: &mut CanvasState,
        localizer: &Localizer,
        dark_mode: bool,
    ) {
        let mut modified = false;

        // 1. Painel Lateral Esquerdo (Largura redimensionável e altura total dinâmica)
        Panel::left("tab3_wall_left_panel")
            .resizable(true)
            .default_size(360.0)
            .size_range(270.0..=540.0)
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
                        ui.heading(localizer.t("tab3.title"));
                        ui.label(
                            RichText::new(localizer.t("tab3.subtitle"))
                                .color(GeoEstabilTheme::text_secondary(dark_mode)),
                        );
                        ui.separator();

                        // Tipologia e Teoria
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong(localizer.t("tab3.wall_type"));
                            modified |= ui
                                .radio_value(
                                    &mut project.wall_model.wall_type,
                                    RetainingWallType::Cantilever,
                                    localizer.t("tab3.type_cantilever"),
                                )
                                .changed();
                            modified |= ui
                                .radio_value(
                                    &mut project.wall_model.wall_type,
                                    RetainingWallType::Gravity,
                                    localizer.t("tab3.type_gravity"),
                                )
                                .changed();

                            ui.separator();
                            ui.strong(localizer.t("tab3.theory"));
                            modified |= ui
                                .radio_value(
                                    &mut project.wall_model.theory,
                                    EarthPressureTheory::Rankine,
                                    localizer.t("tab3.theory_rankine"),
                                )
                                .changed();
                            modified |= ui
                                .radio_value(
                                    &mut project.wall_model.theory,
                                    EarthPressureTheory::Coulomb,
                                    localizer.t("tab3.theory_coulomb"),
                                )
                                .changed();
                        });

                        ui.add_space(8.0);

                        // Geometria Paramétrica
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong(localizer.t("tab3.dimensions"));
                            let is_gravity = matches!(
                                project.wall_model.wall_type,
                                RetainingWallType::Gravity
                            );
                            let geom = &mut project.wall_model.geometry;

                            egui::Grid::new("wall_geom_grid")
                                .num_columns(2)
                                .spacing([10.0, 5.0])
                                .show(ui, |ui| {
                                    ui.label(localizer.t("tab3.stem_height"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(&mut geom.stem_height)
                                                .speed(0.1)
                                                .range(1.5..=15.0)
                                                .suffix(" m"),
                                        )
                                        .changed();
                                    ui.end_row();

                                    rich_math_label(ui, localizer.t("tab3.stem_top"), 12.5, false, dark_mode, None);
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(&mut geom.stem_top_width)
                                                .speed(0.05)
                                                .range(0.2..=2.5)
                                                .suffix(" m"),
                                        )
                                        .changed();
                                    ui.end_row();

                                    if !is_gravity {
                                        ui.label(localizer.t("tab3.stem_base"));
                                        modified |= ui
                                            .add(
                                                egui::DragValue::new(&mut geom.stem_base_width)
                                                    .speed(0.05)
                                                    .range(0.2..=3.0)
                                                    .suffix(" m"),
                                            )
                                            .changed();
                                        ui.end_row();
                                    }

                                    ui.label(localizer.t("tab3.base_width"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(&mut geom.base_width)
                                                .speed(0.1)
                                                .range(0.8..=12.0)
                                                .suffix(" m"),
                                        )
                                        .changed();
                                    ui.end_row();

                                    if !is_gravity {
                                        rich_math_label(ui, localizer.t("tab3.base_thickness"), 12.5, false, dark_mode, None);
                                        modified |= ui
                                            .add(
                                                egui::DragValue::new(&mut geom.base_thickness)
                                                    .speed(0.05)
                                                    .range(0.0..=2.5)
                                                    .suffix(" m"),
                                            )
                                            .changed();
                                        ui.end_row();

                                        rich_math_label(ui, localizer.t("tab3.toe_length"), 12.5, false, dark_mode, None);
                                        modified |= ui
                                            .add(
                                                egui::DragValue::new(&mut geom.toe_length)
                                                    .speed(0.05)
                                                    .range(0.0..=4.0)
                                                    .suffix(" m"),
                                            )
                                            .changed();
                                        ui.end_row();
                                    }
                                });

                            if !is_gravity {
                                rich_math_label(
                                    ui,
                                    &format!(
                                        "{}: B_{{heel}} = {:.2} m",
                                        localizer.pick("Calcanhar Calculado", "Computed Heel"),
                                        geom.heel_length()
                                    ),
                                    12.2,
                                    false,
                                    dark_mode,
                                    None,
                                );
                            }

                            ui.separator();
                            modified |= ui
                                .checkbox(
                                    &mut geom.shear_key_enabled,
                                    localizer.t("tab3.shear_key"),
                                )
                                .changed();
                            if geom.shear_key_enabled {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(localizer.t("tab3.depth"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(&mut geom.shear_key_depth)
                                                .speed(0.05)
                                                .range(0.1..=1.5)
                                                .suffix(" m"),
                                        )
                                        .changed();
                                    ui.label(localizer.t("tab3.width"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(&mut geom.shear_key_width)
                                                .speed(0.05)
                                                .range(0.1..=1.5)
                                                .suffix(" m"),
                                        )
                                        .changed();
                                });
                            }
                        });

                        ui.add_space(8.0);

                        // Carregamentos e Nível d'Água
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong(localizer.t("tab3.loads_water"));
                            egui::Grid::new("wall_loads_grid")
                                .num_columns(2)
                                .spacing([10.0, 5.0])
                                .show(ui, |ui| {
                                    ui.label(localizer.t("tab3.surcharge_q"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(&mut project.wall_model.surcharge_q)
                                                .speed(1.0)
                                                .range(0.0..=200.0)
                                                .suffix(" kPa"),
                                        )
                                        .changed();
                                    ui.end_row();

                                    rich_math_label(ui, localizer.t("tab3.water_hw"), 12.5, false, dark_mode, None);
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project.wall_model.water_height_back,
                                            )
                                            .speed(0.1)
                                            .range(0.0..=12.0)
                                            .suffix(" m"),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label(localizer.t("tab3.backfill_slope"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project.wall_model.geometry.backfill_slope_deg,
                                            )
                                            .speed(0.5)
                                            .range(0.0..=35.0)
                                            .suffix("°"),
                                        )
                                        .changed();
                                    ui.end_row();
                                });
                        });

                        ui.add_space(8.0);

                        // Propriedades dos Solos do Muro
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong(localizer.t("tab3.backfill_soil"));
                            egui::Grid::new("wall_backfill_grid")
                                .num_columns(2)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(localizer.t("tab3.gamma"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project.wall_model.backfill_soil.gamma,
                                            )
                                            .speed(0.1)
                                            .range(12.0..=24.0),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label(localizer.t("tab3.phi"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project
                                                    .wall_model
                                                    .backfill_soil
                                                    .friction_angle_deg,
                                            )
                                            .speed(0.5)
                                            .range(10.0..=45.0),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label(localizer.t("tab3.cohesion"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project.wall_model.backfill_soil.cohesion,
                                            )
                                            .speed(0.5)
                                            .range(0.0..=100.0),
                                        )
                                        .changed();
                                    ui.end_row();
                                });

                            ui.separator();
                            ui.strong(localizer.t("tab3.foundation_soil"));
                            egui::Grid::new("wall_foundation_grid")
                                .num_columns(2)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(localizer.t("tab3.gamma"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project.wall_model.foundation_soil.gamma,
                                            )
                                            .speed(0.1)
                                            .range(12.0..=24.0),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label(localizer.t("tab3.phi"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project
                                                    .wall_model
                                                    .foundation_soil
                                                    .friction_angle_deg,
                                            )
                                            .speed(0.5)
                                            .range(5.0..=45.0),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label(localizer.t("tab3.cohesion"));
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project.wall_model.foundation_soil.cohesion,
                                            )
                                            .speed(0.5)
                                            .range(0.0..=150.0),
                                        )
                                        .changed();
                                    ui.end_row();

                                    rich_math_label(ui, localizer.t("tab3.sigma_adm"), 12.5, false, dark_mode, None);
                                    modified |= ui
                                        .add(
                                            egui::DragValue::new(
                                                &mut project.wall_model.foundation_soil.sigma_adm,
                                            )
                                            .speed(5.0)
                                            .range(50.0..=1000.0)
                                            .suffix(" kPa"),
                                        )
                                        .changed();
                                    ui.end_row();
                                });
                        });
                    });
            });

        if modified {
            project.mark_dirty();
        }

        // 2. Painel Central/Direito: Resultados de Empuxo & Visualização Gráfica 2D
        egui::CentralPanel::default()
            .frame(Frame::NONE.inner_margin(Margin::symmetric(8, 4)))
            .show(ui, |ui| {
                let pressures = calculate_earth_pressures(&project.wall_model);
                let stability = check_wall_external_stability(&project.wall_model);

                // Painel Superior Compacto: Resumo de Empuxos + Controles do Visualizador
                Frame::NONE
                    .fill(GeoEstabilTheme::card_bg(dark_mode))
                    .stroke(Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(8, 5))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(localizer.t("tab3.pressures_hdr")).strong());
                            ui.label(format!("Ka={:.3} Kp={:.3}", pressures.ka, pressures.kp));
                            ui.label(format!(
                                "{}={:.1} {}={:.1}",
                                localizer.pick("Solo", "Soil"),
                                pressures.ea_soil,
                                localizer.pick("Sob", "Sur"),
                                pressures.ea_surcharge
                            ));
                            if pressures.ea_water > 0.0 {
                                ui.label(format!(
                                    "{}={:.1}",
                                    localizer.pick("Água", "Water"),
                                    pressures.ea_water
                                ));
                            }
                            rich_math_label(
                                ui,
                                &format!("E_{{a,h}}={:.1} kN/m", pressures.ea_horizontal),
                                12.0,
                                true,
                                dark_mode,
                                Some(GeoEstabilTheme::slip_circle_red(dark_mode)),
                            );
                            ui.separator();
                            let fs_t_col = if stability.is_overturning_ok {
                                GeoEstabilTheme::success_green(dark_mode)
                            } else {
                                GeoEstabilTheme::slip_circle_red(dark_mode)
                            };
                            rich_math_label(
                                ui,
                                &format!(
                                    "{}={:.2}",
                                    localizer.pick("FS_{tomb}", "FS_{ovt}"),
                                    stability.fs_overturning
                                ),
                                12.0,
                                true,
                                dark_mode,
                                Some(fs_t_col),
                            );
                            let fs_d_col = if stability.is_sliding_ok {
                                GeoEstabilTheme::success_green(dark_mode)
                            } else {
                                GeoEstabilTheme::slip_circle_red(dark_mode)
                            };
                            rich_math_label(
                                ui,
                                &format!(
                                    "{}={:.2}",
                                    localizer.pick("FS_{desl}", "FS_{sld}"),
                                    stability.fs_sliding
                                ),
                                12.0,
                                true,
                                dark_mode,
                                Some(fs_d_col),
                            );
                        });

                        ui.separator();

                        ui.horizontal_wrapped(|ui| {
                            ui.strong(localizer.t("tab3.view_title"));
                            ui.checkbox(
                                &mut canvas_state.show_diagrams,
                                localizer.t("tab3.show_diagrams"),
                            );
                            ui.checkbox(
                                &mut canvas_state.show_dimensions,
                                localizer.t("tab3.show_dimensions"),
                            );
                            ui.checkbox(
                                &mut canvas_state.show_grid,
                                localizer.t("tab3.show_grid"),
                            );
                            if ui.button(localizer.t("tab2.fit_view")).clicked() {
                                canvas_state.auto_fit = true;
                            }
                            if ui.button(localizer.t("tab2.reset_view")).clicked() {
                                canvas_state.reset_view();
                            }
                            ui.separator();
                            ui.label(
                                RichText::new(localizer.t("tab2.zoom_pan"))
                                    .size(11.5)
                                    .color(GeoEstabilTheme::text_secondary(dark_mode)),
                            );
                        });
                    });

                ui.add_space(4.0);

                Canvas2D::show(
                    ui,
                    project,
                    ActiveProjectMode::RetainingWall,
                    None,
                    canvas_state,
                    localizer,
                    dark_mode,
                );
            });
    }
}
