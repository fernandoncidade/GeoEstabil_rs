use crate::core::types::RetainingWallModel;
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::geometry::decompose_wall_geometry;

#[derive(Debug, Clone)]
pub struct WallStabilityReport {
    pub total_vertical_load: f64,
    pub total_stabilizing_moment: f64,
    pub total_overturning_moment: f64,
    pub fs_overturning: f64,
    pub is_overturning_ok: bool,
    pub total_driving_horizontal_force: f64,
    pub total_resisting_horizontal_force: f64,
    pub fs_sliding: f64,
    pub is_sliding_ok: bool,
    pub resultant_position_xr: f64,
    pub eccentricity: f64,
    pub max_allowed_eccentricity: f64,
    pub is_in_middle_third: bool,
    pub sigma_toe_kpa: f64,
    pub sigma_heel_kpa: f64,
    pub sigma_adm_kpa: f64,
    pub is_bearing_stress_ok: bool,
}

pub fn check_wall_external_stability(model: &RetainingWallModel) -> WallStabilityReport {
    let geom_decomp = decompose_wall_geometry(model);
    let pressures = calculate_earth_pressures(model);
    let b = model.geometry.base_width;

    // Componente vertical do empuxo ativo aplicada no tardoz
    let ea_v_arm = b; // No plano posterior da base
    let moment_ea_v = pressures.ea_vertical * ea_v_arm;

    // Momento Estabilizante Total em relação à ponta (O)
    let total_m_est = geom_decomp.total_stabilizing_moment + moment_ea_v;

    // Momento Tombante Total em relação à ponta (O)
    let total_m_tomb = pressures.overturning_moment;

    let fs_overturning = if total_m_tomb > 0.001 {
        (total_m_est / total_m_tomb).clamp(0.0, 99.99)
    } else {
        99.99
    };
    let is_overturning_ok = fs_overturning >= 1.50;

    // Força vertical resultante total N
    let total_n = geom_decomp.total_vertical_dead_load + pressures.ea_vertical;

    // Verificação ao Deslizamento
    let delta_base_rad = model.base_friction_deg.to_radians();
    let ca = model.base_adhesion_kpa;
    // Considera 50% do empuxo passivo como margem de segurança conservadora da prática
    let ep_considered = pressures.ep_horizontal * 0.50;

    // Adiciona dente de ancoragem se presente
    let extra_key_passive = if model.geometry.shear_key_enabled {
        let key_h = model.geometry.shear_key_depth;
        let gamma_f = model.foundation_soil.gamma;
        0.5 * gamma_f * key_h.powi(2) * pressures.kp
    } else {
        0.0
    };

    let total_f_resist = total_n * delta_base_rad.tan() + ca * b + ep_considered + extra_key_passive;
    let total_f_driving = pressures.ea_horizontal;

    let fs_sliding = if total_f_driving > 0.001 {
        (total_f_resist / total_f_driving).clamp(0.0, 99.99)
    } else {
        99.99
    };
    let is_sliding_ok = fs_sliding >= 1.50;

    // Posição da resultante e Excentricidade
    let net_moment = total_m_est - total_m_tomb;
    let xr = if total_n > 0.001 {
        (net_moment / total_n).clamp(0.0, b * 2.0)
    } else {
        b * 0.5
    };

    let eccentricity = (b * 0.5 - xr).abs();
    let max_allowed_eccentricity = b / 6.0;
    let is_in_middle_third = eccentricity <= max_allowed_eccentricity;

    // Tensões de contato na base do solo
    let (sigma_toe, sigma_heel) = if eccentricity <= max_allowed_eccentricity {
        let avg = total_n / b;
        let diff = (6.0 * eccentricity) / b;
        let s_toe = avg * (1.0 + diff);
        let s_heel = (avg * (1.0 - diff)).max(0.0);
        (s_toe, s_heel)
    } else {
        // Descolamento parcial (tração não admitida no solo)
        let s_toe = if xr > 0.01 {
            (2.0 * total_n) / (3.0 * xr)
        } else {
            999.0
        };
        (s_toe, 0.0)
    };

    let sigma_adm = model.foundation_soil.sigma_adm;
    let is_bearing_stress_ok = sigma_toe <= sigma_adm;

    WallStabilityReport {
        total_vertical_load: total_n,
        total_stabilizing_moment: total_m_est,
        total_overturning_moment: total_m_tomb,
        fs_overturning,
        is_overturning_ok,
        total_driving_horizontal_force: total_f_driving,
        total_resisting_horizontal_force: total_f_resist,
        fs_sliding,
        is_sliding_ok,
        resultant_position_xr: xr,
        eccentricity,
        max_allowed_eccentricity,
        is_in_middle_third,
        sigma_toe_kpa: sigma_toe,
        sigma_heel_kpa: sigma_heel,
        sigma_adm_kpa: sigma_adm,
        is_bearing_stress_ok,
    }
}
