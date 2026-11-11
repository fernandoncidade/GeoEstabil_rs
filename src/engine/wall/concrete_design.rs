use crate::core::types::RetainingWallModel;
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::stability::WallStabilityReport;
use crate::locale::translations::Language;

#[derive(Debug, Clone)]
pub struct RebarRecommendation {
    pub bar_diameter_mm: f64,
    pub spacing_cm: f64,
    pub area_provided_cm2_m: f64,
    pub description: String,
}

impl RebarRecommendation {
    pub fn localized_description(&self, lang: Language) -> String {
        match lang {
            Language::PtBr => self.description.clone(),
            Language::EnUs => format!(
                "Ø {:.1} mm @ {:.1} cm (A_s = {:.2} cm²/m)",
                self.bar_diameter_mm, self.spacing_cm, self.area_provided_cm2_m
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SectionReinforcementReport {
    pub section_name: String,
    pub mk_knm_m: f64,
    pub md_knm_m: f64,
    pub vk_kn_m: f64,
    pub vd_kn_m: f64,
    pub section_width_m: f64,
    pub section_height_m: f64,
    pub effective_depth_d_m: f64,
    pub neutral_axis_xi: f64,
    pub is_ductile: bool,
    pub as_calculated_cm2_m: f64,
    pub as_min_cm2_m: f64,
    pub as_required_cm2_m: f64,
    pub recommendation: RebarRecommendation,
    pub v_rd1_kn_m: f64,
    pub is_shear_ok: bool,
    pub crack_width_wk_mm: f64,
    pub is_crack_width_ok: bool,
}

impl SectionReinforcementReport {
    pub fn localized_section_name(&self, lang: Language) -> &str {
        if lang == Language::PtBr {
            return &self.section_name;
        }
        match self.section_name.as_str() {
            "Fuste (Base do Muro no Engaste)" => "Stem (Wall Base at Footing)",
            "Ponta da Sapata (Aba Anterior)" => "Footing Toe (Front Cantilever)",
            "Calcanhar da Sapata (Aba Posterior)" => "Footing Heel (Back Cantilever)",
            other => other,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetainingWallStructuralReport {
    pub stem_section: SectionReinforcementReport,
    pub toe_section: SectionReinforcementReport,
    pub heel_section: SectionReinforcementReport,
    pub horizontal_distribution_as_cm2_m: f64,
    pub horizontal_distribution_rebar: RebarRecommendation,
    pub fcd_mpa: f64,
    pub fyd_mpa: f64,
}

pub fn design_concrete_reinforcement(
    model: &RetainingWallModel,
    stability: &WallStabilityReport,
) -> RetainingWallStructuralReport {
    let conc = &model.concrete;
    let geom = &model.geometry;
    let gamma_f = conc.gamma_f;
    let fcd = (conc.fck_mpa / conc.gamma_c) * 1e3; // kPa
    let fyd = (conc.fyk_mpa / conc.gamma_s) * 1e3; // kPa
    let fcd_mpa = conc.fck_mpa / conc.gamma_c;
    let fyd_mpa = conc.fyk_mpa / conc.gamma_s;

    let pressures = calculate_earth_pressures(model);
    let ka = pressures.ka;
    let gamma_s = model.backfill_soil.gamma;
    let q = model.surcharge_q;
    let h_stem = geom.stem_height;

    // -------------------------------------------------------------
    // 1. FUSTE (Stem) — Seção Crítica no Engaste com a Sapata
    // -------------------------------------------------------------
    let ea_stem_soil = 0.5 * gamma_s * h_stem.powi(2) * ka;
    let arm_soil = h_stem / 3.0;
    let ea_stem_surcharge = q * ka * h_stem;
    let arm_surcharge = h_stem * 0.5;

    let vk_stem = ea_stem_soil + ea_stem_surcharge;
    let mk_stem = ea_stem_soil * arm_soil + ea_stem_surcharge * arm_surcharge;

    let vd_stem = vk_stem * gamma_f;
    let md_stem = mk_stem * gamma_f;

    let stem_h = geom.stem_base_width; // m
    let cover_m = conc.cover_nom_mm / 1000.0;
    let d_stem = (stem_h - cover_m - 0.008).max(0.10); // d estimado com barra de 16mm

    let stem_sec = dimension_flexural_section(
        "Fuste (Base do Muro no Engaste)",
        mk_stem,
        md_stem,
        vk_stem,
        vd_stem,
        1.0,
        stem_h,
        d_stem,
        conc.fck_mpa,
        conc.fyk_mpa,
        conc.gamma_c,
        fcd,
        fyd,
    );

    // -------------------------------------------------------------
    // 2. PONTA DA SAPATA (Toe) — Balanço da Aba Anterior
    // -------------------------------------------------------------
    let b_toe = geom.toe_length;
    let h_base = geom.base_thickness;
    let d_toe = (h_base - cover_m - 0.008).max(0.10);

    // Pressão do solo sob a ponta (média entre extrema e bordo do fuste)
    let b_total = geom.base_width;
    let slope_stress = if b_total > 0.01 {
        (stability.sigma_toe_kpa - stability.sigma_heel_kpa) / b_total
    } else {
        0.0
    };
    let sigma_toe_edge = stability.sigma_toe_kpa;
    let sigma_toe_stem = (sigma_toe_edge - slope_stress * b_toe).max(0.0);
    let avg_toe_stress = (sigma_toe_edge + sigma_toe_stem) * 0.5;

    // Carga resultante para cima menos peso próprio da ponta
    let net_toe_upward = (avg_toe_stress - conc.gamma_concrete * h_base).max(0.0);
    let vk_toe = net_toe_upward * b_toe;
    let mk_toe = 0.5 * net_toe_upward * b_toe.powi(2);

    let vd_toe = vk_toe * gamma_f;
    let md_toe = mk_toe * gamma_f;

    let toe_sec = dimension_flexural_section(
        "Ponta da Sapata (Aba Anterior)",
        mk_toe,
        md_toe,
        vk_toe,
        vd_toe,
        1.0,
        h_base,
        d_toe,
        conc.fck_mpa,
        conc.fyk_mpa,
        conc.gamma_c,
        fcd,
        fyd,
    );

    // -------------------------------------------------------------
    // 3. CALCANHAR DA SAPATA (Heel) — Balanço da Aba Posterior
    // -------------------------------------------------------------
    let b_heel = geom.heel_length();
    let d_heel = (h_base - cover_m - 0.008).max(0.10);

    // Carga atuante de cima para baixo: Peso do solo de reaterro + sobrecarga + peso próprio
    let downward_heel_stress = gamma_s * h_stem + q + conc.gamma_concrete * h_base;
    let avg_heel_upward = (stability.sigma_heel_kpa
        + (stability.sigma_heel_kpa + slope_stress * b_heel))
        * 0.5;
    let net_heel_downward = (downward_heel_stress - avg_heel_upward).max(0.0);

    let vk_heel = net_heel_downward * b_heel;
    let mk_heel = 0.5 * net_heel_downward * b_heel.powi(2);

    let vd_heel = vk_heel * gamma_f;
    let md_heel = mk_heel * gamma_f;

    let heel_sec = dimension_flexural_section(
        "Calcanhar da Sapata (Aba Posterior)",
        mk_heel,
        md_heel,
        vk_heel,
        vd_heel,
        1.0,
        h_base,
        d_heel,
        conc.fck_mpa,
        conc.fyk_mpa,
        conc.gamma_c,
        fcd,
        fyd,
    );

    // -------------------------------------------------------------
    // 4. ARMADURA HORIZONTAL DE DISTRIBUIÇÃO E RETRAÇÃO (Fuste)
    // -------------------------------------------------------------
    // NBR 6118: As_dist >= 0.20 * As e >= 0.9 cm²/m por face (ou 0.15% da seção)
    let as_horiz = (stem_sec.as_required_cm2_m * 0.20)
        .max(0.0015 * 100.0 * (geom.stem_top_width * 100.0) * 0.5)
        .max(1.50);

    let horiz_rebar = recommend_rebar(as_horiz);

    RetainingWallStructuralReport {
        stem_section: stem_sec,
        toe_section: toe_sec,
        heel_section: heel_sec,
        horizontal_distribution_as_cm2_m: as_horiz,
        horizontal_distribution_rebar: horiz_rebar,
        fcd_mpa,
        fyd_mpa,
    }
}

fn concrete_fctm_mpa(fck_mpa: f64) -> f64 {
    // ABNT NBR 6118:2023 Seção 8.2.5
    if fck_mpa <= 50.0 {
        0.30 * fck_mpa.powf(2.0 / 3.0)
    } else {
        2.12 * (1.0 + 0.1 * (fck_mpa + 8.0)).ln()
    }
}

fn min_flexural_steel_ratio(fck_mpa: f64) -> f64 {
    // ABNT NBR 6118:2023 Seção 17.3.5.2.1 / Tabela 17.3 (retangular, CA-50)
    const TABLE_17_3: &[(f64, f64)] = &[
        (20.0, 0.00150),
        (25.0, 0.00150),
        (30.0, 0.00150),
        (35.0, 0.00164),
        (40.0, 0.00179),
        (45.0, 0.00194),
        (50.0, 0.00208),
        (55.0, 0.00211),
        (60.0, 0.00219),
        (65.0, 0.00226),
        (70.0, 0.00233),
        (75.0, 0.00239),
        (80.0, 0.00245),
        (85.0, 0.00251),
        (90.0, 0.00256),
    ];
    if fck_mpa <= 30.0 {
        return 0.00150;
    }
    if fck_mpa >= 90.0 {
        return 0.00256;
    }
    for w in TABLE_17_3.windows(2) {
        if fck_mpa <= w[1].0 {
            let t = (fck_mpa - w[0].0) / (w[1].0 - w[0].0);
            return w[0].1 + t * (w[1].1 - w[0].1);
        }
    }
    0.00150
}

fn dimension_flexural_section(
    name: &str,
    mk: f64,
    md: f64,
    vk: f64,
    vd: f64,
    b_m: f64,
    h_m: f64,
    d_m: f64,
    fck_mpa: f64,
    fyk_mpa: f64,
    gamma_c: f64,
    fcd_kpa: f64,
    fyd_kpa: f64,
) -> SectionReinforcementReport {
    // Parâmetros NBR 6118:2023 (Seções 8.2.10.1 e 17.2.2 e): η_c = (40/f_ck)^(1/3) para f_ck > 40 MPa
    let eta_c = if fck_mpa > 40.0 {
        (40.0 / fck_mpa).cbrt()
    } else {
        1.0
    };
    let (alpha_c, lambda) = if fck_mpa <= 50.0 {
        (0.85 * eta_c, 0.80)
    } else {
        (
            0.85 * eta_c * (1.0 - (fck_mpa - 50.0) / 200.0).max(0.50),
            (0.80 - (fck_mpa - 50.0) / 400.0).max(0.65),
        )
    };

    // Momento fletor adimensional: mu = Md / (b * d^2 * alpha_c * fcd)
    let denom_mu = b_m * d_m.powi(2) * alpha_c * fcd_kpa;
    let mu = if denom_mu > 0.001 {
        (md / denom_mu).clamp(0.0, 0.50)
    } else {
        0.05
    };

    // Linha neutra relativa: xi = (1 - sqrt(1 - 2*mu)) / lambda
    let xi = (1.0 - (1.0 - 2.0 * mu).max(0.0).sqrt()) / lambda;
    let xi_limit = if fck_mpa <= 50.0 { 0.45 } else { 0.35 }; // Limite do Domínio 3 (14.6.4.3)
    let is_ductile = xi <= xi_limit;

    // Braço de alavanca interno: z = d * (1 - 0.5 * lambda * xi)
    let z = d_m * (1.0 - 0.5 * lambda * xi);

    // Armadura calculada: As = Md / (fyd * z) [m²] -> converter para cm²/m
    let as_calc_m2 = if z > 0.01 && fyd_kpa > 0.0 {
        md / (fyd_kpa * z)
    } else {
        0.0
    };
    let as_calc_cm2 = as_calc_m2 * 10_000.0;

    // Taxa mínima de armadura rho_min conforme Tabela 17.3 (NBR 6118:2023)
    let rho_min = min_flexural_steel_ratio(fck_mpa);
    let as_min_cm2 = rho_min * (b_m * 100.0) * (h_m * 100.0);
    let as_req_cm2 = as_calc_cm2.max(as_min_cm2);

    let rebar = recommend_rebar(as_req_cm2);

    // Verificação de Esforço Cortante sem armadura transversal (VRd1 - Seção 19.4.1 NBR 6118:2023)
    let d_mm = d_m * 1000.0;
    let fck_shear = fck_mpa.min(60.0); // 19.4.1 limita tau_Rd ao equivalente a fck = 60 MPa
    let fctm_shear = concrete_fctm_mpa(fck_shear);
    let fctd_shear = (0.70 * fctm_shear) / gamma_c.max(1.0);
    let tau_rd = 0.25 * fctd_shear; // MPa
    let k_shear = (1.0 + (200.0 / d_mm).sqrt()).min(2.0);
    let rho1 = (as_req_cm2 / (b_m * 100.0 * d_m * 100.0)).min(0.02);
    let v_rd1 = (tau_rd * 1e3 * k_shear * (1.2 + 40.0 * rho1)) * b_m * d_m; // kN/m
    let is_shear_ok = v_rd1 >= vd;

    // Abertura característica de fissuras wk (Seção 17.3.3.2 NBR 6118:2023): menor entre as 2 expressões
    let fctm = concrete_fctm_mpa(fck_mpa);
    let eta1 = if (480.0..580.0).contains(&fyk_mpa) {
        2.25 // CA-50 (Tabela 8.2 NBR 6118:2023)
    } else {
        1.00 // CA-25 / CA-60
    };
    let z_serv = (0.90 * d_m).max(0.05);
    let as_ef_m2 = (rebar.area_provided_cm2_m * 1e-4).max(1e-6);
    let sigma_s = (mk / (z_serv * as_ef_m2)).min(fyd_kpa) * 1e-3; // MPa em serviço
    let phi_mm = rebar.bar_diameter_mm;
    let common = (phi_mm / (12.5 * eta1)) * (sigma_s / 210_000.0);
    let wk1 = common * (3.0 * sigma_s / fctm.max(0.5));
    let phi_cm = phi_mm * 0.1;
    let s_band_cm = rebar.spacing_cm.min(15.0 * phi_cm).max(1.0);
    let h_band_cm = ((h_m - d_m) * 200.0).clamp(2.0 * phi_cm, 15.0 * phi_cm);
    let a_cri_cm2 = s_band_cm * h_band_cm;
    let a_bar_cm2 = std::f64::consts::PI * (phi_cm * 0.5).powi(2);
    let rho_ri = (a_bar_cm2 / a_cri_cm2).max(0.0015);
    let wk2 = common * (4.0 / rho_ri + 45.0);
    let wk = wk1.min(wk2);
    let is_crack_width_ok = wk <= 0.30;

    SectionReinforcementReport {
        section_name: name.to_string(),
        mk_knm_m: mk,
        md_knm_m: md,
        vk_kn_m: vk,
        vd_kn_m: vd,
        section_width_m: b_m,
        section_height_m: h_m,
        effective_depth_d_m: d_m,
        neutral_axis_xi: xi,
        is_ductile,
        as_calculated_cm2_m: as_calc_cm2,
        as_min_cm2_m: as_min_cm2,
        as_required_cm2_m: as_req_cm2,
        recommendation: rebar,
        v_rd1_kn_m: v_rd1,
        is_shear_ok,
        crack_width_wk_mm: wk,
        is_crack_width_ok,
    }
}

/// Escolhe bitola comercial padronizada e espaçamento ótimo (s <= 20 cm)
fn recommend_rebar(as_req_cm2: f64) -> RebarRecommendation {
    let bar_sizes = [
        (8.0, 0.503),   // phi 8.0mm, area 0.503 cm2
        (10.0, 0.785),  // phi 10.0mm, area 0.785 cm2
        (12.5, 1.227),  // phi 12.5mm, area 1.227 cm2
        (16.0, 2.011),  // phi 16.0mm, area 2.011 cm2
        (20.0, std::f64::consts::PI),  // phi 20.0mm, area PI = 3.14159... cm2
        (25.0, 4.909),  // phi 25.0mm, area 4.909 cm2
    ];

    for &(phi_mm, area_one) in &bar_sizes {
        let spacing_cm = ((area_one * 100.0) / as_req_cm2).clamp(7.5, 20.0);
        // Arredonda para baixo para múltiplos de 2.5 cm (ex: 10, 12.5, 15, 17.5, 20)
        let s_rounded = (spacing_cm / 2.5).floor() * 2.5;
        let s_final = s_rounded.clamp(7.5, 20.0);
        let area_provided = (area_one * 100.0) / s_final;

        if area_provided >= as_req_cm2 * 0.98 {
            return RebarRecommendation {
                bar_diameter_mm: phi_mm,
                spacing_cm: s_final,
                area_provided_cm2_m: area_provided,
                description: format!("Ø {:.1} mm c/ {:.1} cm (A_s = {:.2} cm²/m)", phi_mm, s_final, area_provided),
            };
        }
    }

    // Fallback para armaduras muito densas
    RebarRecommendation {
        bar_diameter_mm: 20.0,
        spacing_cm: 10.0,
        area_provided_cm2_m: 31.42,
        description: "Ø 20.0 mm c/ 10.0 cm (A_s = 31.42 cm²/m)".to_string(),
    }
}
