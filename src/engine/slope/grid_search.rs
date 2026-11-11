use crate::core::types::{SlopeAnalysisMethod, SlopeModel};
use crate::engine::slope::bishop::calculate_bishop;
use crate::engine::slope::fellenius::calculate_fellenius;
use crate::engine::slope::geometry::find_slip_surface_intersections;
use crate::engine::slope::janbu::calculate_janbu;
use crate::engine::slope::results::{
    CircularSurfaceResult, HeatmapGrid, HeatmapPoint, SlopeAnalysisReport,
};
use crate::engine::slope::slices::generate_slices;
use rayon::prelude::*;
use std::time::Instant;

/// Executa a varredura paramétrica multithreaded com Rayon sobre a malha de centros e raios.
pub fn run_slope_stability_analysis(model: &SlopeModel) -> SlopeAnalysisReport {
    let start_time = Instant::now();

    let x_steps = model.grid_x_steps.max(5);
    let y_steps = model.grid_y_steps.max(5);
    let r_steps = model.radius_steps.max(5);

    let dx = if x_steps > 1 {
        (model.grid_x_max - model.grid_x_min) / (x_steps - 1) as f64
    } else {
        1.0
    };
    let dy = if y_steps > 1 {
        (model.grid_y_max - model.grid_y_min) / (y_steps - 1) as f64
    } else {
        1.0
    };
    let dr = if r_steps > 1 {
        (model.radius_max - model.radius_min) / (r_steps - 1) as f64
    } else {
        1.0
    };

    // Monta todos os centros de pesquisa da malha
    let mut centers = Vec::with_capacity(x_steps * y_steps);
    for ix in 0..x_steps {
        let xc = model.grid_x_min + ix as f64 * dx;
        for iy in 0..y_steps {
            let yc = model.grid_y_min + iy as f64 * dy;
            centers.push((xc, yc));
        }
    }

    // Paralelização com Rayon sobre a malha de centros
    let results: Vec<(Option<CircularSurfaceResult>, HeatmapPoint, usize, usize)> = centers
        .par_iter()
        .map(|&(xc, yc)| {
            let mut best_circle_for_center: Option<CircularSurfaceResult> = None;
            let mut min_fs_center = 99.99;
            let mut best_radius_center = model.radius_min;
            let mut center_evaluated = 0;
            let mut center_valid = 0;

            for ir in 0..r_steps {
                let r = model.radius_min + ir as f64 * dr;
                center_evaluated += 1;

                if let Some((x_entry, x_exit)) =
                    find_slip_surface_intersections(&model.terrain_surface, xc, yc, r)
                {
                    center_valid += 1;
                    let slices = generate_slices(
                        model,
                        xc,
                        yc,
                        r,
                        x_entry,
                        x_exit,
                        model.num_slices.clamp(15, 60),
                    );

                    if slices.len() >= 8 {
                        let fs_fellenius = calculate_fellenius(&slices);
                        let bishop_res = calculate_bishop(&slices);
                        let janbu_res = calculate_janbu(&slices, x_entry, x_exit, r);

                        let primary_fs = match model.method {
                            SlopeAnalysisMethod::BishopSimplified => bishop_res.fs,
                            SlopeAnalysisMethod::FelleniusOrdinary => fs_fellenius,
                            SlopeAnalysisMethod::JanbuSimplified => janbu_res.fs,
                        };

                        if primary_fs > 0.05 && primary_fs < min_fs_center {
                            min_fs_center = primary_fs;
                            best_radius_center = r;

                            best_circle_for_center = Some(CircularSurfaceResult {
                                center_x: xc,
                                center_y: yc,
                                radius: r,
                                x_entry,
                                x_exit,
                                fs_bishop: bishop_res.fs,
                                fs_fellenius,
                                fs_janbu: janbu_res.fs,
                                converged: bishop_res.converged,
                                iterations: bishop_res.iterations,
                                slices: vec![], // Preenchido apenas para o crítico final
                            });
                        }
                    }
                }
            }

            let heatmap_pt = HeatmapPoint {
                center_x: xc,
                center_y: yc,
                best_radius: best_radius_center,
                min_fs: min_fs_center,
            };

            (
                best_circle_for_center,
                heatmap_pt,
                center_evaluated,
                center_valid,
            )
        })
        .collect();

    // Consolidação dos resultados e seleção do Círculo Crítico Global
    let mut total_evaluated = 0;
    let mut total_valid = 0;
    let mut global_best_circle: Option<CircularSurfaceResult> = None;
    let mut global_min_fs = 99.99;
    let mut heatmap_points = Vec::with_capacity(results.len());

    for (best_circ_opt, heatmap_pt, evaluated, valid) in results {
        total_evaluated += evaluated;
        total_valid += valid;
        heatmap_points.push(heatmap_pt);

        if let Some(circ) = best_circ_opt {
            let primary_fs = match model.method {
                SlopeAnalysisMethod::BishopSimplified => circ.fs_bishop,
                SlopeAnalysisMethod::FelleniusOrdinary => circ.fs_fellenius,
                SlopeAnalysisMethod::JanbuSimplified => circ.fs_janbu,
            };

            if primary_fs < global_min_fs {
                global_min_fs = primary_fs;
                global_best_circle = Some(circ);
            }
        }
    }

    // Se encontramos um círculo crítico válido, regeramos as fatias detalhadas dele
    let critical_circle = if let Some(mut circ) = global_best_circle {
        circ.slices = generate_slices(
            model,
            circ.center_x,
            circ.center_y,
            circ.radius,
            circ.x_entry,
            circ.x_exit,
            model.num_slices.clamp(20, 80),
        );
        circ
    } else {
        // Fallback se nenhum círculo válido foi encontrado nos limites da malha
        let xc = (model.grid_x_min + model.grid_x_max) * 0.5;
        let yc = (model.grid_y_min + model.grid_y_max) * 0.5;
        let r = (model.radius_min + model.radius_max) * 0.5;
        CircularSurfaceResult {
            center_x: xc,
            center_y: yc,
            radius: r,
            x_entry: xc - r * 0.8,
            x_exit: xc + r * 0.8,
            fs_bishop: 99.99,
            fs_fellenius: 99.99,
            fs_janbu: 99.99,
            converged: false,
            iterations: 0,
            slices: vec![],
        }
    };

    let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;
    let required_min_fs = model.safety_level.min_fs();
    let current_primary_fs = match model.method {
        SlopeAnalysisMethod::BishopSimplified => critical_circle.fs_bishop,
        SlopeAnalysisMethod::FelleniusOrdinary => critical_circle.fs_fellenius,
        SlopeAnalysisMethod::JanbuSimplified => critical_circle.fs_janbu,
    };
    let is_stable = current_primary_fs >= required_min_fs && current_primary_fs < 90.0;

    let heatmap_max_fs = heatmap_points
        .iter()
        .map(|p| p.min_fs)
        .filter(|&fs| fs < 90.0)
        .fold(0.0, f64::max)
        .max(3.0);

    let heatmap = HeatmapGrid {
        points: heatmap_points,
        min_x: model.grid_x_min,
        max_x: model.grid_x_max,
        min_y: model.grid_y_min,
        max_y: model.grid_y_max,
        global_min_fs,
        global_max_fs: heatmap_max_fs,
    };

    SlopeAnalysisReport {
        critical_circle,
        total_circles_evaluated: total_evaluated,
        valid_circles_count: total_valid,
        elapsed_ms: elapsed,
        heatmap: Some(heatmap),
        timestamp: chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
        is_stable,
        required_min_fs,
    }
}
