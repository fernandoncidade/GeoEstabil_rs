use crate::engine::slope::results::SliceDetail;

/// Calcula o Fator de Segurança pelo Método de Fellenius (Método Sueco Ordinário)
pub fn calculate_fellenius(slices: &[SliceDetail]) -> f64 {
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for slice in slices {
        let phi_rad = slice.friction_angle_deg.to_radians();
        let cos_alpha = slice.base_angle_rad.cos();
        let sin_alpha = slice.base_angle_rad.sin();

        // Força normal total na base
        let n_total = slice.weight * cos_alpha;
        // Força de poro-pressão na base
        let u_force = slice.pore_pressure * slice.base_length;
        // Força normal efetiva (com corte de tração para física realista)
        let n_effective = (n_total - u_force).max(0.0);

        // Resistência ao cisalhamento disponível (Coulomb)
        let shear_resisting = slice.cohesion * slice.base_length + n_effective * phi_rad.tan();
        numerator += shear_resisting;

        // Força motriz tangencial
        let driving = slice.weight * sin_alpha;
        denominator += driving;
    }

    if denominator <= 1e-4 {
        99.99
    } else {
        (numerator / denominator).clamp(0.01, 99.99)
    }
}
