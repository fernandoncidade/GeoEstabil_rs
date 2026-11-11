use crate::engine::slope::fellenius::calculate_fellenius;
use crate::engine::slope::results::SliceDetail;

pub struct BishopResult {
    pub fs: f64,
    pub converged: bool,
    pub iterations: usize,
}

/// Calcula o Fator de Segurança pelo Método de Bishop Simplificado (Iterativo)
pub fn calculate_bishop(slices: &[SliceDetail]) -> BishopResult {
    let mut denominator = 0.0;
    for slice in slices {
        denominator += slice.weight * slice.base_angle_rad.sin();
    }

    if denominator <= 1e-4 {
        return BishopResult {
            fs: 99.99,
            converged: true,
            iterations: 1,
        };
    }

    // Chute inicial com o método de Fellenius
    let mut fs_current = calculate_fellenius(slices).clamp(0.5, 10.0);
    let max_iter = 50;
    let tolerance = 1e-4;

    for iter in 1..=max_iter {
        let mut numerator = 0.0;

        for slice in slices {
            let phi_rad = slice.friction_angle_deg.to_radians();
            let tan_phi = phi_rad.tan();
            let cos_alpha = slice.base_angle_rad.cos();
            let tan_alpha = slice.base_angle_rad.tan();

            // Termo m_alpha do Bishop: cos(alpha) * (1 + tan(alpha)*tan(phi)/FS)
            let mut m_alpha = cos_alpha * (1.0 + (tan_alpha * tan_phi) / fs_current);
            // Proteção contra singularidade numérica
            if m_alpha < 0.10 {
                m_alpha = 0.10;
            }

            // Força efetiva vertical
            let effective_vertical = (slice.weight - slice.pore_pressure * slice.width).max(0.0);

            let term = (slice.cohesion * slice.width + effective_vertical * tan_phi) / m_alpha;
            numerator += term;
        }

        let fs_next = (numerator / denominator).clamp(0.01, 50.0);

        if (fs_next - fs_current).abs() < tolerance {
            return BishopResult {
                fs: fs_next,
                converged: true,
                iterations: iter,
            };
        }

        fs_current = fs_next;
    }

    BishopResult {
        fs: fs_current,
        converged: false,
        iterations: max_iter,
    }
}
