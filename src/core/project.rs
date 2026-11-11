use crate::core::types::{RetainingWallModel, SoilLayer, SlopeModel};
use crate::engine::soils::get_standard_nbr6122_soils;
use crate::locale::translations::Language;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveProjectMode {
    SlopeStability,
    RetainingWall,
}

impl ActiveProjectMode {
    pub fn name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::SlopeStability, Language::PtBr) => "Estabilidade de Taludes (NBR 11682)",
            (Self::SlopeStability, Language::EnUs) => "Slope Stability (NBR 11682)",
            (Self::RetainingWall, Language::PtBr) => "Muros de Contenção (NBR 11682 / NBR 6122 / NBR 6118)",
            (Self::RetainingWall, Language::EnUs) => "Retaining Walls (NBR 11682 / NBR 6122 / NBR 6118)",
        }
    }

    pub fn name_pt(&self) -> &'static str {
        self.name(Language::PtBr)
    }
}

/// Estrutura central do projeto de engenharia geotécnica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoProject {
    pub version: String,
    pub title: String,
    pub client: String,
    pub site_location: String,
    pub engineer_name: String,
    pub crea_cau: String,
    pub art_rrt: String,
    pub date: String,
    pub notes: String,
    pub active_mode: ActiveProjectMode,
    pub slope_model: SlopeModel,
    pub wall_model: RetainingWallModel,
    #[serde(skip)]
    pub file_path: Option<PathBuf>,
    #[serde(skip)]
    pub is_dirty: bool,
}

impl Default for GeoProject {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: "Estudo de Estabilidade e Contenção Geotécnica".to_string(),
            client: "Cliente / Incorporadora / Construtora".to_string(),
            site_location: "Município / UF — Obra de Encosta".to_string(),
            engineer_name: String::new(),
            crea_cau: "Registro Profissional (se aplicável)".to_string(),
            art_rrt: "Identificação / Estudo".to_string(),
            date: chrono::Local::now().format("%d/%m/%Y").to_string(),
            notes: "Simulação auxiliar de estudo baseada nas referências ABNT NBR 11682:2009, NBR 6122:2022 e NBR 6118:2023. Exige conferência manual e validação em ferramentas consolidadas."
                .to_string(),
            active_mode: ActiveProjectMode::SlopeStability,
            slope_model: SlopeModel::default(),
            wall_model: RetainingWallModel::default(),
            file_path: None,
            is_dirty: false,
        }
    }
}

impl GeoProject {
    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path_ref = path.as_ref();
        let file = File::create(path_ref).map_err(|e| format!("Falha ao criar arquivo: {}", e))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)
            .map_err(|e| format!("Falha ao serializar projeto: {}", e))?;
        self.file_path = Some(path_ref.to_path_buf());
        self.is_dirty = false;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|e| format!("Falha ao abrir arquivo: {}", e))?;
        let reader = BufReader::new(file);
        let mut proj: GeoProject = serde_json::from_reader(reader)
            .map_err(|e| format!("Falha ao desserializar projeto: {}", e))?;
        proj.file_path = Some(path_ref.to_path_buf());
        proj.is_dirty = false;
        Ok(proj)
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Retorna o nome do responsável técnico preenchido ou o padrão `"GeoEstabil"` caso vazio.
    pub fn effective_engineer_name(&self) -> &str {
        let trimmed = self.engineer_name.trim();
        if trimmed.is_empty() {
            "GeoEstabil"
        } else {
            trimmed
        }
    }

    /// Atualiza metadados e textos padrão/de catálogo caso ainda estejam no idioma de origem,
    /// preservando integralmente qualquer texto personalizado pelo usuário.
    pub fn translate_defaults(&mut self, from: Language, to: Language) {
        if from == to {
            return;
        }
        let pairs: &[(&str, &str)] = &[
            (
                "Estudo de Estabilidade e Contenção Geotécnica",
                "Geotechnical Stability and Retaining Study",
            ),
            (
                "Cliente / Incorporadora / Construtora",
                "Client / Developer / Contractor",
            ),
            (
                "Município / UF — Obra de Encosta",
                "City / State — Slope Engineering Site",
            ),
            (
                "Registro Profissional (se aplicável)",
                "Professional Registration (if applicable)",
            ),
            (
                "Simulação auxiliar de estudo baseada nas referências ABNT NBR 11682:2009, NBR 6122:2022 e NBR 6118:2023. Exige conferência manual e validação em ferramentas consolidadas.",
                "Auxiliary study simulation referencing ABNT NBR 11682:2009, NBR 6122:2022, and NBR 6118:2023. Requires manual check and validation against established market tools.",
            ),
            // Presets titles
            (
                "Talude Rodoviário em Corte — Solo Residual de Gnaisse",
                "Highway Cut Slope — Gneiss Residual Soil",
            ),
            (
                "Aterro sobre Argila Mole de Baixada",
                "Embankment over Lowland Soft Clay",
            ),
            (
                "Muro de Arrimo em Concreto Armado (Cantilever H=4.5m)",
                "Reinforced Concrete Cantilever Retaining Wall (H=4.5m)",
            ),
            (
                "Muro de Gravidade em Concreto Ciclópico (H=3.5m)",
                "Cyclopean Concrete Gravity Retaining Wall (H=3.5m)",
            ),
            // Default & Preset surcharge names
            (
                "Sobrecarga de Tráfego / Edificação",
                "Traffic / Building Surcharge",
            ),
            (
                "Sobrecarga de Tráfego Rodoviário (DNIT)",
                "Highway Traffic Surcharge (DNIT)",
            ),
            (
                "Carga de Pavimentação e Cargas Acidentais",
                "Pavement Dead & Live Surcharge",
            ),
            // Default & Preset soil layer names & descriptions
            (
                "Solo Residual / Colúvio",
                "Residual / Colluvial Soil",
            ),
            (
                "Argila siltosa avermelhada compacta",
                "Dense reddish silty clay",
            ),
            (
                "Saprolito / Solo de Alteração",
                "Saprolite / Weathered Soil",
            ),
            (
                "Silte arenoso com fragmentos de rocha",
                "Sandy silt with rock fragments",
            ),
            (
                "Solo Residual Maduro (Colúvio)",
                "Mature Residual Soil (Colluvium)",
            ),
            (
                "Argila siltosa vermelha",
                "Red silty clay",
            ),
            (
                "Saprolito / Solo de Alteração de Gnaisse",
                "Saprolite / Weathered Gneiss",
            ),
            (
                "Silte arenoso compacto",
                "Dense sandy silt",
            ),
            (
                "Corpo do Aterro Compactado",
                "Compacted Embankment Body",
            ),
            (
                "Argila arenosa bem compactada",
                "Well-compacted sandy clay",
            ),
            (
                "Argila Mole Orgânica de Baixada",
                "Soft Organic Lowland Clay",
            ),
            (
                "Argila muito mole e saturada (SPT 1 a 2)",
                "Very soft saturated clay (SPT 1 to 2)",
            ),
            (
                "Solo residual ou coluvial",
                "Residual or colluvial soil",
            ),
            // Retaining wall default & preset soils
            (
                "Aterro Arenoso Bem Graduado",
                "Well-Graded Sandy Backfill",
            ),
            (
                "Areia média a grossa compactada",
                "Compacted medium to coarse sand",
            ),
            (
                "Solo Natural de Fundação",
                "Natural Foundation Soil",
            ),
            (
                "Argila siltosa rija",
                "Stiff silty clay",
            ),
            (
                "Aterro Arenoso Drenado",
                "Drained Sandy Backfill",
            ),
            (
                "Areia média compactada",
                "Compacted medium sand",
            ),
            (
                "Solo de Fundação (Argila Siltosa Rija)",
                "Foundation Soil (Stiff Silty Clay)",
            ),
            (
                "Solo de boa consistência (SPT 10 a 14)",
                "Stiff consistency soil (SPT 10 to 14)",
            ),
            (
                "Solo Natural de Encosta",
                "Natural Slope Soil",
            ),
            (
                "Silte argiloso medianamente compacto",
                "Medium-dense clayey silt",
            ),
            (
                "Solo de Apoio da Fundação",
                "Foundation Bearing Soil",
            ),
            (
                "Areia siltosa compacta (SPT 15)",
                "Dense silty sand (SPT 15)",
            ),
        ];

        let translate_str = |s: &mut String| {
            for &(pt, en) in pairs {
                let (src, dst) = match (from, to) {
                    (Language::PtBr, Language::EnUs) => (pt, en),
                    (Language::EnUs, Language::PtBr) => (en, pt),
                    _ => continue,
                };
                if s == src {
                    *s = dst.to_string();
                    return;
                }
            }
            if from == Language::PtBr && to == Language::EnUs {
                if let Some(rest) = s.strip_prefix("Camada de Solo ") {
                    *s = format!("Soil Layer {}", rest);
                    return;
                }
            } else if from == Language::EnUs && to == Language::PtBr {
                if let Some(rest) = s.strip_prefix("Soil Layer ") {
                    *s = format!("Camada de Solo {}", rest);
                    return;
                }
            }
            for soil in get_standard_nbr6122_soils() {
                let src_name = soil.localized_name(from);
                let dst_name = soil.localized_name(to);
                if s == src_name {
                    *s = dst_name.to_string();
                    return;
                }
                let src_desc = format!(
                    "{} ({})",
                    soil.localized_classification(from),
                    soil.localized_spt_range(from)
                );
                let dst_desc = format!(
                    "{} ({})",
                    soil.localized_classification(to),
                    soil.localized_spt_range(to)
                );
                if s == &src_desc {
                    *s = dst_desc;
                    return;
                }
            }
        };

        let translate_layer = |layer: &mut SoilLayer| {
            translate_str(&mut layer.name);
            translate_str(&mut layer.description);
        };

        translate_str(&mut self.title);
        translate_str(&mut self.client);
        translate_str(&mut self.site_location);
        translate_str(&mut self.crea_cau);
        translate_str(&mut self.notes);
        translate_str(&mut self.slope_model.surcharge.name);
        for layer in &mut self.slope_model.layers {
            translate_layer(layer);
        }
        translate_layer(&mut self.wall_model.backfill_soil);
        translate_layer(&mut self.wall_model.foundation_soil);
    }
}
