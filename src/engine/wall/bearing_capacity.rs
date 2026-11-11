use crate::core::types::RetainingWallModel;
use crate::engine::wall::stability::WallStabilityReport;

#[derive(Debug, Clone)]
pub struct BearingCapacityReport {
    pub nc: f64,
    pub nq: f64,
    pub n_gamma: f64,
    pub effective_width_b_prime: f64,
    pub q0_surcharge_kpa: f64,
    pub ic: f64,
    pub iq: f64,
    pub i_gamma: f64,
    pub q_ult_kpa: f64,
    pub fs_bearing: f64,
    pub is_bearing_ok: bool,
}

pub fn calculate_bearing_capacity(
    model: &RetainingWallModel,
    stability: &WallStabilityReport,
) -> BearingCapacityReport {
    let phi_deg = model.foundation_soil.friction_angle_deg;
    let phi_rad = phi_deg.to_radians();
    let c = model.foundation_soil.cohesion;
    let gamma = model.foundation_soil.gamma;
    let df = model.geometry.base_thickness; // Cota de assentamento
    let b = model.geometry.base_width;

    // Fatores de capacidade de carga Nq, Nc, Ngamma
    let (nc, nq, n_gamma) = if phi_deg > 0.5 {
        let nq_val = (std::f64::consts::PI * phi_rad.tan()).exp()
            * ((std::f64::consts::FRAC_PI_4 + phi_rad * 0.5).tan()).powi(2);
        let nc_val = (nq_val - 1.0) / phi_rad.tan();
        let ng_val = 2.0 * (nq_val + 1.0) * phi_rad.tan();
        (nc_val, nq_val, ng_val)
    } else {
        (5.14, 1.0, 0.0)
    };

    // Largura efetiva de Meyerhof: B' = B - 2e
    let b_prime = (b - 2.0 * stability.eccentricity).max(0.10);

    // Sobrecarga efetiva na cota de apoio
    let q0 = gamma * df;

    // Fatores de inclinação de carga (Hansen / Vesic)
    let hl = stability.total_driving_horizontal_force;
    let vl = stability.total_vertical_load;

    let ca_term = if phi_deg > 0.5 {
        b_prime * c / phi_rad.tan()
    } else {
        b_prime * c
    };
    let denom = (vl + ca_term).max(1.0);

    let iq = (1.0 - (0.5 * hl) / denom).clamp(0.1, 1.0).powi(5);
    let ic = if phi_deg > 0.5 {
        (iq - (1.0 - iq) / (nc * phi_rad.tan())).clamp(0.1, 1.0)
    } else {
        (1.0 - (0.4 * hl) / (b_prime * c * 5.14)).clamp(0.1, 1.0)
    };
    let i_gamma = (1.0 - (0.7 * hl) / denom).clamp(0.1, 1.0).powi(5);

    // Capacidade de carga última q_ult
    let term_c = c * nc * ic;
    let term_q = q0 * nq * iq;
    let term_gamma = 0.5 * gamma * b_prime * n_gamma * i_gamma;

    let q_ult = (term_c + term_q + term_gamma).max(10.0);

    // FS contra ruptura do solo de fundação: FS = q_ult / sigma_toe
    let fs_bearing = if stability.sigma_toe_kpa > 0.01 {
        (q_ult / stability.sigma_toe_kpa).clamp(0.0, 99.99)
    } else {
        99.99
    };

    // NBR 6122 exige FS >= 2.0 a 3.0 para ruptura do maciço
    let is_bearing_ok = fs_bearing >= 2.0;

    BearingCapacityReport {
        nc,
        nq,
        n_gamma,
        effective_width_b_prime: b_prime,
        q0_surcharge_kpa: q0,
        ic,
        iq,
        i_gamma,
        q_ult_kpa: q_ult,
        fs_bearing,
        is_bearing_ok,
    }
}
