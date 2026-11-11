use crate::engine::slope::results::SliceDetail;

pub struct JanbuResult {
    pub fs: f64,
    pub f0_factor: f64,
    pub converged: bool,
    pub iterations: usize,
}

/// Calcula o Fator de Segurança pelo Método de Janbu Simplificado (com fator de correção f0)
pub fn calculate_janbu(
    slices: &[SliceDetail],
    x_entry: f64,
    x_exit: f64,
    radius: f64,
) -> JanbuResult {
    let mut denominator = 0.0;
    for slice in slices {
        denominator += slice.weight * slice.base_angle_rad.tan();
    }

    if denominator <= 1e-4 {
        return JanbuResult {
            fs: 99.99,
            f0_factor: 1.0,
            converged: true,
            iterations: 1,
        };
    }

    // Cálculo do fator de forma e profundidade d/L para Janbu
    let chord_length = (x_exit - x_entry).abs().max(1.0);
    // Flecha do arco circular: d = R - sqrt(R^2 - (L/2)^2)
    let half_l = chord_length * 0.5;
    let d = if radius > half_l {
        radius - (radius.powi(2) - half_l.powi(2)).sqrt()
    } else {
        chord_length * 0.25
    };
    let d_over_l = (d / chord_length).clamp(0.0, 0.40);
    let b1 = 0.50; // Para solos c' - phi'
    let f0 = 1.0 + b1 * (d_over_l - 1.4 * d_over_l.powi(2)).max(0.0);

    let mut fs_current = 1.5;
    let max_iter = 50;
    let tolerance = 1e-4;

    for iter in 1..=max_iter {
        let mut numerator = 0.0;

        for slice in slices {
            let phi_rad = slice.friction_angle_deg.to_radians();
            let tan_phi = phi_rad.tan();
            let cos_alpha = slice.base_angle_rad.cos();
            let tan_alpha = slice.base_angle_rad.tan();

            // Termo n_alpha = cos^2(alpha) * (1 + tan(alpha)*tan(phi)/FS)
            let mut n_alpha = cos_alpha.powi(2) * (1.0 + (tan_alpha * tan_phi) / fs_current);
            if n_alpha < 0.10 {
                n_alpha = 0.10;
            }

            let effective_vertical = (slice.weight - slice.pore_pressure * slice.width).max(0.0);
            let term = (slice.cohesion * slice.width + effective_vertical * tan_phi) / n_alpha;
            numerator += term;
        }

        let fs_next = (numerator / denominator).clamp(0.01, 50.0);

        if (fs_next - fs_current).abs() < tolerance {
            return JanbuResult {
                fs: (fs_next * f0).clamp(0.01, 99.99),
                f0_factor: f0,
                converged: true,
                iterations: iter,
            };
        }

        fs_current = fs_next;
    }

    JanbuResult {
        fs: (fs_current * f0).clamp(0.01, 99.99),
        f0_factor: f0,
        converged: false,
        iterations: max_iter,
    }
}
