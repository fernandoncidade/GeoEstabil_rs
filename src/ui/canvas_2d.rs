use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::core::types::{RetainingWallType, SlopeAnalysisMethod};
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::engine::wall::bearing_capacity::calculate_bearing_capacity;
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::stability::check_wall_external_stability;
use crate::locale::translations::Localizer;
use crate::ui::dialogs::doc_renderer::build_math_job_for_painter;
use crate::ui::theme::GeoEstabilTheme;
use egui::{
    Align2, Color32, CornerRadius, FontId, NumExt as _, Painter, Pos2, Rect, Response, Sense,
    Shape, Stroke, Ui, Vec2,
};

fn paint_math(
    painter: &Painter,
    pos: Pos2,
    align: Align2,
    text: &str,
    font_size: f32,
    bold: bool,
    dark_mode: bool,
    color: Color32,
) {
    let job = build_math_job_for_painter(text, font_size, bold, false, dark_mode, Some(color));
    let galley = painter.layout_job(job);
    let r = align.anchor_size(pos, galley.size());
    painter.galley(r.min, galley, color);
}

#[derive(Debug, Clone)]
pub struct CanvasState {
    pub pan_offset: Vec2,
    pub zoom: f32,
    pub auto_fit: bool,
    pub last_rect_size: Option<Vec2>,
    pub show_heatmap: bool,
    pub show_slices: bool,
    pub show_grid: bool,
    pub show_dimensions: bool,
    pub show_diagrams: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan_offset: Vec2::new(100.0, 350.0),
            zoom: 16.0,
            auto_fit: true,
            last_rect_size: None,
            show_heatmap: true,
            show_slices: true,
            show_grid: true,
            show_dimensions: true,
            show_diagrams: true,
        }
    }
}

impl CanvasState {
    pub fn reset_view(&mut self) {
        self.auto_fit = true;
        self.last_rect_size = None;
    }
}

#[derive(Debug, Clone, Copy)]
struct WorldBounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl WorldBounds {
    fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    fn expand_point(&mut self, x: f64, y: f64) {
        if x.is_finite() && y.is_finite() {
            self.min_x = self.min_x.min(x);
            self.max_x = self.max_x.max(x);
            self.min_y = self.min_y.min(y);
            self.max_y = self.max_y.max(y);
        }
    }

    fn compute_fit(&self, rect: Rect, padding_px: f32) -> (Vec2, f32) {
        let usable_w = (rect.width() - padding_px * 2.0).max(80.0);
        let usable_h = (rect.height() - padding_px * 2.0).max(80.0);

        let world_w = (self.max_x - self.min_x).max(1.0) as f32;
        let world_h = (self.max_y - self.min_y).max(1.0) as f32;

        let zoom = (usable_w / world_w)
            .min(usable_h / world_h)
            .clamp(3.0, 160.0);

        let world_cx = 0.5 * (self.min_x + self.max_x) as f32;
        let world_cy = 0.5 * (self.min_y + self.max_y) as f32;

        // world_to_screen(world_cx, world_cy) == rect.center()
        let pan_offset = Vec2::new(
            rect.width() * 0.5 - world_cx * zoom,
            rect.height() * 0.5 + world_cy * zoom,
        );
        (pan_offset, zoom)
    }
}

pub struct Canvas2D;

impl Canvas2D {
    pub fn show(
        ui: &mut Ui,
        project: &GeoProject,
        mode: ActiveProjectMode,
        slope_report: Option<&SlopeAnalysisReport>,
        state: &mut CanvasState,
        localizer: &Localizer,
        dark_mode: bool,
    ) -> Response {
        // Garante área mínima segura e expansão total no espaço restante do painel
        let desired_size = ui.available_size().at_least(Vec2::new(220.0, 220.0));
        let (response, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());
        let rect = response.rect;
        let painter = painter.with_clip_rect(rect);

        // Fundo do canvas
        painter.rect_filled(rect, 4.0, GeoEstabilTheme::canvas_bg(dark_mode));

        // 1. Limites geométricos do modelo para enquadramento dinâmico
        let bounds = match mode {
            ActiveProjectMode::SlopeStability => compute_slope_bounds(project, slope_report),
            ActiveProjectMode::RetainingWall => compute_wall_bounds(project),
        };

        // 2. Sincroniza transformação ao redimensionar ou quando auto_fit estiver ativo
        let base_padding: f32 = match mode {
            ActiveProjectMode::SlopeStability => 52.0,
            ActiveProjectMode::RetainingWall => 54.0,
        };
        let fit_padding = base_padding
            .min(rect.height() * 0.14)
            .min(rect.width() * 0.10)
            .max(22.0);

        if state.auto_fit {
            let (fit_pan, fit_zoom) = bounds.compute_fit(rect, fit_padding);
            state.pan_offset = fit_pan;
            state.zoom = fit_zoom;
        } else if let Some(prev_size) = state.last_rect_size {
            let delta_size = rect.size() - prev_size;
            if delta_size.length_sq() > 0.25 {
                state.pan_offset += delta_size * 0.5;
            }
        }
        state.last_rect_size = Some(rect.size());

        // 3. Interação do usuário: Duplo clique (Auto-Fit), Pan (Arrastar) e Zoom no cursor
        if response.double_clicked() {
            state.auto_fit = true;
            let (fit_pan, fit_zoom) = bounds.compute_fit(rect, fit_padding);
            state.pan_offset = fit_pan;
            state.zoom = fit_zoom;
        }

        if response.dragged() {
            state.auto_fit = false;
            state.pan_offset += response.drag_delta();
        }

        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.01 {
                state.auto_fit = false;
                let zoom_factor = (1.0 + scroll_delta * 0.002).clamp(0.80, 1.25);
                let old_zoom = state.zoom;
                let new_zoom = (old_zoom * zoom_factor).clamp(2.5, 220.0);

                if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                    let wx = (pointer.x - rect.min.x - state.pan_offset.x) / old_zoom;
                    let wy = (rect.min.y + state.pan_offset.y - pointer.y) / old_zoom;
                    state.zoom = new_zoom;
                    state.pan_offset.x = pointer.x - rect.min.x - wx * new_zoom;
                    state.pan_offset.y = pointer.y - rect.min.y + wy * new_zoom;
                } else {
                    state.zoom = new_zoom;
                }
            }
        }

        // 4. Grade Métrica & Modelos (reutilizado por `show` e `show_static_preview`)
        render_canvas_scene(&painter, rect, project, mode, slope_report, state, localizer, dark_mode);

        // 5. Borda externa do Canvas e HUD inferior de telemetria de cursor/escala
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)),
            egui::StrokeKind::Inside,
        );

        draw_canvas_bottom_hud(ui, &painter, rect, state, localizer, dark_mode);

        response
    }

    /// Renderiza visualização estática auto-enquadrada (Abas 2 ou 3) para o preview do Memorial na Aba 5.
    pub fn show_static_preview(
        ui: &mut Ui,
        project: &GeoProject,
        mode: ActiveProjectMode,
        slope_report: Option<&SlopeAnalysisReport>,
        localizer: &Localizer,
        dark_mode: bool,
        height: f32,
    ) {
        let width = ui.available_width().max(320.0);
        let (response, painter) = ui.allocate_painter(Vec2::new(width, height), Sense::hover());
        let rect = response.rect;
        let painter = painter.with_clip_rect(rect);
        painter.rect_filled(rect, 6.0, GeoEstabilTheme::canvas_bg(dark_mode));

        let mut state = CanvasState::default();
        let bounds = match mode {
            ActiveProjectMode::SlopeStability => compute_slope_bounds(project, slope_report),
            ActiveProjectMode::RetainingWall => compute_wall_bounds(project),
        };
        let (fit_pan, fit_zoom) = bounds.compute_fit(rect, 34.0);
        state.pan_offset = fit_pan;
        state.zoom = fit_zoom;

        render_canvas_scene(&painter, rect, project, mode, slope_report, &state, localizer, dark_mode);
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)),
            egui::StrokeKind::Inside,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_canvas_scene(
    painter: &Painter,
    rect: Rect,
    project: &GeoProject,
    mode: ActiveProjectMode,
    slope_report: Option<&SlopeAnalysisReport>,
    state: &CanvasState,
    localizer: &Localizer,
    dark_mode: bool,
) {
    let world_to_screen = |wx: f64, wy: f64| -> Pos2 {
        Pos2::new(
            rect.min.x + state.pan_offset.x + (wx as f32) * state.zoom,
            rect.min.y + state.pan_offset.y - (wy as f32) * state.zoom,
        )
    };

    if state.show_grid {
        draw_metric_grid(painter, rect, state, dark_mode);
    }

    match mode {
        ActiveProjectMode::SlopeStability => {
            draw_slope_mode(
                painter,
                rect,
                project,
                slope_report,
                state,
                localizer,
                dark_mode,
                world_to_screen,
            );
        }
        ActiveProjectMode::RetainingWall => {
            draw_wall_mode(
                painter,
                rect,
                project,
                state,
                localizer,
                dark_mode,
                world_to_screen,
            );
        }
    }
}

fn compute_slope_bounds(
    project: &GeoProject,
    slope_report: Option<&SlopeAnalysisReport>,
) -> WorldBounds {
    let slope = &project.slope_model;
    let mut b = WorldBounds::new(0.0, 40.0, -4.0, 12.0);
    let mut initialized = false;

    let mut init_or_expand = |x: f64, y: f64| {
        if !initialized {
            b = WorldBounds::new(x, x, y, y);
            initialized = true;
        } else {
            b.expand_point(x, y);
        }
    };

    for p in &slope.terrain_surface.points {
        init_or_expand(p.x, p.y);
    }
    for layer in &slope.layers {
        for p in &layer.top_profile.points {
            init_or_expand(p.x, p.y);
        }
    }
    if slope.water_table.enabled {
        for p in &slope.water_table.phreatic_line.points {
            init_or_expand(p.x, p.y);
        }
    }
    if slope.surcharge.enabled {
        let mid_x = 0.5 * (slope.surcharge.x_start + slope.surcharge.x_end);
        let y_mid = slope.terrain_surface.interpolate_y(mid_x).unwrap_or(0.0);
        init_or_expand(slope.surcharge.x_start, y_mid + 2.5);
        init_or_expand(slope.surcharge.x_end, y_mid + 2.5);
    }

    // Subsolo inferior para visibilidade das camadas
    let bedrock_y = b.min_y - 3.5;
    b.expand_point(b.min_x, bedrock_y);

    if let Some(report) = slope_report {
        let c = &report.critical_circle;
        if c.center_x.is_finite() && c.center_y.is_finite() {
            b.expand_point(c.center_x, c.center_y + 1.5);
            // Ponto mais profundo da superfície circular de ruptura
            let lowest_arc_y = c.center_y - c.radius;
            b.expand_point(c.center_x, lowest_arc_y - 1.0);
        }
        if let Some(ref hm) = report.heatmap {
            for pt in &hm.points {
                if pt.min_fs < 90.0 {
                    b.expand_point(pt.center_x, pt.center_y);
                }
            }
        }
    }

    b
}

fn compute_wall_bounds(project: &GeoProject) -> WorldBounds {
    let geom = &project.wall_model.geometry;
    let is_gravity = matches!(project.wall_model.wall_type, RetainingWallType::Gravity);

    let wall_h = if is_gravity {
        geom.stem_height.max(1.0)
    } else {
        geom.total_height().max(1.0)
    };
    let b_base = geom.base_width.max(0.8);
    let slope_ext_x = 3.2;
    let slope_rise = slope_ext_x * geom.backfill_slope_deg.to_radians().tan();
    let surcharge_extra = if project.wall_model.surcharge_q > 0.1 {
        1.2
    } else {
        0.5
    };
    let bottom_extra = if geom.shear_key_enabled {
        geom.shear_key_depth.max(1.8) + 0.6
    } else {
        1.9
    };

    WorldBounds::new(
        -1.6,
        b_base + slope_ext_x + 2.4,
        -bottom_extra,
        wall_h + slope_rise + surcharge_extra,
    )
}

fn draw_metric_grid(painter: &Painter, rect: Rect, state: &CanvasState, dark_mode: bool) {
    // Escolhe passo métrico agradável (~55-90 px por divisão)
    let candidates = [0.5_f64, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0];
    let target_world_step = (70.0 / state.zoom.max(1.0)) as f64;
    let grid_step = candidates
        .iter()
        .copied()
        .find(|&s| s >= target_world_step)
        .unwrap_or(50.0);

    let minor_stroke = Stroke::new(1.0, GeoEstabilTheme::grid_lines(dark_mode));
    let axis_stroke = Stroke::new(
        1.2,
        if dark_mode {
            Color32::from_rgba_premultiplied(148, 163, 184, 70)
        } else {
            Color32::from_rgba_premultiplied(100, 116, 139, 70)
        },
    );
    let tick_color = GeoEstabilTheme::text_secondary(dark_mode);

    let min_wx = (-(state.pan_offset.x) / state.zoom) as f64;
    let max_wx = ((rect.width() - state.pan_offset.x) / state.zoom) as f64;
    let min_wy = (-(rect.height() - state.pan_offset.y) / state.zoom) as f64;
    let max_wy = ((state.pan_offset.y) / state.zoom) as f64;

    let start_x = (min_wx / grid_step).floor() * grid_step;
    let start_y = (min_wy / grid_step).floor() * grid_step;

    let mut x = start_x;
    let mut safety_guard = 0;
    while x <= max_wx && safety_guard < 200 {
        safety_guard += 1;
        let sx = rect.min.x + state.pan_offset.x + (x as f32) * state.zoom;
        let is_origin = x.abs() < 1e-6;
        painter.line_segment(
            [Pos2::new(sx, rect.min.y), Pos2::new(sx, rect.max.y)],
            if is_origin { axis_stroke } else { minor_stroke },
        );
        if sx > rect.min.x + 32.0 && sx < rect.max.x - 24.0 {
            let lbl = if grid_step < 1.0 {
                format!("{:.1}m", x)
            } else {
                format!("{:.0}m", x)
            };
            painter.text(
                Pos2::new(sx + 2.0, rect.max.y - 22.0),
                Align2::LEFT_BOTTOM,
                lbl,
                FontId::monospace(10.0),
                tick_color,
            );
        }
        x += grid_step;
    }

    let mut y = start_y;
    safety_guard = 0;
    while y <= max_wy && safety_guard < 200 {
        safety_guard += 1;
        let sy = rect.min.y + state.pan_offset.y - (y as f32) * state.zoom;
        let is_origin = y.abs() < 1e-6;
        painter.line_segment(
            [Pos2::new(rect.min.x, sy), Pos2::new(rect.max.x, sy)],
            if is_origin { axis_stroke } else { minor_stroke },
        );
        if sy > rect.min.y + 18.0 && sy < rect.max.y - 26.0 {
            let lbl = if grid_step < 1.0 {
                format!("{:.1}", y)
            } else {
                format!("{:.0}", y)
            };
            painter.text(
                Pos2::new(rect.min.x + 6.0, sy - 2.0),
                Align2::LEFT_BOTTOM,
                lbl,
                FontId::monospace(10.0),
                tick_color,
            );
        }
        y += grid_step;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_slope_mode<F>(
    painter: &Painter,
    rect: Rect,
    project: &GeoProject,
    slope_report: Option<&SlopeAnalysisReport>,
    state: &CanvasState,
    localizer: &Localizer,
    dark_mode: bool,
    world_to_screen: F,
) where
    F: Fn(f64, f64) -> Pos2 + Copy,
{
    let slope = &project.slope_model;
    let terrain = &slope.terrain_surface;

    // 1. Camadas Geotécnicas (Polígonos estratificados preenchidos via faixas trapezoidais convexas)
    let x_start = terrain.x_min().min(0.0);
    let x_end = terrain.x_max().max(40.0);
    let deep_floor_y = slope
        .layers
        .iter()
        .map(|l| l.top_profile.y_min())
        .chain(std::iter::once(terrain.y_min()))
        .fold(f64::INFINITY, f64::min)
        - 3.5;

    for (idx, layer) in slope.layers.iter().enumerate() {
        let upper_profile = if idx == 0 {
            terrain
        } else {
            &layer.top_profile
        };
        let lower_profile_opt = slope.layers.get(idx + 1).map(|l| &l.top_profile);

        // Coleta todos os pontos X críticos para construir trapézios exatos sem artefatos côncavos
        let mut xs: Vec<f64> = vec![x_start, x_end];
        for p in &terrain.points {
            xs.push(p.x);
        }
        for p in &upper_profile.points {
            xs.push(p.x);
        }
        if let Some(lp) = lower_profile_opt {
            for p in &lp.points {
                xs.push(p.x);
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        xs.dedup_by(|a, b| (*a - *b).abs() < 1e-4);

        let fill_color = Color32::from_rgba_unmultiplied(
            layer.color_rgba[0],
            layer.color_rgba[1],
            layer.color_rgba[2],
            if dark_mode { 115 } else { 95 },
        );

        for pair in xs.windows(2) {
            let xa = pair[0];
            let xb = pair[1];
            if xb - xa < 1e-4 {
                continue;
            }
            let yt_a = terrain
                .interpolate_y(xa)
                .unwrap_or(0.0)
                .min(upper_profile.interpolate_y(xa).unwrap_or(f64::INFINITY));
            let yt_b = terrain
                .interpolate_y(xb)
                .unwrap_or(0.0)
                .min(upper_profile.interpolate_y(xb).unwrap_or(f64::INFINITY));

            let yb_a = lower_profile_opt
                .and_then(|lp| lp.interpolate_y(xa))
                .unwrap_or(deep_floor_y)
                .min(yt_a);
            let yb_b = lower_profile_opt
                .and_then(|lp| lp.interpolate_y(xb))
                .unwrap_or(deep_floor_y)
                .min(yt_b);

            if yt_a > yb_a + 1e-3 || yt_b > yb_b + 1e-3 {
                let quad = vec![
                    world_to_screen(xa, yt_a),
                    world_to_screen(xb, yt_b),
                    world_to_screen(xb, yb_b),
                    world_to_screen(xa, yb_a),
                ];
                painter.add(Shape::convex_polygon(quad, fill_color, Stroke::NONE));
            }
        }

        // Linha divisória do topo da camada
        if layer.top_profile.points.len() >= 2 {
            let pts: Vec<Pos2> = layer
                .top_profile
                .points
                .iter()
                .map(|p| world_to_screen(p.x, p.y))
                .collect();
            let line_color = Color32::from_rgb(
                layer.color_rgba[0],
                layer.color_rgba[1],
                layer.color_rgba[2],
            );
            painter.add(Shape::line(pts, Stroke::new(1.8, line_color)));
        }
    }

    // 2. Lençol Freático (Linha Piezométrica + Triângulo N.A.)
    if slope.water_table.enabled {
        let wt = &slope.water_table.phreatic_line;
        if wt.points.len() >= 2 {
            let pts: Vec<Pos2> = wt
                .points
                .iter()
                .map(|p| world_to_screen(p.x, p.y))
                .collect();
            let wt_color = GeoEstabilTheme::water_blue(dark_mode);
            painter.add(Shape::line(pts.clone(), Stroke::new(2.2, wt_color)));

            // Símbolo N.A. no meio da linha freática
            let mid_x = 0.5 * (wt.x_min() + wt.x_max());
            if let Some(mid_y) = wt.interpolate_y(mid_x) {
                let na = world_to_screen(mid_x, mid_y);
                let p1 = na + Vec2::new(-6.0, -9.0);
                let p2 = na + Vec2::new(6.0, -9.0);
                painter.add(Shape::convex_polygon(
                    vec![p1, p2, na],
                    wt_color,
                    Stroke::NONE,
                ));
                painter.text(
                    na + Vec2::new(9.0, -4.0),
                    Align2::LEFT_BOTTOM,
                    localizer.t("canvas.wt"),
                    FontId::proportional(11.0),
                    wt_color,
                );
            }
        }
    }

    // 3. Perfil do Terreno Superficial
    if terrain.points.len() >= 2 {
        let pts: Vec<Pos2> = terrain
            .points
            .iter()
            .map(|p| world_to_screen(p.x, p.y))
            .collect();

        let terrain_stroke = Stroke::new(
            2.8,
            if dark_mode {
                Color32::from_rgb(241, 245, 249)
            } else {
                Color32::from_rgb(15, 23, 42)
            },
        );
        painter.add(Shape::line(pts.clone(), terrain_stroke));

        for (i, (&pos, pt)) in pts.iter().zip(terrain.points.iter()).enumerate() {
            painter.circle_filled(pos, 4.0, GeoEstabilTheme::soil_brown(dark_mode));
            painter.text(
                pos + Vec2::new(0.0, -8.0),
                Align2::CENTER_BOTTOM,
                format!("P{}({:.0};{:.0})", i + 1, pt.x, pt.y),
                FontId::proportional(10.0),
                GeoEstabilTheme::text_secondary(dark_mode),
            );
        }
    }

    // 4. Sobrecarga Distribuída q (ancorada à elevação real do terreno)
    if slope.surcharge.enabled {
        let sur = &slope.surcharge;
        let span = (sur.x_end - sur.x_start).max(0.5);
        let arrow_count = ((span / 2.2).round() as usize).clamp(3, 14);
        let arrow_color = GeoEstabilTheme::slip_circle_red(dark_mode);
        let arrow_world_h = (28.0 / state.zoom.max(1.0)) as f64;

        let mut bar_pts = Vec::with_capacity(arrow_count + 1);
        for i in 0..=arrow_count {
            let x = sur.x_start + (span * i as f64) / arrow_count as f64;
            if let Some(yt) = terrain.interpolate_y(x) {
                let p_top = world_to_screen(x, yt + arrow_world_h);
                let p_ground = world_to_screen(x, yt);
                bar_pts.push(p_top);
                painter.arrow(p_top, p_ground - p_top, Stroke::new(1.8, arrow_color));
            }
        }
        if bar_pts.len() >= 2 {
            painter.add(Shape::line(bar_pts, Stroke::new(2.0, arrow_color)));
        }

        let x_mid = 0.5 * (sur.x_start + sur.x_end);
        let y_mid = terrain.interpolate_y(x_mid).unwrap_or(0.0);
        let p_label = world_to_screen(x_mid, y_mid + arrow_world_h) + Vec2::new(0.0, -4.0);
        painter.text(
            p_label,
            Align2::CENTER_BOTTOM,
            format!("q = {:.1} kPa ({})", sur.q_kpa, sur.name),
            FontId::proportional(12.0),
            arrow_color,
        );
    }

    // 5. Mapa de Calor (Heatmap) dos Centros de Ruptura
    if state.show_heatmap {
        if let Some(report) = slope_report {
            if let Some(ref hm) = report.heatmap {
                let dx_w = if slope.grid_x_steps > 1 {
                    (hm.max_x - hm.min_x) / (slope.grid_x_steps - 1) as f64
                } else {
                    1.0
                };
                let dy_w = if slope.grid_y_steps > 1 {
                    (hm.max_y - hm.min_y) / (slope.grid_y_steps - 1) as f64
                } else {
                    1.0
                };
                let cell_w = ((dx_w as f32) * state.zoom * 0.88).clamp(5.0, 28.0);
                let cell_h = ((dy_w as f32) * state.zoom * 0.88).clamp(5.0, 28.0);

                for pt in &hm.points {
                    if pt.min_fs < 50.0 {
                        let c_scr = world_to_screen(pt.center_x, pt.center_y);
                        let c_col = GeoEstabilTheme::heatmap_color(pt.min_fs);
                        let soft = Color32::from_rgba_unmultiplied(c_col.r(), c_col.g(), c_col.b(), 125);
                        let cell_rect = Rect::from_center_size(c_scr, Vec2::new(cell_w, cell_h));
                        painter.rect_filled(cell_rect, 2.0, soft);
                        painter.circle_filled(c_scr, 2.5, c_col);
                    }
                }
            }
        }
    }

    // 6. Fatias e Arco Circular Crítico (recortado entre x_entry e x_exit)
    if let Some(report) = slope_report {
        let circ = &report.critical_circle;

        // Fatias verticais dentro da massa instável
        if state.show_slices && !circ.slices.is_empty() {
            let slice_fill = Color32::from_rgba_unmultiplied(59, 130, 246, if dark_mode { 28 } else { 24 });
            let slice_stroke = Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(148, 163, 184, 165),
            );
            for s in &circ.slices {
                let half_w = s.width * 0.5;
                let y_top_l = terrain.interpolate_y(s.x_mid - half_w).unwrap_or(s.y_top);
                let y_top_r = terrain.interpolate_y(s.x_mid + half_w).unwrap_or(s.y_top);

                let dy_l = circ.radius.powi(2) - (s.x_mid - half_w - circ.center_x).powi(2);
                let dy_r = circ.radius.powi(2) - (s.x_mid + half_w - circ.center_x).powi(2);
                let y_bot_l = if dy_l >= 0.0 {
                    (circ.center_y - dy_l.sqrt()).min(y_top_l)
                } else {
                    s.y_base
                };
                let y_bot_r = if dy_r >= 0.0 {
                    (circ.center_y - dy_r.sqrt()).min(y_top_r)
                } else {
                    s.y_base
                };

                if y_top_l >= y_bot_l && y_top_r >= y_bot_r {
                    painter.add(Shape::convex_polygon(
                        vec![
                            world_to_screen(s.x_mid - half_w, y_top_l),
                            world_to_screen(s.x_mid + half_w, y_top_r),
                            world_to_screen(s.x_mid + half_w, y_bot_r),
                            world_to_screen(s.x_mid - half_w, y_bot_l),
                        ],
                        slice_fill,
                        Stroke::NONE,
                    ));
                }

                let p_top = world_to_screen(s.x_mid, s.y_top);
                let p_bot = world_to_screen(s.x_mid, s.y_base);
                painter.line_segment([p_top, p_bot], slice_stroke);
            }
        }

        // Arco circular recortado entre entrada (x_entry) e saída (x_exit)
        let center_screen = world_to_screen(circ.center_x, circ.center_y);
        let y_entry = terrain.interpolate_y(circ.x_entry).unwrap_or(circ.center_y);
        let y_exit = terrain.interpolate_y(circ.x_exit).unwrap_or(circ.center_y);

        let ang_start = (y_entry - circ.center_y).atan2(circ.x_entry - circ.center_x);
        let ang_end = (y_exit - circ.center_y).atan2(circ.x_exit - circ.center_x);
        let mut sweep = ang_end - ang_start;
        // Garante que o arco percorra a parte inferior do círculo
        if sweep > std::f64::consts::PI {
            sweep -= std::f64::consts::TAU;
        } else if sweep < -std::f64::consts::PI {
            sweep += std::f64::consts::TAU;
        }

        let segments = 64;
        let mut arc_pts = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let theta = ang_start + sweep * (i as f64 / segments as f64);
            let wx = circ.center_x + circ.radius * theta.cos();
            let wy = circ.center_y + circ.radius * theta.sin();
            arc_pts.push(world_to_screen(wx, wy));
        }

        let arc_color = if report.is_stable {
            GeoEstabilTheme::success_green(dark_mode)
        } else {
            GeoEstabilTheme::slip_circle_red(dark_mode)
        };
        painter.add(Shape::line(arc_pts, Stroke::new(3.0, arc_color)));

        // Raios radiais sutis até os extremos do arco
        let ray_stroke = Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(arc_color.r(), arc_color.g(), arc_color.b(), 110),
        );
        let p_entry_scr = world_to_screen(circ.x_entry, y_entry);
        let p_exit_scr = world_to_screen(circ.x_exit, y_exit);
        painter.line_segment([center_screen, p_entry_scr], ray_stroke);
        painter.line_segment([center_screen, p_exit_scr], ray_stroke);
        painter.circle_filled(p_entry_scr, 4.0, arc_color);
        painter.circle_filled(p_exit_scr, 4.0, arc_color);

        // Marcador do centro (+)
        let center_stroke = Stroke::new(2.2, arc_color);
        painter.line_segment(
            [
                center_screen - Vec2::new(8.0, 0.0),
                center_screen + Vec2::new(8.0, 0.0),
            ],
            center_stroke,
        );
        painter.line_segment(
            [
                center_screen - Vec2::new(0.0, 8.0),
                center_screen + Vec2::new(0.0, 8.0),
            ],
            center_stroke,
        );

        let (method_tag, primary_fs) = match slope.method {
            SlopeAnalysisMethod::BishopSimplified => ("Bishop", circ.fs_bishop),
            SlopeAnalysisMethod::FelleniusOrdinary => ("Fellenius", circ.fs_fellenius),
            SlopeAnalysisMethod::JanbuSimplified => ("Janbu", circ.fs_janbu),
        };
        let label_pos = (center_screen + Vec2::new(12.0, -6.0))
            .clamp(rect.min + Vec2::new(10.0, 24.0), rect.max - Vec2::new(180.0, 40.0));
        painter.text(
            label_pos,
            Align2::LEFT_BOTTOM,
            format!(
                "{} ({:.1}; {:.1}) R={:.1}m\nFS ({}) = {:.3} [{}]",
                localizer.pick("Centro", "Center"),
                circ.center_x,
                circ.center_y,
                circ.radius,
                method_tag,
                primary_fs,
                if report.is_stable {
                    localizer.t("canvas.stable")
                } else {
                    localizer.t("canvas.unstable")
                }
            ),
            FontId::proportional(12.5),
            arc_color,
        );
    }

    // 7. Legendas Overlay (Camadas de Solo + Mapa de Calor FS)
    draw_slope_overlays(painter, rect, project, slope_report, state.show_heatmap, localizer, dark_mode);
}

fn draw_slope_overlays(
    painter: &Painter,
    rect: Rect,
    project: &GeoProject,
    slope_report: Option<&SlopeAnalysisReport>,
    show_heatmap: bool,
    localizer: &Localizer,
    dark_mode: bool,
) {
    let hud_bg = if dark_mode {
        Color32::from_rgba_unmultiplied(24, 27, 34, 225)
    } else {
        Color32::from_rgba_unmultiplied(248, 250, 252, 230)
    };
    let border = Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode));

    // Legenda de Camadas (Canto superior esquerdo)
    let layers = &project.slope_model.layers;
    if !layers.is_empty() && rect.width() > 360.0 {
        let rows = layers.len().min(5);
        let box_h = 24.0 + (rows as f32) * 18.0;
        let box_w = 275.0_f32.min(rect.width() * 0.48);
        let legend_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(box_w, box_h));
        painter.rect_filled(legend_rect, CornerRadius::same(5), hud_bg);
        painter.rect_stroke(legend_rect, CornerRadius::same(5), border, egui::StrokeKind::Inside);
        let legend_painter = painter.with_clip_rect(legend_rect);

        legend_painter.text(
            legend_rect.min + Vec2::new(8.0, 5.0),
            Align2::LEFT_TOP,
            localizer.t("canvas.stratigraphy"),
            FontId::proportional(11.0),
            GeoEstabilTheme::text_primary(dark_mode),
        );

        for (i, layer) in layers.iter().take(5).enumerate() {
            let y = legend_rect.min.y + 23.0 + (i as f32) * 18.0;
            let sw = Rect::from_min_size(Pos2::new(legend_rect.min.x + 8.0, y + 2.0), Vec2::new(10.0, 10.0));
            let col = Color32::from_rgb(layer.color_rgba[0], layer.color_rgba[1], layer.color_rgba[2]);
            legend_painter.rect_filled(sw, 2.0, col);
            paint_math(
                &legend_painter,
                Pos2::new(legend_rect.min.x + 23.0, y),
                Align2::LEFT_TOP,
                &format!(
                    "{}: c'={:.0} φ'={:.0}° γ={:.1}",
                    layer.name, layer.cohesion, layer.friction_angle_deg, layer.gamma
                ),
                10.5,
                false,
                dark_mode,
                GeoEstabilTheme::text_secondary(dark_mode),
            );
        }
    }

    // Legenda do Heatmap ou Aviso de Cálculo (Canto superior direito)
    if rect.width() > 420.0 {
        if let Some(report) = slope_report {
            if show_heatmap {
                let hm_rect = Rect::from_min_size(
                    Pos2::new(rect.max.x - 165.0, rect.min.y + 10.0),
                    Vec2::new(155.0, 74.0),
                );
                painter.rect_filled(hm_rect, CornerRadius::same(5), hud_bg);
                painter.rect_stroke(hm_rect, CornerRadius::same(5), border, egui::StrokeKind::Inside);
                painter.text(
                    hm_rect.min + Vec2::new(8.0, 5.0),
                    Align2::LEFT_TOP,
                    localizer.t("canvas.heatmap"),
                    FontId::proportional(11.0),
                    GeoEstabilTheme::text_primary(dark_mode),
                );
                let bar_w = 139.0;
                let steps = 20;
                for i in 0..steps {
                    let t = i as f64 / (steps - 1) as f64;
                    let fs_sample = 0.8 + t * 1.4;
                    let col = GeoEstabilTheme::heatmap_color(fs_sample);
                    let r = Rect::from_min_size(
                        Pos2::new(hm_rect.min.x + 8.0 + (i as f32) * (bar_w / steps as f32), hm_rect.min.y + 24.0),
                        Vec2::new(bar_w / steps as f32 + 0.5, 12.0),
                    );
                    painter.rect_filled(r, 0.0, col);
                }
                painter.text(
                    Pos2::new(hm_rect.min.x + 8.0, hm_rect.min.y + 39.0),
                    Align2::LEFT_TOP,
                    "≤1.0      1.3      1.5      ≥2.0",
                    FontId::monospace(9.5),
                    GeoEstabilTheme::text_secondary(dark_mode),
                );
                paint_math(
                    painter,
                    Pos2::new(hm_rect.min.x + 8.0, hm_rect.min.y + 54.0),
                    Align2::LEFT_TOP,
                    &format!(
                        "{} = {:.3} ({:.1} ms)",
                        localizer.pick("FS_{mín}", "FS_{min}"),
                        report.critical_circle.fs_bishop,
                        report.elapsed_ms
                    ),
                    10.5,
                    true,
                    dark_mode,
                    if report.is_stable {
                        GeoEstabilTheme::success_green(dark_mode)
                    } else {
                        GeoEstabilTheme::slip_circle_red(dark_mode)
                    },
                );
            }
        } else {
            let hint_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - 245.0, rect.min.y + 10.0),
                Vec2::new(235.0, 38.0),
            );
            painter.rect_filled(hint_rect, CornerRadius::same(5), hud_bg);
            painter.rect_stroke(hint_rect, CornerRadius::same(5), border, egui::StrokeKind::Inside);
            painter.text(
                hint_rect.center(),
                Align2::CENTER_CENTER,
                localizer.t("canvas.calc_hint"),
                FontId::proportional(10.5),
                GeoEstabilTheme::warning_amber(dark_mode),
            );
        }
    }
}

fn draw_wall_mode<F>(
    painter: &Painter,
    rect: Rect,
    project: &GeoProject,
    state: &CanvasState,
    localizer: &Localizer,
    dark_mode: bool,
    world_to_screen: F,
) where
    F: Fn(f64, f64) -> Pos2 + Copy,
{
    let model = &project.wall_model;
    let geom = &model.geometry;
    let stability = check_wall_external_stability(model);
    let pressures = calculate_earth_pressures(model);
    let bearing = calculate_bearing_capacity(model, &stability);

    let is_gravity = matches!(model.wall_type, RetainingWallType::Gravity);
    let h_stem = geom.stem_height.max(1.0);
    let h_base = if is_gravity { 0.0 } else { geom.base_thickness.max(0.0) };
    let h_total = h_base + h_stem;
    let b_base = geom.base_width.max(0.8);
    let b_toe = if is_gravity { 0.0 } else { geom.toe_length.clamp(0.0, b_base) };
    let b_stem_base = if is_gravity {
        b_base
    } else {
        geom.stem_base_width.clamp(0.2, b_base - b_toe)
    };
    let b_stem_top = geom.stem_top_width.clamp(0.2, b_stem_base);

    let slope_ext = 3.2;
    let slope_rad = geom.backfill_slope_deg.to_radians();
    let slope_rise = slope_ext * slope_rad.tan();

    // 1. Solo de Fundação abaixo da base (y = -1.6 .. 0.0)
    let found_fill = Color32::from_rgba_unmultiplied(120, 80, 50, if dark_mode { 90 } else { 75 });
    painter.add(Shape::convex_polygon(
        vec![
            world_to_screen(-1.5, 0.0),
            world_to_screen(b_base + slope_ext + 1.5, 0.0),
            world_to_screen(b_base + slope_ext + 1.5, -1.6),
            world_to_screen(-1.5, -1.6),
        ],
        found_fill,
        Stroke::NONE,
    ));
    painter.line_segment(
        [world_to_screen(-1.5, 0.0), world_to_screen(b_base + slope_ext + 1.5, 0.0)],
        Stroke::new(1.5, GeoEstabilTheme::soil_brown(dark_mode)),
    );

    // 2. Solo de Embutimento Frontal sobre a ponta da sapata (Df = h_base)
    if !is_gravity && h_base > 0.01 && b_toe > 0.01 {
        let df_fill = Color32::from_rgba_unmultiplied(161, 98, 7, if dark_mode { 100 } else { 85 });
        painter.add(Shape::convex_polygon(
            vec![
                world_to_screen(-1.5, h_base),
                world_to_screen(b_toe, h_base),
                world_to_screen(b_toe, 0.0),
                world_to_screen(-1.5, 0.0),
            ],
            df_fill,
            Stroke::NONE,
        ));
        painter.line_segment(
            [world_to_screen(-1.5, h_base), world_to_screen(b_toe, h_base)],
            Stroke::new(1.8, GeoEstabilTheme::soil_brown(dark_mode)),
        );
    }

    // 3. Solo de Reaterro no Tardoz (decomposto em polígonos convexos sem artefatos!)
    let backfill_fill = Color32::from_rgba_unmultiplied(217, 158, 93, if dark_mode { 95 } else { 85 });
    let x_wall_back_bot = if is_gravity { b_base } else { b_toe + b_stem_base };
    let x_wall_back_top = if is_gravity { b_stem_top } else { b_toe + b_stem_top };

    // Trapézio encostado no fuste (de y=h_base até y=h_total)
    painter.add(Shape::convex_polygon(
        vec![
            world_to_screen(x_wall_back_top, h_total),
            world_to_screen(b_base, h_total),
            world_to_screen(b_base, h_base),
            world_to_screen(x_wall_back_bot, h_base),
        ],
        backfill_fill,
        Stroke::NONE,
    ));
    // Maciço estendido atrás de x = b_base (inclui eventual rampa beta no topo)
    painter.add(Shape::convex_polygon(
        vec![
            world_to_screen(b_base, h_total),
            world_to_screen(b_base + slope_ext, h_total + slope_rise),
            world_to_screen(b_base + slope_ext, 0.0),
            world_to_screen(b_base, 0.0),
        ],
        backfill_fill,
        Stroke::NONE,
    ));

    // Contorno superior do terrapleno
    let ground_stroke = Stroke::new(2.2, GeoEstabilTheme::soil_brown(dark_mode));
    painter.line_segment(
        [
            world_to_screen(x_wall_back_top, h_total),
            world_to_screen(b_base, h_total),
        ],
        ground_stroke,
    );
    painter.line_segment(
        [
            world_to_screen(b_base, h_total),
            world_to_screen(b_base + slope_ext, h_total + slope_rise),
        ],
        ground_stroke,
    );

    // 4. Nível d'Água no Tardoz (se hw > 0)
    let hw = model.water_height_back.clamp(0.0, h_total);
    if hw > 0.01 {
        let water_fill = Color32::from_rgba_unmultiplied(56, 189, 248, 60);
        let x_water_left = if is_gravity {
            let t = (hw / h_stem).clamp(0.0, 1.0);
            b_base + t * (b_stem_top - b_base)
        } else if hw <= h_base {
            b_base
        } else {
            let t = ((hw - h_base) / h_stem).clamp(0.0, 1.0);
            x_wall_back_bot + t * (x_wall_back_top - x_wall_back_bot)
        };
        painter.add(Shape::convex_polygon(
            vec![
                world_to_screen(x_water_left, hw),
                world_to_screen(b_base + slope_ext, hw),
                world_to_screen(b_base + slope_ext, 0.0),
                world_to_screen(b_base, 0.0),
            ],
            water_fill,
            Stroke::NONE,
        ));
        painter.line_segment(
            [
                world_to_screen(x_water_left, hw),
                world_to_screen(b_base + slope_ext, hw),
            ],
            Stroke::new(2.0, GeoEstabilTheme::water_blue(dark_mode)),
        );
    }

    // 5. Sobrecarga no Terrapleno q
    if model.surcharge_q > 0.1 {
        let q_color = GeoEstabilTheme::warning_amber(dark_mode);
        let arrows = 5;
        for i in 0..=arrows {
            let dx = (slope_ext - 0.2) * (i as f64 / arrows as f64) + 0.1;
            let gx = b_base + dx;
            let gy = h_total + dx * slope_rad.tan();
            let p_top = world_to_screen(gx, gy + 0.65);
            let p_bot = world_to_screen(gx, gy);
            painter.arrow(p_top, p_bot - p_top, Stroke::new(1.6, q_color));
        }
        painter.line_segment(
            [
                world_to_screen(b_base + 0.1, h_total + 0.1 * slope_rad.tan() + 0.65),
                world_to_screen(
                    b_base + slope_ext - 0.1,
                    h_total + (slope_ext - 0.1) * slope_rad.tan() + 0.65,
                ),
            ],
            Stroke::new(1.8, q_color),
        );
        painter.text(
            world_to_screen(b_base + slope_ext * 0.5, h_total + slope_rise * 0.5 + 0.75),
            Align2::CENTER_BOTTOM,
            format!("q = {:.1} kPa", model.surcharge_q),
            FontId::proportional(11.5),
            q_color,
        );
    }

    // 6. Corpo do Muro de Contenção (Cantilever vs Gravidade + Dente de Ancoragem)
    let conc_fill = GeoEstabilTheme::concrete_gray(dark_mode);
    let conc_stroke = Stroke::new(
        2.2,
        if dark_mode {
            Color32::from_rgb(241, 245, 249)
        } else {
            Color32::from_rgb(15, 23, 42)
        },
    );

    if is_gravity {
        // Trapezoidal: base (0..b_base, 0), crista (0..b_stem_top, h_stem)
        painter.add(Shape::convex_polygon(
            vec![
                world_to_screen(0.0, 0.0),
                world_to_screen(b_base, 0.0),
                world_to_screen(b_stem_top, h_stem),
                world_to_screen(0.0, h_stem),
            ],
            conc_fill,
            conc_stroke,
        ));
    } else {
        // Sapata
        if h_base > 0.01 {
            painter.add(Shape::convex_polygon(
                vec![
                    world_to_screen(0.0, 0.0),
                    world_to_screen(b_base, 0.0),
                    world_to_screen(b_base, h_base),
                    world_to_screen(0.0, h_base),
                ],
                conc_fill,
                conc_stroke,
            ));
        }
        // Fuste
        painter.add(Shape::convex_polygon(
            vec![
                world_to_screen(b_toe, h_base),
                world_to_screen(b_toe + b_stem_base, h_base),
                world_to_screen(b_toe + b_stem_top, h_total),
                world_to_screen(b_toe, h_total),
            ],
            conc_fill,
            conc_stroke,
        ));
    }

    // Dente de ancoragem (chave de cisalhamento)
    if geom.shear_key_enabled && geom.shear_key_depth > 0.02 && geom.shear_key_width > 0.02 {
        let kx_center = if is_gravity {
            b_base * 0.5
        } else {
            b_toe + b_stem_base * 0.5
        };
        let kw = geom.shear_key_width;
        let kd = geom.shear_key_depth;
        painter.add(Shape::convex_polygon(
            vec![
                world_to_screen(kx_center - kw * 0.5, 0.0),
                world_to_screen(kx_center + kw * 0.5, 0.0),
                world_to_screen(kx_center + kw * 0.5, -kd),
                world_to_screen(kx_center - kw * 0.5, -kd),
            ],
            conc_fill,
            conc_stroke,
        ));
    }

    // 7. Diagramas Proporcionais de Empuxo Lateral (Ea, Ep) & Pressão no Solo
    if state.show_diagrams {
        let ka = pressures.ka;
        let sigma_q = model.surcharge_q * ka;
        let sigma_soil_bot = model.backfill_soil.gamma * h_total * ka;
        let sigma_max = (sigma_q + sigma_soil_bot + 9.81 * hw).max(1.0);
        // Normaliza largura gráfica do diagrama para ~1.5 m em escala de mundo
        let diag_scale = 1.5 / sigma_max;

        let x_ref = b_base + 0.15;
        let ea_color = GeoEstabilTheme::slip_circle_red(dark_mode);
        let ea_fill = Color32::from_rgba_unmultiplied(239, 68, 68, 55);

        let w_top = (sigma_q * diag_scale).max(0.0);
        let w_bot = ((sigma_q + sigma_soil_bot) * diag_scale).max(0.05);
        painter.add(Shape::convex_polygon(
            vec![
                world_to_screen(x_ref, h_total),
                world_to_screen(x_ref + w_top, h_total),
                world_to_screen(x_ref + w_bot, 0.0),
                world_to_screen(x_ref, 0.0),
            ],
            ea_fill,
            Stroke::new(1.5, ea_color),
        ));

        // Cunha hidrostática sobreposta se hw > 0
        if hw > 0.05 {
            let w_water = (9.81 * hw * diag_scale).max(0.05);
            let w_color = GeoEstabilTheme::water_blue(dark_mode);
            painter.add(Shape::convex_polygon(
                vec![
                    world_to_screen(x_ref + w_bot, hw),
                    world_to_screen(x_ref + w_bot + w_water, 0.0),
                    world_to_screen(x_ref + w_bot, 0.0),
                ],
                Color32::from_rgba_unmultiplied(56, 189, 248, 80),
                Stroke::new(1.2, w_color),
            ));
        }

        // Vetor resultante do empuxo ativo horizontal Ea
        let p_thrust_target = world_to_screen(b_base, pressures.ea_arm_y);
        let p_thrust_origin = world_to_screen(b_base + 2.2, pressures.ea_arm_y);
        painter.arrow(
            p_thrust_origin,
            p_thrust_target - p_thrust_origin,
            Stroke::new(2.8, ea_color),
        );
        paint_math(
            painter,
            p_thrust_origin + Vec2::new(6.0, 0.0),
            Align2::LEFT_CENTER,
            &format!(
                "E_{{a,h}} = {:.1} kN/m\n(y = {:.2} m)",
                pressures.ea_horizontal, pressures.ea_arm_y
            ),
            11.5,
            true,
            dark_mode,
            ea_color,
        );

        // Vetor de empuxo passivo na ponta (se embutido)
        if pressures.ep_horizontal > 0.5 && h_base > 0.05 {
            let ep_color = GeoEstabilTheme::success_green(dark_mode);
            let p_ep_start = world_to_screen(-1.1, h_base / 3.0);
            let p_ep_end = world_to_screen(0.0, h_base / 3.0);
            painter.arrow(p_ep_start, p_ep_end - p_ep_start, Stroke::new(2.2, ep_color));
            paint_math(
                painter,
                p_ep_start + Vec2::new(-4.0, -8.0),
                Align2::RIGHT_BOTTOM,
                &format!("E_{{p}} = {:.1} kN/m", pressures.ep_horizontal),
                10.5,
                true,
                dark_mode,
                ep_color,
            );
        }

        // Diagrama proporcional de tensões na base da sapata
        let max_sigma = stability.sigma_toe_kpa.max(stability.sigma_heel_kpa).max(1.0);
        let stress_depth_max = 1.15;
        let d_toe = (stability.sigma_toe_kpa / max_sigma).clamp(0.05, 1.0) * stress_depth_max;
        let d_heel = (stability.sigma_heel_kpa / max_sigma).clamp(0.0, 1.0) * stress_depth_max;

        let stress_col = if stability.is_bearing_stress_ok {
            GeoEstabilTheme::secondary_blue(dark_mode)
        } else {
            GeoEstabilTheme::slip_circle_red(dark_mode)
        };
        let stress_fill = Color32::from_rgba_unmultiplied(stress_col.r(), stress_col.g(), stress_col.b(), 65);

        let p_b0 = world_to_screen(0.0, 0.0);
        let p_bb = world_to_screen(b_base, 0.0);
        let p_st = world_to_screen(0.0, -d_toe);
        let p_sh = world_to_screen(b_base, -d_heel);

        if d_heel > 0.01 {
            painter.add(Shape::convex_polygon(
                vec![p_b0, p_bb, p_sh, p_st],
                stress_fill,
                Stroke::new(1.8, stress_col),
            ));
        } else {
            // Descolamento parcial: contato efetivo comprimido = 3 * xr
            let x_contact = (3.0 * stability.resultant_position_xr).clamp(0.05, b_base);
            let p_zc = world_to_screen(x_contact, 0.0);
            painter.add(Shape::convex_polygon(
                vec![p_b0, p_zc, p_st],
                stress_fill,
                Stroke::new(1.8, stress_col),
            ));
        }

        paint_math(
            painter,
            p_st + Vec2::new(-4.0, 4.0),
            Align2::RIGHT_TOP,
            &format!(
                "{} = {:.1} kPa",
                localizer.pick("σ_{ponta}", "σ_{toe}"),
                stability.sigma_toe_kpa
            ),
            11.5,
            true,
            dark_mode,
            stress_col,
        );
        paint_math(
            painter,
            p_sh + Vec2::new(4.0, 4.0),
            Align2::LEFT_TOP,
            &format!(
                "{} = {:.1} kPa",
                localizer.pick("σ_{calcanhar}", "σ_{heel}"),
                stability.sigma_heel_kpa
            ),
            11.5,
            true,
            dark_mode,
            stress_col,
        );

        // Marcador da resultante e excentricidade
        let p_xr = world_to_screen(stability.resultant_position_xr, 0.0);
        let xr_color = if stability.is_in_middle_third {
            GeoEstabilTheme::success_green(dark_mode)
        } else {
            GeoEstabilTheme::warning_amber(dark_mode)
        };
        painter.circle_filled(p_xr, 4.5, xr_color);
        paint_math(
            painter,
            world_to_screen(b_base * 0.5, -(stress_depth_max + 0.15)),
            Align2::CENTER_TOP,
            &format!(
                "x_{{R}} = {:.2} m | e = {:.3} m (lim B/6 = {:.3} m)",
                stability.resultant_position_xr,
                stability.eccentricity,
                stability.max_allowed_eccentricity
            ),
            11.0,
            true,
            dark_mode,
            xr_color,
        );
    }

    // 8. Cotas Dimensionais do Muro
    if state.show_dimensions {
        let dim_col = GeoEstabilTheme::text_secondary(dark_mode);
        let dim_stroke = Stroke::new(1.0, dim_col);
        // Cota de Altura Total H
        let x_dim = -0.55;
        let p_d0 = world_to_screen(x_dim, 0.0);
        let p_d1 = world_to_screen(x_dim, h_total);
        painter.line_segment([p_d0, p_d1], dim_stroke);
        painter.line_segment([p_d0 - Vec2::new(4.0, 0.0), p_d0 + Vec2::new(4.0, 0.0)], dim_stroke);
        painter.line_segment([p_d1 - Vec2::new(4.0, 0.0), p_d1 + Vec2::new(4.0, 0.0)], dim_stroke);
        painter.text(
            world_to_screen(x_dim - 0.08, h_total * 0.5),
            Align2::RIGHT_CENTER,
            format!("H = {:.2}m", h_total),
            FontId::proportional(11.0),
            dim_col,
        );

        // Cota da Largura da Base B
        let y_dim_b = h_total + 0.35;
        let p_b_l = world_to_screen(0.0, y_dim_b);
        let p_b_r = world_to_screen(b_base, y_dim_b);
        painter.line_segment([p_b_l, p_b_r], dim_stroke);
        painter.text(
            world_to_screen(b_base * 0.5, y_dim_b) + Vec2::new(0.0, -4.0),
            Align2::CENTER_BOTTOM,
            format!("B = {:.2}m", b_base),
            FontId::proportional(11.0),
            dim_col,
        );
    }

    // 9. Card HUD de Estabilidade (Canto superior direito)
    if rect.width() > 380.0 {
        let hud_bg = if dark_mode {
            Color32::from_rgba_unmultiplied(24, 27, 34, 228)
        } else {
            Color32::from_rgba_unmultiplied(248, 250, 252, 235)
        };
        let hud_rect = Rect::from_min_size(
            Pos2::new(rect.max.x - 215.0, rect.min.y + 10.0),
            Vec2::new(205.0, 96.0),
        );
        painter.rect_filled(hud_rect, CornerRadius::same(5), hud_bg);
        painter.rect_stroke(
            hud_rect,
            CornerRadius::same(5),
            Stroke::new(1.0, GeoEstabilTheme::panel_border(dark_mode)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            hud_rect.min + Vec2::new(8.0, 6.0),
            Align2::LEFT_TOP,
            localizer.t("canvas.wall_checks"),
            FontId::proportional(11.0),
            GeoEstabilTheme::text_primary(dark_mode),
        );

        let lines = [
            (
                format!("{}: FS = {:.2} (≥1.50)", localizer.t("tab4.overturning"), stability.fs_overturning),
                stability.is_overturning_ok,
            ),
            (
                format!("{}: FS = {:.2} (≥1.50)", localizer.t("tab4.sliding"), stability.fs_sliding),
                stability.is_sliding_ok,
            ),
            (
                format!(
                    "{}: e = {:.3}m",
                    localizer.pick("Terço Médio", "Middle Third"),
                    stability.eccentricity
                ),
                stability.is_in_middle_third,
            ),
            (
                format!(
                    "{}: FS = {:.2} (≥2.00)",
                    localizer.pick("Cap. Hansen", "Hansen Bearing"),
                    bearing.fs_bearing
                ),
                bearing.is_bearing_ok && stability.is_bearing_stress_ok,
            ),
        ];

        for (i, (text, ok)) in lines.iter().enumerate() {
            let col = if *ok {
                GeoEstabilTheme::success_green(dark_mode)
            } else {
                GeoEstabilTheme::slip_circle_red(dark_mode)
            };
            painter.text(
                hud_rect.min + Vec2::new(8.0, 25.0 + (i as f32) * 16.5),
                Align2::LEFT_TOP,
                format!("{} {}", if *ok { "✔" } else { "✖" }, text),
                FontId::proportional(10.5),
                col,
            );
        }
    }
}

fn draw_canvas_bottom_hud(
    ui: &Ui,
    painter: &Painter,
    rect: Rect,
    state: &CanvasState,
    localizer: &Localizer,
    dark_mode: bool,
) {
    let hud_bg = if dark_mode {
        Color32::from_rgba_unmultiplied(24, 27, 34, 220)
    } else {
        Color32::from_rgba_unmultiplied(248, 250, 252, 220)
    };
    let hud_h = 20.0;
    let strip = Rect::from_min_max(Pos2::new(rect.min.x + 1.0, rect.max.y - hud_h), rect.max - Vec2::new(1.0, 1.0));
    painter.rect_filled(strip, 0.0, hud_bg);

    let pointer_info = ui
        .input(|i| i.pointer.hover_pos())
        .filter(|p| rect.contains(*p))
        .map(|p| {
            let wx = (p.x - rect.min.x - state.pan_offset.x) / state.zoom;
            let wy = (rect.min.y + state.pan_offset.y - p.y) / state.zoom;
            format!("X = {:.2} m  |  Y = {:.2} m", wx, wy)
        })
        .unwrap_or_else(|| {
            if rect.width() > 640.0 {
                localizer.t("canvas.hover_hint").to_string()
            } else {
                "X = -- m  |  Y = -- m".to_string()
            }
        });

    let mode_tag = if state.auto_fit {
        localizer.t("canvas.autofit_on")
    } else {
        localizer.t("canvas.autofit_manual")
    };

    painter.text(
        Pos2::new(strip.min.x + 8.0, strip.center().y),
        Align2::LEFT_CENTER,
        format!(
            "{}   •   {}: {:.1} px/m   •   {}",
            pointer_info,
            localizer.pick("Escala", "Scale"),
            state.zoom,
            mode_tag
        ),
        FontId::monospace(10.5),
        GeoEstabilTheme::text_secondary(dark_mode),
    );
}
