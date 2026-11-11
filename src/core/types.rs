use crate::locale::translations::Language;
use serde::{Deserialize, Serialize};

/// Ponto bidimensional no plano cartesiano (x = estaca/distância horizontal, y = cota/elevação).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point2D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Polilinha representando uma fronteira de solo, perfil de terreno ou nível d'água.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    pub points: Vec<Point2D>,
}

impl Polyline {
    pub fn new(points: Vec<Point2D>) -> Self {
        Self { points }
    }

    /// Interpola a elevação y para uma dada coordenada x ao longo da polilinha.
    pub fn interpolate_y(&self, x: f64) -> Option<f64> {
        if self.points.len() < 2 {
            return self.points.first().map(|p| p.y);
        }

        if x <= self.points.first().unwrap().x {
            return Some(self.points.first().unwrap().y);
        }
        if x >= self.points.last().unwrap().x {
            return Some(self.points.last().unwrap().y);
        }

        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];
            if (p1.x..=p2.x).contains(&x) || (p2.x..=p1.x).contains(&x) {
                let dx = p2.x - p1.x;
                if dx.abs() < 1e-9 {
                    return Some(p1.y.max(p2.y));
                }
                let t = (x - p1.x) / dx;
                return Some(p1.y + t * (p2.y - p1.y));
            }
        }

        None
    }

    pub fn x_min(&self) -> f64 {
        self.points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min)
    }

    pub fn x_max(&self) -> f64 {
        self.points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn y_min(&self) -> f64 {
        self.points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min)
    }

    pub fn y_max(&self) -> f64 {
        self.points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max)
    }
}

/// Camada geotécnica estratigráfica com parâmetros de resistência ao cisalhamento (Mohr-Coulomb).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoilLayer {
    pub id: usize,
    pub name: String,
    pub description: String,
    /// Peso específico natural / úmido [kN/m³]
    pub gamma: f64,
    /// Peso específico saturado [kN/m³]
    pub gamma_sat: f64,
    /// Coesão efetiva c' ou resistência não-drenada Su [kPa]
    pub cohesion: f64,
    /// Ângulo de atrito interno efetivo φ' [graus]
    pub friction_angle_deg: f64,
    /// Tensão admissível estimada para fundações rasas [kPa] (NBR 6122)
    pub sigma_adm: f64,
    /// Polilinha do topo da camada (ou base da camada superior)
    pub top_profile: Polyline,
    /// Cor RGBA para visualização gráfica
    pub color_rgba: [u8; 4],
}

impl SoilLayer {
    pub fn friction_angle_rad(&self) -> f64 {
        self.friction_angle_deg.to_radians()
    }
}

/// Nível Freático (Lençol Freático / Piezométrico)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterTable {
    pub enabled: bool,
    /// Peso específico da água γw [kN/m³] (padrão 9.81 ou 10.0)
    pub gamma_w: f64,
    /// Polilinha representando a linha freática
    pub phreatic_line: Polyline,
}

impl Default for WaterTable {
    fn default() -> Self {
        Self {
            enabled: false,
            gamma_w: 9.81,
            phreatic_line: Polyline::new(vec![
                Point2D::new(0.0, 5.0),
                Point2D::new(40.0, 5.0),
            ]),
        }
    }
}

/// Sobrecarga superficial acidental ou permanente (ex: tráfego rodoviário, edificação na crista).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Surcharge {
    pub enabled: bool,
    pub name: String,
    /// Carga distribuída vertical q [kPa = kN/m²]
    pub q_kpa: f64,
    pub x_start: f64,
    pub x_end: f64,
}

impl Default for Surcharge {
    fn default() -> Self {
        Self {
            enabled: false,
            name: "Sobrecarga de Tráfego / Edificação".to_string(),
            q_kpa: 20.0,
            x_start: 18.0,
            x_end: 35.0,
        }
    }
}

/// Método de Equilíbrio Limite para Estabilidade de Taludes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlopeAnalysisMethod {
    BishopSimplified,
    FelleniusOrdinary,
    JanbuSimplified,
}

impl SlopeAnalysisMethod {
    pub fn name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::BishopSimplified, Language::PtBr) => "Bishop Simplificado (Iterativo)",
            (Self::BishopSimplified, Language::EnUs) => "Simplified Bishop (Iterative)",
            (Self::FelleniusOrdinary, Language::PtBr) => "Fellenius (Ordinário / Sueco)",
            (Self::FelleniusOrdinary, Language::EnUs) => "Fellenius (Ordinary / Swedish)",
            (Self::JanbuSimplified, Language::PtBr) => "Janbu Simplificado (com fator f0)",
            (Self::JanbuSimplified, Language::EnUs) => "Simplified Janbu (with f0 factor)",
        }
    }

    pub fn name_pt(&self) -> &'static str {
        self.name(Language::PtBr)
    }
}

/// Grau de Segurança requerido conforme ABNT NBR 11682:2009 (Tabela 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevelNBR11682 {
    /// FS mínimo = 1.50 (Vidas humanas em risco elevado, edificações residenciais, rodovias principais)
    High,
    /// FS mínimo = 1.30 (Risco médio a vidas humanas, áreas com ocupação esporádica)
    Medium,
    /// FS mínimo = 1.20 (Risco reduzido, taludes provisórios, sem risco a vidas)
    Low,
}

impl SafetyLevelNBR11682 {
    pub fn min_fs(&self) -> f64 {
        match self {
            Self::High => 1.50,
            Self::Medium => 1.30,
            Self::Low => 1.20,
        }
    }

    pub fn description(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::High, Language::PtBr) => "Alto (FS mínimo ≥ 1.50) — Risco a vidas humanas ou danos materiais elevados",
            (Self::High, Language::EnUs) => "High (Min FS ≥ 1.50) — Risk to human lives or high property damage",
            (Self::Medium, Language::PtBr) => "Médio (FS mínimo ≥ 1.30) — Danos materiais moderados, ocupação esporádica",
            (Self::Medium, Language::EnUs) => "Medium (Min FS ≥ 1.30) — Moderate property damage, sporadic occupancy",
            (Self::Low, Language::PtBr) => "Baixo (FS mínimo ≥ 1.20) — Taludes provisórios, sem risco a vidas",
            (Self::Low, Language::EnUs) => "Low (Min FS ≥ 1.20) — Temporary slopes, no risk to human lives",
        }
    }

    pub fn description_pt(&self) -> &'static str {
        self.description(Language::PtBr)
    }
}

/// Modelo completo do Talude para análise
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlopeModel {
    pub terrain_surface: Polyline,
    pub layers: Vec<SoilLayer>,
    pub water_table: WaterTable,
    pub surcharge: Surcharge,
    pub safety_level: SafetyLevelNBR11682,
    pub method: SlopeAnalysisMethod,
    /// Número de fatias por superfície de ruptura circular
    pub num_slices: usize,
    /// Limites da malha de busca de centros (Grid Search)
    pub grid_x_min: f64,
    pub grid_x_max: f64,
    pub grid_x_steps: usize,
    pub grid_y_min: f64,
    pub grid_y_max: f64,
    pub grid_y_steps: usize,
    /// Faixa de raios a pesquisar para cada centro
    pub radius_min: f64,
    pub radius_max: f64,
    pub radius_steps: usize,
}

impl Default for SlopeModel {
    fn default() -> Self {
        Self {
            terrain_surface: Polyline::new(vec![
                Point2D::new(0.0, 0.0),
                Point2D::new(10.0, 0.0),
                Point2D::new(20.0, 8.0),
                Point2D::new(40.0, 8.0),
            ]),
            layers: vec![
                SoilLayer {
                    id: 1,
                    name: "Solo Residual / Colúvio".to_string(),
                    description: "Argila siltosa avermelhada compacta".to_string(),
                    gamma: 18.0,
                    gamma_sat: 19.5,
                    cohesion: 12.0,
                    friction_angle_deg: 26.0,
                    sigma_adm: 180.0,
                    top_profile: Polyline::new(vec![
                        Point2D::new(0.0, 0.0),
                        Point2D::new(10.0, 0.0),
                        Point2D::new(20.0, 8.0),
                        Point2D::new(40.0, 8.0),
                    ]),
                    color_rgba: [205, 133, 63, 200], // Peru brown
                },
                SoilLayer {
                    id: 2,
                    name: "Saprolito / Solo de Alteração".to_string(),
                    description: "Silte arenoso com fragmentos de rocha".to_string(),
                    gamma: 19.0,
                    gamma_sat: 20.0,
                    cohesion: 25.0,
                    friction_angle_deg: 32.0,
                    sigma_adm: 300.0,
                    top_profile: Polyline::new(vec![
                        Point2D::new(0.0, -3.0),
                        Point2D::new(12.0, -3.0),
                        Point2D::new(22.0, 4.0),
                        Point2D::new(40.0, 4.0),
                    ]),
                    color_rgba: [160, 82, 45, 220], // Sienna
                },
            ],
            water_table: WaterTable::default(),
            surcharge: Surcharge::default(),
            safety_level: SafetyLevelNBR11682::High,
            method: SlopeAnalysisMethod::BishopSimplified,
            num_slices: 35,
            grid_x_min: 5.0,
            grid_x_max: 25.0,
            grid_x_steps: 25,
            grid_y_min: 8.0,
            grid_y_max: 22.0,
            grid_y_steps: 20,
            radius_min: 8.0,
            radius_max: 26.0,
            radius_steps: 20,
        }
    }
}

/// Tipologia de Muro de Contenção / Arrimo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetainingWallType {
    /// Muro de Flexão / Cantilever em Concreto Armado (Perfil L ou T invertido)
    Cantilever,
    /// Muro de Gravidade (Concreto Ciclópico, Gabião ou Alvenaria de Pedra)
    Gravity,
}

impl RetainingWallType {
    pub fn name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Cantilever, Language::PtBr) => "Muro de Flexão em Concreto Armado (Cantilever)",
            (Self::Cantilever, Language::EnUs) => "Reinforced Concrete Cantilever Wall",
            (Self::Gravity, Language::PtBr) => "Muro de Gravidade (Concreto Ciclópico / Gabião)",
            (Self::Gravity, Language::EnUs) => "Gravity Wall (Cyclopean Concrete / Gabion)",
        }
    }

    pub fn name_pt(&self) -> &'static str {
        self.name(Language::PtBr)
    }
}

/// Teoria de Empuxo de Terra
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EarthPressureTheory {
    /// Teoria de Rankine (superfície plana, atrito solo-muro δ = 0)
    Rankine,
    /// Teoria de Coulomb (cunha de terra com atrito solo-muro δ e inclinação do tardoz)
    Coulomb,
}

impl EarthPressureTheory {
    pub fn name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Rankine, Language::PtBr) => "Teoria de Rankine",
            (Self::Rankine, Language::EnUs) => "Rankine Theory",
            (Self::Coulomb, Language::PtBr) => "Teoria de Coulomb (com atrito solo-muro δ)",
            (Self::Coulomb, Language::EnUs) => "Coulomb Theory (with wall friction δ)",
        }
    }

    pub fn name_pt(&self) -> &'static str {
        self.name(Language::PtBr)
    }
}

/// Geometria paramétrica do Muro de Contenção
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallGeometry {
    /// Altura livre total do muro H [m] (do topo da sapata até a crista)
    pub stem_height: f64,
    /// Espessura da crista do fuste b_top [m]
    pub stem_top_width: f64,
    /// Espessura da base do fuste b_base [m]
    pub stem_base_width: f64,
    /// Largura total da sapata corrida B [m]
    pub base_width: f64,
    /// Espessura da sapata de fundação h_s [m]
    pub base_thickness: f64,
    /// Comprimento da ponta (aba anterior) B_toe [m]
    pub toe_length: f64,
    /// Inclinação do talude de montante / aterro β [graus]
    pub backfill_slope_deg: f64,
    /// Inclinação da face interna do tardoz α [graus] (90° = vertical)
    pub wall_back_slope_deg: f64,
    /// Dente de ancoragem (chave de cisalhamento) ativo na base?
    pub shear_key_enabled: bool,
    /// Altura do dente de ancoragem [m]
    pub shear_key_depth: f64,
    /// Largura do dente de ancoragem [m]
    pub shear_key_width: f64,
}

impl Default for WallGeometry {
    fn default() -> Self {
        Self {
            stem_height: 4.0,
            stem_top_width: 0.30,
            stem_base_width: 0.50,
            base_width: 2.60,
            base_thickness: 0.50,
            toe_length: 0.70,
            backfill_slope_deg: 0.0,
            wall_back_slope_deg: 90.0,
            shear_key_enabled: false,
            shear_key_depth: 0.40,
            shear_key_width: 0.40,
        }
    }
}

impl WallGeometry {
    /// Comprimento do calcanhar (aba posterior) B_heel [m]
    pub fn heel_length(&self) -> f64 {
        (self.base_width - self.toe_length - self.stem_base_width).max(0.0)
    }

    /// Altura total do sistema H_total = H + h_s [m]
    pub fn total_height(&self) -> f64 {
        self.stem_height + self.base_thickness
    }
}

/// Parâmetros de materiais estruturais (ABNT NBR 6118:2023)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcreteStructuralProperties {
    /// Resistência característica à compressão do concreto fck [MPa]
    pub fck_mpa: f64,
    /// Tensão de escoamento característica do aço fyk [MPa] (padrão CA-50 = 500 MPa)
    pub fyk_mpa: f64,
    /// Peso específico do concreto armado γc [kN/m³] (padrão 25.0 kN/m³)
    pub gamma_concrete: f64,
    /// Recobrimento nominal da armadura c_nom [mm] (conforme Classe de Agressividade Ambiental)
    pub cover_nom_mm: f64,
    /// Coeficiente de minoração do concreto γc (padrão 1.40)
    pub gamma_c: f64,
    /// Coeficiente de minoração do aço γs (padrão 1.15)
    pub gamma_s: f64,
    /// Coeficiente de majoração das ações γf (padrão 1.40)
    pub gamma_f: f64,
}

impl Default for ConcreteStructuralProperties {
    fn default() -> Self {
        Self {
            fck_mpa: 25.0,
            fyk_mpa: 500.0,
            gamma_concrete: 25.0,
            cover_nom_mm: 35.0, // CAA II (Urbana)
            gamma_c: 1.40,
            gamma_s: 1.15,
            gamma_f: 1.40,
        }
    }
}

/// Modelo completo de Muro de Contenção
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetainingWallModel {
    pub wall_type: RetainingWallType,
    pub theory: EarthPressureTheory,
    pub geometry: WallGeometry,
    pub concrete: ConcreteStructuralProperties,
    /// Solo de aterro / reaterro contido
    pub backfill_soil: SoilLayer,
    /// Solo de fundação sobre o qual se assenta a sapata corrida
    pub foundation_soil: SoilLayer,
    /// Atrito solo-muro δ / φ' (relação de atrito, ex: 0.667 = 2/3)
    pub delta_ratio: f64,
    /// Sobrecarga no terrapleno q [kPa]
    pub surcharge_q: f64,
    /// Nível de água no tardoz hw [m] acima da base da sapata (0.0 = seco)
    pub water_height_back: f64,
    /// Coesão de adesão na interface solo-base ca [kPa] (geralmente ca <= c')
    pub base_adhesion_kpa: f64,
    /// Atrito na interface solo-base δ_base [graus]
    pub base_friction_deg: f64,
}

impl Default for RetainingWallModel {
    fn default() -> Self {
        Self {
            wall_type: RetainingWallType::Cantilever,
            theory: EarthPressureTheory::Rankine,
            geometry: WallGeometry::default(),
            concrete: ConcreteStructuralProperties::default(),
            backfill_soil: SoilLayer {
                id: 1,
                name: "Aterro Arenoso Bem Graduado".to_string(),
                description: "Areia média a grossa compactada".to_string(),
                gamma: 18.5,
                gamma_sat: 20.0,
                cohesion: 2.0,
                friction_angle_deg: 32.0,
                sigma_adm: 200.0,
                top_profile: Polyline::new(vec![]),
                color_rgba: [218, 165, 32, 200],
            },
            foundation_soil: SoilLayer {
                id: 2,
                name: "Solo Natural de Fundação".to_string(),
                description: "Argila siltosa rija".to_string(),
                gamma: 19.0,
                gamma_sat: 20.5,
                cohesion: 20.0,
                friction_angle_deg: 24.0,
                sigma_adm: 250.0,
                top_profile: Polyline::new(vec![]),
                color_rgba: [160, 82, 45, 200],
            },
            delta_ratio: 0.667, // 2/3 de phi
            surcharge_q: 10.0,  // 10 kPa (sobrecarga típica de calçada/veículos leves)
            water_height_back: 0.0, // Drenagem efetiva
            base_adhesion_kpa: 10.0,
            base_friction_deg: 24.0,
        }
    }
}
