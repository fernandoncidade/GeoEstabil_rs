use crate::core::project::GeoProject;
use crate::engine::wall::bearing_capacity::calculate_bearing_capacity;
use crate::engine::wall::concrete_design::design_concrete_reinforcement;
use crate::engine::wall::stability::check_wall_external_stability;
use crate::locale::translations::Localizer;
use crate::ui::dialogs::doc_renderer::rich_math_label;
use crate::ui::theme::GeoEstabilTheme;
use egui::{RichText, ScrollArea, Ui};

pub struct Tab4StructuralDesign;

impl Tab4StructuralDesign {
    pub fn show(ui: &mut Ui, project: &mut GeoProject, localizer: &Localizer, dark_mode: bool) {
        let lang = localizer.language;
        let stability = check_wall_external_stability(&project.wall_model);
        let bearing = calculate_bearing_capacity(&project.wall_model, &stability);
        let structural = design_concrete_reinforcement(&project.wall_model, &stability);

        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading(localizer.t("tab4.title"));
            ui.label(localizer.t("tab4.subtitle"));
            ui.add_space(10.0);

            // =========================================================================
            // 1. CARDS DE ESTABILIDADE GEOTÉCNICA EXTERNA (NBR 6122 / NBR 11682)
            // =========================================================================
            ui.heading(localizer.t("tab4.sec_a"));
            ui.columns(5, |cols| {
                // Tombamento
                cols[0].group(|ui| {
                    ui.strong(localizer.t("tab4.overturning"));
                    let status = stability.is_overturning_ok;
                    let color = if status {
                        GeoEstabilTheme::success_green(dark_mode)
                    } else {
                        GeoEstabilTheme::slip_circle_red(dark_mode)
                    };
                    ui.label(
                        RichText::new(format!("FS = {:.2}", stability.fs_overturning))
                            .size(16.0)
                            .color(color)
                            .strong(),
                    );
                    ui.label(localizer.t("tab4.req_150"));
                    ui.label(if status {
                        localizer.t("tab4.pass")
                    } else {
                        localizer.t("tab4.fail")
                    });
                });

                // Deslizamento
                cols[1].group(|ui| {
                    ui.strong(localizer.t("tab4.sliding"));
                    let status = stability.is_sliding_ok;
                    let color = if status {
                        GeoEstabilTheme::success_green(dark_mode)
                    } else {
                        GeoEstabilTheme::slip_circle_red(dark_mode)
                    };
                    ui.label(
                        RichText::new(format!("FS = {:.2}", stability.fs_sliding))
                            .size(16.0)
                            .color(color)
                            .strong(),
                    );
                    ui.label(localizer.t("tab4.req_150"));
                    ui.label(if status {
                        localizer.t("tab4.pass")
                    } else {
                        localizer.t("tab4.fail")
                    });
                });

                // Excentricidade
                cols[2].group(|ui| {
                    ui.strong(localizer.t("tab4.eccentricity"));
                    let status = stability.is_in_middle_third;
                    let color = if status {
                        GeoEstabilTheme::success_green(dark_mode)
                    } else {
                        GeoEstabilTheme::slip_circle_red(dark_mode)
                    };
                    ui.label(
                        RichText::new(format!("e = {:.3} m", stability.eccentricity))
                            .size(15.0)
                            .color(color)
                            .strong(),
                    );
                    ui.label(format!("B/6 = {:.3} m", stability.max_allowed_eccentricity));
                    ui.label(if status {
                        localizer.t("tab4.middle_third")
                    } else {
                        localizer.t("tab4.uplift")
                    });
                });

                // Tensão Máxima no Solo
                cols[3].group(|ui| {
                    rich_math_label(
                        ui,
                        localizer.t("tab4.max_stress"),
                        12.8,
                        true,
                        dark_mode,
                        None,
                    );
                    let status = stability.is_bearing_stress_ok;
                    let color = if status {
                        GeoEstabilTheme::success_green(dark_mode)
                    } else {
                        GeoEstabilTheme::slip_circle_red(dark_mode)
                    };
                    ui.label(
                        RichText::new(format!("{:.1} kPa", stability.sigma_toe_kpa))
                            .size(15.0)
                            .color(color)
                            .strong(),
                    );
                    rich_math_label(
                        ui,
                        &format!("σ_{{adm}} = {:.1} kPa", stability.sigma_adm_kpa),
                        12.2,
                        false,
                        dark_mode,
                        None,
                    );
                    rich_math_label(
                        ui,
                        if status {
                            localizer.t("tab4.stress_ok")
                        } else {
                            localizer.t("tab4.overstress")
                        },
                        12.2,
                        false,
                        dark_mode,
                        None,
                    );
                });

                // Capacidade de Carga Última
                cols[4].group(|ui| {
                    ui.strong(localizer.t("tab4.bearing"));
                    let status = bearing.is_bearing_ok;
                    let color = if status {
                        GeoEstabilTheme::success_green(dark_mode)
                    } else {
                        GeoEstabilTheme::slip_circle_red(dark_mode)
                    };
                    ui.label(
                        RichText::new(format!("FS = {:.2}", bearing.fs_bearing))
                            .size(16.0)
                            .color(color)
                            .strong(),
                    );
                    rich_math_label(
                        ui,
                        &format!("q_{{ult}} = {:.1} kPa", bearing.q_ult_kpa),
                        12.2,
                        false,
                        dark_mode,
                        None,
                    );
                    ui.label(if status {
                        localizer.t("tab4.bearing_ok")
                    } else {
                        localizer.t("tab4.bearing_fail")
                    });
                });
            });

            ui.add_space(15.0);

            // =========================================================================
            // 2. PARÂMETROS ESTRUTURAIS DO CONCRETO ARMADO (NBR 6118:2023)
            // =========================================================================
            ui.heading(localizer.t("tab4.sec_b"));
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    rich_math_label(ui, localizer.t("tab4.fck"), 12.8, false, dark_mode, None);
                    ui.add(
                        egui::DragValue::new(&mut project.wall_model.concrete.fck_mpa)
                            .speed(1.0)
                            .range(15.0..=60.0),
                    );
                    rich_math_label(ui, localizer.t("tab4.fyk"), 12.8, false, dark_mode, None);
                    ui.add(
                        egui::DragValue::new(&mut project.wall_model.concrete.fyk_mpa)
                            .speed(10.0)
                            .range(250.0..=600.0),
                    );
                    rich_math_label(ui, localizer.t("tab4.cover"), 12.8, false, dark_mode, None);
                    ui.add(
                        egui::DragValue::new(&mut project.wall_model.concrete.cover_nom_mm)
                            .speed(1.0)
                            .range(20.0..=55.0),
                    );
                    rich_math_label(ui, localizer.t("tab4.gamma_f"), 12.8, false, dark_mode, None);
                    ui.add(
                        egui::DragValue::new(&mut project.wall_model.concrete.gamma_f)
                            .speed(0.05)
                            .range(1.2..=1.6),
                    );
                });
                rich_math_label(
                    ui,
                    &format!(
                        "{}: f_{{cd}} = {:.2} MPa | f_{{yd}} = {:.2} MPa | {} γ_{{c}} = {:.1} kN/m³",
                        localizer.pick("Resistências de Cálculo", "Design Strengths"),
                        structural.fcd_mpa,
                        structural.fyd_mpa,
                        localizer.pick("Concreto", "Concrete"),
                        project.wall_model.concrete.gamma_concrete
                    ),
                    12.5,
                    false,
                    dark_mode,
                    None,
                );
            });

            ui.add_space(15.0);

            // =========================================================================
            // 3. DIMENSIONAMENTO DAS SEÇÕES CRÍTICAS (FUSTE, PONTA E CALCANHAR)
            // =========================================================================
            ui.heading(localizer.t("tab4.sec_c"));

            let sections = [
                &structural.stem_section,
                &structural.toe_section,
                &structural.heel_section,
            ];

            for sec in &sections {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.heading(sec.localized_section_name(lang));
                        rich_math_label(
                            ui,
                            &sec.recommendation.localized_description(lang),
                            15.0,
                            true,
                            dark_mode,
                            Some(GeoEstabilTheme::rebar_orange(dark_mode)),
                        );
                    });
                    ui.separator();

                    let fck_s = project.wall_model.concrete.fck_mpa;
                    let lambda_s = if fck_s <= 50.0 {
                        0.80
                    } else {
                        (0.80 - (fck_s - 50.0) / 400.0).max(0.65)
                    };
                    ui.columns(4, |cols| {
                        rich_math_label(
                            &mut cols[0],
                            &format!(
                                "{}:\nM_{{k}} = {:.1} kNm/m\nM_{{d}} = {:.1} kNm/m",
                                localizer.pick("Momento Fletor", "Bending Moment"),
                                sec.mk_knm_m,
                                sec.md_knm_m
                            ),
                            12.4,
                            false,
                            dark_mode,
                            None,
                        );
                        rich_math_label(
                            &mut cols[1],
                            &format!(
                                "{}:\nV_{{k}} = {:.1} kN/m\nV_{{d}} = {:.1} kN/m",
                                localizer.pick("Força Cortante", "Shear Force"),
                                sec.vk_kn_m,
                                sec.vd_kn_m
                            ),
                            12.4,
                            false,
                            dark_mode,
                            None,
                        );
                        rich_math_label(
                            &mut cols[2],
                            &format!(
                                "{}:\nξ = {:.3} ({})\n{} z = {:.3} m",
                                localizer.pick("Linha Neutra", "Neutral Axis"),
                                sec.neutral_axis_xi,
                                if sec.is_ductile {
                                    localizer.t("tab4.ductile")
                                } else {
                                    localizer.t("tab4.brittle")
                                },
                                localizer.pick("Braço", "Lever Arm"),
                                sec.effective_depth_d_m * (1.0 - 0.5 * lambda_s * sec.neutral_axis_xi)
                            ),
                            12.4,
                            false,
                            dark_mode,
                            None,
                        );
                        rich_math_label(
                            &mut cols[3],
                            &format!(
                                "{}:\nA_{{s,calc}} = {:.2} cm²/m\n{} = {:.2} cm²/m ({})\n{} = {:.2} cm²/m",
                                localizer.pick("Área de Aço Requerida", "Required Steel Area"),
                                sec.as_calculated_cm2_m,
                                localizer.pick("A_{s,mín}", "A_{s,min}"),
                                sec.as_min_cm2_m,
                                localizer.pick("mín. NBR", "min. NBR"),
                                localizer.pick("A_{s,adot}", "A_{s,prov}"),
                                sec.as_required_cm2_m
                            ),
                            12.4,
                            false,
                            dark_mode,
                            None,
                        );
                    });

                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        let shear_color = if sec.is_shear_ok {
                            GeoEstabilTheme::success_green(dark_mode)
                        } else {
                            GeoEstabilTheme::slip_circle_red(dark_mode)
                        };
                        rich_math_label(
                            ui,
                            &format!(
                                "{} V_{{Rd1}} = {:.1} kN/m → {} (V_{{d}} = {:.1} kN/m)",
                                localizer.pick("Cortante", "Shear"),
                                sec.v_rd1_kn_m,
                                if sec.is_shear_ok {
                                    localizer.t("tab4.shear_ok")
                                } else {
                                    localizer.t("tab4.shear_fail")
                                },
                                sec.vd_kn_m
                            ),
                            12.6,
                            true,
                            dark_mode,
                            Some(shear_color),
                        );

                        ui.separator();
                        let crack_color = if sec.is_crack_width_ok {
                            GeoEstabilTheme::success_green(dark_mode)
                        } else {
                            GeoEstabilTheme::warning_amber(dark_mode)
                        };
                        rich_math_label(
                            ui,
                            &format!(
                                "{}: w_{{k}} ≈ {:.2} mm ({}: 0.30 mm)",
                                localizer.pick("Abertura de Fissuras", "Crack Width"),
                                sec.crack_width_wk_mm,
                                localizer.pick("Limite", "Limit")
                            ),
                            12.6,
                            false,
                            dark_mode,
                            Some(crack_color),
                        );
                    });
                });
                ui.add_space(8.0);
            }

            // Armadura Horizontal de Distribuição
            ui.group(|ui| {
                ui.heading(localizer.t("tab4.horiz_title"));
                rich_math_label(
                    ui,
                    &format!(
                        "A_{{s,dist}} = {:.2} cm²/m {}",
                        structural.horizontal_distribution_as_cm2_m,
                        localizer.pick(
                            "por face (≥ 20% A_{s} e ≥ 0.15% da seção de concreto).",
                            "per face (≥ 20% A_{s} and ≥ 0.15% of concrete section)."
                        )
                    ),
                    12.6,
                    false,
                    dark_mode,
                    None,
                );
                rich_math_label(
                    ui,
                    &format!(
                        "{}: {}",
                        localizer.pick("Armadura Adotada por Face", "Adopted Rebar per Face"),
                        structural
                            .horizontal_distribution_rebar
                            .localized_description(lang)
                    ),
                    13.0,
                    true,
                    dark_mode,
                    Some(GeoEstabilTheme::rebar_orange(dark_mode)),
                );
            });
        });
    }
}
