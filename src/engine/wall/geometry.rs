use crate::core::types::{RetainingWallModel, RetainingWallType};

#[derive(Debug, Clone)]
pub struct WallComponentWeight {
    pub name: String,
    pub area_m2: f64,
    pub weight_kn_m: f64,
    pub arm_x_m: f64,
    pub moment_knm_m: f64,
}

#[derive(Debug, Clone)]
pub struct WallGeometricDecomposition {
    pub components: Vec<WallComponentWeight>,
    pub total_vertical_dead_load: f64,
    pub total_stabilizing_moment: f64,
    pub center_of_gravity_x: f64,
    pub heel_soil_weight: f64,
    pub heel_soil_arm: f64,
}

pub fn decompose_wall_geometry(model: &RetainingWallModel) -> WallGeometricDecomposition {
    let mut components = Vec::new();
    let geom = &model.geometry;
    let gamma_c = model.concrete.gamma_concrete;
    let gamma_s = model.backfill_soil.gamma;

    match model.wall_type {
        RetainingWallType::Cantilever => {
            // 1. Sapata de Fundação (Retângulo)
            let base_area = geom.base_width * geom.base_thickness;
            let base_weight = base_area * gamma_c;
            let base_arm = geom.base_width * 0.5;
            components.push(WallComponentWeight {
                name: "Sapata de Fundação (Base)".to_string(),
                area_m2: base_area,
                weight_kn_m: base_weight,
                arm_x_m: base_arm,
                moment_knm_m: base_weight * base_arm,
            });

            // 2. Fuste - Parte Retangular (crista com espessura b_top)
            let stem_rect_area = geom.stem_top_width * geom.stem_height;
            let stem_rect_weight = stem_rect_area * gamma_c;
            let stem_rect_arm = geom.toe_length + geom.stem_top_width * 0.5;
            components.push(WallComponentWeight {
                name: "Fuste Retangular de Concreto".to_string(),
                area_m2: stem_rect_area,
                weight_kn_m: stem_rect_weight,
                arm_x_m: stem_rect_arm,
                moment_knm_m: stem_rect_weight * stem_rect_arm,
            });

            // 3. Fuste - Parte Triangular (alargamento na base do fuste)
            let stem_tri_width = (geom.stem_base_width - geom.stem_top_width).max(0.0);
            if stem_tri_width > 0.001 {
                let stem_tri_area = 0.5 * stem_tri_width * geom.stem_height;
                let stem_tri_weight = stem_tri_area * gamma_c;
                let stem_tri_arm = geom.toe_length + geom.stem_top_width + stem_tri_width * (1.0 / 3.0);
                components.push(WallComponentWeight {
                    name: "Fuste Triangular (Tardoz Inclinado)".to_string(),
                    area_m2: stem_tri_area,
                    weight_kn_m: stem_tri_weight,
                    arm_x_m: stem_tri_arm,
                    moment_knm_m: stem_tri_weight * stem_tri_arm,
                });
            }

            // 4. Prisma de Solo sobre o Calcanhar (Heel)
            let heel_length = geom.heel_length();
            let mut heel_soil_weight = 0.0;
            let mut heel_soil_arm = 0.0;
            if heel_length > 0.01 {
                let soil_rect_area = heel_length * geom.stem_height;
                heel_soil_weight = soil_rect_area * gamma_s;
                heel_soil_arm = geom.toe_length + geom.stem_base_width + heel_length * 0.5;
                components.push(WallComponentWeight {
                    name: "Solo de Reaterro sobre o Calcanhar".to_string(),
                    area_m2: soil_rect_area,
                    weight_kn_m: heel_soil_weight,
                    arm_x_m: heel_soil_arm,
                    moment_knm_m: heel_soil_weight * heel_soil_arm,
                });
            }

            // 5. Dente de Ancoragem (se ativo)
            if geom.shear_key_enabled && geom.shear_key_depth > 0.01 {
                let key_area = geom.shear_key_width * geom.shear_key_depth;
                let key_weight = key_area * gamma_c;
                let key_arm = geom.toe_length + geom.stem_base_width * 0.5;
                components.push(WallComponentWeight {
                    name: "Dente de Ancoragem (Chave de Cisalhamento)".to_string(),
                    area_m2: key_area,
                    weight_kn_m: key_weight,
                    arm_x_m: key_arm,
                    moment_knm_m: key_weight * key_arm,
                });
            }

            let total_v = components.iter().map(|c| c.weight_kn_m).sum::<f64>();
            let total_m = components.iter().map(|c| c.moment_knm_m).sum::<f64>();
            let cg_x = if total_v > 0.0 { total_m / total_v } else { 0.0 };

            WallGeometricDecomposition {
                components,
                total_vertical_dead_load: total_v,
                total_stabilizing_moment: total_m,
                center_of_gravity_x: cg_x,
                heel_soil_weight,
                heel_soil_arm,
            }
        }
        RetainingWallType::Gravity => {
            // Muro de gravidade trapezoidal
            // Retângulo crista
            let rect_area = geom.stem_top_width * geom.stem_height;
            let rect_weight = rect_area * gamma_c;
            let rect_arm = geom.stem_top_width * 0.5;
            components.push(WallComponentWeight {
                name: "Corpo Central de Gravidade".to_string(),
                area_m2: rect_area,
                weight_kn_m: rect_weight,
                arm_x_m: rect_arm,
                moment_knm_m: rect_weight * rect_arm,
            });

            // Triângulo posterior/tardoz
            let tri_width = (geom.base_width - geom.stem_top_width).max(0.0);
            if tri_width > 0.001 {
                let tri_area = 0.5 * tri_width * geom.stem_height;
                let tri_weight = tri_area * gamma_c;
                let tri_arm = geom.stem_top_width + tri_width * (2.0 / 3.0);
                components.push(WallComponentWeight {
                    name: "Talude Interno do Muro".to_string(),
                    area_m2: tri_area,
                    weight_kn_m: tri_weight,
                    arm_x_m: tri_arm,
                    moment_knm_m: tri_weight * tri_arm,
                });
            }

            let total_v = components.iter().map(|c| c.weight_kn_m).sum::<f64>();
            let total_m = components.iter().map(|c| c.moment_knm_m).sum::<f64>();
            let cg_x = if total_v > 0.0 { total_m / total_v } else { 0.0 };

            WallGeometricDecomposition {
                components,
                total_vertical_dead_load: total_v,
                total_stabilizing_moment: total_m,
                center_of_gravity_x: cg_x,
                heel_soil_weight: 0.0,
                heel_soil_arm: 0.0,
            }
        }
    }
}
