use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::core::types::SlopeAnalysisMethod;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::engine::wall::bearing_capacity::calculate_bearing_capacity;
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::stability::check_wall_external_stability;
use crate::locale::translations::Language;
use crate::ui::theme::GeoEstabilTheme;
use std::fmt::Write as _;

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScriptKind {
    Normal,
    Sub,
    Sup,
}

fn token_bounds_around(chars: &[char], idx: usize) -> (usize, usize) {
    let mut s = idx;
    while s > 0 && !chars[s - 1].is_whitespace() {
        s -= 1;
    }
    let mut e = idx;
    while e < chars.len() && !chars[e].is_whitespace() {
        e += 1;
    }
    (s, e)
}

fn is_non_math_underscore(chars: &[char], idx: usize) -> bool {
    // Allow braced subscripts unconditionally (e.g., X_{foo_bar})
    if idx + 1 < chars.len() && chars[idx + 1] == '{' {
        return false;
    }
    let (s, e) = token_bounds_around(chars, idx);
    let token: String = chars[s..e].iter().collect();
    let lower = token.to_ascii_lowercase();
    if lower.contains("://")
        || lower.starts_with("www.")
        || lower.contains('@')
        || lower.contains('\\')
        || lower.contains('/')
        || [
            ".rs", ".md", ".txt", ".json", ".pdf", ".html", ".htm", ".svg", ".dxf", ".exe", ".iss",
            ".toml",
        ]
        .iter()
        .any(|ext| lower.contains(ext))
    {
        return true;
    }
    // Two underscores inside a plain unbraced token (e.g., snake_case_ident) indicate code/identifier, not math subscript
    if token.matches('_').count() > 1 {
        return true;
    }
    false
}

/// Tokeniza expressões matemáticas (incluindo índices `_` estilo `V_base`, `_{...}` e expoentes `^`)
/// para gerar `<tspan>` SVG / `<sub>`/`<sup>` HTML / sub-sobrescritos PDF sem `_`, `{`, `}` ou `^` literais.
pub(crate) fn split_math_runs(text: &str) -> Vec<(String, ScriptKind)> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut runs: Vec<(String, ScriptKind)> = Vec::new();
    let mut buf = String::new();
    let mut i = 0usize;

    let flush_buf = |runs: &mut Vec<(String, ScriptKind)>, b: &mut String| {
        if !b.is_empty() {
            runs.push((std::mem::take(b), ScriptKind::Normal));
        }
    };

    while i < n {
        let ch = chars[i];
        if ch == '_'
            && i + 1 < n
            && (i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == ')' || chars[i - 1] == '\'' || chars[i - 1] == '′'))
            && !is_non_math_underscore(&chars, i)
        {
            if chars[i + 1] == '{' {
                if let Some(end_rel) = chars[i + 2..].iter().position(|&c| c == '}') {
                    flush_buf(&mut runs, &mut buf);
                    let inner: String = chars[i + 2..i + 2 + end_rel].iter().collect();
                    runs.push((inner, ScriptKind::Sub));
                    i = i + 2 + end_rel + 1;
                    continue;
                }
            } else {
                let mut end = i + 1;
                while end < n {
                    let c = chars[end];
                    if c.is_alphanumeric()
                        || (c == ',' && end + 1 < n && chars[end + 1].is_alphanumeric())
                        || c == '\''
                        || c == '′'
                    {
                        end += 1;
                    } else {
                        break;
                    }
                }
                if end > i + 1 {
                    flush_buf(&mut runs, &mut buf);
                    let inner: String = chars[i + 1..end].iter().collect();
                    runs.push((inner, ScriptKind::Sub));
                    i = end;
                    continue;
                }
            }
        }
        if ch == '^' && i + 1 < n {
            if chars[i + 1] == '{' {
                if let Some(end_rel) = chars[i + 2..].iter().position(|&c| c == '}') {
                    flush_buf(&mut runs, &mut buf);
                    let inner: String = chars[i + 2..i + 2 + end_rel].iter().collect();
                    runs.push((inner, ScriptKind::Sup));
                    i = i + 2 + end_rel + 1;
                    continue;
                }
            } else {
                let mut end = i + 1;
                while end < n {
                    let c = chars[end];
                    if c.is_alphanumeric() || (end == i + 1 && (c == '-' || c == '+')) {
                        end += 1;
                    } else {
                        break;
                    }
                }
                if end > i + 1 {
                    flush_buf(&mut runs, &mut buf);
                    let inner: String = chars[i + 1..end].iter().collect();
                    runs.push((inner, ScriptKind::Sup));
                    i = end;
                    continue;
                }
            }
        }
        if ch == '²' {
            flush_buf(&mut runs, &mut buf);
            runs.push(("2".to_string(), ScriptKind::Sup));
            i += 1;
            continue;
        }
        if ch == '³' {
            flush_buf(&mut runs, &mut buf);
            runs.push(("3".to_string(), ScriptKind::Sup));
            i += 1;
            continue;
        }
        buf.push(ch);
        i += 1;
    }
    flush_buf(&mut runs, &mut buf);
    runs
}

#[allow(clippy::too_many_arguments)]
fn push_svg_rich_text(
    svg: &mut String,
    x: f64,
    y: f64,
    font_size: f64,
    anchor: &str,
    fill: &str,
    bold: bool,
    expr: &str,
) {
    let weight = if bold { " font-weight=\"bold\"" } else { "" };
    let _ = write!(
        svg,
        "<text x=\"{x:.1}\" y=\"{y:.1}\" text-anchor=\"{anchor}\" fill=\"{fill}\" font-family=\"'Segoe UI', Arial, sans-serif\" font-size=\"{font_size:.1}\"{weight}>"
    );
    let sub_shift = (font_size * 0.28).max(2.5);
    let sup_shift = (font_size * 0.34).max(3.0);
    let sub_size = (font_size * 0.74).max(7.5);
    let mut cur_offset = 0.0_f64;

    for (txt, kind) in split_math_runs(expr) {
        let escaped = escape_xml(&txt);
        match kind {
            ScriptKind::Normal => {
                if cur_offset.abs() > 0.01 {
                    let _ = write!(svg, "<tspan dy=\"{:.1}\">{}</tspan>", -cur_offset, escaped);
                    cur_offset = 0.0;
                } else {
                    svg.push_str(&escaped);
                }
            }
            ScriptKind::Sub => {
                let dy = sub_shift - cur_offset;
                cur_offset = sub_shift;
                let _ = write!(
                    svg,
                    "<tspan dy=\"{dy:.1}\" font-size=\"{sub_size:.1}\">{escaped}</tspan>"
                );
            }
            ScriptKind::Sup => {
                let dy = -sup_shift - cur_offset;
                cur_offset = -sup_shift;
                let _ = write!(
                    svg,
                    "<tspan dy=\"{dy:.1}\" font-size=\"{sub_size:.1}\">{escaped}</tspan>"
                );
            }
        }
    }
    svg.push_str("</text>\n");
}

fn nice_grid_step(span: f64) -> f64 {
    let raw = (span / 6.0).max(0.25);
    let pow10 = 10.0_f64.powf(raw.log10().floor());
    let norm = raw / pow10;
    let step = if norm <= 1.5 {
        1.0
    } else if norm <= 3.5 {
        2.0
    } else if norm <= 7.0 {
        5.0
    } else {
        10.0
    };
    step * pow10
}

fn heatmap_hex(fs: f64) -> String {
    let c = GeoEstabilTheme::heatmap_color(fs);
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

pub struct VectorExporter;

impl VectorExporter {
    pub fn export_slope_svg(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
    ) -> String {
        Self::export_slope_svg_lang(project, slope_report, Language::PtBr)
    }

    pub fn export_slope_svg_lang(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> String {
        Self::render_slope_svg_doc(project, slope_report, lang, false)
    }

    /// Gera SVG embutível (responsivo) para a Aba 2 — Estabilidade de Taludes (sem cabeçalho XML extra).
    pub fn render_inline_slope_svg(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> String {
        Self::render_slope_svg_doc(project, slope_report, lang, true)
    }

    fn render_slope_svg_doc(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
        responsive_inline: bool,
    ) -> String {
        let width = 1000.0_f64;
        let height = 580.0_f64;
        let mut svg = String::with_capacity(24_000);
        if responsive_inline {
            let _ = writeln!(
                svg,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.0}\" style=\"width:100%;height:auto;max-width:960px;border:1px solid #cbd5e1;border-radius:6px;background:#ffffff;\">"
            );
        } else {
            let _ = writeln!(
                svg,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.0}\" width=\"{width:.0}\" height=\"{height:.0}\">"
            );
        }
        Self::append_slope_panel_contents(&mut svg, project, slope_report, lang, 0.0, 0.0, width, height);
        svg.push_str("</svg>\n");
        svg
    }

    #[allow(clippy::too_many_arguments)]
    fn append_slope_panel_contents(
        svg: &mut String,
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
        ox: f64,
        oy: f64,
        panel_w: f64,
        panel_h: f64,
    ) {
        let pt = lang == Language::PtBr;
        let slope = &project.slope_model;
        let terrain = &slope.terrain_surface;

        let mut x_min = terrain.x_min().min(0.0);
        let mut x_max = terrain.x_max().max(40.0);
        let mut y_min = terrain
            .y_min()
            .min(slope.layers.iter().map(|l| l.top_profile.y_min()).fold(0.0_f64, f64::min) - 3.5);
        let mut y_max = terrain.y_max().max(12.0) + 3.0;
        if let Some(rep) = slope_report {
            y_max = y_max.max(rep.critical_circle.center_y + 2.5);
            y_min = y_min.min(rep.critical_circle.center_y - rep.critical_circle.radius - 1.0);
            x_min = x_min.min(rep.critical_circle.x_entry - 2.0);
            x_max = x_max.max(rep.critical_circle.x_exit + 2.0);
        }

        let plot_x0 = ox + 58.0;
        let plot_y0 = oy + 52.0;
        let plot_w = panel_w - 86.0;
        let plot_h = panel_h - 94.0;

        let sx = plot_w / (x_max - x_min).max(1.0);
        let sy = plot_h / (y_max - y_min).max(1.0);
        let scale = sx.min(sy);
        let content_w = (x_max - x_min) * scale;
        let content_h = (y_max - y_min) * scale;
        let pad_x = plot_x0 + (plot_w - content_w) * 0.5;
        let pad_y = plot_y0 + (plot_h - content_h) * 0.5;

        let to_s = |wx: f64, wy: f64| -> (f64, f64) {
            (pad_x + (wx - x_min) * scale, pad_y + content_h - (wy - y_min) * scale)
        };

        let clip_id = format!("clip_slope_{:.0}_{:.0}", ox, oy);
        let _ = writeln!(
            svg,
            "<defs><clipPath id=\"{clip_id}\"><rect x=\"{plot_x0:.1}\" y=\"{plot_y0:.1}\" width=\"{plot_w:.1}\" height=\"{plot_h:.1}\"/></clipPath></defs>"
        );
        let _ = writeln!(
            svg,
            "<rect x=\"{ox:.1}\" y=\"{oy:.1}\" width=\"{panel_w:.1}\" height=\"{panel_h:.1}\" fill=\"#ffffff\" stroke=\"#cbd5e1\" stroke-width=\"1\"/>"
        );
        let _ = writeln!(
            svg,
            "<rect x=\"{plot_x0:.1}\" y=\"{plot_y0:.1}\" width=\"{plot_w:.1}\" height=\"{plot_h:.1}\" fill=\"#f8fafc\" stroke=\"#cbd5e1\" stroke-width=\"1\"/>"
        );

        let panel_title = if pt {
            format!("GeoEstabil — Aba 2: Estabilidade de Taludes (ABNT NBR 11682:2009) — {}", project.title)
        } else {
            format!("GeoEstabil — Tab 2: Slope Stability Analysis (ABNT NBR 11682:2009) — {}", project.title)
        };
        push_svg_rich_text(svg, ox + 16.0, oy + 26.0, 14.0, "start", "#1b365d", true, &panel_title);

        let _ = writeln!(svg, "<g clip-path=\"url(#{clip_id})\">");

        // Grade métrica + eixos
        let step_x = nice_grid_step(x_max - x_min);
        let mut gx = (x_min / step_x).floor() * step_x;
        while gx <= x_max + 0.001 {
            let (sx_pos, _) = to_s(gx, y_min);
            if sx_pos >= plot_x0 && sx_pos <= plot_x0 + plot_w {
                let _ = writeln!(
                    svg,
                    "<line x1=\"{sx_pos:.1}\" y1=\"{plot_y0:.1}\" x2=\"{sx_pos:.1}\" y2=\"{:.1}\" stroke=\"#e2e8f0\" stroke-width=\"1\"/>",
                    plot_y0 + plot_h
                );
            }
            gx += step_x;
        }
        let step_y = nice_grid_step(y_max - y_min);
        let mut gy = (y_min / step_y).floor() * step_y;
        while gy <= y_max + 0.001 {
            let (_, sy_pos) = to_s(x_min, gy);
            if sy_pos >= plot_y0 && sy_pos <= plot_y0 + plot_h {
                let _ = writeln!(
                    svg,
                    "<line x1=\"{plot_x0:.1}\" y1=\"{sy_pos:.1}\" x2=\"{:.1}\" y2=\"{sy_pos:.1}\" stroke=\"#e2e8f0\" stroke-width=\"1\"/>",
                    plot_x0 + plot_w
                );
            }
            gy += step_y;
        }

        // Camadas estratigráficas preenchidas
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
            let upper_profile = if idx == 0 { terrain } else { &layer.top_profile };
            let lower_profile_opt = slope.layers.get(idx + 1).map(|l| &l.top_profile);
            let mut xs: Vec<f64> = vec![x_start, x_end];
            xs.extend(terrain.points.iter().map(|p| p.x));
            xs.extend(upper_profile.points.iter().map(|p| p.x));
            if let Some(lp) = lower_profile_opt {
                xs.extend(lp.points.iter().map(|p| p.x));
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            xs.dedup_by(|a, b| (*a - *b).abs() < 1e-4);

            let hex_fill = format!("#{:02x}{:02x}{:02x}", layer.color_rgba[0], layer.color_rgba[1], layer.color_rgba[2]);
            for pair in xs.windows(2) {
                let (xa, xb) = (pair[0], pair[1]);
                if xb - xa < 1e-4 {
                    continue;
                }
                let yt_a = terrain.interpolate_y(xa).unwrap_or(0.0).min(upper_profile.interpolate_y(xa).unwrap_or(f64::INFINITY));
                let yt_b = terrain.interpolate_y(xb).unwrap_or(0.0).min(upper_profile.interpolate_y(xb).unwrap_or(f64::INFINITY));
                let yb_a = lower_profile_opt.and_then(|lp| lp.interpolate_y(xa)).unwrap_or(deep_floor_y).min(yt_a);
                let yb_b = lower_profile_opt.and_then(|lp| lp.interpolate_y(xb)).unwrap_or(deep_floor_y).min(yt_b);
                if yt_a > yb_a + 1e-3 || yt_b > yb_b + 1e-3 {
                    let (p1, p2, p3, p4) = (to_s(xa, yt_a), to_s(xb, yt_b), to_s(xb, yb_b), to_s(xa, yb_a));
                    let _ = writeln!(
                        svg,
                        "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{hex_fill}\" fill-opacity=\"0.34\" stroke=\"none\"/>",
                        p1.0, p1.1, p2.0, p2.1, p3.0, p3.1, p4.0, p4.1
                    );
                }
            }
            if layer.top_profile.points.len() >= 2 {
                let pts: String = layer
                    .top_profile
                    .points
                    .iter()
                    .map(|p| {
                        let s = to_s(p.x, p.y);
                        format!("{:.1},{:.1}", s.0, s.1)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let _ = writeln!(
                    svg,
                    "<polyline points=\"{pts}\" fill=\"none\" stroke=\"{hex_fill}\" stroke-width=\"1.8\"/>"
                );
            }
        }

        // Lençol freático + marcador N.A. / W.T.
        if slope.water_table.enabled && slope.water_table.phreatic_line.points.len() >= 2 {
            let wt = &slope.water_table.phreatic_line;
            let pts: String = wt
                .points
                .iter()
                .map(|p| {
                    let s = to_s(p.x, p.y);
                    format!("{:.1},{:.1}", s.0, s.1)
                })
                .collect::<Vec<_>>()
                .join(" ");
            let _ = writeln!(
                svg,
                "<polyline points=\"{pts}\" fill=\"none\" stroke=\"#0284c7\" stroke-width=\"2.2\" stroke-dasharray=\"7,4\"/>"
            );
            let mid_x = 0.5 * (wt.x_min() + wt.x_max());
            if let Some(mid_y) = wt.interpolate_y(mid_x) {
                let na = to_s(mid_x, mid_y);
                let _ = writeln!(
                    svg,
                    "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#0284c7\"/>",
                    na.0 - 5.5,
                    na.1 - 8.5,
                    na.0 + 5.5,
                    na.1 - 8.5,
                    na.0,
                    na.1
                );
                let wt_lbl = if pt { "N.A." } else { "W.T." };
                push_svg_rich_text(svg, na.0 + 9.0, na.1 - 3.0, 10.5, "start", "#0284c7", true, wt_lbl);
            }
        }

        // Perfil do Terreno + vértices P1, P2...
        if terrain.points.len() >= 2 {
            let pts: String = terrain
                .points
                .iter()
                .map(|p| {
                    let s = to_s(p.x, p.y);
                    format!("{:.1},{:.1}", s.0, s.1)
                })
                .collect::<Vec<_>>()
                .join(" ");
            let _ = writeln!(
                svg,
                "<polyline points=\"{pts}\" fill=\"none\" stroke=\"#0f172a\" stroke-width=\"2.8\"/>"
            );
            for (i, p) in terrain.points.iter().enumerate() {
                let sp = to_s(p.x, p.y);
                let _ = writeln!(
                    svg,
                    "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3.8\" fill=\"#a16207\"/>",
                    sp.0, sp.1
                );
                push_svg_rich_text(
                    svg,
                    sp.0,
                    sp.1 - 7.0,
                    9.5,
                    "middle",
                    "#475569",
                    false,
                    &format!("P{}({:.0};{:.0})", i + 1, p.x, p.y),
                );
            }
        }

        // Sobrecarga distribuída q
        if slope.surcharge.enabled {
            let sur = &slope.surcharge;
            let span = (sur.x_end - sur.x_start).max(0.5);
            let arrow_count = ((span / 2.2).round() as usize).clamp(3, 12);
            let arrow_h_world = (24.0 / scale.max(1.0)).clamp(0.4, 2.2);
            let mut top_pts = Vec::new();
            for i in 0..=arrow_count {
                let wx = sur.x_start + span * (i as f64 / arrow_count as f64);
                if let Some(yt) = terrain.interpolate_y(wx) {
                    let pt_top = to_s(wx, yt + arrow_h_world);
                    let pt_bot = to_s(wx, yt);
                    top_pts.push(pt_top);
                    let _ = writeln!(
                        svg,
                        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#dc2626\" stroke-width=\"1.6\"/>",
                        pt_top.0, pt_top.1, pt_bot.0, pt_bot.1 - 2.0
                    );
                    let _ = writeln!(
                        svg,
                        "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#dc2626\"/>",
                        pt_bot.0, pt_bot.1, pt_bot.0 - 3.0, pt_bot.1 - 6.5, pt_bot.0 + 3.0, pt_bot.1 - 6.5
                    );
                }
            }
            if top_pts.len() >= 2 {
                let s_pts: String = top_pts.iter().map(|p| format!("{:.1},{:.1}", p.0, p.1)).collect::<Vec<_>>().join(" ");
                let _ = writeln!(svg, "<polyline points=\"{s_pts}\" fill=\"none\" stroke=\"#dc2626\" stroke-width=\"1.8\"/>");
            }
            let x_mid = 0.5 * (sur.x_start + sur.x_end);
            let y_mid = terrain.interpolate_y(x_mid).unwrap_or(0.0);
            let lbl = to_s(x_mid, y_mid + arrow_h_world);
            push_svg_rich_text(
                svg,
                lbl.0,
                lbl.1 - 5.0,
                11.0,
                "middle",
                "#dc2626",
                true,
                &format!("q = {:.1} kPa ({})", sur.q_kpa, sur.name),
            );
        }

        // Heatmap FS dos centros de ruptura
        if let Some(rep) = slope_report {
            if let Some(ref hm) = rep.heatmap {
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
                let cell_w = (dx_w * scale * 0.88).clamp(5.0, 26.0);
                let cell_h = (dy_w * scale * 0.88).clamp(5.0, 26.0);
                for pt_hm in &hm.points {
                    if pt_hm.min_fs < 50.0 {
                        let sc = to_s(pt_hm.center_x, pt_hm.center_y);
                        let hex = heatmap_hex(pt_hm.min_fs);
                        let _ = writeln!(
                            svg,
                            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{cell_w:.1}\" height=\"{cell_h:.1}\" rx=\"2\" fill=\"{hex}\" fill-opacity=\"0.42\"/>",
                            sc.0 - cell_w * 0.5,
                            sc.1 - cell_h * 0.5
                        );
                        let _ = writeln!(
                            svg,
                            "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2.2\" fill=\"{hex}\"/>",
                            sc.0, sc.1
                        );
                    }
                }
            }

            // Fatias verticais dentro da massa instável
            let circ = &rep.critical_circle;
            for sl in &circ.slices {
                let hw = sl.width * 0.5;
                let y_tl = terrain.interpolate_y(sl.x_mid - hw).unwrap_or(sl.y_top);
                let y_tr = terrain.interpolate_y(sl.x_mid + hw).unwrap_or(sl.y_top);
                let dy_l = circ.radius.powi(2) - (sl.x_mid - hw - circ.center_x).powi(2);
                let dy_r = circ.radius.powi(2) - (sl.x_mid + hw - circ.center_x).powi(2);
                let y_bl = if dy_l >= 0.0 { (circ.center_y - dy_l.sqrt()).min(y_tl) } else { sl.y_base };
                let y_br = if dy_r >= 0.0 { (circ.center_y - dy_r.sqrt()).min(y_tr) } else { sl.y_base };
                if y_tl >= y_bl && y_tr >= y_br {
                    let (q1, q2, q3, q4) = (to_s(sl.x_mid - hw, y_tl), to_s(sl.x_mid + hw, y_tr), to_s(sl.x_mid + hw, y_br), to_s(sl.x_mid - hw, y_bl));
                    let _ = writeln!(
                        svg,
                        "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#3b82f6\" fill-opacity=\"0.12\" stroke=\"#94a3b8\" stroke-width=\"0.7\"/>",
                        q1.0, q1.1, q2.0, q2.1, q3.0, q3.1, q4.0, q4.1
                    );
                }
            }

            // Arco circular crítico (entre entrada e saída)
            let cs = to_s(circ.center_x, circ.center_y);
            let y_entry = terrain.interpolate_y(circ.x_entry).unwrap_or(circ.center_y);
            let y_exit = terrain.interpolate_y(circ.x_exit).unwrap_or(circ.center_y);
            let ang_start = (y_entry - circ.center_y).atan2(circ.x_entry - circ.center_x);
            let ang_end = (y_exit - circ.center_y).atan2(circ.x_exit - circ.center_x);
            let mut sweep = ang_end - ang_start;
            if sweep > std::f64::consts::PI {
                sweep -= std::f64::consts::TAU;
            } else if sweep < -std::f64::consts::PI {
                sweep += std::f64::consts::TAU;
            }
            let arc_pts: String = (0..=64)
                .map(|i| {
                    let th = ang_start + sweep * (i as f64 / 64.0);
                    let p = to_s(circ.center_x + circ.radius * th.cos(), circ.center_y + circ.radius * th.sin());
                    format!("{:.1},{:.1}", p.0, p.1)
                })
                .collect::<Vec<_>>()
                .join(" ");
            let arc_col = if rep.is_stable { "#16a34a" } else { "#dc2626" };
            let pe = to_s(circ.x_entry, y_entry);
            let px = to_s(circ.x_exit, y_exit);
            let _ = writeln!(
                svg,
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{arc_col}\" stroke-opacity=\"0.45\" stroke-width=\"1\"/>",
                cs.0, cs.1, pe.0, pe.1
            );
            let _ = writeln!(
                svg,
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{arc_col}\" stroke-opacity=\"0.45\" stroke-width=\"1\"/>",
                cs.0, cs.1, px.0, px.1
            );
            let _ = writeln!(svg, "<polyline points=\"{arc_pts}\" fill=\"none\" stroke=\"{arc_col}\" stroke-width=\"3\"/>");
            let _ = writeln!(svg, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{arc_col}\"/>", pe.0, pe.1);
            let _ = writeln!(svg, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{arc_col}\"/>", px.0, px.1);
            let _ = writeln!(
                svg,
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{arc_col}\" stroke-width=\"2\"/>",
                cs.0 - 7.0, cs.1, cs.0 + 7.0, cs.1
            );
            let _ = writeln!(
                svg,
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{arc_col}\" stroke-width=\"2\"/>",
                cs.0, cs.1 - 7.0, cs.0, cs.1 + 7.0
            );

            let (m_lbl, m_fs) = match slope.method {
                SlopeAnalysisMethod::BishopSimplified => ("Bishop", circ.fs_bishop),
                SlopeAnalysisMethod::FelleniusOrdinary => ("Fellenius", circ.fs_fellenius),
                SlopeAnalysisMethod::JanbuSimplified => ("Janbu", circ.fs_janbu),
            };
            let st_txt = if rep.is_stable {
                if pt { "ESTÁVEL" } else { "STABLE" }
            } else if pt {
                "INSTÁVEL"
            } else {
                "UNSTABLE"
            };
            let tx = (cs.0 + 12.0).clamp(plot_x0 + 10.0, plot_x0 + plot_w - 220.0);
            let ty = (cs.1 - 12.0).clamp(plot_y0 + 30.0, plot_y0 + plot_h - 10.0);
            push_svg_rich_text(
                svg,
                tx,
                ty - 13.0,
                11.0,
                "start",
                arc_col,
                true,
                &format!("{} ({:.1}; {:.1}) R = {:.1} m", if pt { "Centro" } else { "Center" }, circ.center_x, circ.center_y, circ.radius),
            );
            push_svg_rich_text(
                svg,
                tx,
                ty,
                11.5,
                "start",
                arc_col,
                true,
                &format!("FS_{{{m_lbl}}} = {m_fs:.3} [{st_txt}]"),
            );
        }

        let _ = writeln!(svg, "</g>");

        // Rótulos numéricos dos eixos fora do recorte
        let mut tick_x = (x_min / step_x).ceil() * step_x;
        while tick_x <= x_max + 0.001 {
            let (sx_pos, _) = to_s(tick_x, y_min);
            if sx_pos >= plot_x0 + 12.0 && sx_pos <= plot_x0 + plot_w - 12.0 {
                push_svg_rich_text(
                    svg,
                    sx_pos,
                    plot_y0 + plot_h + 15.0,
                    9.5,
                    "middle",
                    "#64748b",
                    false,
                    &format!("{:.0} m", tick_x),
                );
            }
            tick_x += step_x;
        }
        let mut tick_y = (y_min / step_y).ceil() * step_y;
        while tick_y <= y_max + 0.001 {
            let (_, sy_pos) = to_s(x_min, tick_y);
            if sy_pos >= plot_y0 + 10.0 && sy_pos <= plot_y0 + plot_h - 8.0 {
                push_svg_rich_text(
                    svg,
                    plot_x0 - 6.0,
                    sy_pos + 3.0,
                    9.5,
                    "end",
                    "#64748b",
                    false,
                    &format!("{:.0} m", tick_y),
                );
            }
            tick_y += step_y;
        }

        // Legenda de Camadas (Superior Esquerdo)
        if !slope.layers.is_empty() {
            let rows = slope.layers.len().min(4);
            let lw = 310.0;
            let lh = 26.0 + (rows as f64) * 17.0;
            let lx = plot_x0 + 10.0;
            let ly = plot_y0 + 10.0;
            let _ = writeln!(
                svg,
                "<rect x=\"{lx:.1}\" y=\"{ly:.1}\" width=\"{lw:.1}\" height=\"{lh:.1}\" rx=\"5\" fill=\"#ffffff\" fill-opacity=\"0.92\" stroke=\"#cbd5e1\"/>"
            );
            push_svg_rich_text(
                svg,
                lx + 8.0,
                ly + 15.0,
                10.5,
                "start",
                "#0f172a",
                true,
                if pt { "Estratigrafia (Mohr-Coulomb):" } else { "Stratigraphy (Mohr-Coulomb):" },
            );
            for (i, layer) in slope.layers.iter().take(4).enumerate() {
                let row_y = ly + 31.0 + (i as f64) * 17.0;
                let hex = format!("#{:02x}{:02x}{:02x}", layer.color_rgba[0], layer.color_rgba[1], layer.color_rgba[2]);
                let _ = writeln!(
                    svg,
                    "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"9\" height=\"9\" rx=\"2\" fill=\"{hex}\"/>",
                    lx + 8.0,
                    row_y - 8.0
                );
                let short_name = if layer.name.chars().count() > 22 {
                    format!("{}...", layer.name.chars().take(21).collect::<String>().trim_end())
                } else {
                    layer.name.clone()
                };
                push_svg_rich_text(
                    svg,
                    lx + 22.0,
                    row_y,
                    9.5,
                    "start",
                    "#334155",
                    false,
                    &format!("{short_name}: c'={:.0} kPa, φ'={:.0}°, γ={:.1} kN/m³", layer.cohesion, layer.friction_angle_deg, layer.gamma),
                );
            }
        }

        // Legenda Heatmap (Superior Direito)
        let hm_w = 175.0;
        let hm_h = 74.0;
        let hx = plot_x0 + plot_w - hm_w - 10.0;
        let hy = plot_y0 + 10.0;
        let _ = writeln!(
            svg,
            "<rect x=\"{hx:.1}\" y=\"{hy:.1}\" width=\"{hm_w:.1}\" height=\"{hm_h:.1}\" rx=\"5\" fill=\"#ffffff\" fill-opacity=\"0.92\" stroke=\"#cbd5e1\"/>"
        );
        push_svg_rich_text(
            svg,
            hx + 8.0,
            hy + 15.0,
            10.5,
            "start",
            "#0f172a",
            true,
            if pt { "Heatmap FS (Centros)" } else { "FS Heatmap (Centers)" },
        );
        let bar_w = hm_w - 16.0;
        let segs = 20;
        for i in 0..segs {
            let fs_sample = 0.8 + (i as f64 / (segs - 1) as f64) * 1.4;
            let hex = heatmap_hex(fs_sample);
            let _ = writeln!(
                svg,
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"10\" fill=\"{hex}\"/>",
                hx + 8.0 + (i as f64) * (bar_w / segs as f64),
                hy + 22.0,
                bar_w / segs as f64 + 0.6
            );
        }
        push_svg_rich_text(svg, hx + 8.0, hy + 43.0, 9.0, "start", "#475569", false, "≤1.0      1.3      1.5      ≥2.0");
        if let Some(rep) = slope_report {
            let col = if rep.is_stable { "#16a34a" } else { "#dc2626" };
            push_svg_rich_text(
                svg,
                hx + 8.0,
                hy + 61.0,
                10.5,
                "start",
                col,
                true,
                &format!(
                    "{} = {:.3} ({:.1} ms)",
                    if pt { "FS_{mín}" } else { "FS_{min}" },
                    rep.critical_circle.fs_bishop,
                    rep.elapsed_ms
                ),
            );
        } else {
            push_svg_rich_text(
                svg,
                hx + 8.0,
                hy + 61.0,
                9.2,
                "start",
                "#d97706",
                true,
                if pt { "Aguardando varredura Aba 2" } else { "Run Tab 2 sweep for FS" },
            );
        }
    }

    pub fn export_retaining_wall_svg_lang(project: &GeoProject, lang: Language) -> String {
        Self::render_wall_svg_doc(project, lang, false)
    }

    /// Gera SVG embutível (responsivo) para a Aba 3 — Muro de Contenção (sem cabeçalho XML extra).
    pub fn render_inline_wall_svg(project: &GeoProject, lang: Language) -> String {
        Self::render_wall_svg_doc(project, lang, true)
    }

    fn render_wall_svg_doc(
        project: &GeoProject,
        lang: Language,
        responsive_inline: bool,
    ) -> String {
        let width = 1000.0_f64;
        let height = 580.0_f64;
        let mut svg = String::with_capacity(24_000);
        if responsive_inline {
            let _ = writeln!(
                svg,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.0}\" style=\"width:100%;height:auto;max-width:960px;border:1px solid #cbd5e1;border-radius:6px;background:#ffffff;\">"
            );
        } else {
            let _ = writeln!(
                svg,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.0}\" width=\"{width:.0}\" height=\"{height:.0}\">"
            );
        }
        Self::append_wall_panel_contents(&mut svg, project, lang, 0.0, 0.0, width, height);
        svg.push_str("</svg>\n");
        svg
    }

    #[allow(clippy::too_many_arguments)]
    fn append_wall_panel_contents(
        svg: &mut String,
        project: &GeoProject,
        lang: Language,
        ox: f64,
        oy: f64,
        panel_w: f64,
        panel_h: f64,
    ) {
        let pt = lang == Language::PtBr;
        let model = &project.wall_model;
        let g = &model.geometry;
        let is_gravity = model.wall_type != crate::core::types::RetainingWallType::Cantilever;
        let h_total = g.total_height().max(1.0);
        let b_base = g.base_width.max(1.0);
        let h_base = if is_gravity { 0.0 } else { g.base_thickness };
        let b_toe = if is_gravity { 0.0 } else { g.toe_length };
        let b_stem_base = g.stem_base_width;
        let b_stem_top = g.stem_top_width;
        let slope_ext = 4.6_f64;
        let slope_rad = g.backfill_slope_deg.to_radians();
        let slope_rise = slope_ext * slope_rad.tan();

        let stability = check_wall_external_stability(model);
        let bearing = calculate_bearing_capacity(model, &stability);
        let pressures = calculate_earth_pressures(model);

        let x_min = -1.6_f64;
        let x_max = b_base + slope_ext + 1.2;
        let y_min = -1.75_f64;
        let y_max = h_total + slope_rise.max(0.8) + 1.05;

        let plot_x0 = ox + 58.0;
        let plot_y0 = oy + 52.0;
        let plot_w = panel_w - 86.0;
        let plot_h = panel_h - 94.0;

        let scale = (plot_w / (x_max - x_min)).min(plot_h / (y_max - y_min));
        let cw = (x_max - x_min) * scale;
        let ch = (y_max - y_min) * scale;
        let px0 = plot_x0 + (plot_w - cw) * 0.5;
        let py0 = plot_y0 + (plot_h - ch) * 0.5;
        let to_s = |wx: f64, wy: f64| -> (f64, f64) { (px0 + (wx - x_min) * scale, py0 + ch - (wy - y_min) * scale) };

        let clip_id = format!("clip_wall_{:.0}_{:.0}", ox, oy);
        let _ = writeln!(
            svg,
            "<defs><clipPath id=\"{clip_id}\"><rect x=\"{plot_x0:.1}\" y=\"{plot_y0:.1}\" width=\"{plot_w:.1}\" height=\"{plot_h:.1}\"/></clipPath></defs>"
        );
        let _ = writeln!(
            svg,
            "<rect x=\"{ox:.1}\" y=\"{oy:.1}\" width=\"{panel_w:.1}\" height=\"{panel_h:.1}\" fill=\"#ffffff\" stroke=\"#cbd5e1\" stroke-width=\"1\"/>"
        );
        let _ = writeln!(
            svg,
            "<rect x=\"{plot_x0:.1}\" y=\"{plot_y0:.1}\" width=\"{plot_w:.1}\" height=\"{plot_h:.1}\" fill=\"#f8fafc\" stroke=\"#cbd5e1\" stroke-width=\"1\"/>"
        );

        let wall_tag = model.wall_type.name(lang);
        let header = if pt {
            format!("GeoEstabil — Aba 3: Muro de Contenção ({wall_tag}, ABNT NBR 6122:2022) — {}", project.title)
        } else {
            format!("GeoEstabil — Tab 3: Retaining Wall ({wall_tag}, ABNT NBR 6122:2022) — {}", project.title)
        };
        push_svg_rich_text(svg, ox + 16.0, oy + 26.0, 14.0, "start", "#1b365d", true, &header);

        let _ = writeln!(svg, "<g clip-path=\"url(#{clip_id})\">");

        // Grade métrica
        let step_x = nice_grid_step(x_max - x_min);
        let mut gx = (x_min / step_x).floor() * step_x;
        while gx <= x_max + 0.001 {
            let (sx_pos, _) = to_s(gx, y_min);
            let _ = writeln!(
                svg,
                "<line x1=\"{sx_pos:.1}\" y1=\"{plot_y0:.1}\" x2=\"{sx_pos:.1}\" y2=\"{:.1}\" stroke=\"#e2e8f0\" stroke-width=\"1\"/>",
                plot_y0 + plot_h
            );
            gx += step_x;
        }
        let step_y = nice_grid_step(y_max - y_min);
        let mut gy = (y_min / step_y).floor() * step_y;
        while gy <= y_max + 0.001 {
            let (_, sy_pos) = to_s(x_min, gy);
            let _ = writeln!(
                svg,
                "<line x1=\"{plot_x0:.1}\" y1=\"{sy_pos:.1}\" x2=\"{:.1}\" y2=\"{sy_pos:.1}\" stroke=\"#e2e8f0\" stroke-width=\"1\"/>",
                plot_x0 + plot_w
            );
            gy += step_y;
        }

        // 1. Solo de fundação abaixo da base
        let (f1, f2, f3, f4) = (to_s(-1.5, 0.0), to_s(b_base + slope_ext + 1.1, 0.0), to_s(b_base + slope_ext + 1.1, -1.6), to_s(-1.5, -1.6));
        let _ = writeln!(
            svg,
            "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#785032\" fill-opacity=\"0.24\"/>",
            f1.0, f1.1, f2.0, f2.1, f3.0, f3.1, f4.0, f4.1
        );
        let _ = writeln!(
            svg,
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#a16207\" stroke-width=\"1.5\"/>",
            f1.0, f1.1, f2.0, f2.1
        );

        // 2. Embutimento frontal sobre a ponta (D_f)
        if !is_gravity && h_base > 0.01 && b_toe > 0.01 {
            let (e1, e2, e3, e4) = (to_s(-1.5, h_base), to_s(b_toe, h_base), to_s(b_toe, 0.0), to_s(-1.5, 0.0));
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#a16207\" fill-opacity=\"0.28\"/>",
                e1.0, e1.1, e2.0, e2.1, e3.0, e3.1, e4.0, e4.1
            );
            let _ = writeln!(
                svg,
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#a16207\" stroke-width=\"1.8\"/>",
                e1.0, e1.1, e2.0, e2.1
            );
        }

        // 3. Reaterro no tardoz + superfície inclinada beta
        let x_wall_back_bot = if is_gravity { b_base } else { b_toe + b_stem_base };
        let x_wall_back_top = if is_gravity { b_stem_top } else { b_toe + b_stem_top };
        let (r1, r2, r3, r4) = (
            to_s(x_wall_back_top, h_total),
            to_s(b_base, h_total),
            to_s(b_base, h_base),
            to_s(x_wall_back_bot, h_base),
        );
        let _ = writeln!(
            svg,
            "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#d99e5d\" fill-opacity=\"0.30\"/>",
            r1.0, r1.1, r2.0, r2.1, r3.0, r3.1, r4.0, r4.1
        );
        let (m1, m2, m3, m4) = (
            to_s(b_base, h_total),
            to_s(b_base + slope_ext, h_total + slope_rise),
            to_s(b_base + slope_ext, 0.0),
            to_s(b_base, 0.0),
        );
        let _ = writeln!(
            svg,
            "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#d99e5d\" fill-opacity=\"0.30\"/>",
            m1.0, m1.1, m2.0, m2.1, m3.0, m3.1, m4.0, m4.1
        );
        let _ = writeln!(
            svg,
            "<polyline points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"none\" stroke=\"#a16207\" stroke-width=\"2.2\"/>",
            r1.0, r1.1, r2.0, r2.1, m2.0, m2.1
        );

        // 4. Nível d'Água no tardoz
        let hw = model.water_height_back.clamp(0.0, h_total);
        if hw > 0.01 {
            let x_wl = if is_gravity {
                let t = (hw / g.stem_height.max(0.1)).clamp(0.0, 1.0);
                b_base + t * (b_stem_top - b_base)
            } else if hw <= h_base {
                b_base
            } else {
                let t = ((hw - h_base) / g.stem_height.max(0.1)).clamp(0.0, 1.0);
                x_wall_back_bot + t * (x_wall_back_top - x_wall_back_bot)
            };
            let (w1, w2, w3, w4) = (
                to_s(x_wl, hw),
                to_s(b_base + slope_ext, hw),
                to_s(b_base + slope_ext, 0.0),
                to_s(b_base, 0.0),
            );
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#38bdf8\" fill-opacity=\"0.24\"/>",
                w1.0, w1.1, w2.0, w2.1, w3.0, w3.1, w4.0, w4.1
            );
            let _ = writeln!(
                svg,
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#0284c7\" stroke-width=\"2\" stroke-dasharray=\"6,4\"/>",
                w1.0, w1.1, w2.0, w2.1
            );
        }

        // 5. Sobrecarga no terrapleno q
        if model.surcharge_q > 0.1 {
            let arrows = 5;
            let mut line_pts = Vec::new();
            for i in 0..=arrows {
                let dx = (slope_ext - 0.3) * (i as f64 / arrows as f64) + 0.15;
                let gx = b_base + dx;
                let gy = h_total + dx * slope_rad.tan();
                let pt_top = to_s(gx, gy + 0.55);
                let pt_bot = to_s(gx, gy);
                line_pts.push(pt_top);
                let _ = writeln!(
                    svg,
                    "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#d97706\" stroke-width=\"1.5\"/>",
                    pt_top.0, pt_top.1, pt_bot.0, pt_bot.1 - 2.0
                );
                let _ = writeln!(
                    svg,
                    "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#d97706\"/>",
                    pt_bot.0, pt_bot.1, pt_bot.0 - 2.8, pt_bot.1 - 5.5, pt_bot.0 + 2.8, pt_bot.1 - 5.5
                );
            }
            if line_pts.len() >= 2 {
                let first = line_pts.first().unwrap();
                let last = line_pts.last().unwrap();
                let _ = writeln!(
                    svg,
                    "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#d97706\" stroke-width=\"1.8\"/>",
                    first.0, first.1, last.0, last.1
                );
            }
            let pl = to_s(b_base + slope_ext * 0.5, h_total + slope_rise * 0.5 + 0.65);
            push_svg_rich_text(
                svg,
                pl.0,
                pl.1,
                10.5,
                "middle",
                "#b45309",
                true,
                &format!("q = {:.1} kPa", model.surcharge_q),
            );
        }

        // 6. Corpo do muro + Dente de ancoragem
        if is_gravity {
            let (q1, q2, q3, q4) = (to_s(0.0, 0.0), to_s(b_base, 0.0), to_s(b_stem_top, h_total), to_s(0.0, h_total));
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#cbd5e1\" stroke=\"#0f172a\" stroke-width=\"2.2\"/>",
                q1.0, q1.1, q2.0, q2.1, q3.0, q3.1, q4.0, q4.1
            );
        } else {
            if h_base > 0.01 {
                let (s1, s2) = (to_s(0.0, h_base), to_s(b_base, 0.0));
                let _ = writeln!(
                    svg,
                    "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#cbd5e1\" stroke=\"#0f172a\" stroke-width=\"2.2\"/>",
                    s1.0, s1.1, s2.0 - s1.0, s2.1 - s1.1
                );
            }
            let (f1, f2, f3, f4) = (
                to_s(b_toe, h_base),
                to_s(b_toe + b_stem_base, h_base),
                to_s(b_toe + b_stem_top, h_total),
                to_s(b_toe, h_total),
            );
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#cbd5e1\" stroke=\"#0f172a\" stroke-width=\"2.2\"/>",
                f1.0, f1.1, f2.0, f2.1, f3.0, f3.1, f4.0, f4.1
            );
        }
        if g.shear_key_enabled && g.shear_key_depth > 0.02 && g.shear_key_width > 0.02 {
            let kx = if is_gravity { b_base * 0.5 } else { b_toe + b_stem_base * 0.5 };
            let (k1, k2) = (to_s(kx - g.shear_key_width * 0.5, 0.0), to_s(kx + g.shear_key_width * 0.5, -g.shear_key_depth));
            let _ = writeln!(
                svg,
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#94a3b8\" stroke=\"#0f172a\" stroke-width=\"2\"/>",
                k1.0, k1.1, k2.0 - k1.0, k2.1 - k1.1
            );
        }

        // 7. Diagramas de empuxo lateral & pressão hidrostática
        let ka = pressures.ka;
        let sigma_q = model.surcharge_q * ka;
        let sigma_soil_bot = model.backfill_soil.gamma * h_total * ka;
        let sigma_max_lat = (sigma_q + sigma_soil_bot + 9.81 * hw).max(1.0);
        let d_scale = 1.45 / sigma_max_lat;
        let x_ref = b_base + 0.15;
        let w_top = sigma_q * d_scale;
        let w_bot = ((sigma_q + sigma_soil_bot) * d_scale).max(0.05);
        let (a1, a2, a3, a4) = (to_s(x_ref, h_total), to_s(x_ref + w_top, h_total), to_s(x_ref + w_bot, 0.0), to_s(x_ref, 0.0));
        let _ = writeln!(
            svg,
            "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#ef4444\" fill-opacity=\"0.22\" stroke=\"#dc2626\" stroke-width=\"1.5\"/>",
            a1.0, a1.1, a2.0, a2.1, a3.0, a3.1, a4.0, a4.1
        );
        if hw > 0.05 {
            let ww = (9.81 * hw * d_scale).max(0.05);
            let (hw1, hw2, hw3) = (to_s(x_ref + w_bot, hw), to_s(x_ref + w_bot + ww, 0.0), to_s(x_ref + w_bot, 0.0));
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#38bdf8\" fill-opacity=\"0.34\" stroke=\"#0284c7\" stroke-width=\"1.3\"/>",
                hw1.0, hw1.1, hw2.0, hw2.1, hw3.0, hw3.1
            );
        }

        // Vetor resultante Ea,h
        let pt_tgt = to_s(b_base, pressures.ea_arm_y);
        let pt_org = to_s(b_base + 2.15, pressures.ea_arm_y);
        let _ = writeln!(
            svg,
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#dc2626\" stroke-width=\"2.6\"/>",
            pt_org.0, pt_org.1, pt_tgt.0 + 2.0, pt_tgt.1
        );
        let _ = writeln!(
            svg,
            "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#dc2626\"/>",
            pt_tgt.0, pt_tgt.1, pt_tgt.0 + 8.5, pt_tgt.1 - 4.0, pt_tgt.0 + 8.5, pt_tgt.1 + 4.0
        );
        let eah_x = (pt_org.0 + 6.0).clamp(plot_x0 + 10.0, (plot_x0 + plot_w - 135.0).max(plot_x0 + 10.0));
        let eah_y = if pt_org.1 <= plot_y0 + 116.0 && eah_x >= plot_x0 + plot_w - 265.0 {
            plot_y0 + 124.0
        } else {
            pt_org.1 - 4.0
        };
        push_svg_rich_text(
            svg,
            eah_x,
            eah_y,
            10.8,
            "start",
            "#b91c1c",
            true,
            &format!("E_{{a,h}} = {:.1} kN/m", pressures.ea_horizontal),
        );
        push_svg_rich_text(
            svg,
            eah_x,
            eah_y + 15.0,
            9.5,
            "start",
            "#b91c1c",
            false,
            &format!("(y = {:.2} m)", pressures.ea_arm_y),
        );

        // Vetor passivo Ep
        if pressures.ep_horizontal > 0.5 && h_base > 0.05 {
            let ep_s = to_s(-1.05, h_base / 3.0);
            let ep_e = to_s(0.0, h_base / 3.0);
            let _ = writeln!(
                svg,
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#16a34a\" stroke-width=\"2.2\"/>",
                ep_s.0, ep_s.1, ep_e.0 - 2.0, ep_e.1
            );
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"#16a34a\"/>",
                ep_e.0, ep_e.1, ep_e.0 - 7.5, ep_e.1 - 3.5, ep_e.0 - 7.5, ep_e.1 + 3.5
            );
            let ep_tx = ep_s.0.max(plot_x0 + 82.0);
            push_svg_rich_text(
                svg,
                ep_tx - 4.0,
                ep_s.1 - 6.0,
                10.0,
                "end",
                "#15803d",
                true,
                &format!("E_p = {:.1} kN/m", pressures.ep_horizontal),
            );
        }

        // 8. Diagrama de tensões na base + marcador xR
        let max_sig = stability.sigma_toe_kpa.max(stability.sigma_heel_kpa).max(1.0);
        let depth_max = 1.05;
        let d_toe = (stability.sigma_toe_kpa / max_sig).clamp(0.05, 1.0) * depth_max;
        let d_heel = (stability.sigma_heel_kpa / max_sig).clamp(0.0, 1.0) * depth_max;
        let sig_col = if stability.is_bearing_stress_ok { "#2563eb" } else { "#dc2626" };
        let (pb0, pbb, pst, psh) = (to_s(0.0, 0.0), to_s(b_base, 0.0), to_s(0.0, -d_toe), to_s(b_base, -d_heel));
        if d_heel > 0.01 {
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{sig_col}\" fill-opacity=\"0.22\" stroke=\"{sig_col}\" stroke-width=\"1.8\"/>",
                pb0.0, pb0.1, pbb.0, pbb.1, psh.0, psh.1, pst.0, pst.1
            );
        } else {
            let x_c = (3.0 * stability.resultant_position_xr).clamp(0.05, b_base);
            let pzc = to_s(x_c, 0.0);
            let _ = writeln!(
                svg,
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{sig_col}\" fill-opacity=\"0.22\" stroke=\"{sig_col}\" stroke-width=\"1.8\"/>",
                pb0.0, pb0.1, pzc.0, pzc.1, pst.0, pst.1
            );
        }
        let toe_x = (pst.0 - 4.0).max(plot_x0 + 115.0);
        push_svg_rich_text(
            svg,
            toe_x,
            pst.1 + 12.0,
            10.5,
            "end",
            sig_col,
            true,
            &format!("{} = {:.1} kPa", if pt { "σ_{ponta}" } else { "σ_{toe}" }, stability.sigma_toe_kpa),
        );
        let heel_x = (psh.0 + 4.0).min(plot_x0 + plot_w - 125.0);
        push_svg_rich_text(
            svg,
            heel_x,
            psh.1 + 12.0,
            10.5,
            "start",
            sig_col,
            true,
            &format!("{} = {:.1} kPa", if pt { "σ_{calcanhar}" } else { "σ_{heel}" }, stability.sigma_heel_kpa),
        );

        let xr_pt = to_s(stability.resultant_position_xr, 0.0);
        let xr_col = if stability.is_in_middle_third { "#16a34a" } else { "#d97706" };
        let _ = writeln!(svg, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4.2\" fill=\"{xr_col}\"/>", xr_pt.0, xr_pt.1);
        let xr_lbl_pos = to_s(b_base * 0.5, -(depth_max + 0.16));
        push_svg_rich_text(
            svg,
            xr_lbl_pos.0.clamp(plot_x0 + 165.0, plot_x0 + plot_w - 165.0),
            (xr_lbl_pos.1 + 12.0).min(plot_y0 + plot_h - 6.0),
            10.2,
            "middle",
            xr_col,
            true,
            &format!("x_R = {:.2} m | e = {:.3} m (lim B/6 = {:.3} m)", stability.resultant_position_xr, stability.eccentricity, stability.max_allowed_eccentricity),
        );

        // 9. Cotas dimensionais H e B
        let (hd0, hd1) = (to_s(-0.48, 0.0), to_s(-0.48, h_total));
        let _ = writeln!(svg, "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#64748b\" stroke-width=\"1\"/>", hd0.0, hd0.1, hd1.0, hd1.1);
        push_svg_rich_text(svg, hd0.0 - 5.0, (hd0.1 + hd1.1) * 0.5 + 3.0, 10.0, "end", "#475569", true, &format!("H = {:.2} m", h_total));
        let (bd0, bd1) = (to_s(0.0, h_total + 0.25), to_s(b_base, h_total + 0.25));
        let _ = writeln!(svg, "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#64748b\" stroke-width=\"1\"/>", bd0.0, bd0.1, bd1.0, bd1.1);
        push_svg_rich_text(svg, (bd0.0 + bd1.0) * 0.5, bd0.1 - 5.0, 10.0, "middle", "#475569", true, &format!("B = {:.2} m", b_base));

        let _ = writeln!(svg, "</g>");

        // Rótulos de eixos
        let mut tx = (x_min / step_x).ceil() * step_x;
        while tx <= x_max + 0.001 {
            let (sx_pos, _) = to_s(tx, y_min);
            if sx_pos >= plot_x0 + 12.0 && sx_pos <= plot_x0 + plot_w - 12.0 {
                push_svg_rich_text(svg, sx_pos, plot_y0 + plot_h + 15.0, 9.5, "middle", "#64748b", false, &format!("{:.0} m", tx));
            }
            tx += step_x;
        }
        let mut ty = (y_min / step_y).ceil() * step_y;
        while ty <= y_max + 0.001 {
            let (_, sy_pos) = to_s(x_min, ty);
            if sy_pos >= plot_y0 + 10.0 && sy_pos <= plot_y0 + plot_h - 8.0 {
                push_svg_rich_text(svg, plot_x0 - 6.0, sy_pos + 3.0, 9.5, "end", "#64748b", false, &format!("{:.0} m", ty));
            }
            ty += step_y;
        }

        // Card HUD de Verificações Normativas (Superior Direito)
        let card_w = 240.0;
        let card_h = 98.0;
        let cx = plot_x0 + plot_w - card_w - 10.0;
        let cy = plot_y0 + 10.0;
        let _ = writeln!(
            svg,
            "<rect x=\"{cx:.1}\" y=\"{cy:.1}\" width=\"{card_w:.1}\" height=\"{card_h:.1}\" rx=\"5\" fill=\"#ffffff\" fill-opacity=\"0.94\" stroke=\"#cbd5e1\"/>"
        );
        push_svg_rich_text(
            svg,
            cx + 8.0,
            cy + 16.0,
            10.5,
            "start",
            "#0f172a",
            true,
            if pt { "Verificações (NBR 11682 / 6122)" } else { "Checks (NBR 11682 / 6122)" },
        );
        let checks = [
            (
                if pt {
                    format!("FS_{{tomb}} = {:.2} (≥ 1.50)", stability.fs_overturning)
                } else {
                    format!("FS_{{ovt}} = {:.2} (≥ 1.50)", stability.fs_overturning)
                },
                stability.is_overturning_ok,
            ),
            (
                if pt {
                    format!("FS_{{desl}} = {:.2} (≥ 1.50)", stability.fs_sliding)
                } else {
                    format!("FS_{{sld}} = {:.2} (≥ 1.50)", stability.fs_sliding)
                },
                stability.is_sliding_ok,
            ),
            (
                format!("{}: e = {:.3} m", if pt { "Terço Médio" } else { "Middle Third" }, stability.eccentricity),
                stability.is_in_middle_third,
            ),
            (
                format!("{}: FS = {:.2} (≥ 2.00)", if pt { "Cap. Hansen" } else { "Hansen Bearing" }, bearing.fs_bearing),
                bearing.is_bearing_ok && stability.is_bearing_stress_ok,
            ),
        ];
        for (idx, (lbl, ok)) in checks.iter().enumerate() {
            let col = if *ok { "#16a34a" } else { "#dc2626" };
            let badge = if *ok { "✔" } else { "✖" };
            push_svg_rich_text(
                svg,
                cx + 8.0,
                cy + 34.0 + (idx as f64) * 16.5,
                10.0,
                "start",
                col,
                true,
                &format!("{badge} {lbl}"),
            );
        }
    }

    /// Prancha vetorial completa em SVG registrando conjuntamente os diagramas e gráficos das Abas 2 e 3.
    pub fn export_complete_sheet_svg_lang(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> String {
        let width = 1040.0_f64;
        let height = 1240.0_f64;
        let mut svg = String::with_capacity(48_000);
        let _ = writeln!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.0} {height:.0}\" width=\"{width:.0}\" height=\"{height:.0}\">"
        );
        let _ = writeln!(svg, "<rect width=\"100%\" height=\"100%\" fill=\"#f1f5f9\"/>");
        let _ = writeln!(svg, "<rect x=\"16\" y=\"14\" width=\"{:.0}\" height=\"42\" rx=\"6\" fill=\"#1b365d\"/>", width - 32.0);
        push_svg_rich_text(
            &mut svg,
            28.0,
            40.0,
            15.0,
            "start",
            "#ffffff",
            true,
            &format!("GeoEstabil — {}", project.title),
        );
        Self::append_slope_panel_contents(&mut svg, project, slope_report, lang, 16.0, 68.0, width - 32.0, 565.0);
        Self::append_wall_panel_contents(&mut svg, project, lang, 16.0, 648.0, width - 32.0, 574.0);
        svg.push_str("</svg>\n");
        svg
    }

    pub fn export_active_svg_lang(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> String {
        let _ = ActiveProjectMode::SlopeStability;
        Self::export_complete_sheet_svg_lang(project, slope_report, lang)
    }
}
