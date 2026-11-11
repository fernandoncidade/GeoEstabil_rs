use geoestabil_rs::core::project::{ActiveProjectMode, GeoProject};
use geoestabil_rs::core::types::*;
use geoestabil_rs::data::database::GeoDatabase;
use geoestabil_rs::engine::slope::bishop::calculate_bishop;
use geoestabil_rs::engine::slope::fellenius::calculate_fellenius;
use geoestabil_rs::engine::slope::geometry::find_slip_surface_intersections;
use geoestabil_rs::engine::slope::grid_search::run_slope_stability_analysis;
use geoestabil_rs::engine::slope::janbu::calculate_janbu;
use geoestabil_rs::engine::slope::slices::generate_slices;
use geoestabil_rs::engine::wall::bearing_capacity::calculate_bearing_capacity;
use geoestabil_rs::engine::wall::concrete_design::design_concrete_reinforcement;
use geoestabil_rs::engine::wall::earth_pressure::calculate_earth_pressures;
use geoestabil_rs::engine::wall::stability::check_wall_external_stability;
use geoestabil_rs::ui::app::GeoEstabilApp;
use geoestabil_rs::ui::canvas_2d::CanvasState;
use geoestabil_rs::locale::translations::Localizer;
use geoestabil_rs::ui::dialogs::{
    AboutDialog, ManualDialog, SettingsDialog, SoilCatalogDialog,
};

#[test]
fn test_rankine_earth_pressures() {
    let mut model = RetainingWallModel::default();
    model.theory = EarthPressureTheory::Rankine;
    model.backfill_soil.friction_angle_deg = 30.0;
    model.backfill_soil.cohesion = 0.0;
    model.backfill_soil.gamma = 18.0;
    model.geometry.stem_height = 4.0;
    model.geometry.base_thickness = 0.50; // Total height = 4.5m
    model.surcharge_q = 0.0;
    model.water_height_back = 0.0;

    let pressures = calculate_earth_pressures(&model);

    // Ka para phi = 30° deve ser exatamente 1/3 ~ 0.3333
    assert!((pressures.ka - 1.0 / 3.0).abs() < 1e-3);
    // Kp para phi = 30° deve ser exatamente 3.0
    assert!((pressures.kp - 3.0).abs() < 1e-3);

    // Ea = 0.5 * gamma * H^2 * Ka = 0.5 * 18 * 4.5^2 * (1/3) = 60.75 kN/m
    assert!((pressures.ea_soil - 60.75).abs() < 0.1);
    assert!((pressures.ea_horizontal - 60.75).abs() < 0.1);
}

#[test]
fn test_wall_stability_checks() {
    let model = RetainingWallModel::default();
    let stability = check_wall_external_stability(&model);

    // Muro padrão deve ser estável contra tombamento (FS >= 1.50) e deslizamento (FS >= 1.50)
    assert!(stability.fs_overturning >= 1.50);
    assert!(stability.is_overturning_ok);
    assert!(stability.fs_sliding >= 1.20);
    assert!(stability.eccentricity < model.geometry.base_width / 2.0);
}

#[test]
fn test_bearing_capacity_hansen() {
    let model = RetainingWallModel::default();
    let stability = check_wall_external_stability(&model);
    let bearing = calculate_bearing_capacity(&model, &stability);

    assert!(bearing.q_ult_kpa > 100.0);
    assert!(bearing.fs_bearing >= 1.50);
}

#[test]
fn test_concrete_reinforcement_design() {
    let model = RetainingWallModel::default();
    let stability = check_wall_external_stability(&model);
    let reinf = design_concrete_reinforcement(&model, &stability);

    // Fuste deve ter armadura de cálculo e respeitar taxa mínima
    assert!(reinf.stem_section.as_required_cm2_m >= reinf.stem_section.as_min_cm2_m);
    assert!(reinf.stem_section.is_ductile); // Linha neutra xi <= 0.45
    assert!(reinf.stem_section.is_shear_ok); // VRd1 >= Vd
    assert!(reinf.stem_section.crack_width_wk_mm <= 0.30);
    assert!(reinf.stem_section.crack_width_wk_mm > 0.005);

    // Verifica NBR 6118:2023 Grupo II (C60: η_c = (40/60)^(1/3), limite dúctil x/d <= 0.35, rho_min = 0.219%)
    let mut high_fck = RetainingWallModel::default();
    high_fck.concrete.fck_mpa = 60.0;
    let st_high = check_wall_external_stability(&high_fck);
    let reinf_high = design_concrete_reinforcement(&high_fck, &st_high);
    assert!(reinf_high.stem_section.is_ductile);
    assert!(reinf_high.stem_section.as_min_cm2_m > reinf.stem_section.as_min_cm2_m);
}

#[test]
fn test_slope_slices_and_bishop() {
    let model = SlopeModel::default();
    let xc = 15.0;
    let yc = 15.0;
    let r = 16.0;

    let intersections = find_slip_surface_intersections(&model.terrain_surface, xc, yc, r);
    assert!(intersections.is_some());

    let (x_in, x_out) = intersections.unwrap();
    assert!(x_out > x_in);

    let slices = generate_slices(&model, xc, yc, r, x_in, x_out, 30);
    assert!(slices.len() >= 10);

    let fs_fell = calculate_fellenius(&slices);
    assert!(fs_fell > 0.5 && fs_fell < 10.0);

    let bishop = calculate_bishop(&slices);
    assert!(bishop.converged);
    assert!(bishop.fs > 0.5 && bishop.fs < 10.0);
    // Bishop é tipicamente superior a Fellenius
    assert!(bishop.fs >= fs_fell * 0.95);

    let janbu = calculate_janbu(&slices, x_in, x_out, r);
    assert!(janbu.converged);
    assert!(janbu.fs > 0.5 && janbu.fs < 10.0);
}

#[test]
fn test_slope_rayon_grid_search() {
    let mut model = SlopeModel::default();
    // Grid leve para teste unitário rápido
    model.grid_x_steps = 8;
    model.grid_y_steps = 8;
    model.radius_steps = 8;
    model.num_slices = 25;

    let report = run_slope_stability_analysis(&model);
    assert!(report.total_circles_evaluated > 0);
    assert!(report.valid_circles_count > 0);
    assert!(report.critical_circle.fs_bishop > 0.5);
    assert!(report.heatmap.is_some());
}

#[test]
fn test_tabs_2_and_3_dynamic_canvas_layout_and_resize() {
    let mut project = GeoProject::default();
    project.slope_model.grid_x_steps = 10;
    project.slope_model.grid_y_steps = 10;
    project.slope_model.radius_steps = 10;
    project.slope_model.num_slices = 25;
    let initial_report = run_slope_stability_analysis(&project.slope_model);

    let mut app = GeoEstabilApp {
        project,
        active_tab: 1,
        slope_report: Some(initial_report),
        slope_canvas_state: CanvasState::default(),
        wall_canvas_state: CanvasState::default(),
        dark_mode: true,
        localizer: Localizer::default(),
        db: GeoDatabase::open_in_memory().ok(),
        about_dialog: AboutDialog::default(),
        manual_dialog: ManualDialog::default(),
        soil_catalog_dialog: SoilCatalogDialog::default(),
        settings_dialog: SettingsDialog::default(),
    };

    let ctx = egui::Context::default();
    let run_frames = |app: &mut GeoEstabilApp, w: f32, h: f32| -> usize {
        let mut last_shapes = 0;
        for _ in 0..2 {
            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(w, h),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(raw_input, |ui| app.render_ui(ui));
            last_shapes = output.shapes.len();
            output.textures_delta.clear();
        }
        last_shapes
    };

    // 1. Aba 2 (Estabilidade de Taludes): redimensionamento progressivo e encolhimento
    app.active_tab = 1;
    app.project.active_mode = ActiveProjectMode::SlopeStability;

    let shapes_980 = run_frames(&mut app, 980.0, 620.0);
    let size_980 = app
        .slope_canvas_state
        .last_rect_size
        .expect("Canvas da Aba 2 deve ter tamanho válido em 980x620");
    let zoom_980 = app.slope_canvas_state.zoom;
    println!("Tab2 @ 980x620 => canvas {:?}, zoom={:.2}, shapes={}", size_980, zoom_980, shapes_980);
    assert!(
        size_980.x >= 500.0 && size_980.y >= 280.0,
        "Canvas da Aba 2 colapsou em 980x620: {:?}",
        size_980
    );
    assert!(shapes_980 > 30, "Canvas da Aba 2 deve emitir formas gráficas");

    let shapes_1280 = run_frames(&mut app, 1280.0, 820.0);
    let size_1280 = app.slope_canvas_state.last_rect_size.unwrap();
    let zoom_1280 = app.slope_canvas_state.zoom;
    println!("Tab2 @ 1280x820 => canvas {:?}, zoom={:.2}, shapes={}", size_1280, zoom_1280, shapes_1280);
    assert!(
        size_1280.x >= 800.0 && size_1280.y >= 500.0,
        "Canvas da Aba 2 inesperado em 1280x820: {:?}",
        size_1280
    );
    assert!(size_1280.x > size_980.x && size_1280.y > size_980.y);
    assert!(zoom_1280 > zoom_980, "Auto-fit da Aba 2 deve ampliar escala quando janela cresce");
    assert!(shapes_1280 > 30);

    run_frames(&mut app, 1600.0, 900.0);
    let size_1600 = app.slope_canvas_state.last_rect_size.unwrap();
    let zoom_1600 = app.slope_canvas_state.zoom;
    println!("Tab2 @ 1600x900 => canvas {:?}, zoom={:.2}", size_1600, zoom_1600);
    assert!(size_1600.x > size_1280.x && size_1600.y > size_1280.y);
    assert!(zoom_1600 >= zoom_1280);

    // Encolhe de volta para 980x620 e confirma adaptação automática
    run_frames(&mut app, 980.0, 620.0);
    let size_back = app.slope_canvas_state.last_rect_size.unwrap();
    assert!(size_back.x < size_1600.x && size_back.y < size_1600.y);
    assert!((size_back.x - size_980.x).abs() < 2.0 && (size_back.y - size_980.y).abs() < 2.0);

    // 2. Aba 3 (Muro de Contenção): redimensionamento progressivo, encolhimento e pan manual
    app.active_tab = 2;
    app.project.active_mode = ActiveProjectMode::RetainingWall;

    let wall_shapes_980 = run_frames(&mut app, 980.0, 620.0);
    let wall_size_980 = app
        .wall_canvas_state
        .last_rect_size
        .expect("Canvas da Aba 3 deve ter tamanho válido em 980x620");
    let wall_zoom_980 = app.wall_canvas_state.zoom;
    println!("Tab3 @ 980x620 => canvas {:?}, zoom={:.2}, shapes={}", wall_size_980, wall_zoom_980, wall_shapes_980);
    assert!(
        wall_size_980.x >= 500.0 && wall_size_980.y >= 260.0,
        "Canvas da Aba 3 colapsou em 980x620: {:?}",
        wall_size_980
    );
    assert!(wall_shapes_980 > 20, "Canvas da Aba 3 deve emitir formas do muro e diagramas");

    run_frames(&mut app, 1280.0, 820.0);
    let wall_size_1280 = app.wall_canvas_state.last_rect_size.unwrap();
    let wall_zoom_1280 = app.wall_canvas_state.zoom;
    println!("Tab3 @ 1280x820 => canvas {:?}, zoom={:.2}", wall_size_1280, wall_zoom_1280);
    assert!(
        wall_size_1280.x >= 780.0 && wall_size_1280.y >= 500.0,
        "Canvas da Aba 3 inesperado em 1280x820: {:?}",
        wall_size_1280
    );
    assert!(wall_size_1280.x > wall_size_980.x && wall_size_1280.y > wall_size_980.y);
    assert!(wall_zoom_1280 > wall_zoom_980, "Auto-fit da Aba 3 deve ampliar escala ao crescer");

    run_frames(&mut app, 1600.0, 900.0);
    let wall_size_1600 = app.wall_canvas_state.last_rect_size.unwrap();
    let wall_zoom_1600 = app.wall_canvas_state.zoom;
    println!("Tab3 @ 1600x900 => canvas {:?}, zoom={:.2}", wall_size_1600, wall_zoom_1600);
    assert!(wall_size_1600.x > wall_size_1280.x && wall_size_1600.y > wall_size_1280.y);

    // Encolhe com auto_fit ativo
    run_frames(&mut app, 980.0, 620.0);
    let wall_size_back = app.wall_canvas_state.last_rect_size.unwrap();
    assert!(wall_size_back.x < wall_size_1600.x && wall_size_back.y < wall_size_1600.y);

    // Modo manual: zoom permanece constante enquanto janela redimensiona, e pan acompanha centro
    app.wall_canvas_state.auto_fit = false;
    let manual_zoom_before = app.wall_canvas_state.zoom;
    let manual_pan_before = app.wall_canvas_state.pan_offset;
    run_frames(&mut app, 1280.0, 820.0);
    assert!((app.wall_canvas_state.zoom - manual_zoom_before).abs() < 1e-4);
    assert!(app.wall_canvas_state.pan_offset.x > manual_pan_before.x);
    assert!(app.wall_canvas_state.pan_offset.y > manual_pan_before.y);

    // 3. Smoke check nas demais abas (1, 4 e 5) em 980x620 e 1280x820
    for tab_idx in [0_usize, 3, 4] {
        app.active_tab = tab_idx;
        let s1 = run_frames(&mut app, 980.0, 620.0);
        let s2 = run_frames(&mut app, 1280.0, 820.0);
        assert!(s1 > 5 && s2 > 5);
    }
}
