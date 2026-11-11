use crate::core::types::SlopeModel;
use crate::engine::slope::results::SliceDetail;

pub fn generate_slices(
    model: &SlopeModel,
    xc: f64,
    yc: f64,
    radius: f64,
    x_entry: f64,
    x_exit: f64,
    num_slices: usize,
) -> Vec<SliceDetail> {
    let mut slices = Vec::with_capacity(num_slices);
    let total_width = x_exit - x_entry;
    if total_width <= 0.0 || num_slices == 0 {
        return slices;
    }

    let b = total_width / num_slices as f64;

    for i in 0..num_slices {
        let x_mid = x_entry + (i as f64 + 0.5) * b;

        let r_sq = radius.powi(2) - (x_mid - xc).powi(2);
        if r_sq <= 0.0 {
            continue;
        }

        let y_base = yc - r_sq.sqrt();
        let y_top = match model.terrain_surface.interpolate_y(x_mid) {
            Some(y) => y,
            None => continue,
        };

        let height = (y_top - y_base).max(0.0);
        if height < 0.01 {
            continue;
        }

        // Inclinação da base da fatia: sin(alpha) = (x_mid - xc) / R
        let sin_alpha = ((x_mid - xc) / radius).clamp(-0.9999, 0.9999);
        let alpha_rad = sin_alpha.asin();
        let cos_alpha = alpha_rad.cos().max(0.01);
        let base_length = b / cos_alpha;

        // Lençol freático e poropressão na base
        let mut pore_pressure = 0.0;
        let mut y_water = None;
        if model.water_table.enabled {
            if let Some(yw) = model.water_table.phreatic_line.interpolate_y(x_mid) {
                y_water = Some(yw);
                if yw > y_base {
                    let head = (yw - y_base).min(y_top - y_base);
                    pore_pressure = model.water_table.gamma_w * head;
                }
            }
        }

        // Identificação da camada de solo na base da fatia
        let (cohesion, friction_angle_deg, layer_name) = get_soil_at_base(model, x_mid, y_base);

        // Cálculo do peso próprio integrando as camadas presentes na fatia
        let soil_weight = calculate_slice_soil_weight(model, x_mid, y_base, y_top, b, y_water);

        // Sobrecarga superficial atuante na fatia
        let mut surcharge_force = 0.0;
        if model.surcharge.enabled
            && x_mid >= model.surcharge.x_start
            && x_mid <= model.surcharge.x_end
        {
            surcharge_force = model.surcharge.q_kpa * b;
        }

        let total_weight = soil_weight + surcharge_force;

        slices.push(SliceDetail {
            index: i + 1,
            x_mid,
            width: b,
            y_top,
            y_base,
            height,
            base_angle_rad: alpha_rad,
            base_angle_deg: alpha_rad.to_degrees(),
            base_length,
            weight: total_weight,
            surcharge_force,
            pore_pressure,
            cohesion,
            friction_angle_deg,
            layer_name,
        });
    }

    slices
}

/// Identifica a camada presente no ponto base (x, y_base)
fn get_soil_at_base(model: &SlopeModel, x: f64, y: f64) -> (f64, f64, String) {
    if model.layers.is_empty() {
        return (10.0, 30.0, "Solo Padrão".to_string());
    }

    // Procura de cima para baixo nas camadas
    for layer in &model.layers {
        if let Some(y_top_layer) = layer.top_profile.interpolate_y(x) {
            if y <= y_top_layer {
                return (
                    layer.cohesion,
                    layer.friction_angle_deg,
                    layer.name.clone(),
                );
            }
        }
    }

    // Se estiver abaixo de todas ou acima, pegar a mais próxima
    let last = model.layers.last().unwrap();
    (last.cohesion, last.friction_angle_deg, last.name.clone())
}

/// Calcula o peso da coluna de solo estratificado na fatia
fn calculate_slice_soil_weight(
    model: &SlopeModel,
    x: f64,
    y_base: f64,
    y_top: f64,
    width: f64,
    y_water: Option<f64>,
) -> f64 {
    if model.layers.is_empty() {
        return 18.0 * (y_top - y_base) * width;
    }

    let mut weight = 0.0;

    // Coleta as cotas das fronteiras de camadas no ponto x
    let mut boundaries = Vec::new();
    boundaries.push((y_base, 0.0, 0.0)); // Início na base

    for layer in &model.layers {
        if let Some(y_layer) = layer.top_profile.interpolate_y(x) {
            if y_layer > y_base && y_layer < y_top {
                boundaries.push((y_layer, layer.gamma, layer.gamma_sat));
            }
        }
    }
    boundaries.push((y_top, 0.0, 0.0)); // Fim no topo
    boundaries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for i in 0..boundaries.len() - 1 {
        let y_bot_sub = boundaries[i].0;
        let y_top_sub = boundaries[i + 1].0;
        let h_sub = y_top_sub - y_bot_sub;
        if h_sub <= 0.0 {
            continue;
        }

        let y_mid_sub = (y_bot_sub + y_top_sub) * 0.5;
        // Pega os parâmetros da camada correspondente
        let layer = get_layer_at(model, x, y_mid_sub);
        let gamma = if let Some(yw) = y_water {
            if y_mid_sub < yw {
                layer.gamma_sat
            } else {
                layer.gamma
            }
        } else {
            layer.gamma
        };

        weight += gamma * h_sub * width;
    }

    weight
}

fn get_layer_at<'a>(model: &'a SlopeModel, x: f64, y: f64) -> &'a crate::core::types::SoilLayer {
    for layer in &model.layers {
        if let Some(y_top_layer) = layer.top_profile.interpolate_y(x) {
            if y <= y_top_layer {
                return layer;
            }
        }
    }
    model.layers.last().unwrap()
}
