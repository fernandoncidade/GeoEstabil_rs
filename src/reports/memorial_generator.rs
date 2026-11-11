use crate::core::project::GeoProject;
use crate::core::types::EarthPressureTheory;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::engine::wall::bearing_capacity::calculate_bearing_capacity;
use crate::engine::wall::concrete_design::design_concrete_reinforcement;
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::stability::check_wall_external_stability;
use crate::locale::translations::Language;
use crate::reports::pdf_writer::{
    normalize_math_symbols, split_equation_segments, EqSegment, PdfDocument,
};
use crate::utils::export_dxf_svg::{split_math_runs, ScriptKind, VectorExporter};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Converte expressões com índices `_` / `_{...}`, expoentes `^` / `^{...}` e macros LaTeX
/// para marcação HTML5 semântica (`<sub>`, `<sup>`) sem nenhum sublinhado literal exposto.
fn math_html(expr: &str) -> String {
    let norm = normalize_math_symbols(expr);
    let mut out = String::with_capacity(norm.len() + 16);
    for (txt, kind) in split_math_runs(&norm) {
        let esc = escape_html(&txt);
        match kind {
            ScriptKind::Normal => out.push_str(&esc),
            ScriptKind::Sub => {
                let _ = write!(out, "<sub>{esc}</sub>");
            }
            ScriptKind::Sup => {
                let _ = write!(out, "<sup>{esc}</sup>");
            }
        }
    }
    out
}

fn render_eq_segments_html(raw: &str) -> String {
    let norm = normalize_math_symbols(raw);
    let mut out = String::with_capacity(norm.len() + 32);
    for seg in split_equation_segments(&norm) {
        match seg {
            EqSegment::Text(t) => out.push_str(&math_html(&t)),
            EqSegment::Fraction { num, den } => {
                let _ = write!(
                    out,
                    "<span class=\"frac\"><span class=\"num\">{}</span><span class=\"den\">{}</span></span>",
                    math_html(&num),
                    math_html(&den)
                );
            }
        }
    }
    out
}

fn push_html_eq_card(html: &mut String, tag: &str, expr: &str, note: Option<&str>) {
    html.push_str("<div class=\"eq-card\">\n<div class=\"eq-row\">\n<div class=\"eq-math\">");
    html.push_str(&render_eq_segments_html(expr));
    html.push_str("</div>\n");
    if !tag.is_empty() {
        let _ = write!(html, "<span class=\"eq-tag\">{}</span>", escape_html(tag));
    }
    html.push_str("</div>\n");
    if let Some(n) = note {
        let _ = write!(html, "<div class=\"eq-note\">{}</div>\n", render_eq_segments_html(n));
    }
    html.push_str("</div>\n");
}

pub struct MemorialGenerator;

impl MemorialGenerator {
    /// Gera memorial completo HTML5 (compatível com impressão direta em PDF A4 pelo navegador).
    pub fn generate_html(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
    ) -> String {
        Self::generate_html_lang(project, slope_report, Language::PtBr)
    }

    pub fn generate_html_lang(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> String {
        let pt = lang == Language::PtBr;
        let wall_pressures = calculate_earth_pressures(&project.wall_model);
        let wall_stability = check_wall_external_stability(&project.wall_model);
        let wall_bearing = calculate_bearing_capacity(&project.wall_model, &wall_stability);
        let wall_reinf = design_concrete_reinforcement(&project.wall_model, &wall_stability);

        let mut html = String::with_capacity(32_768);
        let html_lang = if pt { "pt-BR" } else { "en-US" };
        let title_prefix = if pt { "Memorial de Cálculo Geotécnico" } else { "Geotechnical Calculation Report" };
        let _ = writeln!(html, "<!DOCTYPE html>\n<html lang=\"{html_lang}\">\n<head>");
        html.push_str("<meta charset=\"UTF-8\">\n");
        let _ = writeln!(html, "<title>{title_prefix} — {}</title>", escape_html(&project.title));
        html.push_str(
            "<style>
@page { size: A4; margin: 16mm; }
* { box-sizing: border-box; }
body { font-family: 'Segoe UI', 'Liberation Sans', Arial, sans-serif; color: #1e293b; margin: 28px auto; padding: 0 12px; max-width: 960px; line-height: 1.52; font-size: 13.5px; overflow-wrap: anywhere; word-break: break-word; }
h1, h2, h3 { color: #0f172a; page-break-after: avoid; overflow-wrap: anywhere; }
h1 { border-bottom: 3px solid #1e3a8a; padding-bottom: 8px; font-size: 22px; margin-bottom: 10px; }
h2 { border-left: 5px solid #2563eb; padding-left: 10px; background: #f8fafc; padding-top: 6px; padding-bottom: 6px; margin-top: 24px; font-size: 16px; }
h3 { color: #1e40af; font-size: 14px; margin-top: 16px; }
table { width: 100%; border-collapse: collapse; margin: 10px 0 14px 0; font-size: 12.2px; page-break-inside: avoid; table-layout: auto; }
th, td { border: 1px solid #cbd5e1; padding: 6px 9px; text-align: left; overflow-wrap: anywhere; word-break: break-word; vertical-align: middle; }
th { background-color: #1e3a8a; color: #ffffff; font-weight: 600; }
tr:nth-child(even) td { background-color: #f8fafc; }
sub, sup { font-size: 0.74em; line-height: 0; }
.badge-ok { background: #16a34a; color: #ffffff; padding: 2px 8px; border-radius: 4px; font-weight: bold; font-size: 11px; white-space: nowrap; display: inline-block; }
.badge-fail { background: #dc2626; color: #ffffff; padding: 2px 8px; border-radius: 4px; font-weight: bold; font-size: 11px; white-space: nowrap; display: inline-block; }
.box { border: 1px solid #cbd5e1; border-radius: 6px; padding: 12px 14px; background: #f8fafc; margin-bottom: 16px; page-break-inside: avoid; overflow-wrap: anywhere; }
.eq-card { border: 1px solid #bfdbfe; border-left: 4px solid #2563eb; background: #eff6ff; border-radius: 6px; padding: 10px 14px; margin: 12px 0; page-break-inside: avoid; max-width: 100%; }
.eq-row { display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; min-width: 0; }
.eq-math { font-family: 'Cambria Math', 'Georgia', 'Times New Roman', serif; font-size: 15px; font-weight: 600; color: #0f172a; display: flex; align-items: center; flex-wrap: wrap; gap: 5px; min-width: 0; max-width: 100%; }
.frac { display: inline-flex; flex-direction: column; align-items: center; vertical-align: middle; margin: 0 3px; text-align: center; }
.frac .num { border-bottom: 1.5px solid #0f172a; padding: 0 6px 2px 6px; font-size: 13.5px; }
.frac .den { padding: 2px 6px 0 6px; font-size: 13.5px; }
.eq-tag { font-size: 11px; font-weight: 700; color: #1e40af; background: #dbeafe; padding: 2px 8px; border-radius: 4px; white-space: nowrap; }
.eq-note { margin-top: 6px; font-size: 12px; color: #334155; font-style: italic; overflow-wrap: anywhere; }
.svg-figure { margin: 14px 0 18px 0; text-align: center; page-break-inside: avoid; max-width: 100%; }
.svg-figure svg { max-width: 100% !important; height: auto !important; display: block; margin: 0 auto; }
.svg-figure figcaption { margin-top: 6px; font-size: 11.8px; color: #475569; font-weight: 600; }
.footer { margin-top: 32px; border-top: 1px solid #cbd5e1; padding-top: 12px; font-size: 11px; color: #64748b; text-align: center; }
@media print { body { margin: 0; } .no-print { display: none; } }
</style>
</head>
<body>\n",
        );

        let header_title = if pt {
            "MEMORIAL DESCRITIVO E DE CÁLCULO GEOTÉCNICO / ESTRUTURAL"
        } else {
            "GEOTECHNICAL &amp; STRUCTURAL ENGINEERING CALCULATION REPORT"
        };
        let _ = writeln!(html, "<h1>{header_title}</h1>\n<div class=\"box\">\n<table>");

        if pt {
            let _ = writeln!(
                html,
                "<tr><th>Empreendimento / Obra</th><td>{}</td><th>Data</th><td>{}</td></tr>",
                escape_html(&project.title),
                escape_html(&project.date)
            );
            let _ = writeln!(
                html,
                "<tr><th>Cliente / Interessado</th><td>{}</td><th>Localização</th><td>{}</td></tr>",
                escape_html(&project.client),
                escape_html(&project.site_location)
            );
            let _ = writeln!(
                html,
                "<tr><th>Responsável Técnico</th><td>{}</td><th>Conselho Profissional</th><td>{}</td></tr>",
                escape_html(project.effective_engineer_name()),
                escape_html(&project.crea_cau)
            );
            let _ = writeln!(
                html,
                "<tr><th>Registro / ART-RRT</th><td>{}</td><th>Modo de Estudo</th><td>{}</td></tr>",
                escape_html(&project.art_rrt),
                escape_html(project.active_mode.name(lang))
            );
        } else {
            let _ = writeln!(
                html,
                "<tr><th>Project Title</th><td>{}</td><th>Date</th><td>{}</td></tr>",
                escape_html(&project.title),
                escape_html(&project.date)
            );
            let _ = writeln!(
                html,
                "<tr><th>Client / Owner</th><td>{}</td><th>Site Location</th><td>{}</td></tr>",
                escape_html(&project.client),
                escape_html(&project.site_location)
            );
            let _ = writeln!(
                html,
                "<tr><th>Lead Engineer</th><td>{}</td><th>License / Board</th><td>{}</td></tr>",
                escape_html(project.effective_engineer_name()),
                escape_html(&project.crea_cau)
            );
            let _ = writeln!(
                html,
                "<tr><th>Record / Filing</th><td>{}</td><th>Study Mode</th><td>{}</td></tr>",
                escape_html(&project.art_rrt),
                escape_html(project.active_mode.name(lang))
            );
        }
        html.push_str("</table>\n</div>\n");

        // 1. Normative Framework
        if pt {
            html.push_str("<h2>1. ENQUADRAMENTO E CRITÉRIOS NORMATIVOS (ABNT)</h2>\n");
            html.push_str("<table><tr><th>Norma Técnica</th><th>Escopo e Aplicação</th><th>Critério Exigido no Projeto</th></tr>\n");
            let _ = writeln!(
                html,
                "<tr><td><strong>ABNT NBR 11682:2009</strong></td><td>Estabilidade de Encostas e Obras de Contenção</td><td>Segurança Alta — {} (Risco Alto)</td></tr>",
                math_html(&format!("FS ≥ {:.2}", project.slope_model.safety_level.min_fs()))
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>ABNT NBR 6122:2022</strong></td><td>Projeto e Execução de Fundações Rasas e Contenções</td><td>{}, {}, {}</td></tr>",
                math_html("FS_{tomb} ≥ 1.50"),
                math_html("FS_{desl} ≥ 1.50"),
                math_html("e ≤ B/6, σ_{ponta} ≤ σ_{adm}")
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>ABNT NBR 6118:2023</strong></td><td>Projeto de Estruturas de Concreto Armado</td><td>ELU ({}), {}, {}, {}</td></tr>",
                math_html("γ_f = 1.40, ξ ≤ 0.45"),
                math_html("ρ_{mín} = 0.15%"),
                math_html("V_{Rd1}"),
                math_html("w_k ≤ 0.30 mm")
            );
            html.push_str("</table>\n");
        } else {
            html.push_str("<h2>1. NORMATIVE FRAMEWORK &amp; DESIGN CRITERIA (ABNT)</h2>\n");
            html.push_str("<table><tr><th>Technical Standard</th><th>Scope &amp; Application</th><th>Project Criterion</th></tr>\n");
            let _ = writeln!(
                html,
                "<tr><td><strong>ABNT NBR 11682:2009</strong></td><td>Slope Stability &amp; Retaining Structures</td><td>High Safety Level — {}</td></tr>",
                math_html(&format!("FS ≥ {:.2}", project.slope_model.safety_level.min_fs()))
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>ABNT NBR 6122:2022</strong></td><td>Design &amp; Construction of Foundations</td><td>{}, {}, {}</td></tr>",
                math_html("FS_{ovt} ≥ 1.50"),
                math_html("FS_{sld} ≥ 1.50"),
                math_html("e ≤ B/6, σ_{toe} ≤ σ_{adm}")
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>ABNT NBR 6118:2023</strong></td><td>Design of Reinforced Concrete Structures</td><td>ULS ({}), {}, {}, {}</td></tr>",
                math_html("γ_f = 1.40, ξ ≤ 0.45"),
                math_html("ρ_{min} = 0.15%"),
                math_html("V_{Rd1}"),
                math_html("w_k ≤ 0.30 mm")
            );
            html.push_str("</table>\n");
        }

        // 2. Geotechnical Parameters
        if pt {
            html.push_str("<h2>2. CARACTERIZAÇÃO ESTRATIGRÁFICA E PARÂMETROS DOS SOLOS</h2>\n");
            let _ = writeln!(
                html,
                "<table><tr><th>Camada</th><th>Descrição Litológica</th><th>{} [kN/m³]</th><th>{} [kN/m³]</th><th>{} [kPa]</th><th>{} [°]</th><th>{} [kPa]</th></tr>",
                math_html("γ_{nat}"),
                math_html("γ_{sat}"),
                math_html("c'"),
                math_html("φ'"),
                math_html("σ_{adm}"),
            );
        } else {
            html.push_str("<h2>2. STRATIGRAPHIC CHARACTERIZATION &amp; SOIL PARAMETERS</h2>\n");
            let _ = writeln!(
                html,
                "<table><tr><th>Layer</th><th>Lithological Description</th><th>{} [kN/m³]</th><th>{} [kN/m³]</th><th>{} [kPa]</th><th>{} [°]</th><th>{} [kPa]</th></tr>",
                math_html("γ_{nat}"),
                math_html("γ_{sat}"),
                math_html("c'"),
                math_html("φ'"),
                math_html("σ_{adm}"),
            );
        }
        for l in &project.slope_model.layers {
            let _ = writeln!(
                html,
                "<tr><td>{}</td><td>{}</td><td>{:.1}</td><td>{:.1}</td><td>{:.1}</td><td>{:.1}</td><td>{:.1}</td></tr>",
                escape_html(&l.name),
                escape_html(&l.description),
                l.gamma,
                l.gamma_sat,
                l.cohesion,
                l.friction_angle_deg,
                l.sigma_adm
            );
        }
        html.push_str("</table>\n");

        // 3. Slope Stability (Tab 2)
        if pt {
            html.push_str("<h2>3. ANÁLISE DE ESTABILIDADE DE TALUDES (ABA 2 — ABNT NBR 11682:2009)</h2>\n");
            push_html_eq_card(
                &mut html,
                "Bishop Simplificado",
                "FS = \\frac{Σ [ c' · Δx + (W - u · Δx + q · Δx) · tan φ' ] / m_α}{Σ (W + q · Δx) · sin α}",
                Some("Onde m_α = cos α + (sin α · tan φ') / FS (processo iterativo por fatias discretas)."),
            );
        } else {
            html.push_str("<h2>3. SLOPE STABILITY ANALYSIS (TAB 2 — ABNT NBR 11682:2009)</h2>\n");
            push_html_eq_card(
                &mut html,
                "Simplified Bishop",
                "FS = \\frac{Σ [ c' · Δx + (W - u · Δx + q · Δx) · tan φ' ] / m_α}{Σ (W + q · Δx) · sin α}",
                Some("Where m_α = cos α + (sin α · tan φ') / FS (iterative discrete slice equilibrium)."),
            );
        }

        // Diagrama SVG completo da Aba 2 (sempre incluído: estratigrafia, N.A., sobrecarga q, Heatmap FS, fatias e círculo crítico)
        html.push_str("<figure class=\"svg-figure\">\n");
        html.push_str(&VectorExporter::render_inline_slope_svg(project, slope_report, lang));
        let slope_cap = if pt {
            "Figura 1 — Aba 2: Seção Geotécnica, Estratigrafia, Mapa de Calor (Heatmap FS), Fatias e Superfície Crítica."
        } else {
            "Figure 1 — Tab 2: Geotechnical Profile, Stratigraphy, FS Heatmap, Slices & Critical Slip Surface."
        };
        let _ = writeln!(html, "<figcaption>{}</figcaption>\n</figure>", escape_html(slope_cap));

        if let Some(slope) = slope_report {
            let status_badge = |ok: bool| {
                let lbl = match (ok, pt) {
                    (true, true) => "ESTÁVEL (ATENDE)",
                    (false, true) => "INSTÁVEL (ABAIXO DO LIMITE)",
                    (true, false) => "STABLE (APPROVED)",
                    (false, false) => "UNSTABLE (BELOW LIMIT)",
                };
                if ok {
                    format!("<span class=\"badge-ok\">{lbl}</span>")
                } else {
                    format!("<span class=\"badge-fail\">{lbl}</span>")
                }
            };
            if pt {
                let _ = writeln!(
                    html,
                    "<p>Varredura de <strong>{} superfícies circulares</strong> ({} válidas) executada em <strong>{:.1} ms</strong>. Centro crítico: {}, Raio {}, entrada {}, saída {}.</p>",
                    slope.total_circles_evaluated,
                    slope.valid_circles_count,
                    slope.elapsed_ms,
                    math_html(&format!("(X_c = {:.2} m, Y_c = {:.2} m)", slope.critical_circle.center_x, slope.critical_circle.center_y)),
                    math_html(&format!("R = {:.2} m", slope.critical_circle.radius)),
                    math_html(&format!("x_{{ent}} = {:.2} m", slope.critical_circle.x_entry)),
                    math_html(&format!("x_{{saí}} = {:.2} m", slope.critical_circle.x_exit)),
                );
                let _ = writeln!(
                    html,
                    "<table><tr><th>Método de Equilíbrio Limite</th><th>Fator de Segurança Obtido</th><th>{}</th><th>Status Normativo</th></tr>",
                    math_html("FS_{mín} Exigido")
                );
                let _ = writeln!(
                    html,
                    "<tr><td><strong>Bishop Simplificado</strong></td><td>{}</td><td>{:.2}</td><td>{}</td></tr>",
                    math_html(&format!("FS = {:.3}", slope.critical_circle.fs_bishop)),
                    slope.required_min_fs,
                    status_badge(slope.critical_circle.fs_bishop >= slope.required_min_fs)
                );
                let _ = writeln!(
                    html,
                    "<tr><td>Fellenius (Ordinário / Sueco)</td><td>{}</td><td>{:.2}</td><td>{}</td></tr>",
                    math_html(&format!("FS = {:.3}", slope.critical_circle.fs_fellenius)),
                    slope.required_min_fs,
                    status_badge(slope.critical_circle.fs_fellenius >= slope.required_min_fs)
                );
                let _ = writeln!(
                    html,
                    "<tr><td>Janbu Simplificado</td><td>{}</td><td>{:.2}</td><td>{}</td></tr></table>",
                    math_html(&format!("FS = {:.3}", slope.critical_circle.fs_janbu)),
                    slope.required_min_fs,
                    status_badge(slope.critical_circle.fs_janbu >= slope.required_min_fs)
                );
                let _ = writeln!(
                    html,
                    "<h3>Discretização em Fatias da Massa Instável ({} fatias)</h3>",
                    slope.critical_circle.slices.len()
                );
                let _ = writeln!(
                    html,
                    "<table><tr><th>Fatia</th><th>{} [m]</th><th>{} [m]</th><th>{} [m]</th><th>{} [kN/m]</th><th>{} [°]</th><th>{} [kPa]</th><th>{} [kPa]</th><th>{} [°]</th><th>Camada</th></tr>",
                    math_html("x_{méd}"),
                    math_html("Δx"),
                    math_html("h"),
                    math_html("W"),
                    math_html("α"),
                    math_html("u"),
                    math_html("c'"),
                    math_html("φ'"),
                );
            } else {
                let _ = writeln!(
                    html,
                    "<p>Grid sweep of <strong>{} circular slip surfaces</strong> ({} valid) completed in <strong>{:.1} ms</strong>. Critical circle: {}, radius {}, entry {}, exit {}.</p>",
                    slope.total_circles_evaluated,
                    slope.valid_circles_count,
                    slope.elapsed_ms,
                    math_html(&format!("(X_c = {:.2} m, Y_c = {:.2} m)", slope.critical_circle.center_x, slope.critical_circle.center_y)),
                    math_html(&format!("R = {:.2} m", slope.critical_circle.radius)),
                    math_html(&format!("x_{{entry}} = {:.2} m", slope.critical_circle.x_entry)),
                    math_html(&format!("x_{{exit}} = {:.2} m", slope.critical_circle.x_exit)),
                );
                let _ = writeln!(
                    html,
                    "<table><tr><th>Limit Equilibrium Method</th><th>Factor of Safety</th><th>{}</th><th>Normative Status</th></tr>",
                    math_html("Required FS_{min}")
                );
                let _ = writeln!(
                    html,
                    "<tr><td><strong>Simplified Bishop</strong></td><td>{}</td><td>{:.2}</td><td>{}</td></tr>",
                    math_html(&format!("FS = {:.3}", slope.critical_circle.fs_bishop)),
                    slope.required_min_fs,
                    status_badge(slope.critical_circle.fs_bishop >= slope.required_min_fs)
                );
                let _ = writeln!(
                    html,
                    "<tr><td>Fellenius (Ordinary / Swedish)</td><td>{}</td><td>{:.2}</td><td>{}</td></tr>",
                    math_html(&format!("FS = {:.3}", slope.critical_circle.fs_fellenius)),
                    slope.required_min_fs,
                    status_badge(slope.critical_circle.fs_fellenius >= slope.required_min_fs)
                );
                let _ = writeln!(
                    html,
                    "<tr><td>Simplified Janbu</td><td>{}</td><td>{:.2}</td><td>{}</td></tr></table>",
                    math_html(&format!("FS = {:.3}", slope.critical_circle.fs_janbu)),
                    slope.required_min_fs,
                    status_badge(slope.critical_circle.fs_janbu >= slope.required_min_fs)
                );
                let _ = writeln!(
                    html,
                    "<h3>Slice Discretization of Critical Sliding Mass ({} slices)</h3>",
                    slope.critical_circle.slices.len()
                );
                let _ = writeln!(
                    html,
                    "<table><tr><th>Slice</th><th>{} [m]</th><th>{} [m]</th><th>{} [m]</th><th>{} [kN/m]</th><th>{} [°]</th><th>{} [kPa]</th><th>{} [kPa]</th><th>{} [°]</th><th>Layer</th></tr>",
                    math_html("x_{mid}"),
                    math_html("Δx"),
                    math_html("h"),
                    math_html("W"),
                    math_html("α"),
                    math_html("u"),
                    math_html("c'"),
                    math_html("φ'"),
                );
            }
            for s in &slope.critical_circle.slices {
                let _ = writeln!(
                    html,
                    "<tr><td>{}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td>{:.1}</td><td>{:.1}</td><td>{:.1}</td><td>{:.1}</td><td>{:.1}</td><td>{}</td></tr>",
                    s.index,
                    s.x_mid,
                    s.width,
                    s.height,
                    s.weight,
                    s.base_angle_deg,
                    s.pore_pressure,
                    s.cohesion,
                    s.friction_angle_deg,
                    escape_html(&s.layer_name)
                );
            }
            html.push_str("</table>\n");
        } else {
            let msg = if pt {
                "<p><em>Nenhuma varredura de equilíbrio limite foi executada ainda na Aba 2 para este projeto.</em></p>"
            } else {
                "<p><em>No Limit Equilibrium grid sweep has been executed yet on Tab 2 for this project.</em></p>"
            };
            html.push_str(msg);
        }

        // 4. Retaining Wall & External Stability (Tab 3)
        let pass_badge = |ok: bool| match (ok, pt) {
            (true, true) => "<span class=\"badge-ok\">APROVADO</span>",
            (false, true) => "<span class=\"badge-fail\">REPROVADO</span>",
            (true, false) => "<span class=\"badge-ok\">APPROVED</span>",
            (false, false) => "<span class=\"badge-fail\">FAILED</span>",
        };
        let stress_expr = if wall_stability.is_in_middle_third {
            if pt {
                "σ_{ponta,calcanhar} = \\frac{N}{B} · (1 ± \\frac{6 · e}{B})"
            } else {
                "σ_{toe,heel} = \\frac{N}{B} · (1 ± \\frac{6 · e}{B})"
            }
        } else if pt {
            "σ_{ponta} = \\frac{2 · N}{3 · x_R}"
        } else {
            "σ_{toe} = \\frac{2 · N}{3 · x_R}"
        };

        if pt {
            let _ = writeln!(
                html,
                "<h2>4. MURO DE CONTENÇÃO E ESTABILIDADE EXTERNA (ABA 3 — {})</h2>",
                escape_html(project.wall_model.wall_type.name(lang))
            );
            let _ = writeln!(
                html,
                "<p><strong>Geometria:</strong> {}, {}, {}, {}. <strong>Teoria de Empuxo:</strong> {}.</p>",
                math_html(&format!("H_{{total}} = {:.2} m", project.wall_model.geometry.total_height())),
                math_html(&format!("B = {:.2} m", project.wall_model.geometry.base_width)),
                math_html(&format!("h_{{sapata}} = {:.2} m", project.wall_model.geometry.base_thickness)),
                math_html(&format!("β = {:.1}°", project.wall_model.geometry.backfill_slope_deg)),
                escape_html(project.wall_model.theory.name(lang))
            );
            push_html_eq_card(
                &mut html,
                "Empuxo / Estabilidade",
                &format!(
                    "{}   |   {}",
                    if matches!(project.wall_model.theory, EarthPressureTheory::Rankine) {
                        "K_a = tan²(45° - φ'/2)"
                    } else {
                        "K_a = Coulomb(φ', δ, β, θ)"
                    },
                    "FS_{tomb} = \\frac{∑ M_{est}}{∑ M_{tomb}}"
                ),
                Some(&format!(
                    "Empuxo ativo resultante: K_a = {:.4}, K_p = {:.4}, E_{{a,h}} = {:.2} kN/m (M_{{tomb}} = {:.2} kNm/m)",
                    wall_pressures.ka, wall_pressures.kp, wall_pressures.ea_horizontal, wall_pressures.overturning_moment
                )),
            );
            push_html_eq_card(
                &mut html,
                "Tensões na Base",
                stress_expr,
                Some(if wall_stability.is_in_middle_third {
                    "Condição comprimida em toda a base (excentricidade dentro do terço médio: e ≤ B/6)."
                } else {
                    "Base parcialmente tracionada (descolamento no calcanhar com redistribuição triangular sobre 3 · x_R)."
                }),
            );
        } else {
            let _ = writeln!(
                html,
                "<h2>4. RETAINING WALL &amp; EXTERNAL STABILITY (TAB 3 — {})</h2>",
                escape_html(project.wall_model.wall_type.name(lang))
            );
            let _ = writeln!(
                html,
                "<p><strong>Geometry:</strong> {}, {}, {}, {}. <strong>Thrust Theory:</strong> {}.</p>",
                math_html(&format!("H_{{total}} = {:.2} m", project.wall_model.geometry.total_height())),
                math_html(&format!("B = {:.2} m", project.wall_model.geometry.base_width)),
                math_html(&format!("h_{{base}} = {:.2} m", project.wall_model.geometry.base_thickness)),
                math_html(&format!("β = {:.1}°", project.wall_model.geometry.backfill_slope_deg)),
                escape_html(project.wall_model.theory.name(lang))
            );
            push_html_eq_card(
                &mut html,
                "Thrust / Stability",
                &format!(
                    "{}   |   {}",
                    if matches!(project.wall_model.theory, EarthPressureTheory::Rankine) {
                        "K_a = tan²(45° - φ'/2)"
                    } else {
                        "K_a = Coulomb(φ', δ, β, θ)"
                    },
                    "FS_{ovt} = \\frac{∑ M_{stab}}{∑ M_{ovt}}"
                ),
                Some(&format!(
                    "Active thrust summary: K_a = {:.4}, K_p = {:.4}, E_{{a,h}} = {:.2} kN/m (M_{{ovt}} = {:.2} kNm/m)",
                    wall_pressures.ka, wall_pressures.kp, wall_pressures.ea_horizontal, wall_pressures.overturning_moment
                )),
            );
            push_html_eq_card(
                &mut html,
                "Base Stress",
                stress_expr,
                Some(if wall_stability.is_in_middle_third {
                    "Full base compression (resultant inside middle third: e ≤ B/6)."
                } else {
                    "Cracked heel uplift (triangular stress block distributed over effective length 3 · x_R)."
                }),
            );
        }

        // Diagrama SVG completo da Aba 3 (geometria, empuxos, vetores Ea,h e Ep, tensões na base, xR e HUD)
        html.push_str("<figure class=\"svg-figure\">\n");
        html.push_str(&VectorExporter::render_inline_wall_svg(project, lang));
        let wall_cap = if pt {
            "Figura 2 — Aba 3: Seção do Muro, Diagramas de Empuxo Lateral e Hidrostático, Tensões na Base e Verificações."
        } else {
            "Figure 2 — Tab 3: Retaining Wall Cross-Section, Lateral & Hydrostatic Pressures, Base Stress & Checks."
        };
        let _ = writeln!(html, "<figcaption>{}</figcaption>\n</figure>", escape_html(wall_cap));

        if pt {
            html.push_str("<table><tr><th>Componente de Empuxo</th><th>Notação</th><th>Valor</th><th>Braço / Observação</th></tr>\n");
            let _ = writeln!(html, "<tr><td>Coeficiente de Empuxo Ativo</td><td>{}</td><td>{:.4}</td><td>—</td></tr>", math_html("K_a"), wall_pressures.ka);
            let _ = writeln!(html, "<tr><td>Coeficiente de Empuxo Passivo</td><td>{}</td><td>{:.4}</td><td>—</td></tr>", math_html("K_p"), wall_pressures.kp);
            let _ = writeln!(html, "<tr><td>Empuxo do Solo</td><td>{}</td><td>{:.2} kN/m</td><td>{}</td></tr>", math_html("E_{a,solo}"), wall_pressures.ea_soil, math_html(&format!("y = {:.2} m", wall_pressures.ea_arm_y)));
            let _ = writeln!(html, "<tr><td>Empuxo da Sobrecarga {}</td><td>{}</td><td>{:.2} kN/m</td><td>{}</td></tr>", math_html(&format!("q = {:.1} kPa", project.wall_model.surcharge_q)), math_html("E_{a,q}"), wall_pressures.ea_surcharge, math_html(&format!("y = {:.2} m", project.wall_model.geometry.total_height() * 0.5)));
            let _ = writeln!(html, "<tr><td>Empuxo Hidrostático (Água)</td><td>{}</td><td>{:.2} kN/m</td><td>{}</td></tr>", math_html("E_w"), wall_pressures.ea_water, math_html(&format!("y = {:.2} m", project.wall_model.water_height_back / 3.0)));
            let _ = writeln!(html, "<tr><td><strong>Empuxo Ativo Horizontal Total</strong></td><td><strong>{}</strong></td><td><strong>{:.2} kN/m</strong></td><td>{}</td></tr></table>", math_html("E_{a,h}"), wall_pressures.ea_horizontal, math_html(&format!("M_{{tomb}} = {:.2} kNm/m", wall_pressures.overturning_moment)));

            html.push_str("<table><tr><th>Verificação de Estabilidade Externa</th><th>Valor Calculado</th><th>Critério / Limite</th><th>Status</th></tr>\n");
            let _ = writeln!(
                html,
                "<tr><td><strong>Tombamento</strong></td><td>{} ({}; {})</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("FS_{{tomb}} = {:.2}", wall_stability.fs_overturning)),
                math_html(&format!("M_{{est}} = {:.1} kNm/m", wall_stability.total_stabilizing_moment)),
                math_html(&format!("M_{{tomb}} = {:.1} kNm/m", wall_stability.total_overturning_moment)),
                math_html("FS ≥ 1.50"),
                pass_badge(wall_stability.is_overturning_ok)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Deslizamento</strong></td><td>{} ({}; {})</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("FS_{{desl}} = {:.2}", wall_stability.fs_sliding)),
                math_html(&format!("F_{{resist}} = {:.1} kN/m", wall_stability.total_resisting_horizontal_force)),
                math_html(&format!("F_{{atu}} = {:.1} kN/m", wall_stability.total_driving_horizontal_force)),
                math_html("FS ≥ 1.50"),
                pass_badge(wall_stability.is_sliding_ok)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Excentricidade / Terço Médio</strong></td><td>{}; {}</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("x_R = {:.2} m", wall_stability.resultant_position_xr)),
                math_html(&format!("e = {:.3} m", wall_stability.eccentricity)),
                math_html(&format!("e ≤ B/6 = {:.3} m", wall_stability.max_allowed_eccentricity)),
                pass_badge(wall_stability.is_in_middle_third)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Tensões na Base da Fundação</strong></td><td>{}; {}</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("σ_{{ponta}} = {:.1} kPa", wall_stability.sigma_toe_kpa)),
                math_html(&format!("σ_{{calcanhar}} = {:.1} kPa", wall_stability.sigma_heel_kpa)),
                math_html(&format!("σ_{{máx}} ≤ σ_{{adm}} = {:.1} kPa", wall_stability.sigma_adm_kpa)),
                pass_badge(wall_stability.is_bearing_stress_ok)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Capacidade de Carga (Hansen / Vesic)</strong></td><td>{}; {}</td><td>{}</td><td>{}</td></tr></table>",
                math_html(&format!("q_{{ult}} = {:.1} kPa", wall_bearing.q_ult_kpa)),
                math_html(&format!("FS = {:.2}", wall_bearing.fs_bearing)),
                math_html("FS ≥ 2.00"),
                pass_badge(wall_bearing.is_bearing_ok)
            );
        } else {
            html.push_str("<table><tr><th>Earth Pressure Component</th><th>Notation</th><th>Value</th><th>Arm / Remark</th></tr>\n");
            let _ = writeln!(html, "<tr><td>Active Earth Pressure Coeff.</td><td>{}</td><td>{:.4}</td><td>—</td></tr>", math_html("K_a"), wall_pressures.ka);
            let _ = writeln!(html, "<tr><td>Passive Earth Pressure Coeff.</td><td>{}</td><td>{:.4}</td><td>—</td></tr>", math_html("K_p"), wall_pressures.kp);
            let _ = writeln!(html, "<tr><td>Soil Active Thrust</td><td>{}</td><td>{:.2} kN/m</td><td>{}</td></tr>", math_html("E_{a,soil}"), wall_pressures.ea_soil, math_html(&format!("y = {:.2} m", wall_pressures.ea_arm_y)));
            let _ = writeln!(html, "<tr><td>Surcharge Thrust {}</td><td>{}</td><td>{:.2} kN/m</td><td>{}</td></tr>", math_html(&format!("q = {:.1} kPa", project.wall_model.surcharge_q)), math_html("E_{a,q}"), wall_pressures.ea_surcharge, math_html(&format!("y = {:.2} m", project.wall_model.geometry.total_height() * 0.5)));
            let _ = writeln!(html, "<tr><td>Hydrostatic Water Thrust</td><td>{}</td><td>{:.2} kN/m</td><td>{}</td></tr>", math_html("E_w"), wall_pressures.ea_water, math_html(&format!("y = {:.2} m", project.wall_model.water_height_back / 3.0)));
            let _ = writeln!(html, "<tr><td><strong>Total Horizontal Active Thrust</strong></td><td><strong>{}</strong></td><td><strong>{:.2} kN/m</strong></td><td>{}</td></tr></table>", math_html("E_{a,h}"), wall_pressures.ea_horizontal, math_html(&format!("M_{{ovt}} = {:.2} kNm/m", wall_pressures.overturning_moment)));

            html.push_str("<table><tr><th>External Stability Check</th><th>Computed Value</th><th>Criterion / Limit</th><th>Status</th></tr>\n");
            let _ = writeln!(
                html,
                "<tr><td><strong>Overturning</strong></td><td>{} ({}; {})</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("FS_{{ovt}} = {:.2}", wall_stability.fs_overturning)),
                math_html(&format!("M_{{res}} = {:.1} kNm/m", wall_stability.total_stabilizing_moment)),
                math_html(&format!("M_{{ovt}} = {:.1} kNm/m", wall_stability.total_overturning_moment)),
                math_html("FS ≥ 1.50"),
                pass_badge(wall_stability.is_overturning_ok)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Base Sliding</strong></td><td>{} ({}; {})</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("FS_{{sld}} = {:.2}", wall_stability.fs_sliding)),
                math_html(&format!("F_{{res}} = {:.1} kN/m", wall_stability.total_resisting_horizontal_force)),
                math_html(&format!("F_{{drv}} = {:.1} kN/m", wall_stability.total_driving_horizontal_force)),
                math_html("FS ≥ 1.50"),
                pass_badge(wall_stability.is_sliding_ok)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Eccentricity / Middle Third</strong></td><td>{}; {}</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("x_R = {:.2} m", wall_stability.resultant_position_xr)),
                math_html(&format!("e = {:.3} m", wall_stability.eccentricity)),
                math_html(&format!("e ≤ B/6 = {:.3} m", wall_stability.max_allowed_eccentricity)),
                pass_badge(wall_stability.is_in_middle_third)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Base Bearing Pressures</strong></td><td>{}; {}</td><td>{}</td><td>{}</td></tr>",
                math_html(&format!("σ_{{toe}} = {:.1} kPa", wall_stability.sigma_toe_kpa)),
                math_html(&format!("σ_{{heel}} = {:.1} kPa", wall_stability.sigma_heel_kpa)),
                math_html(&format!("σ_{{max}} ≤ σ_{{adm}} = {:.1} kPa", wall_stability.sigma_adm_kpa)),
                pass_badge(wall_stability.is_bearing_stress_ok)
            );
            let _ = writeln!(
                html,
                "<tr><td><strong>Bearing Capacity (Hansen / Vesic)</strong></td><td>{}; {}</td><td>{}</td><td>{}</td></tr></table>",
                math_html(&format!("q_{{ult}} = {:.1} kPa", wall_bearing.q_ult_kpa)),
                math_html(&format!("FS = {:.2}", wall_bearing.fs_bearing)),
                math_html("FS ≥ 2.00"),
                pass_badge(wall_bearing.is_bearing_ok)
            );
        }

        // 5. Structural Design NBR 6118:2023
        let conc = &project.wall_model.concrete;
        let conc_params_str = format!(
            "{} | {} | {}",
            math_html(&format!("f_{{ck}} = {:.0} MPa (f_{{cd}} = {:.2} MPa)", conc.fck_mpa, wall_reinf.fcd_mpa)),
            math_html(&format!("f_{{yk}} = {:.0} MPa (f_{{yd}} = {:.2} MPa)", conc.fyk_mpa, wall_reinf.fyd_mpa)),
            math_html(&format!("c_{{nom}} = {:.0} mm | γ_f = {:.2}", conc.cover_nom_mm, conc.gamma_f)),
        );
        if pt {
            html.push_str("<h2>5. DIMENSIONAMENTO ESTRUTURAL EM CONCRETO ARMADO (ABNT NBR 6118:2023)</h2>\n");
            let _ = writeln!(html, "<p><strong>Parâmetros Estruturais:</strong> {conc_params_str}</p>");
            push_html_eq_card(
                &mut html,
                "NBR 6118:2023",
                "M_d = γ_f · M_k   |   ξ = \\frac{x}{d} ≤ 0.45",
                Some("Verificação ao cisalhamento (V_d ≤ V_{Rd1}) e abertura de fissuras (w_k ≤ 0.30 mm)."),
            );
            let _ = writeln!(
                html,
                "<table><tr><th>Elemento</th><th>{} [kNm/m]</th><th>{} [kN/m]</th><th>{}</th><th>{} [cm²/m]</th><th>Armadura Detalhada</th><th>{} [kN/m]</th><th>{} [mm]</th></tr>",
                math_html("M_d"),
                math_html("V_d"),
                math_html("ξ = x/d"),
                math_html("A_s / A_{s,mín}"),
                math_html("V_{Rd1}"),
                math_html("w_k"),
            );
        } else {
            html.push_str("<h2>5. REINFORCED CONCRETE STRUCTURAL DESIGN (ABNT NBR 6118:2023)</h2>\n");
            let _ = writeln!(html, "<p><strong>Concrete &amp; Steel Parameters:</strong> {conc_params_str}</p>");
            push_html_eq_card(
                &mut html,
                "NBR 6118:2023",
                "M_d = γ_f · M_k   |   ξ = \\frac{x}{d} ≤ 0.45",
                Some("Shear resistance without stirrups (V_d ≤ V_{Rd1}) and crack width SLS (w_k ≤ 0.30 mm)."),
            );
            let _ = writeln!(
                html,
                "<table><tr><th>Element</th><th>{} [kNm/m]</th><th>{} [kN/m]</th><th>{}</th><th>{} [cm²/m]</th><th>Selected Rebar</th><th>{} [kN/m]</th><th>{} [mm]</th></tr>",
                math_html("M_d"),
                math_html("V_d"),
                math_html("ξ = x/d"),
                math_html("A_s / A_{s,min}"),
                math_html("V_{Rd1}"),
                math_html("w_k"),
            );
        }

        for sec in [&wall_reinf.stem_section, &wall_reinf.toe_section, &wall_reinf.heel_section] {
            let shear_lbl = match (sec.is_shear_ok, pt) {
                (true, true) => "ATENDE",
                (false, true) => "VERIF.",
                (true, false) => "OK",
                (false, false) => "CHECK",
            };
            let wk_lbl = match (sec.is_crack_width_ok, pt) {
                (true, _) => "OK",
                (false, true) => "ACIMA",
                (false, false) => "EXCEEDS",
            };
            let _ = writeln!(
                html,
                "<tr><td><strong>{}</strong></td><td>{:.2}</td><td>{:.2}</td><td>{:.3}</td><td>{:.2} / {:.2}</td><td><strong>{}</strong></td><td>{:.1} ({shear_lbl})</td><td>{:.3} ({wk_lbl})</td></tr>",
                escape_html(sec.localized_section_name(lang)),
                sec.md_knm_m,
                sec.vd_kn_m,
                sec.neutral_axis_xi,
                sec.as_calculated_cm2_m,
                sec.as_min_cm2_m,
                escape_html(&sec.recommendation.localized_description(lang)),
                sec.v_rd1_kn_m,
                sec.crack_width_wk_mm,
            );
        }
        html.push_str("</table>\n");
        let dist_lbl = if pt { "Armadura Horizontal de Retração e Distribuição" } else { "Horizontal Shrinkage &amp; Distribution Steel" };
        let _ = writeln!(
            html,
            "<p><strong>{dist_lbl}:</strong> {} → <strong>{}</strong></p>\n",
            math_html(&format!("A_{{s,dist}} = {:.2} cm²/m", wall_reinf.horizontal_distribution_as_cm2_m)),
            escape_html(&wall_reinf.horizontal_distribution_rebar.localized_description(lang)),
        );

        // 6. Study Notes, Verification & Legal Disclaimer
        if pt {
            html.push_str("<h2>6. NOTAS DE ESTUDO, CONFERÊNCIA EXTERNA E ISENÇÃO</h2>\n");
            let _ = writeln!(html, "<div class=\"box\">\n<p>{}</p>", escape_html(&project.notes));
            let _ = writeln!(
                html,
                "<p>Os cálculos apresentados utilizam as equações de referência de ABNT NBR 11682:2009, ABNT NBR 6122:2022 e ABNT NBR 6118:2023 apenas para estudo acadêmico, ensino e experimentação computacional, podendo conter falhas ou limitações numéricas. NÃO utilize este relatório como peça técnica definitiva em obras ou projetos reais sem conferência manual completa, validação cruzada em outras ferramentas consolidadas de mercado, ensaios de campo e supervisão formal de profissional habilitado.</p>"
            );
            let _ = writeln!(
                html,
                "<p>Software fornecido &quot;no estado em que se encontra&quot; (&quot;AS IS&quot;), com suporte prestado exclusivamente por e-mail (linceu_lighthouse@outlook.com) conforme disponibilidade e sem obrigação de urgência, ficando o autor integralmente isento de responsabilidade por uso indevido ou aplicação produtiva.</p>"
            );
            let _ = writeln!(
                html,
                "<p style=\"margin-top:26px;\">____________________________________________________<br><strong>{}</strong> | {} | {}</p>\n</div>",
                escape_html(project.effective_engineer_name()),
                escape_html(&project.crea_cau),
                escape_html(&project.art_rrt),
            );
            html.push_str("<div class=\"footer\">Relatório auxiliar de estudo gerado pelo GeoEstabil (Ref. ABNT NBR 11682 / 6122 / 6118 — Uso acadêmico AS-IS; exige conferência externa)</div>\n");
        } else {
            html.push_str("<h2>6. STUDY NOTES, EXTERNAL VERIFICATION &amp; DISCLAIMER</h2>\n");
            let _ = writeln!(html, "<div class=\"box\">\n<p>{}</p>", escape_html(&project.notes));
            let _ = writeln!(
                html,
                "<p>Computed outputs follow ABNT NBR 11682:2009, ABNT NBR 6122:2022, and ABNT NBR 6118:2023 reference equations strictly for academic study, teaching, and computational experimentation, and may contain bugs or numerical limitations. DO NOT use this report as definitive engineering documentation for real works or production projects without full manual verification, cross-checking against established commercial market tools, field testing, and licensed professional oversight.</p>"
            );
            let _ = writeln!(
                html,
                "<p>Provided strictly &quot;AS IS&quot; without warranties; support is offered exclusively via email (linceu_lighthouse@outlook.com) according to availability with no obligation of urgency, wholly exempting the author from liability for improper or production use.</p>"
            );
            let _ = writeln!(
                html,
                "<p style=\"margin-top:26px;\">____________________________________________________<br><strong>{}</strong> | {} | {}</p>\n</div>",
                escape_html(project.effective_engineer_name()),
                escape_html(&project.crea_cau),
                escape_html(&project.art_rrt),
            );
            html.push_str("<div class=\"footer\">Auxiliary study report generated by GeoEstabil (Ref. ABNT NBR 11682 / 6122 / 6118 — Academic AS-IS use; external verification required)</div>\n");
        }
        html.push_str("</body>\n</html>\n");
        html
    }

    /// Gera versão Markdown estruturada para pré-visualização interativa na Aba 5 (usa marcadores `[[DIAGRAM:...]]`).
    pub fn generate_markdown(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
    ) -> String {
        Self::generate_markdown_lang(project, slope_report, Language::PtBr)
    }

    pub fn generate_markdown_lang(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> String {
        let pt = lang == Language::PtBr;
        let wall_pressures = calculate_earth_pressures(&project.wall_model);
        let wall_stability = check_wall_external_stability(&project.wall_model);
        let wall_bearing = calculate_bearing_capacity(&project.wall_model, &wall_stability);
        let wall_reinf = design_concrete_reinforcement(&project.wall_model, &wall_stability);
        let conc = &project.wall_model.concrete;

        let pass_fail = |ok: bool| match (ok, pt) {
            (true, true) => "APROVADO",
            (false, true) => "REPROVADO",
            (true, false) => "APPROVED",
            (false, false) => "FAILED",
        };

        let mut md = String::with_capacity(8_192);
        if pt {
            let _ = writeln!(md, "# MEMORIAL DESCRITIVO E DE CÁLCULO — {}\n", project.title);
            let _ = writeln!(md, "| Campo do Projeto | Registro Técnico |");
            let _ = writeln!(md, "| :--- | :--- |");
            let _ = writeln!(md, "| **Cliente / Interessado** | {} |", project.client);
            let _ = writeln!(md, "| **Localização** | {} |", project.site_location);
            let _ = writeln!(md, "| **Responsável Técnico** | {} ({}) |", project.effective_engineer_name(), project.crea_cau);
            let _ = writeln!(md, "| **ART / RRT** | {} |", project.art_rrt);
            let _ = writeln!(md, "| **Data / Modo** | {} — {} |\n", project.date, project.active_mode.name(lang));

            md.push_str("## 1. Referências Normativas de Estudo (ABNT)\n");
            md.push_str("| Norma | Aplicação | Critério Adotado |\n");
            md.push_str("| :--- | :--- | :--- |\n");
            let _ = writeln!(md, "| **ABNT NBR 11682:2009** | Estabilidade de Encostas | Segurança Alta: $FS \\ge {:.2}$ |", project.slope_model.safety_level.min_fs());
            md.push_str("| **ABNT NBR 6122:2022** | Fundações e Contenções | $FS_{tomb} \\ge 1.50$, $FS_{desl} \\ge 1.50$, $e \\le B/6$, $\\sigma_{ponta} \\le \\sigma_{adm}$ |\n");
            md.push_str("| **ABNT NBR 6118:2023** | Estruturas de Concreto | $\\gamma_f = 1.40$, $\\xi \\le 0.45$, $\\rho_{mín} = 0.15\\%$, $V_{Rd1}$, $w_k \\le 0.30$ mm |\n\n");

            md.push_str("## 2. Camadas de Solo e Parâmetros Geotécnicos\n");
            md.push_str("| Camada | $\\gamma_{nat}$ [kN/m³] | $\\gamma_{sat}$ [kN/m³] | $c'$ [kPa] | $\\varphi'$ [°] | $\\sigma_{adm}$ [kPa] |\n");
            md.push_str("| :--- | :---: | :---: | :---: | :---: | :---: |\n");
            for l in &project.slope_model.layers {
                let _ = writeln!(md, "| {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |", l.name, l.gamma, l.gamma_sat, l.cohesion, l.friction_angle_deg, l.sigma_adm);
            }
            md.push('\n');

            md.push_str("## 3. Estabilidade de Taludes (Aba 2 — Equilíbrio Limite)\n");
            md.push_str("$$FS = \\frac{\\Sigma [ c' \\cdot \\Delta x + (W - u \\cdot \\Delta x + q \\cdot \\Delta x) \\cdot \\tan \\varphi' ] / m_\\alpha}{\\Sigma (W + q \\cdot \\Delta x) \\cdot \\sin \\alpha}$$\n\n");
            md.push_str("[[DIAGRAM:SLOPE_TAB2]]\n\n");
            if let Some(slope) = slope_report {
                md.push_str("| Método de Análise | $FS$ Obtido | $FS_{mín}$ Exigido | Status |\n");
                md.push_str("| :--- | :---: | :---: | :---: |\n");
                let _ = writeln!(md, "| **Bishop Simplificado** | **{:.3}** | {:.2} | {} |", slope.critical_circle.fs_bishop, slope.required_min_fs, pass_fail(slope.critical_circle.fs_bishop >= slope.required_min_fs));
                let _ = writeln!(md, "| Fellenius (Sueco) | {:.3} | {:.2} | {} |", slope.critical_circle.fs_fellenius, slope.required_min_fs, pass_fail(slope.critical_circle.fs_fellenius >= slope.required_min_fs));
                let _ = writeln!(md, "| Janbu Simplificado | {:.3} | {:.2} | {} |\n", slope.critical_circle.fs_janbu, slope.required_min_fs, pass_fail(slope.critical_circle.fs_janbu >= slope.required_min_fs));
                let _ = writeln!(md, "- **Círculo Crítico:** $X_c = {:.2}$ m, $Y_c = {:.2}$ m, $R = {:.2}$ m | Fatias: {} | Tempo: {:.1} ms\n", slope.critical_circle.center_x, slope.critical_circle.center_y, slope.critical_circle.radius, slope.critical_circle.slices.len(), slope.elapsed_ms);
            } else {
                md.push_str("- *Nenhuma varredura executada ainda na Aba 2.*\n\n");
            }

            let _ = writeln!(md, "## 4. Muro de Contenção e Estabilidade Externa — {} (Aba 3)\n", project.wall_model.wall_type.name(lang));
            md.push_str("$$FS_{tomb} = \\frac{\\Sigma M_{est}}{\\Sigma M_{tomb}} \\ge 1.50 \\quad|\\quad FS_{desl} = \\frac{\\Sigma F_{resist}}{E_{a,h}} \\ge 1.50$$\n\n");
            let md_stress_eq = if wall_stability.is_in_middle_third {
                "$$\\sigma_{ponta,calcanhar} = \\frac{N}{B} \\cdot (1 ± \\frac{6 \\cdot e}{B})$$\n\n"
            } else {
                "$$\\sigma_{ponta} = \\frac{2 \\cdot N}{3 \\cdot x_R}$$\n\n"
            };
            md.push_str(md_stress_eq);
            md.push_str("[[DIAGRAM:WALL_TAB3]]\n\n");
            md.push_str("| Grandeza / Verificação | Valor Calculado | Critério / Referência | Status |\n");
            md.push_str("| :--- | :--- | :--- | :---: |\n");
            let _ = writeln!(md, "| **Coeficientes de Empuxo** | $K_a = {:.4}$, $K_p = {:.4}$ | {} | INFO |", wall_pressures.ka, wall_pressures.kp, project.wall_model.theory.name(lang));
            let _ = writeln!(md, "| **Empuxo Ativo Horizontal** | $E_{{a,h}} = {:.2}$ kN/m ($y = {:.2}$ m) | $M_{{tomb}} = {:.2}$ kNm/m | INFO |", wall_pressures.ea_horizontal, wall_pressures.ea_arm_y, wall_pressures.overturning_moment);
            let _ = writeln!(md, "| **Segurança ao Tombamento** | $FS_{{tomb}} = {:.2}$ | $FS \\ge 1.50$ | {} |", wall_stability.fs_overturning, pass_fail(wall_stability.is_overturning_ok));
            let _ = writeln!(md, "| **Segurança ao Deslizamento** | $FS_{{desl}} = {:.2}$ | $FS \\ge 1.50$ | {} |", wall_stability.fs_sliding, pass_fail(wall_stability.is_sliding_ok));
            let _ = writeln!(md, "| **Excentricidade / Terço Médio** | $x_R = {:.2}$ m ($e = {:.3}$ m) | $e \\le B/6 = {:.3}$ m | {} |", wall_stability.resultant_position_xr, wall_stability.eccentricity, wall_stability.max_allowed_eccentricity, pass_fail(wall_stability.is_in_middle_third));
            let _ = writeln!(md, "| **Tensões na Base** | $\\sigma_{{ponta}} = {:.1}$ kPa / $\\sigma_{{calcanhar}} = {:.1}$ kPa | $\\sigma_{{adm}} = {:.1}$ kPa | {} |", wall_stability.sigma_toe_kpa, wall_stability.sigma_heel_kpa, wall_stability.sigma_adm_kpa, pass_fail(wall_stability.is_bearing_stress_ok));
            let _ = writeln!(md, "| **Capacidade de Carga (Hansen)** | $q_{{ult}} = {:.1}$ kPa ($FS = {:.2}$) | $FS \\ge 2.00$ | {} |\n", wall_bearing.q_ult_kpa, wall_bearing.fs_bearing, pass_fail(wall_bearing.is_bearing_ok));

            md.push_str("## 5. Dimensionamento Estrutural (ABNT NBR 6118:2023)\n");
            let _ = writeln!(md, "- **Materiais:** $f_{{ck}} = {:.0}$ MPa ($f_{{cd}} = {:.2}$ MPa) | $f_{{yk}} = {:.0}$ MPa ($f_{{yd}} = {:.2}$ MPa) | $c_{{nom}} = {:.0}$ mm\n", conc.fck_mpa, wall_reinf.fcd_mpa, conc.fyk_mpa, wall_reinf.fyd_mpa, conc.cover_nom_mm);
            md.push_str("$$M_d = \\gamma_f \\cdot M_k \\quad|\\quad \\xi = \\frac{x}{d} \\le 0.45$$\n\n");
            md.push_str("| Seção Estrutural | $M_d$ [kNm/m] | $V_d$ [kN/m] | $\\xi$ | $A_{{s,req}}$ [cm²/m] | Armadura Detalhada | $V_{{Rd1}}$ [kN/m] | $w_k$ [mm] |\n");
            md.push_str("| :--- | :---: | :---: | :---: | :---: | :--- | :---: | :---: |\n");
            for sec in [&wall_reinf.stem_section, &wall_reinf.toe_section, &wall_reinf.heel_section] {
                let _ = writeln!(md, "| {} | {:.1} | {:.1} | {:.3} | {:.2} | **{}** | {:.1} | {:.3} |", sec.localized_section_name(lang), sec.md_knm_m, sec.vd_kn_m, sec.neutral_axis_xi, sec.as_required_cm2_m, sec.recommendation.localized_description(lang), sec.v_rd1_kn_m, sec.crack_width_wk_mm);
            }
            let _ = writeln!(md, "\n- **Armadura Horizontal de Distribuição:** $A_{{s,dist}} = {:.2}$ cm²/m → **{}**\n", wall_reinf.horizontal_distribution_as_cm2_m, wall_reinf.horizontal_distribution_rebar.localized_description(lang));
            md.push_str("## 6. Notas de Estudo, Conferência e Isenção\n");
            let _ = writeln!(md, "- **Observações de Estudo:** {}", project.notes);
            md.push_str("- **Advertência Acadêmica e Isenção (AS IS):** Ferramenta auxiliar voltada exclusivamente a estudo acadêmico, ensino e experimentação; pode conter falhas de cálculo ou aproximações. NÃO usar de forma definitiva em produção sem conferência manual completa, validação em ferramentas consolidadas de mercado e acompanhamento por profissional habilitado. Suporte exclusivamente via e-mail (`linceu_lighthouse@outlook.com`), conforme disponibilidade e sem urgência.\n");
            let _ = writeln!(md, "- **Identificação:** **{}** | {} | {} | Data: {}\n", project.effective_engineer_name(), project.crea_cau, project.art_rrt, project.date);
        } else {
            let _ = writeln!(md, "# GEOTECHNICAL & STRUCTURAL CALCULATION REPORT — {}\n", project.title);
            let _ = writeln!(md, "| Project Field | Technical Record |");
            let _ = writeln!(md, "| :--- | :--- |");
            let _ = writeln!(md, "| **Client / Owner** | {} |", project.client);
            let _ = writeln!(md, "| **Site Location** | {} |", project.site_location);
            let _ = writeln!(md, "| **Lead Engineer** | {} ({}) |", project.effective_engineer_name(), project.crea_cau);
            let _ = writeln!(md, "| **Record / Study Ref.** | {} |", project.art_rrt);
            let _ = writeln!(md, "| **Date / Mode** | {} — {} |\n", project.date, project.active_mode.name(lang));

            md.push_str("## 1. Referenced Technical Standards (ABNT)\n");
            md.push_str("| Standard | Application Scope | Adopted Criterion |\n");
            md.push_str("| :--- | :--- | :--- |\n");
            let _ = writeln!(md, "| **ABNT NBR 11682:2009** | Slope Stability | High Safety Level: $FS \\ge {:.2}$ |", project.slope_model.safety_level.min_fs());
            md.push_str("| **ABNT NBR 6122:2022** | Foundations &amp; Walls | $FS_{ovt} \\ge 1.50$, $FS_{sld} \\ge 1.50$, $e \\le B/6$, $\\sigma_{toe} \\le \\sigma_{adm}$ |\n");
            md.push_str("| **ABNT NBR 6118:2023** | Reinforced Concrete | $\\gamma_f = 1.40$, $\\xi \\le 0.45$, $\\rho_{min} = 0.15\\%$, $V_{Rd1}$, $w_k \\le 0.30$ mm |\n\n");

            md.push_str("## 2. Soil Layers & Geotechnical Parameters\n");
            md.push_str("| Layer | $\\gamma_{nat}$ [kN/m³] | $\\gamma_{sat}$ [kN/m³] | $c'$ [kPa] | $\\varphi'$ [°] | $\\sigma_{adm}$ [kPa] |\n");
            md.push_str("| :--- | :---: | :---: | :---: | :---: | :---: |\n");
            for l in &project.slope_model.layers {
                let _ = writeln!(md, "| {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |", l.name, l.gamma, l.gamma_sat, l.cohesion, l.friction_angle_deg, l.sigma_adm);
            }
            md.push('\n');

            md.push_str("## 3. Slope Stability (Tab 2 — Limit Equilibrium)\n");
            md.push_str("$$FS = \\frac{\\Sigma [ c' \\cdot \\Delta x + (W - u \\cdot \\Delta x + q \\cdot \\Delta x) \\cdot \\tan \\varphi' ] / m_\\alpha}{\\Sigma (W + q \\cdot \\Delta x) \\cdot \\sin \\alpha}$$\n\n");
            md.push_str("[[DIAGRAM:SLOPE_TAB2]]\n\n");
            if let Some(slope) = slope_report {
                md.push_str("| Analysis Method | Computed $FS$ | Required $FS_{min}$ | Status |\n");
                md.push_str("| :--- | :---: | :---: | :---: |\n");
                let _ = writeln!(md, "| **Simplified Bishop** | **{:.3}** | {:.2} | {} |", slope.critical_circle.fs_bishop, slope.required_min_fs, pass_fail(slope.critical_circle.fs_bishop >= slope.required_min_fs));
                let _ = writeln!(md, "| Fellenius (Swedish) | {:.3} | {:.2} | {} |", slope.critical_circle.fs_fellenius, slope.required_min_fs, pass_fail(slope.critical_circle.fs_fellenius >= slope.required_min_fs));
                let _ = writeln!(md, "| Simplified Janbu | {:.3} | {:.2} | {} |\n", slope.critical_circle.fs_janbu, slope.required_min_fs, pass_fail(slope.critical_circle.fs_janbu >= slope.required_min_fs));
                let _ = writeln!(md, "- **Critical Slip Circle:** $X_c = {:.2}$ m, $Y_c = {:.2}$ m, $R = {:.2}$ m | Slices: {} | Elapsed: {:.1} ms\n", slope.critical_circle.center_x, slope.critical_circle.center_y, slope.critical_circle.radius, slope.critical_circle.slices.len(), slope.elapsed_ms);
            } else {
                md.push_str("- *No Limit Equilibrium sweep executed yet on Tab 2.*\n\n");
            }

            let _ = writeln!(md, "## 4. Retaining Wall & External Stability — {} (Tab 3)\n", project.wall_model.wall_type.name(lang));
            md.push_str("$$FS_{ovt} = \\frac{\\Sigma M_{res}}{\\Sigma M_{ovt}} \\ge 1.50 \\quad|\\quad FS_{sld} = \\frac{\\Sigma F_{res}}{E_{a,h}} \\ge 1.50$$\n\n");
            let md_stress_eq_en = if wall_stability.is_in_middle_third {
                "$$\\sigma_{toe,heel} = \\frac{N}{B} \\cdot (1 ± \\frac{6 \\cdot e}{B})$$\n\n"
            } else {
                "$$\\sigma_{toe} = \\frac{2 \\cdot N}{3 \\cdot x_R}$$\n\n"
            };
            md.push_str(md_stress_eq_en);
            md.push_str("[[DIAGRAM:WALL_TAB3]]\n\n");
            md.push_str("| Quantity / Check | Computed Value | Criterion / Reference | Status |\n");
            md.push_str("| :--- | :--- | :--- | :---: |\n");
            let _ = writeln!(md, "| **Earth Pressure Coeffs.** | $K_a = {:.4}$, $K_p = {:.4}$ | {} | INFO |", wall_pressures.ka, wall_pressures.kp, project.wall_model.theory.name(lang));
            let _ = writeln!(md, "| **Horizontal Active Thrust** | $E_{{a,h}} = {:.2}$ kN/m ($y = {:.2}$ m) | $M_{{ovt}} = {:.2}$ kNm/m | INFO |", wall_pressures.ea_horizontal, wall_pressures.ea_arm_y, wall_pressures.overturning_moment);
            let _ = writeln!(md, "| **Overturning Safety** | $FS_{{ovt}} = {:.2}$ | $FS \\ge 1.50$ | {} |", wall_stability.fs_overturning, pass_fail(wall_stability.is_overturning_ok));
            let _ = writeln!(md, "| **Base Sliding Safety** | $FS_{{sld}} = {:.2}$ | $FS \\ge 1.50$ | {} |", wall_stability.fs_sliding, pass_fail(wall_stability.is_sliding_ok));
            let _ = writeln!(md, "| **Eccentricity / Middle Third** | $x_R = {:.2}$ m ($e = {:.3}$ m) | $e \\le B/6 = {:.3}$ m | {} |", wall_stability.resultant_position_xr, wall_stability.eccentricity, wall_stability.max_allowed_eccentricity, pass_fail(wall_stability.is_in_middle_third));
            let _ = writeln!(md, "| **Base Bearing Stress** | $\\sigma_{{toe}} = {:.1}$ kPa / $\\sigma_{{heel}} = {:.1}$ kPa | $\\sigma_{{adm}} = {:.1}$ kPa | {} |", wall_stability.sigma_toe_kpa, wall_stability.sigma_heel_kpa, wall_stability.sigma_adm_kpa, pass_fail(wall_stability.is_bearing_stress_ok));
            let _ = writeln!(md, "| **Bearing Capacity (Hansen)** | $q_{{ult}} = {:.1}$ kPa ($FS = {:.2}$) | $FS \\ge 2.00$ | {} |\n", wall_bearing.q_ult_kpa, wall_bearing.fs_bearing, pass_fail(wall_bearing.is_bearing_ok));

            md.push_str("## 5. Reinforced Concrete Structural Design (ABNT NBR 6118:2023)\n");
            let _ = writeln!(md, "- **Materials:** $f_{{ck}} = {:.0}$ MPa ($f_{{cd}} = {:.2}$ MPa) | $f_{{yk}} = {:.0}$ MPa ($f_{{yd}} = {:.2}$ MPa) | $c_{{nom}} = {:.0}$ mm\n", conc.fck_mpa, wall_reinf.fcd_mpa, conc.fyk_mpa, wall_reinf.fyd_mpa, conc.cover_nom_mm);
            md.push_str("$$M_d = \\gamma_f \\cdot M_k \\quad|\\quad \\xi = \\frac{x}{d} \\le 0.45$$\n\n");
            md.push_str("| Structural Section | $M_d$ [kNm/m] | $V_d$ [kN/m] | $\\xi$ | $A_{{s,req}}$ [cm²/m] | Detailed Rebar | $V_{{Rd1}}$ [kN/m] | $w_k$ [mm] |\n");
            md.push_str("| :--- | :---: | :---: | :---: | :---: | :--- | :---: | :---: |\n");
            for sec in [&wall_reinf.stem_section, &wall_reinf.toe_section, &wall_reinf.heel_section] {
                let _ = writeln!(md, "| {} | {:.1} | {:.1} | {:.3} | {:.2} | **{}** | {:.1} | {:.3} |", sec.localized_section_name(lang), sec.md_knm_m, sec.vd_kn_m, sec.neutral_axis_xi, sec.as_required_cm2_m, sec.recommendation.localized_description(lang), sec.v_rd1_kn_m, sec.crack_width_wk_mm);
            }
            let _ = writeln!(md, "\n- **Horizontal Distribution Steel:** $A_{{s,dist}} = {:.2}$ cm²/m → **{}**\n", wall_reinf.horizontal_distribution_as_cm2_m, wall_reinf.horizontal_distribution_rebar.localized_description(lang));
            md.push_str("## 6. Study Notes, Verification & Disclaimer\n");
            let _ = writeln!(md, "- **Study Notes:** {}", project.notes);
            md.push_str("- **Academic Notice & Disclaimer (AS IS):** Auxiliary tool intended exclusively for academic study, teaching, and experimentation; may contain bugs or numerical approximations. DO NOT rely upon as definitive software in production without complete manual checking, cross-validation against established commercial market tools, and oversight by a licensed engineer. Support exclusively via email (`linceu_lighthouse@outlook.com`) per availability, without urgency.\n");
            let _ = writeln!(md, "- **Identification:** **{}** | {} | {} | Date: {}\n", project.effective_engineer_name(), project.crea_cau, project.art_rrt, project.date);
        }
        md
    }

    /// Gera documento Markdown completo para exportação em arquivo `.md`, embutindo ambos os diagramas SVG (Abas 2 e 3).
    pub fn generate_markdown_with_diagrams_lang(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> String {
        let pt = lang == Language::PtBr;
        let base_md = Self::generate_markdown_lang(project, slope_report, lang);
        let slope_fig = format!(
            "<figure>\n{}\n<figcaption>{}</figcaption>\n</figure>",
            VectorExporter::render_inline_slope_svg(project, slope_report, lang),
            if pt {
                "Figura 1 — Aba 2: Seção Geotécnica, Heatmap FS, Fatias e Círculo Crítico."
            } else {
                "Figure 1 — Tab 2: Geotechnical Profile, FS Heatmap, Slices & Critical Circle."
            }
        );
        let wall_fig = format!(
            "<figure>\n{}\n<figcaption>{}</figcaption>\n</figure>",
            VectorExporter::render_inline_wall_svg(project, lang),
            if pt {
                "Figura 2 — Aba 3: Muro de Contenção, Diagramas de Empuxo, Tensões na Base e Verificações."
            } else {
                "Figure 2 — Tab 3: Retaining Wall, Thrust Diagrams, Base Bearing Stress & Checks."
            }
        );
        base_md
            .replace("[[DIAGRAM:SLOPE_TAB2]]", &slope_fig)
            .replace("[[DIAGRAM:WALL_TAB3]]", &wall_fig)
    }

    pub fn generate_pdf(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
    ) -> Vec<u8> {
        Self::generate_pdf_lang(project, slope_report, Language::PtBr)
    }

    /// Gera documento PDF vetorial A4 nativo com tipografia matemática (sub/sobrescritos, `/Symbol`,
    /// frações empilhadas) e os diagramas vetoriais das Abas 2 (Talude+Heatmap+Fatias) e 3 (Muro+Empuxos+Base).
    pub fn generate_pdf_lang(
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> Vec<u8> {
        let pt = lang == Language::PtBr;
        let wall_pressures = calculate_earth_pressures(&project.wall_model);
        let wall_stability = check_wall_external_stability(&project.wall_model);
        let wall_bearing = calculate_bearing_capacity(&project.wall_model, &wall_stability);
        let wall_reinf = design_concrete_reinforcement(&project.wall_model, &wall_stability);

        let mut pdf = PdfDocument::new(lang);
        let subtitle = if pt {
            "Estabilidade de Encostas e Muros de Arrimo — Referências ABNT NBR 11682:2009 / 6122:2022 / 6118:2023"
        } else {
            "Slope Stability & Retaining Wall Study — Ref. ABNT NBR 11682:2009 / 6122:2022 / 6118:2023"
        };
        pdf.draw_banner(&project.title, subtitle);

        let status_txt = |ok: bool| match (ok, pt) {
            (true, true) => "APROVADO",
            (false, true) => "REPROVADO",
            (true, false) => "APPROVED",
            (false, false) => "FAILED",
        };

        if pt {
            pdf.draw_info_card(&[
                ("Cliente: ", &project.client),
                ("Local: ", &project.site_location),
                ("Engenheiro(a): ", &format!("{} ({})", project.effective_engineer_name(), project.crea_cau)),
                ("Data / ART: ", &format!("{} | {}", project.date, project.art_rrt)),
            ]);

            pdf.draw_section_heading("1. Referências Normativas de Estudo (ABNT)");
            pdf.draw_table(
                &["Norma Técnica", "Escopo Principal", "Critério de Segurança Exigido"],
                &[
                    vec![
                        "ABNT NBR 11682:2009".into(),
                        "Estabilidade de Encostas e Taludes".into(),
                        format!("FS ≥ {:.2} (Segurança Alta)", project.slope_model.safety_level.min_fs()),
                    ],
                    vec![
                        "ABNT NBR 6122:2022".into(),
                        "Fundações Rasas e Muros de Arrimo".into(),
                        "FS_{tomb} ≥ 1.50, FS_{desl} ≥ 1.50, e ≤ B/6, σ_{ponta} ≤ σ_{adm}".into(),
                    ],
                    vec![
                        "ABNT NBR 6118:2023".into(),
                        "Estruturas de Concreto Armado".into(),
                        "ELU (γ_f = 1.40, ξ ≤ 0.45), V_{Rd1}, w_k ≤ 0.30 mm".into(),
                    ],
                ],
                &[0.24, 0.36, 0.40],
            );

            pdf.draw_section_heading("2. Caracterização Geotécnica e Parâmetros dos Solos");
            let soil_rows: Vec<Vec<String>> = project
                .slope_model
                .layers
                .iter()
                .map(|l| {
                    vec![
                        l.name.clone(),
                        l.description.clone(),
                        format!("{:.1}", l.gamma),
                        format!("{:.1}", l.gamma_sat),
                        format!("{:.1}", l.cohesion),
                        format!("{:.1}", l.friction_angle_deg),
                        format!("{:.1}", l.sigma_adm),
                    ]
                })
                .collect();
            pdf.draw_table(
                &["Camada", "Descrição", "γ_{nat}", "γ_{sat}", "c' [kPa]", "φ' [°]", "σ_{adm} [kPa]"],
                &soil_rows,
                &[0.16, 0.28, 0.10, 0.10, 0.12, 0.10, 0.14],
            );
        } else {
            pdf.draw_info_card(&[
                ("Client: ", &project.client),
                ("Location: ", &project.site_location),
                ("Engineer: ", &format!("{} ({})", project.effective_engineer_name(), project.crea_cau)),
                ("Date / Filing: ", &format!("{} | {}", project.date, project.art_rrt)),
            ]);

            pdf.draw_section_heading("1. Referenced Technical Standards (ABNT)");
            pdf.draw_table(
                &["Technical Standard", "Engineering Scope", "Required Safety Criterion"],
                &[
                    vec![
                        "ABNT NBR 11682:2009".into(),
                        "Slope Stability & Embankments".into(),
                        format!("FS ≥ {:.2} (High Risk Level)", project.slope_model.safety_level.min_fs()),
                    ],
                    vec![
                        "ABNT NBR 6122:2022".into(),
                        "Shallow Foundations & Retaining Walls".into(),
                        "FS_{ovt} ≥ 1.50, FS_{sld} ≥ 1.50, e ≤ B/6, σ_{toe} ≤ σ_{adm}".into(),
                    ],
                    vec![
                        "ABNT NBR 6118:2023".into(),
                        "Reinforced Concrete Structural Design".into(),
                        "ULS (γ_f = 1.40, ξ ≤ 0.45), V_{Rd1}, w_k ≤ 0.30 mm".into(),
                    ],
                ],
                &[0.24, 0.36, 0.40],
            );

            pdf.draw_section_heading("2. Geotechnical Characterization & Soil Parameters");
            let soil_rows: Vec<Vec<String>> = project
                .slope_model
                .layers
                .iter()
                .map(|l| {
                    vec![
                        l.name.clone(),
                        l.description.clone(),
                        format!("{:.1}", l.gamma),
                        format!("{:.1}", l.gamma_sat),
                        format!("{:.1}", l.cohesion),
                        format!("{:.1}", l.friction_angle_deg),
                        format!("{:.1}", l.sigma_adm),
                    ]
                })
                .collect();
            pdf.draw_table(
                &["Layer", "Description", "γ_{nat}", "γ_{sat}", "c' [kPa]", "φ' [°]", "σ_{adm} [kPa]"],
                &soil_rows,
                &[0.16, 0.28, 0.10, 0.10, 0.12, 0.10, 0.14],
            );
        }

        // Section 3: Slope Stability (Tab 2) — diagram always emitted!
        if pt {
            pdf.draw_section_heading("3. Análise de Estabilidade de Taludes (Aba 2 — Equilíbrio Limite)");
            pdf.draw_equation_box(
                "Bishop",
                "FS = \\frac{Σ [ c' · Δx + (W - u · Δx + q · Δx) · tan φ' ] / m_α}{Σ (W + q · Δx) · sin α}",
                Some("Onde m_α = cos α + (sin α · tan φ') / FS (método das fatias de equilíbrio limite)."),
            );
            pdf.draw_slope_vector_figure(
                project,
                slope_report,
                "Figura 1 — Aba 2: Seção Geotécnica, Heatmap FS, Fatias e Círculo Crítico de Ruptura",
            );
        } else {
            pdf.draw_section_heading("3. Slope Stability Analysis (Tab 2 — Limit Equilibrium)");
            pdf.draw_equation_box(
                "Bishop",
                "FS = \\frac{Σ [ c' · Δx + (W - u · Δx + q · Δx) · tan φ' ] / m_α}{Σ (W + q · Δx) · sin α}",
                Some("Where m_α = cos α + (sin α · tan φ') / FS (iterative limit-equilibrium slices method)."),
            );
            pdf.draw_slope_vector_figure(
                project,
                slope_report,
                "Figure 1 — Tab 2: Geotechnical Cross-Section, FS Heatmap, Slices & Critical Slip Circle",
            );
        }

        if let Some(slope) = slope_report {
            if pt {
                pdf.draw_info_card(&[
                    (
                        "Centro Crítico: ",
                        &format!(
                            "X_c = {:.2} m, Y_c = {:.2} m, R = {:.2} m",
                            slope.critical_circle.center_x,
                            slope.critical_circle.center_y,
                            slope.critical_circle.radius
                        ),
                    ),
                    (
                        "Varredura Paramétrica: ",
                        &format!(
                            "{} círculos ({} válidos) em {:.1} ms",
                            slope.total_circles_evaluated, slope.valid_circles_count, slope.elapsed_ms
                        ),
                    ),
                ]);
                pdf.draw_table(
                    &["Método de Equilíbrio Limite", "FS Calculado", "FS_{mín} Exigido", "Status"],
                    &[
                        vec![
                            "Bishop Simplificado".into(),
                            format!("{:.3}", slope.critical_circle.fs_bishop),
                            format!("{:.2}", slope.required_min_fs),
                            status_txt(slope.critical_circle.fs_bishop >= slope.required_min_fs).into(),
                        ],
                        vec![
                            "Fellenius (Ordinário / Sueco)".into(),
                            format!("{:.3}", slope.critical_circle.fs_fellenius),
                            format!("{:.2}", slope.required_min_fs),
                            status_txt(slope.critical_circle.fs_fellenius >= slope.required_min_fs).into(),
                        ],
                        vec![
                            "Janbu Simplificado".into(),
                            format!("{:.3}", slope.critical_circle.fs_janbu),
                            format!("{:.2}", slope.required_min_fs),
                            status_txt(slope.critical_circle.fs_janbu >= slope.required_min_fs).into(),
                        ],
                    ],
                    &[0.38, 0.20, 0.22, 0.20],
                );
            } else {
                pdf.draw_info_card(&[
                    (
                        "Critical Circle: ",
                        &format!(
                            "X_c = {:.2} m, Y_c = {:.2} m, R = {:.2} m",
                            slope.critical_circle.center_x,
                            slope.critical_circle.center_y,
                            slope.critical_circle.radius
                        ),
                    ),
                    (
                        "Grid Sweep: ",
                        &format!(
                            "{} circles ({} valid) in {:.1} ms",
                            slope.total_circles_evaluated, slope.valid_circles_count, slope.elapsed_ms
                        ),
                    ),
                ]);
                pdf.draw_table(
                    &["Limit Equilibrium Method", "Computed FS", "Required FS_{min}", "Status"],
                    &[
                        vec![
                            "Simplified Bishop".into(),
                            format!("{:.3}", slope.critical_circle.fs_bishop),
                            format!("{:.2}", slope.required_min_fs),
                            status_txt(slope.critical_circle.fs_bishop >= slope.required_min_fs).into(),
                        ],
                        vec![
                            "Fellenius (Ordinary / Swedish)".into(),
                            format!("{:.3}", slope.critical_circle.fs_fellenius),
                            format!("{:.2}", slope.required_min_fs),
                            status_txt(slope.critical_circle.fs_fellenius >= slope.required_min_fs).into(),
                        ],
                        vec![
                            "Simplified Janbu".into(),
                            format!("{:.3}", slope.critical_circle.fs_janbu),
                            format!("{:.2}", slope.required_min_fs),
                            status_txt(slope.critical_circle.fs_janbu >= slope.required_min_fs).into(),
                        ],
                    ],
                    &[0.38, 0.20, 0.22, 0.20],
                );
            }

            let slice_headers: &[&str] = if pt {
                &["Fatia", "x_{méd} [m]", "Δx [m]", "h [m]", "W [kN/m]", "α [°]", "u [kPa]", "c' [kPa]", "φ' [°]"]
            } else {
                &["Slice", "x_{mid} [m]", "Δx [m]", "h [m]", "W [kN/m]", "α [°]", "u [kPa]", "c' [kPa]", "φ' [°]"]
            };
            pdf.draw_sub_heading(if pt {
                "Discretização das Fatias do Círculo Crítico:"
            } else {
                "Critical Slip Surface Slice Discretization:"
            });
            let slice_rows: Vec<Vec<String>> = slope
                .critical_circle
                .slices
                .iter()
                .map(|s| {
                    vec![
                        format!("{}", s.index),
                        format!("{:.2}", s.x_mid),
                        format!("{:.2}", s.width),
                        format!("{:.2}", s.height),
                        format!("{:.1}", s.weight),
                        format!("{:.1}", s.base_angle_deg),
                        format!("{:.1}", s.pore_pressure),
                        format!("{:.1}", s.cohesion),
                        format!("{:.1}", s.friction_angle_deg),
                    ]
                })
                .collect();
            pdf.draw_table(slice_headers, &slice_rows, &[0.08, 0.13, 0.10, 0.10, 0.13, 0.11, 0.11, 0.12, 0.12]);
        }

        // Section 4 & 5: Retaining Wall (Tab 3) & Structural Concrete Design
        let conc = &project.wall_model.concrete;
        let conc_banner = format!(
            "f_{{ck}} = {:.0} MPa (f_{{cd}} = {:.2} MPa) | f_{{yk}} = {:.0} MPa (f_{{yd}} = {:.2} MPa) | c_{{nom}} = {:.0} mm",
            conc.fck_mpa, wall_reinf.fcd_mpa, conc.fyk_mpa, wall_reinf.fyd_mpa, conc.cover_nom_mm
        );
        if pt {
            pdf.draw_section_heading("4. Muro de Contenção e Estabilidade Externa (Aba 3 — NBR 6122:2022)");
            let wall_stress_eq = if wall_stability.is_in_middle_third {
                "σ_{ponta,calcanhar} = \\frac{N}{B} · (1 ± \\frac{6 · e}{B})"
            } else {
                "σ_{ponta} = \\frac{2 · N}{3 · x_R}"
            };
            pdf.draw_equation_box(
                "NBR 6122",
                "FS_{tomb} = \\frac{∑ M_{est}}{∑ M_{tomb}} ≥ 1.50   |   FS_{desl} = \\frac{∑ F_{resist}}{E_{a,h}} ≥ 1.50",
                Some(&format!("Empuxo ativo resultante: K_a = {:.4}, K_p = {:.4}, E_{{a,h}} = {:.2} kN/m (M_{{tomb}} = {:.2} kNm/m)", wall_pressures.ka, wall_pressures.kp, wall_pressures.ea_horizontal, wall_pressures.overturning_moment)),
            );
            pdf.draw_equation_box(
                "Base",
                wall_stress_eq,
                Some(if wall_stability.is_in_middle_third {
                    "Condição de base totalmente comprimida (excentricidade dentro do terço médio: e ≤ B/6)."
                } else {
                    "Base parcialmente tracionada (descolamento no calcanhar; comprimento efetivo comprimido = 3 · x_R)."
                }),
            );
            pdf.draw_wall_vector_figure(
                project,
                "Figura 2 — Aba 3: Geometria do Muro, Empuxos Laterais E_{a,h} / E_p, Tensões σ_{ponta} / σ_{calcanhar} e HUD",
            );
            pdf.draw_table(
                &["Verificação Geotécnica", "Valor Obtido", "Critério", "Status"],
                &[
                    vec![
                        "Tombamento".into(),
                        format!(
                            "FS_{{tomb}} = {:.2} (M_{{est}}={:.1}, M_{{tomb}}={:.1})",
                            wall_stability.fs_overturning, wall_stability.total_stabilizing_moment, wall_stability.total_overturning_moment
                        ),
                        "FS ≥ 1.50".into(),
                        status_txt(wall_stability.is_overturning_ok).into(),
                    ],
                    vec![
                        "Deslizamento".into(),
                        format!(
                            "FS_{{desl}} = {:.2} (F_{{res}}={:.1}, F_{{atu}}={:.1})",
                            wall_stability.fs_sliding, wall_stability.total_resisting_horizontal_force, wall_stability.total_driving_horizontal_force
                        ),
                        "FS ≥ 1.50".into(),
                        status_txt(wall_stability.is_sliding_ok).into(),
                    ],
                    vec![
                        "Terço Médio / Excentricidade".into(),
                        format!("x_R = {:.2} m | e = {:.3} m", wall_stability.resultant_position_xr, wall_stability.eccentricity),
                        format!("e ≤ B/6 = {:.3} m", wall_stability.max_allowed_eccentricity),
                        status_txt(wall_stability.is_in_middle_third).into(),
                    ],
                    vec![
                        "Tensões na Base".into(),
                        format!("σ_{{ponta}} = {:.1} kPa (σ_{{calcanhar}} = {:.1} kPa)", wall_stability.sigma_toe_kpa, wall_stability.sigma_heel_kpa),
                        format!("σ_{{máx}} ≤ {:.1} kPa", wall_stability.sigma_adm_kpa),
                        status_txt(wall_stability.is_bearing_stress_ok).into(),
                    ],
                    vec![
                        "Capacidade de Carga (Hansen)".into(),
                        format!("q_{{ult}} = {:.1} kPa (FS = {:.2})", wall_bearing.q_ult_kpa, wall_bearing.fs_bearing),
                        "FS ≥ 2.00".into(),
                        status_txt(wall_bearing.is_bearing_ok).into(),
                    ],
                ],
                &[0.24, 0.42, 0.18, 0.16],
            );

            pdf.draw_section_heading("5. Dimensionamento Estrutural em Concreto Armado (NBR 6118:2023)");
            pdf.draw_info_card(&[("Materiais: ", &conc_banner)]);
            pdf.draw_equation_box(
                "NBR 6118",
                "ξ = \\frac{x}{d} ≤ 0.45   |   M_d = γ_f · M_k",
                Some("Verificação da ductilidade na flexão simples, força cortante V_d ≤ V_{Rd1} e fissuração w_k ≤ 0.30 mm."),
            );
            let reinf_rows: Vec<Vec<String>> = [&wall_reinf.stem_section, &wall_reinf.toe_section, &wall_reinf.heel_section]
                .iter()
                .map(|sec| {
                    vec![
                        sec.localized_section_name(lang).into(),
                        format!("{:.1} / {:.1}", sec.md_knm_m, sec.vd_kn_m),
                        format!("{:.3}", sec.neutral_axis_xi),
                        format!("{:.2} / {:.2}", sec.as_calculated_cm2_m, sec.as_min_cm2_m),
                        sec.recommendation.localized_description(lang),
                        format!("{:.1} kN/m", sec.v_rd1_kn_m),
                        format!("{:.3} mm", sec.crack_width_wk_mm),
                    ]
                })
                .collect();
            pdf.draw_table(
                &["Elemento", "M_d / V_d", "ξ", "A_s / A_{s,mín}", "Armadura", "V_{Rd1}", "w_k"],
                &reinf_rows,
                &[0.18, 0.13, 0.08, 0.14, 0.25, 0.12, 0.10],
            );

            pdf.draw_section_heading("6. Notas de Estudo, Conferência Externa e Isenção");
            pdf.draw_bullet_list(&[
                project.notes.clone(),
                format!("Armadura horizontal de retração e distribuição: A_{{s,dist}} = {:.2} cm²/m → {}", wall_reinf.horizontal_distribution_as_cm2_m, wall_reinf.horizontal_distribution_rebar.localized_description(lang)),
                "Ferramenta auxiliar de estudo acadêmico (ref. ABNT NBR 11682:2009, 6122:2022 e 6118:2023), fornecida AS-IS; exige conferência manual e validação em ferramentas consolidadas antes de uso profissional. Suporte apenas por e-mail (linceu_lighthouse@outlook.com), conforme disponibilidade e sem urgência.".into(),
                format!("Identificação: {} | {} | Registro: {} | Data: {}", project.effective_engineer_name(), project.crea_cau, project.art_rrt, project.date),
            ]);
        } else {
            pdf.draw_section_heading("4. Retaining Wall & External Stability (Tab 3 — NBR 6122:2022)");
            let wall_stress_eq = if wall_stability.is_in_middle_third {
                "σ_{toe,heel} = \\frac{N}{B} · (1 ± \\frac{6 · e}{B})"
            } else {
                "σ_{toe} = \\frac{2 · N}{3 · x_R}"
            };
            pdf.draw_equation_box(
                "NBR 6122",
                "FS_{ovt} = \\frac{∑ M_{res}}{∑ M_{ovt}} ≥ 1.50   |   FS_{sld} = \\frac{∑ F_{res}}{E_{a,h}} ≥ 1.50",
                Some(&format!("Resultant earth thrust: K_a = {:.4}, K_p = {:.4}, E_{{a,h}} = {:.2} kN/m (M_{{ovt}} = {:.2} kNm/m)", wall_pressures.ka, wall_pressures.kp, wall_pressures.ea_horizontal, wall_pressures.overturning_moment)),
            );
            pdf.draw_equation_box(
                "Base",
                wall_stress_eq,
                Some(if wall_stability.is_in_middle_third {
                    "Full base compression (resultant falls inside the middle third: e ≤ B/6)."
                } else {
                    "Cracked heel uplift condition (triangular compression distributed over 3 · x_R)."
                }),
            );
            pdf.draw_wall_vector_figure(
                project,
                "Figure 2 — Tab 3: Retaining Wall Geometry, Thrusts E_{a,h} / E_p, Base Stresses σ_{toe} / σ_{heel} & Checks HUD",
            );
            pdf.draw_table(
                &["Geotechnical Check", "Computed Result", "Criterion", "Status"],
                &[
                    vec![
                        "Overturning".into(),
                        format!(
                            "FS_{{ovt}} = {:.2} (M_{{stab}}={:.1}, M_{{ovt}}={:.1})",
                            wall_stability.fs_overturning, wall_stability.total_stabilizing_moment, wall_stability.total_overturning_moment
                        ),
                        "FS ≥ 1.50".into(),
                        status_txt(wall_stability.is_overturning_ok).into(),
                    ],
                    vec![
                        "Base Sliding".into(),
                        format!(
                            "FS_{{sld}} = {:.2} (F_{{res}}={:.1}, F_{{drv}}={:.1})",
                            wall_stability.fs_sliding, wall_stability.total_resisting_horizontal_force, wall_stability.total_driving_horizontal_force
                        ),
                        "FS ≥ 1.50".into(),
                        status_txt(wall_stability.is_sliding_ok).into(),
                    ],
                    vec![
                        "Middle Third / Eccentricity".into(),
                        format!("x_R = {:.2} m | e = {:.3} m", wall_stability.resultant_position_xr, wall_stability.eccentricity),
                        format!("e ≤ B/6 = {:.3} m", wall_stability.max_allowed_eccentricity),
                        status_txt(wall_stability.is_in_middle_third).into(),
                    ],
                    vec![
                        "Base Bearing Stress".into(),
                        format!("σ_{{toe}} = {:.1} kPa (σ_{{heel}} = {:.1} kPa)", wall_stability.sigma_toe_kpa, wall_stability.sigma_heel_kpa),
                        format!("σ_{{max}} ≤ {:.1} kPa", wall_stability.sigma_adm_kpa),
                        status_txt(wall_stability.is_bearing_stress_ok).into(),
                    ],
                    vec![
                        "Bearing Capacity (Hansen)".into(),
                        format!("q_{{ult}} = {:.1} kPa (FS = {:.2})", wall_bearing.q_ult_kpa, wall_bearing.fs_bearing),
                        "FS ≥ 2.00".into(),
                        status_txt(wall_bearing.is_bearing_ok).into(),
                    ],
                ],
                &[0.24, 0.42, 0.18, 0.16],
            );

            pdf.draw_section_heading("5. Reinforced Concrete Structural Design (NBR 6118:2023)");
            pdf.draw_info_card(&[("Materials: ", &conc_banner)]);
            pdf.draw_equation_box(
                "NBR 6118",
                "ξ = \\frac{x}{d} ≤ 0.45   |   M_d = γ_f · M_k",
                Some("ULS flexural ductility control, unreinforced shear capacity V_d ≤ V_{Rd1}, and crack width w_k ≤ 0.30 mm."),
            );
            let reinf_rows: Vec<Vec<String>> = [&wall_reinf.stem_section, &wall_reinf.toe_section, &wall_reinf.heel_section]
                .iter()
                .map(|sec| {
                    vec![
                        sec.localized_section_name(lang).into(),
                        format!("{:.1} / {:.1}", sec.md_knm_m, sec.vd_kn_m),
                        format!("{:.3}", sec.neutral_axis_xi),
                        format!("{:.2} / {:.2}", sec.as_calculated_cm2_m, sec.as_min_cm2_m),
                        sec.recommendation.localized_description(lang),
                        format!("{:.1} kN/m", sec.v_rd1_kn_m),
                        format!("{:.3} mm", sec.crack_width_wk_mm),
                    ]
                })
                .collect();
            pdf.draw_table(
                &["Element", "M_d / V_d", "ξ", "A_s / A_{s,min}", "Rebar", "V_{Rd1}", "w_k"],
                &reinf_rows,
                &[0.18, 0.13, 0.08, 0.14, 0.25, 0.12, 0.10],
            );

            pdf.draw_section_heading("6. Study Notes, External Verification & Disclaimer");
            pdf.draw_bullet_list(&[
                project.notes.clone(),
                format!("Horizontal distribution steel: A_{{s,dist}} = {:.2} cm²/m → {}", wall_reinf.horizontal_distribution_as_cm2_m, wall_reinf.horizontal_distribution_rebar.localized_description(lang)),
                "Auxiliary academic study tool (ref. ABNT NBR 11682:2009, 6122:2022, and 6118:2023) supplied AS-IS; requires full manual and established market-tool verification before professional use. Email-only support (linceu_lighthouse@outlook.com) by availability with no urgency.".into(),
                format!("Identification: {} | {} | Record: {} | Date: {}", project.effective_engineer_name(), project.crea_cau, project.art_rrt, project.date),
            ]);
        }

        pdf.finish()
    }

    pub fn write_html_or_pdf(
        path: &Path,
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> Result<PathBuf, String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        match ext.as_deref() {
            Some("pdf") => {
                let bytes = Self::generate_pdf_lang(project, slope_report, lang);
                std::fs::write(path, &bytes).map_err(|e| e.to_string())?;
                Ok(path.to_path_buf())
            }
            Some("html") | Some("htm") => {
                let html = Self::generate_html_lang(project, slope_report, lang);
                std::fs::write(path, html.as_bytes()).map_err(|e| e.to_string())?;
                Ok(path.to_path_buf())
            }
            _ => {
                let target = path.with_extension("pdf");
                let bytes = Self::generate_pdf_lang(project, slope_report, lang);
                std::fs::write(&target, &bytes).map_err(|e| e.to_string())?;
                Ok(target)
            }
        }
    }

    pub fn write_markdown(
        path: &Path,
        project: &GeoProject,
        slope_report: Option<&SlopeAnalysisReport>,
        lang: Language,
    ) -> Result<PathBuf, String> {
        let target = if path.extension().is_none() {
            path.with_extension("md")
        } else {
            path.to_path_buf()
        };
        let md = Self::generate_markdown_with_diagrams_lang(project, slope_report, lang);
        std::fs::write(&target, md.as_bytes()).map_err(|e| e.to_string())?;
        Ok(target)
    }
}
