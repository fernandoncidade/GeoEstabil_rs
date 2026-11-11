use crate::core::types::{EarthPressureTheory, RetainingWallModel};

#[derive(Debug, Clone)]
pub struct EarthPressureResults {
    pub ka: f64,
    pub kp: f64,
    pub delta_rad: f64,
    pub ea_soil: f64,
    pub ea_surcharge: f64,
    pub ea_water: f64,
    pub ea_total: f64,
    pub ea_horizontal: f64,
    pub ea_vertical: f64,
    pub ea_arm_y: f64,
    pub overturning_moment: f64,
    pub ep_passive: f64,
    pub ep_horizontal: f64,
    pub tension_crack_depth: f64,
}

pub fn calculate_earth_pressures(model: &RetainingWallModel) -> EarthPressureResults {
    let phi_deg = model.backfill_soil.friction_angle_deg;
    let phi_rad = phi_deg.to_radians();
    let c = model.backfill_soil.cohesion;
    let gamma = model.backfill_soil.gamma;
    let gamma_sat = model.backfill_soil.gamma_sat;
    let gamma_w = 9.81;

    let h_total = model.geometry.total_height();
    let delta_deg = phi_deg * model.delta_ratio;
    let delta_rad = delta_deg.to_radians();

    let beta_deg = model.geometry.backfill_slope_deg;
    let beta_rad = beta_deg.to_radians();

    // Inclinação do tardoz com a vertical: theta = 90° - alpha
    let alpha_deg = model.geometry.wall_back_slope_deg;
    let theta_rad = (90.0 - alpha_deg).to_radians();

    // Coeficientes Ka e Kp
    let (ka, kp) = match model.theory {
        EarthPressureTheory::Rankine => {
            let ka_rankine = ((std::f64::consts::FRAC_PI_4 - phi_rad * 0.5).tan()).powi(2);
            let kp_rankine = ((std::f64::consts::FRAC_PI_4 + phi_rad * 0.5).tan()).powi(2);
            (ka_rankine, kp_rankine)
        }
        EarthPressureTheory::Coulomb => {
            // Formulação analítica de Coulomb para empuxo ativo
            let sin_phi_plus_delta = (phi_rad + delta_rad).sin();
            let sin_phi_minus_beta = (phi_rad - beta_rad).sin();
            let cos_theta_plus_delta = (theta_rad + delta_rad).cos();
            let cos_theta_minus_beta = (theta_rad - beta_rad).cos();

            let rad_arg = (sin_phi_plus_delta * sin_phi_minus_beta)
                / (cos_theta_plus_delta * cos_theta_minus_beta);

            let ka_coulomb = if rad_arg >= 0.0 && sin_phi_minus_beta >= 0.0 {
                let term = 1.0 + rad_arg.sqrt();
                let num = (phi_rad - theta_rad).cos().powi(2);
                let den = theta_rad.cos().powi(2) * cos_theta_plus_delta * term.powi(2);
                num / den
            } else {
                ((std::f64::consts::FRAC_PI_4 - phi_rad * 0.5).tan()).powi(2)
            };

            // Kp de Coulomb aproximado
            let kp_coulomb = ((std::f64::consts::FRAC_PI_4 + phi_rad * 0.5).tan()).powi(2);
            (ka_coulomb, kp_coulomb)
        }
    };

    // Profundidade de fenda de tração (Rankine c' - phi'): z0 = 2c / (gamma * sqrt(Ka))
    let tension_crack_depth = if c > 0.0 && ka > 0.0 {
        (2.0 * c / (gamma * ka.sqrt())).min(h_total * 0.5)
    } else {
        0.0
    };

    // Nível d'água no tardoz
    let hw = model.water_height_back.clamp(0.0, h_total);
    let h_dry = (h_total - hw).max(0.0);

    // Empuxo do Solo:
    // Parte seca (triangular ou trapezoidal se houver água abaixo)
    let ea_soil = if hw <= 0.001 {
        0.5 * gamma * h_total.powi(2) * ka
    } else {
        let p_dry_base = gamma * h_dry * ka;
        let ea_dry = 0.5 * gamma * h_dry.powi(2) * ka;
        let gamma_sub = (gamma_sat - gamma_w).max(8.0);
        let ea_sub_rect = p_dry_base * hw;
        let ea_sub_tri = 0.5 * gamma_sub * hw.powi(2) * ka;
        ea_dry + ea_sub_rect + ea_sub_tri
    };

    // Empuxo da Sobrecarga q (distribuição retangular uniforme): sigma_q = q * Ka
    let ea_surcharge = model.surcharge_q * ka * h_total;

    // Empuxo Hidrostático da Água: Ew = 0.5 * gamma_w * hw^2
    let ea_water = if hw > 0.0 {
        0.5 * gamma_w * hw.powi(2)
    } else {
        0.0
    };

    let ea_total = ea_soil + ea_surcharge + ea_water;

    // Componentes horizontal e vertical do empuxo ativo
    // Para Rankine puro, delta = 0 => Ea_h = Ea, Ea_v = 0.
    // Para Coulomb, Ea inclina com ângulo delta em relação à normal ao tardoz.
    let angle_to_horizontal = if matches!(model.theory, EarthPressureTheory::Coulomb) {
        delta_rad + theta_rad
    } else {
        0.0
    };

    let ea_horizontal = (ea_soil + ea_surcharge) * angle_to_horizontal.cos() + ea_water;
    let ea_vertical = (ea_soil + ea_surcharge) * angle_to_horizontal.sin();

    // Braços de alavanca em relação à base da sapata
    let arm_soil = if hw <= 0.001 { h_total / 3.0 } else { h_total * 0.35 };
    let arm_surcharge = h_total * 0.5;
    let arm_water = hw / 3.0;

    let overturning_moment = (ea_soil * angle_to_horizontal.cos()) * arm_soil
        + (ea_surcharge * angle_to_horizontal.cos()) * arm_surcharge
        + ea_water * arm_water;

    let ea_arm_y = if ea_horizontal > 0.0 {
        overturning_moment / ea_horizontal
    } else {
        h_total / 3.0
    };

    // Empuxo passivo na frente do muro (profundidade de embutimento Df = h_base)
    let df = model.geometry.base_thickness;
    let gamma_fund = model.foundation_soil.gamma;
    let c_fund = model.foundation_soil.cohesion;
    let ep_passive = 0.5 * gamma_fund * df.powi(2) * kp + 2.0 * c_fund * kp.sqrt() * df;
    let ep_horizontal = ep_passive;

    EarthPressureResults {
        ka,
        kp,
        delta_rad,
        ea_soil,
        ea_surcharge,
        ea_water,
        ea_total,
        ea_horizontal,
        ea_vertical,
        ea_arm_y,
        overturning_moment,
        ep_passive,
        ep_horizontal,
        tension_crack_depth,
    }
}
