use crate::core::types::Polyline;

/// Encontra os pontos de interseção entre o arco inferior de um círculo e o perfil do terreno.
/// Retorna `Some((x_entry, x_exit))` onde `x_entry < x_exit`, ou `None` se não houver corte válido.
pub fn find_slip_surface_intersections(
    terrain: &Polyline,
    xc: f64,
    yc: f64,
    radius: f64,
) -> Option<(f64, f64)> {
    if terrain.points.len() < 2 {
        return None;
    }

    let x_left = (xc - radius).max(terrain.x_min());
    let x_right = (xc + radius).min(terrain.x_max());

    if x_left >= x_right {
        return None;
    }

    // Amostragem fina para detectar pontos de cruzamento (f(x) = y_circle(x) - y_terrain(x) = 0)
    let samples = 120;
    let dx = (x_right - x_left) / samples as f64;
    let mut crossings = Vec::new();

    let mut prev_diff: Option<f64> = None;
    let mut prev_x = x_left;

    for i in 0..=samples {
        let x = x_left + i as f64 * dx;
        let r_sq = radius.powi(2) - (x - xc).powi(2);
        if r_sq < 0.0 {
            continue;
        }
        let y_circle = yc - r_sq.sqrt();
        if let Some(y_terrain) = terrain.interpolate_y(x) {
            let diff = y_circle - y_terrain;
            if let Some(p_diff) = prev_diff {
                // Se cruzou o zero (um ponto acima e outro abaixo)
                if (p_diff <= 0.0 && diff > 0.0) || (p_diff >= 0.0 && diff < 0.0) {
                    // Refinamento por bisseção
                    let exact_x = bisection_crossing(terrain, xc, yc, radius, prev_x, x);
                    crossings.push(exact_x);
                }
            }
            prev_diff = Some(diff);
            prev_x = x;
        }
    }

    // Se temos exatamente 2 cruzamentos
    if crossings.len() >= 2 {
        let x1 = crossings.first().copied().unwrap();
        let x2 = crossings.last().copied().unwrap();
        if (x2 - x1).abs() > 0.5 {
            // Verificar se o meio do arco está abaixo do terreno
            let x_mid = (x1 + x2) * 0.5;
            let r_sq_mid = radius.powi(2) - (x_mid - xc).powi(2);
            if r_sq_mid > 0.0 {
                let y_circle_mid = yc - r_sq_mid.sqrt();
                if let Some(y_terrain_mid) = terrain.interpolate_y(x_mid) {
                    let depth = y_terrain_mid - y_circle_mid;
                    if depth > 0.30 {
                        // Profundidade mínima de 30 cm para evitar fatias infinitesimais
                        return Some((x1.min(x2), x1.max(x2)));
                    }
                }
            }
        }
    }

    None
}

/// Refinamento por bisseção do ponto de cruzamento exato entre o círculo e o terreno
fn bisection_crossing(
    terrain: &Polyline,
    xc: f64,
    yc: f64,
    radius: f64,
    mut x_lo: f64,
    mut x_hi: f64,
) -> f64 {
    for _ in 0..16 {
        let x_mid = (x_lo + x_hi) * 0.5;
        let r_sq = (radius.powi(2) - (x_mid - xc).powi(2)).max(0.0);
        let y_circle = yc - r_sq.sqrt();
        let y_terrain = terrain.interpolate_y(x_mid).unwrap_or(0.0);
        let diff = y_circle - y_terrain;

        let r_sq_lo = (radius.powi(2) - (x_lo - xc).powi(2)).max(0.0);
        let y_circle_lo = yc - r_sq_lo.sqrt();
        let y_terrain_lo = terrain.interpolate_y(x_lo).unwrap_or(0.0);
        let diff_lo = y_circle_lo - y_terrain_lo;

        if diff.abs() < 1e-5 {
            return x_mid;
        }

        if (diff_lo > 0.0 && diff > 0.0) || (diff_lo < 0.0 && diff < 0.0) {
            x_lo = x_mid;
        } else {
            x_hi = x_mid;
        }
    }
    (x_lo + x_hi) * 0.5
}
