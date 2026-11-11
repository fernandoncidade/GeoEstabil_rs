use crate::core::types::{Polyline, SoilLayer};
use crate::locale::translations::Language;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nbr6122SoilPreset {
    pub name: &'static str,
    pub classification: &'static str,
    pub spt_range: &'static str,
    pub gamma: f64,
    pub gamma_sat: f64,
    pub cohesion: f64,
    pub friction_angle_deg: f64,
    pub sigma_adm: f64,
    pub color_rgba: [u8; 4],
}

pub fn get_standard_nbr6122_soils() -> Vec<Nbr6122SoilPreset> {
    vec![
        Nbr6122SoilPreset {
            name: "Areia Fofa",
            classification: "Solo Granular Não-Coesivo",
            spt_range: "N_SPT ≤ 4",
            gamma: 17.0,
            gamma_sat: 18.5,
            cohesion: 0.0,
            friction_angle_deg: 28.0,
            sigma_adm: 100.0,
            color_rgba: [240, 230, 140, 220], // Khaki
        },
        Nbr6122SoilPreset {
            name: "Areia Pouco Compacta",
            classification: "Solo Granular Não-Coesivo",
            spt_range: "N_SPT 5 a 8",
            gamma: 18.0,
            gamma_sat: 19.5,
            cohesion: 0.0,
            friction_angle_deg: 30.0,
            sigma_adm: 150.0,
            color_rgba: [238, 232, 170, 220], // Pale Goldenrod
        },
        Nbr6122SoilPreset {
            name: "Areia Medianamente Compacta",
            classification: "Solo Granular Não-Coesivo",
            spt_range: "N_SPT 9 a 18",
            gamma: 19.0,
            gamma_sat: 20.5,
            cohesion: 0.0,
            friction_angle_deg: 33.0,
            sigma_adm: 250.0,
            color_rgba: [218, 165, 32, 220], // Goldenrod
        },
        Nbr6122SoilPreset {
            name: "Areia Compacta a Muito Compacta",
            classification: "Solo Granular Não-Coesivo",
            spt_range: "N_SPT > 18",
            gamma: 20.0,
            gamma_sat: 21.5,
            cohesion: 0.0,
            friction_angle_deg: 38.0,
            sigma_adm: 400.0,
            color_rgba: [184, 134, 11, 220], // Dark Goldenrod
        },
        Nbr6122SoilPreset {
            name: "Silte Arenoso / Solo Residual Jovem",
            classification: "Solo Intermediário / Saprolito",
            spt_range: "N_SPT 8 a 15",
            gamma: 18.5,
            gamma_sat: 19.5,
            cohesion: 10.0,
            friction_angle_deg: 29.0,
            sigma_adm: 200.0,
            color_rgba: [205, 133, 63, 220], // Peru
        },
        Nbr6122SoilPreset {
            name: "Silte Argiloso / Solo Residual Maduro",
            classification: "Solo Coesivo",
            spt_range: "N_SPT 6 a 12",
            gamma: 18.0,
            gamma_sat: 19.0,
            cohesion: 15.0,
            friction_angle_deg: 25.0,
            sigma_adm: 180.0,
            color_rgba: [160, 82, 45, 220], // Sienna
        },
        Nbr6122SoilPreset {
            name: "Argila Mole (Aluvião / Mangue)",
            classification: "Solo Fino Compressível",
            spt_range: "N_SPT ≤ 2",
            gamma: 14.5,
            gamma_sat: 15.0,
            cohesion: 10.0,
            friction_angle_deg: 0.0,
            sigma_adm: 40.0,
            color_rgba: [112, 128, 144, 220], // Slate Gray
        },
        Nbr6122SoilPreset {
            name: "Argila Média",
            classification: "Solo Fino Coesivo",
            spt_range: "N_SPT 3 a 5",
            gamma: 16.5,
            gamma_sat: 17.5,
            cohesion: 25.0,
            friction_angle_deg: 15.0,
            sigma_adm: 100.0,
            color_rgba: [128, 128, 128, 220], // Gray
        },
        Nbr6122SoilPreset {
            name: "Argila Rija",
            classification: "Solo Fino Coesivo",
            spt_range: "N_SPT 6 a 10",
            gamma: 18.0,
            gamma_sat: 19.0,
            cohesion: 40.0,
            friction_angle_deg: 20.0,
            sigma_adm: 200.0,
            color_rgba: [139, 69, 19, 220], // Saddle Brown
        },
        Nbr6122SoilPreset {
            name: "Argila Muito Rija a Dura",
            classification: "Solo Fino Coesivo",
            spt_range: "N_SPT > 10",
            gamma: 19.5,
            gamma_sat: 20.5,
            cohesion: 70.0,
            friction_angle_deg: 24.0,
            sigma_adm: 350.0,
            color_rgba: [101, 67, 33, 220], // Dark Brown
        },
        Nbr6122SoilPreset {
            name: "Aterro de Engenharia Compactado",
            classification: "Solo de Empréstimo Controlado",
            spt_range: "Grau de Compactação ≥ 95% PN",
            gamma: 19.5,
            gamma_sat: 20.5,
            cohesion: 12.0,
            friction_angle_deg: 32.0,
            sigma_adm: 250.0,
            color_rgba: [210, 105, 30, 220], // Chocolate
        },
    ]
}

impl Nbr6122SoilPreset {
    pub fn localized_name(&self, lang: Language) -> &'static str {
        if lang == Language::PtBr {
            return self.name;
        }
        match self.name {
            "Areia Fofa" => "Loose Sand",
            "Areia Pouco Compacta" => "Medium-Loose Sand",
            "Areia Medianamente Compacta" => "Medium-Dense Sand",
            "Areia Compacta a Muito Compacta" => "Dense to Very Dense Sand",
            "Silte Arenoso / Solo Residual Jovem" => "Sandy Silt / Young Residual Soil",
            "Silte Argiloso / Solo Residual Maduro" => "Clayey Silt / Mature Residual Soil",
            "Argila Mole (Aluvião / Mangue)" => "Soft Clay (Alluvial / Marine)",
            "Argila Média" => "Medium Clay",
            "Argila Rija" => "Stiff Clay",
            "Argila Muito Rija a Dura" => "Very Stiff to Hard Clay",
            "Aterro de Engenharia Compactado" => "Engineered Compacted Fill",
            other => other,
        }
    }

    pub fn localized_classification(&self, lang: Language) -> &'static str {
        if lang == Language::PtBr {
            return self.classification;
        }
        match self.classification {
            "Solo Granular Não-Coesivo" => "Cohesionless Granular Soil",
            "Solo Intermediário / Saprolito" => "Intermediate Soil / Saprolite",
            "Solo Coesivo" => "Cohesive Soil",
            "Solo Fino Compressível" => "Compressible Fine-Grained Soil",
            "Solo Fino Coesivo" => "Fine Cohesive Soil",
            "Solo de Empréstimo Controlado" => "Controlled Borrow Fill",
            other => other,
        }
    }

    pub fn localized_spt_range(&self, lang: Language) -> &'static str {
        if lang == Language::PtBr {
            return self.spt_range;
        }
        match self.spt_range {
            "N_SPT ≤ 4" => "N_SPT ≤ 4",
            "N_SPT 5 a 8" => "N_SPT 5 to 8",
            "N_SPT 9 a 18" => "N_SPT 9 to 18",
            "N_SPT > 18" => "N_SPT > 18",
            "N_SPT 8 a 15" => "N_SPT 8 to 15",
            "N_SPT 6 a 12" => "N_SPT 6 to 12",
            "N_SPT ≤ 2" => "N_SPT ≤ 2",
            "N_SPT 3 a 5" => "N_SPT 3 to 5",
            "N_SPT 6 a 10" => "N_SPT 6 to 10",
            "N_SPT > 10" => "N_SPT > 10",
            "Grau de Compactação ≥ 95% PN" => "Compaction ≥ 95% Std Proctor",
            other => other,
        }
    }

    pub fn to_soil_layer(&self, id: usize) -> SoilLayer {
        self.to_soil_layer_lang(id, Language::PtBr)
    }

    pub fn to_soil_layer_lang(&self, id: usize, lang: Language) -> SoilLayer {
        SoilLayer {
            id,
            name: self.localized_name(lang).to_string(),
            description: format!(
                "{} ({})",
                self.localized_classification(lang),
                self.localized_spt_range(lang)
            ),
            gamma: self.gamma,
            gamma_sat: self.gamma_sat,
            cohesion: self.cohesion,
            friction_angle_deg: self.friction_angle_deg,
            sigma_adm: self.sigma_adm,
            top_profile: Polyline::new(vec![]),
            color_rgba: self.color_rgba,
        }
    }
}
