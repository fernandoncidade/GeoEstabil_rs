use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::core::types::*;
use crate::locale::translations::Language;

pub struct PresetCase {
    pub name: &'static str,
    pub description: &'static str,
    pub mode: ActiveProjectMode,
    pub project: GeoProject,
}

impl PresetCase {
    pub fn localized_name(&self, lang: Language) -> &'static str {
        if lang == Language::PtBr {
            return self.name;
        }
        match self.name {
            "Caso 1: Talude Rodoviário em Corte — Solo Residual de Gnaisse (NBR 11682)" => {
                "Case 1: Highway Cut Slope — Gneiss Residual Soil (NBR 11682)"
            }
            "Caso 2: Aterro sobre Argila Mole com Lençol Freático Elevado" => {
                "Case 2: Embankment over Soft Clay with High Water Table"
            }
            "Caso 3: Muro de Arrimo Cantilever de Concreto Armado H=4.5m (NBR 6118 / NBR 6122)" => {
                "Case 3: RC Cantilever Retaining Wall H=4.5m (NBR 6118 / NBR 6122)"
            }
            "Caso 4: Muro de Gravidade em Concreto Ciclópico H=3.5m" => {
                "Case 4: Cyclopean Concrete Gravity Wall H=3.5m"
            }
            other => other,
        }
    }

    pub fn localized_description(&self, lang: Language) -> &'static str {
        if lang == Language::PtBr {
            return self.description;
        }
        match self.name {
            "Caso 1: Talude Rodoviário em Corte — Solo Residual de Gnaisse (NBR 11682)" => {
                "10m high highway cut slope in gneiss residual soil (1:1 inclination), 20 kPa traffic surcharge on crest, and lowered phreatic line. High Risk (min FS ≥ 1.50)."
            }
            "Caso 2: Aterro sobre Argila Mole com Lençol Freático Elevado" => {
                "6m compacted embankment over very soft compressible clay foundation with shallow groundwater table. Global stability critical check."
            }
            "Caso 3: Muro de Arrimo Cantilever de Concreto Armado H=4.5m (NBR 6118 / NBR 6122)" => {
                "L/T inverted reinforced concrete cantilever retaining wall (H=4.5m), drained granular backfill, footing B=2.70m, 10 kPa sidewalk surcharge, full external and RC flexural design."
            }
            "Caso 4: Muro de Gravidade em Concreto Ciclópico H=3.5m" => {
                "Trapezoidal cyclopean concrete gravity retaining wall (H=3.5m), crest b=0.50m, base B=1.90m, silty retained soil, and external stability check."
            }
            other => other,
        }
    }

    pub fn project_for_lang(mut self, lang: Language) -> GeoProject {
        self.project.translate_defaults(Language::PtBr, lang);
        self.project
    }
}

pub fn get_preset_cases() -> Vec<PresetCase> {
    vec![
        PresetCase {
            name: "Caso 1: Talude Rodoviário em Corte — Solo Residual de Gnaisse (NBR 11682)",
            description: "Talude de corte em solo residual de gnaisse com altura de 10m, inclinação 1:1, sobrecarga de tráfego de 20 kPa na crista e nível freático rebaixado. Risco Alto (FS mínimo ≥ 1.50).",
            mode: ActiveProjectMode::SlopeStability,
            project: {
                let mut p = GeoProject::default();
                p.title = "Talude Rodoviário em Corte — Solo Residual de Gnaisse".to_string();
                p.active_mode = ActiveProjectMode::SlopeStability;
                p.slope_model = SlopeModel {
                    terrain_surface: Polyline::new(vec![
                        Point2D::new(0.0, 0.0),
                        Point2D::new(10.0, 0.0),
                        Point2D::new(20.0, 10.0),
                        Point2D::new(45.0, 10.0),
                    ]),
                    layers: vec![
                        SoilLayer {
                            id: 1,
                            name: "Solo Residual Maduro (Colúvio)".to_string(),
                            description: "Argila siltosa vermelha".to_string(),
                            gamma: 18.0,
                            gamma_sat: 19.5,
                            cohesion: 15.0,
                            friction_angle_deg: 27.0,
                            sigma_adm: 200.0,
                            top_profile: Polyline::new(vec![
                                Point2D::new(0.0, 0.0),
                                Point2D::new(10.0, 0.0),
                                Point2D::new(20.0, 10.0),
                                Point2D::new(45.0, 10.0),
                            ]),
                            color_rgba: [205, 133, 63, 200],
                        },
                        SoilLayer {
                            id: 2,
                            name: "Saprolito / Solo de Alteração de Gnaisse".to_string(),
                            description: "Silte arenoso compacto".to_string(),
                            gamma: 19.5,
                            gamma_sat: 20.5,
                            cohesion: 30.0,
                            friction_angle_deg: 33.0,
                            sigma_adm: 350.0,
                            top_profile: Polyline::new(vec![
                                Point2D::new(0.0, -3.0),
                                Point2D::new(12.0, -3.0),
                                Point2D::new(22.0, 5.0),
                                Point2D::new(45.0, 5.0),
                            ]),
                            color_rgba: [139, 69, 19, 220],
                        },
                    ],
                    water_table: WaterTable {
                        enabled: true,
                        gamma_w: 9.81,
                        phreatic_line: Polyline::new(vec![
                            Point2D::new(0.0, 0.0),
                            Point2D::new(12.0, 1.0),
                            Point2D::new(22.0, 6.0),
                            Point2D::new(45.0, 6.0),
                        ]),
                    },
                    surcharge: Surcharge {
                        enabled: true,
                        name: "Sobrecarga de Tráfego Rodoviário (DNIT)".to_string(),
                        q_kpa: 20.0,
                        x_start: 22.0,
                        x_end: 45.0,
                    },
                    safety_level: SafetyLevelNBR11682::High,
                    method: SlopeAnalysisMethod::BishopSimplified,
                    num_slices: 40,
                    grid_x_min: 6.0,
                    grid_x_max: 24.0,
                    grid_x_steps: 25,
                    grid_y_min: 10.0,
                    grid_y_max: 26.0,
                    grid_y_steps: 25,
                    radius_min: 10.0,
                    radius_max: 30.0,
                    radius_steps: 25,
                };
                p
            },
        },
        PresetCase {
            name: "Caso 2: Aterro sobre Argila Mole com Lençol Freático Elevado",
            description: "Aterro compactado de 6m sobre fundação em argila mole muito compressível com nível d'água aflorante. Verificação crítica de estabilidade global.",
            mode: ActiveProjectMode::SlopeStability,
            project: {
                let mut p = GeoProject::default();
                p.title = "Aterro sobre Argila Mole de Baixada".to_string();
                p.active_mode = ActiveProjectMode::SlopeStability;
                p.slope_model = SlopeModel {
                    terrain_surface: Polyline::new(vec![
                        Point2D::new(0.0, 0.0),
                        Point2D::new(15.0, 0.0),
                        Point2D::new(27.0, 6.0),
                        Point2D::new(50.0, 6.0),
                    ]),
                    layers: vec![
                        SoilLayer {
                            id: 1,
                            name: "Corpo do Aterro Compactado".to_string(),
                            description: "Argila arenosa bem compactada".to_string(),
                            gamma: 19.0,
                            gamma_sat: 20.0,
                            cohesion: 10.0,
                            friction_angle_deg: 28.0,
                            sigma_adm: 250.0,
                            top_profile: Polyline::new(vec![
                                Point2D::new(0.0, 0.0),
                                Point2D::new(15.0, 0.0),
                                Point2D::new(27.0, 6.0),
                                Point2D::new(50.0, 6.0),
                            ]),
                            color_rgba: [210, 105, 30, 200],
                        },
                        SoilLayer {
                            id: 2,
                            name: "Argila Mole Orgânica de Baixada".to_string(),
                            description: "Argila muito mole e saturada (SPT 1 a 2)".to_string(),
                            gamma: 15.0,
                            gamma_sat: 15.5,
                            cohesion: 12.0,
                            friction_angle_deg: 6.0,
                            sigma_adm: 50.0,
                            top_profile: Polyline::new(vec![
                                Point2D::new(0.0, 0.0),
                                Point2D::new(50.0, 0.0),
                            ]),
                            color_rgba: [105, 105, 105, 230],
                        },
                    ],
                    water_table: WaterTable {
                        enabled: true,
                        gamma_w: 9.81,
                        phreatic_line: Polyline::new(vec![
                            Point2D::new(0.0, 0.0),
                            Point2D::new(50.0, 0.0),
                        ]),
                    },
                    surcharge: Surcharge {
                        enabled: true,
                        name: "Carga de Pavimentação e Cargas Acidentais".to_string(),
                        q_kpa: 15.0,
                        x_start: 28.0,
                        x_end: 50.0,
                    },
                    safety_level: SafetyLevelNBR11682::High,
                    method: SlopeAnalysisMethod::BishopSimplified,
                    num_slices: 40,
                    grid_x_min: 10.0,
                    grid_x_max: 30.0,
                    grid_x_steps: 25,
                    grid_y_min: 6.0,
                    grid_y_max: 22.0,
                    grid_y_steps: 25,
                    radius_min: 7.0,
                    radius_max: 26.0,
                    radius_steps: 25,
                };
                p
            },
        },
        PresetCase {
            name: "Caso 3: Muro de Arrimo Cantilever de Concreto Armado H=4.5m (NBR 6118 / NBR 6122)",
            description: "Muro de contenção em perfil L/T invertido de concreto armado com H=4.5m, aterro granular drenado, sapata com B=2.60m, sobrecarga de calçada de 10 kPa e verificação completa contra tombamento, deslizamento e flexão do fuste.",
            mode: ActiveProjectMode::RetainingWall,
            project: {
                let mut p = GeoProject::default();
                p.title = "Muro de Arrimo em Concreto Armado (Cantilever H=4.5m)".to_string();
                p.active_mode = ActiveProjectMode::RetainingWall;
                p.wall_model = RetainingWallModel {
                    wall_type: RetainingWallType::Cantilever,
                    theory: EarthPressureTheory::Rankine,
                    geometry: WallGeometry {
                        stem_height: 4.0,
                        stem_top_width: 0.30,
                        stem_base_width: 0.50,
                        base_width: 2.70,
                        base_thickness: 0.50,
                        toe_length: 0.70,
                        backfill_slope_deg: 0.0,
                        wall_back_slope_deg: 90.0,
                        shear_key_enabled: false,
                        shear_key_depth: 0.40,
                        shear_key_width: 0.40,
                    },
                    concrete: ConcreteStructuralProperties {
                        fck_mpa: 25.0,
                        fyk_mpa: 500.0,
                        gamma_concrete: 25.0,
                        cover_nom_mm: 35.0,
                        gamma_c: 1.40,
                        gamma_s: 1.15,
                        gamma_f: 1.40,
                    },
                    backfill_soil: SoilLayer {
                        id: 1,
                        name: "Aterro Arenoso Drenado".to_string(),
                        description: "Areia média compactada".to_string(),
                        gamma: 18.5,
                        gamma_sat: 20.0,
                        cohesion: 2.0,
                        friction_angle_deg: 32.0,
                        sigma_adm: 220.0,
                        top_profile: Polyline::new(vec![]),
                        color_rgba: [218, 165, 32, 200],
                    },
                    foundation_soil: SoilLayer {
                        id: 2,
                        name: "Solo de Fundação (Argila Siltosa Rija)".to_string(),
                        description: "Solo de boa consistência (SPT 10 a 14)".to_string(),
                        gamma: 19.0,
                        gamma_sat: 20.5,
                        cohesion: 22.0,
                        friction_angle_deg: 25.0,
                        sigma_adm: 260.0,
                        top_profile: Polyline::new(vec![]),
                        color_rgba: [160, 82, 45, 200],
                    },
                    delta_ratio: 0.667,
                    surcharge_q: 10.0,
                    water_height_back: 0.0,
                    base_adhesion_kpa: 12.0,
                    base_friction_deg: 24.0,
                };
                p
            },
        },
        PresetCase {
            name: "Caso 4: Muro de Gravidade em Concreto Ciclópico H=3.5m",
            description: "Muro de gravidade trapezoidal em concreto ciclópico com H=3.5m, crista b=0.50m, base B=1.90m, solo siltoso contido e verificação de estabilidade externa.",
            mode: ActiveProjectMode::RetainingWall,
            project: {
                let mut p = GeoProject::default();
                p.title = "Muro de Gravidade em Concreto Ciclópico (H=3.5m)".to_string();
                p.active_mode = ActiveProjectMode::RetainingWall;
                p.wall_model = RetainingWallModel {
                    wall_type: RetainingWallType::Gravity,
                    theory: EarthPressureTheory::Coulomb,
                    geometry: WallGeometry {
                        stem_height: 3.50,
                        stem_top_width: 0.50,
                        stem_base_width: 1.90,
                        base_width: 1.90,
                        base_thickness: 0.0, // Face contínua do muro de gravidade
                        toe_length: 0.0,
                        backfill_slope_deg: 10.0,
                        wall_back_slope_deg: 80.0,
                        shear_key_enabled: false,
                        shear_key_depth: 0.0,
                        shear_key_width: 0.0,
                    },
                    concrete: ConcreteStructuralProperties {
                        fck_mpa: 15.0,
                        fyk_mpa: 500.0,
                        gamma_concrete: 22.0, // Concreto ciclópico com pedras de mão
                        cover_nom_mm: 50.0,
                        gamma_c: 1.40,
                        gamma_s: 1.15,
                        gamma_f: 1.40,
                    },
                    backfill_soil: SoilLayer {
                        id: 1,
                        name: "Solo Natural de Encosta".to_string(),
                        description: "Silte argiloso medianamente compacto".to_string(),
                        gamma: 18.0,
                        gamma_sat: 19.5,
                        cohesion: 8.0,
                        friction_angle_deg: 28.0,
                        sigma_adm: 180.0,
                        top_profile: Polyline::new(vec![]),
                        color_rgba: [205, 133, 63, 200],
                    },
                    foundation_soil: SoilLayer {
                        id: 2,
                        name: "Solo de Apoio da Fundação".to_string(),
                        description: "Areia siltosa compacta (SPT 15)".to_string(),
                        gamma: 19.5,
                        gamma_sat: 20.5,
                        cohesion: 15.0,
                        friction_angle_deg: 32.0,
                        sigma_adm: 300.0,
                        top_profile: Polyline::new(vec![]),
                        color_rgba: [139, 69, 19, 200],
                    },
                    delta_ratio: 0.667,
                    surcharge_q: 5.0,
                    water_height_back: 0.0,
                    base_adhesion_kpa: 10.0,
                    base_friction_deg: 30.0,
                };
                p
            },
        },
    ]
}
