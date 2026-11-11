use serde::{Deserialize, Serialize};

/// Detalhes calculados de uma fatia vertical discreta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceDetail {
    pub index: usize,
    pub x_mid: f64,
    pub width: f64,
    pub y_top: f64,
    pub y_base: f64,
    pub height: f64,
    pub base_angle_rad: f64,
    pub base_angle_deg: f64,
    pub base_length: f64,
    pub weight: f64,
    pub surcharge_force: f64,
    pub pore_pressure: f64,
    pub cohesion: f64,
    pub friction_angle_deg: f64,
    pub layer_name: String,
}

/// Resultado da análise de uma superfície circular específica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularSurfaceResult {
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
    pub x_entry: f64,
    pub x_exit: f64,
    pub fs_bishop: f64,
    pub fs_fellenius: f64,
    pub fs_janbu: f64,
    pub converged: bool,
    pub iterations: usize,
    pub slices: Vec<SliceDetail>,
}

/// Ponto da malha de calor com centro (x, y) e FS mínimo naquele centro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapPoint {
    pub center_x: f64,
    pub center_y: f64,
    pub best_radius: f64,
    pub min_fs: f64,
}

/// Matriz de dados do Mapa de Calor (Heatmap)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapGrid {
    pub points: Vec<HeatmapPoint>,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub global_min_fs: f64,
    pub global_max_fs: f64,
}

/// Resultado consolidado da análise de estabilidade com varredura paramétrica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlopeAnalysisReport {
    pub critical_circle: CircularSurfaceResult,
    pub total_circles_evaluated: usize,
    pub valid_circles_count: usize,
    pub elapsed_ms: f64,
    pub heatmap: Option<HeatmapGrid>,
    pub timestamp: String,
    pub is_stable: bool,
    pub required_min_fs: f64,
}
