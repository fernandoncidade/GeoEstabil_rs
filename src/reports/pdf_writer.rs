use crate::core::project::GeoProject;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::engine::wall::bearing_capacity::calculate_bearing_capacity;
use crate::engine::wall::earth_pressure::calculate_earth_pressures;
use crate::engine::wall::stability::check_wall_external_stability;
use crate::locale::translations::Language;
use crate::ui::theme::GeoEstabilTheme;
use crate::utils::export_dxf_svg::{split_math_runs, ScriptKind};
use std::fmt::Write as _;

const PAGE_W: f64 = 595.28; // A4 width (pt)
const PAGE_H: f64 = 841.89; // A4 height (pt)
const MARGIN_X: f64 = 42.0;
const MARGIN_TOP: f64 = 46.0;
const MARGIN_BOTTOM: f64 = 46.0;
const USABLE_W: f64 = PAGE_W - 2.0 * MARGIN_X; // 511.28 pt

#[derive(Clone, Copy, PartialEq, Eq)]
enum PdfFont {
    Helvetica,
    HelveticaBold,
    Symbol,
    HelveticaOblique,
}

impl PdfFont {
    fn tag(self) -> &'static str {
        match self {
            PdfFont::Helvetica => "/F1",
            PdfFont::HelveticaBold => "/F2",
            PdfFont::Symbol => "/F3",
            PdfFont::HelveticaOblique => "/F4",
        }
    }
}

/// Normaliza expressões LaTeX/Unicode para tokenização limpa por `split_math_runs`.
pub(crate) fn normalize_math_symbols(raw: &str) -> String {
    let mut s = raw
        .replace("\\sigma", "σ")
        .replace("\\gamma", "γ")
        .replace("\\varphi", "φ")
        .replace("\\phi", "φ")
        .replace("\\alpha", "α")
        .replace("\\beta", "β")
        .replace("\\delta", "δ")
        .replace("\\theta", "θ")
        .replace("\\eta", "η")
        .replace("\\xi", "ξ")
        .replace("\\rho", "ρ")
        .replace("\\tau", "τ")
        .replace("\\psi", "ψ")
        .replace("\\Delta", "Δ")
        .replace("\\Sigma", "Σ")
        .replace("\\cdot", "·")
        .replace("\\times", "×")
        .replace("\\le", "≤")
        .replace("\\ge", "≥")
        .replace("\\neq", "≠")
        .replace("\\approx", "≈")
        .replace("\\to", "→")
        .replace("\\rightarrow", "→")
        .replace("\\%", "%")
        .replace("\\quad", "   ")
        .replace("\\qquad", "     ")
        .replace("\\left", "")
        .replace("\\right", "")
        .replace("₀", "_0")
        .replace("₁", "_1")
        .replace("₂", "_2")
        .replace("₃", "_3")
        .replace("₄", "_4")
        .replace("₅", "_5")
        .replace("₆", "_6")
        .replace("₇", "_7")
        .replace("₈", "_8")
        .replace("₉", "_9")
        .replace('✔', "[OK]")
        .replace('✓', "[OK]")
        .replace('✖', "[X]")
        .replace('⚠', "[!]")
        .replace("✅", "[OK]")
        .replace("⚠️", "[!]");
    while let Some(pos) = s.find("\\text{") {
        let after = &s[pos + "\\text".len()..];
        if let Some((inner, rest)) = read_braced_group(after) {
            s = format!("{}{}{}", &s[..pos], inner, rest);
        } else {
            break;
        }
    }
    s
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EqSegment {
    Text(String),
    Fraction { num: String, den: String },
}

/// Segmenta expressões matemáticas com múltiplas frações `\frac{num}{den}` (ex: `σ = \frac{N}{B} · (1 ± \frac{6e}{B})`)
/// ou fração simples `[LHS =] Num / Den` em segmentos sequenciais renderizáveis.
pub(crate) fn split_equation_segments(expr: &str) -> Vec<EqSegment> {
    let trimmed = expr.trim();
    if trimmed.contains("\\frac{") {
        let mut segs = Vec::new();
        let mut cursor = trimmed;
        while let Some(pos) = cursor.find("\\frac{") {
            let before = &cursor[..pos];
            if !before.is_empty() {
                segs.push(EqSegment::Text(before.to_string()));
            }
            let after = &cursor[pos + "\\frac".len()..];
            if let Some((num, rest1)) = read_braced_group(after) {
                let rest1_trim = rest1.trim_start();
                if let Some((den, rest2)) = read_braced_group(rest1_trim) {
                    segs.push(EqSegment::Fraction {
                        num: num.trim().to_string(),
                        den: den.trim().to_string(),
                    });
                    cursor = rest2;
                    continue;
                }
            }
            segs.push(EqSegment::Text(cursor.to_string()));
            cursor = "";
            break;
        }
        if !cursor.is_empty() {
            segs.push(EqSegment::Text(cursor.to_string()));
        }
        if !segs.is_empty() {
            return segs;
        }
    }

    if let Some((lhs, num, den, suffix)) = extract_fraction_components(trimmed) {
        let mut segs = Vec::new();
        if !lhs.is_empty() {
            segs.push(EqSegment::Text(format!("{lhs} ")));
        }
        segs.push(EqSegment::Fraction { num, den });
        if !suffix.is_empty() {
            segs.push(EqSegment::Text(format!(" {suffix}")));
        }
        return segs;
    }

    vec![EqSegment::Text(trimmed.to_string())]
}

/// Extrai componentes `(prefix_lhs, numerator, denominator, suffix)` de `\frac{num}{den}`
/// ou de expressões do tipo `[LHS =] Num / Den` para renderização em fração empilhada.
pub(crate) fn extract_fraction_components(expr: &str) -> Option<(String, String, String, String)> {
    let trimmed = expr.trim();
    if let Some(pos) = trimmed.find("\\frac{") {
        let prefix = trimmed[..pos].trim_end().to_string();
        let after = &trimmed[pos + "\\frac".len()..];
        let (num, rest1) = read_braced_group(after)?;
        let (den, rest2) = read_braced_group(rest1.trim_start())?;
        return Some((
            prefix,
            num.trim().to_string(),
            den.trim().to_string(),
            rest2.trim().to_string(),
        ));
    }

    if let Some(eq_idx) = trimmed.find('=') {
        let lhs = trimmed[..=eq_idx].trim().to_string();
        let rhs = trimmed[eq_idx + 1..].trim();
        if let Some((num, den)) = split_top_level_slash(rhs) {
            return Some((lhs, num, den, String::new()));
        }
    } else if let Some((num, den)) = split_top_level_slash(trimmed) {
        return Some((String::new(), num, den, String::new()));
    }
    None
}

fn read_braced_group(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    if *bytes.first()? != b'{' {
        return None;
    }
    let mut depth = 0usize;
    for (idx, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((&s[1..idx], &s[idx + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level_slash(rhs: &str) -> Option<(String, String)> {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut slash_pos: Option<usize> = None;
    for (idx, ch) in rhs.char_indices() {
        match ch {
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '/' if paren == 0 && bracket == 0 && brace == 0 => {
                // Evita quebrar unidades curtas como "kN/m" ou "B/6"
                let before = rhs[..idx].trim_end();
                let after = rhs[idx + 1..].trim_start();
                if before.len() >= 4 && after.len() >= 4 && slash_pos.is_none() {
                    slash_pos = Some(idx);
                }
            }
            _ => {}
        }
    }
    let idx = slash_pos?;
    let mut num = rhs[..idx].trim().to_string();
    let mut den = rhs[idx + 1..].trim().to_string();
    if num.starts_with('[') && num.ends_with(']') {
        num = num[1..num.len() - 1].trim().to_string();
    }
    if den.starts_with('[') && den.ends_with(']') {
        den = den[1..den.len() - 1].trim().to_string();
    }
    Some((num, den))
}

fn map_char_pdf(ch: char, base_font: PdfFont) -> (PdfFont, u8) {
    // 1. Gregos e operadores matemáticos no font Base-14 /Symbol
    if let Some(sym_byte) = unicode_to_symbol_byte(ch) {
        return (PdfFont::Symbol, sym_byte);
    }
    // 2. WinAnsi (ASCII + Latin-1 + Windows-1252)
    let code = ch as u32;
    if (0x20..=0x7E).contains(&code) || (0xA0..=0xFF).contains(&code) {
        return (base_font, code as u8);
    }
    let w1252 = match ch {
        '–' => 0x96,
        '—' => 0x97,
        '•' => 0x95,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '…' => 0x85,
        '™' => 0x99,
        '′' => b'\'',
        '″' => b'"',
        '−' => b'-',
        _ => b' ',
    };
    (base_font, w1252)
}

/// Tabela única de mapeamento Unicode ↔ Adobe Standard SymbolEncoding (/F3 + /ToUnicode CMap).
const SYMBOL_UNICODE_MAP: &[(char, u8)] = &[
    ('Α', 0x41),
    ('Β', 0x42),
    ('Δ', 0x44),
    ('Ε', 0x45),
    ('Ζ', 0x5A),
    ('Η', 0x48),
    ('Θ', 0x51),
    ('Ι', 0x49),
    ('Κ', 0x4B),
    ('Λ', 0x4C),
    ('Μ', 0x4D),
    ('Ν', 0x4E),
    ('Ξ', 0x58),
    ('Ο', 0x4F),
    ('Π', 0x50),
    ('Ρ', 0x52),
    ('Σ', 0x53),
    ('Τ', 0x54),
    ('Υ', 0x55),
    ('Φ', 0x46),
    ('Χ', 0x43),
    ('Ψ', 0x59),
    ('Ω', 0x57),
    ('α', 0x61),
    ('β', 0x62),
    ('γ', 0x67),
    ('δ', 0x64),
    ('ε', 0x65),
    ('ζ', 0x7A),
    ('η', 0x68),
    ('θ', 0x71),
    ('ι', 0x69),
    ('κ', 0x6B),
    ('λ', 0x6C),
    ('μ', 0x6D),
    ('ν', 0x6E),
    ('ξ', 0x78),
    ('ο', 0x6F),
    ('π', 0x70),
    ('ρ', 0x72),
    ('σ', 0x73),
    ('ς', 0x56),
    ('τ', 0x74),
    ('υ', 0x75),
    ('φ', 0x66),
    ('ϕ', 0x6A),
    ('χ', 0x63),
    ('ψ', 0x79),
    ('ω', 0x77),
    ('≤', 0xA3),
    ('≥', 0xB3),
    ('≠', 0xB9),
    ('≈', 0xBB),
    ('≡', 0xBA),
    ('→', 0xAE),
    ('←', 0xAC),
    ('↔', 0xAB),
    ('√', 0xD6),
    ('∞', 0xA5),
    ('∑', 0xE5),
    ('∏', 0xD5),
    ('∫', 0xF2),
    ('∂', 0xB6),
    ('∇', 0xD1),
    ('∈', 0xCE),
    ('±', 0xB1),
    ('×', 0xB4),
    ('÷', 0xB8),
    ('°', 0xB0),
    ('″', 0xB2),
];

fn unicode_to_symbol_byte(ch: char) -> Option<u8> {
    SYMBOL_UNICODE_MAP.iter().find_map(|&(u, b)| (u == ch).then_some(b))
}

/// Constrói o fluxo CMap `/ToUnicode` do font `/F3` (`/Symbol`) para garantir que copiar/extrair
/// texto do PDF preserve `σ`, `γ`, `φ`, `ξ`, `≥`, `≤` etc. no clipboard sem virar `s`/`g`/`f`.
fn build_symbol_tounicode_cmap() -> Vec<u8> {
    let mut entries = std::collections::BTreeMap::<u8, u32>::new();
    for &(ch, code) in SYMBOL_UNICODE_MAP {
        entries.entry(code).or_insert(ch as u32);
    }
    let mut cmap = String::with_capacity(2048);
    cmap.push_str(
        "/CIDInit /ProcSet findresource begin\n\
         12 dict begin\n\
         begincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n\
         /CMapType 2 def\n\
         1 begincodespacerange\n\
         <00> <FF>\n\
         endcodespacerange\n",
    );
    let _ = writeln!(cmap, "{} beginbfchar", entries.len());
    for (byte, uni) in entries {
        let _ = writeln!(cmap, "<{byte:02X}> <{uni:04X}>");
    }
    cmap.push_str(
        "endbfchar\n\
         endcmap\n\
         CMapName currentdict /CMap defineresource pop\n\
         end\n\
         end\n",
    );
    cmap.into_bytes()
}

fn char_width_em(ch: char, bold: bool) -> f64 {
    let base = match ch {
        ' ' => 0.278,
        'i' | 'l' | 'I' | '|' | '.' | ',' | ':' | ';' | '\'' | '!' => 0.278,
        'f' | 'j' | 'r' | 't' | '(' | ')' | '[' | ']' | '{' | '}' | '/' | '-' => 0.34,
        'k' | 's' | 'v' | 'x' | 'y' | 'z' | 'c' | 'ç' => 0.50,
        'm' | 'w' | 'M' | 'W' | 'Σ' | 'Ω' | 'Δ' | 'Φ' | 'Ψ' => 0.82,
        '×' => 0.58,
        'a'..='z' | '0'..='9' | 'á'..='ü' => 0.556,
        'A'..='Z' | 'Á'..='Ü' => 0.67,
        '—' => 0.90,
        '–' => 0.56,
        'α' | 'β' | 'γ' | 'δ' | 'σ' | 'φ' | 'θ' | 'ρ' | 'μ' | 'τ' | 'ξ' | 'η' | 'ψ' => 0.56,
        '≤' | '≥' | '≠' | '≈' | '→' | '+' | '=' => 0.58,
        _ => 0.54,
    };
    if bold && ch.is_alphanumeric() {
        base * 1.09
    } else {
        base * 1.03
    }
}

/// Split de tokens gigantes sem espaços que ultrapassam `max_width`, preservando grupos `_{...}` / `^{...}` atômicos.
fn split_oversized_token(token: &str, max_width: f64, font_size: f64, bold: bool) -> Vec<String> {
    if measure_rich_text(token, font_size, bold) <= max_width {
        return vec![token.to_string()];
    }
    let chars: Vec<char> = token.chars().collect();
    let n = chars.len();
    let mut chunks = Vec::new();
    let mut cur = String::new();
    let mut i = 0usize;

    while i < n {
        // Lê uma unidade atômica (preserva `X_{sub}` / `X^{sup}` ou caractere único)
        let mut atom = String::new();
        atom.push(chars[i]);
        i += 1;
        while i < n && (chars[i] == '_' || chars[i] == '^') {
            atom.push(chars[i]);
            i += 1;
            if i < n && chars[i] == '{' {
                while i < n {
                    let c = chars[i];
                    atom.push(c);
                    i += 1;
                    if c == '}' {
                        break;
                    }
                }
            } else {
                while i < n && (chars[i].is_alphanumeric() || chars[i] == '\'' || chars[i] == '′') {
                    atom.push(chars[i]);
                    i += 1;
                }
            }
        }
        let cand = format!("{cur}{atom}");
        if measure_rich_text(&cand, font_size, bold) <= max_width || cur.is_empty() {
            cur = cand;
        } else {
            chunks.push(cur);
            cur = atom;
        }
    }
    if !cur.is_empty() {
        chunks.push(cur);
    }
    chunks
}

fn fit_rich_text_ellipsis(text: &str, max_width: f64, font_size: f64, bold: bool) -> String {
    let normalized = normalize_math_symbols(text);
    if measure_rich_text(&normalized, font_size, bold) <= max_width {
        return normalized;
    }
    let chars: Vec<char> = normalized.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        let mut atom = String::new();
        atom.push(chars[i]);
        i += 1;
        while i < chars.len() && (chars[i] == '_' || chars[i] == '^') {
            atom.push(chars[i]);
            i += 1;
            if i < chars.len() && chars[i] == '{' {
                while i < chars.len() {
                    let c = chars[i];
                    atom.push(c);
                    i += 1;
                    if c == '}' {
                        break;
                    }
                }
            }
        }
        let cand = format!("{out}{atom}...");
        if measure_rich_text(&cand, font_size, bold) > max_width {
            break;
        }
        out.push_str(&atom);
    }
    format!("{}...", out.trim_end())
}

fn measure_rich_text(text: &str, font_size: f64, bold: bool) -> f64 {
    let normalized = normalize_math_symbols(text);
    let mut w = 0.0_f64;
    for (run, kind) in split_math_runs(&normalized) {
        let scale = match kind {
            ScriptKind::Normal => font_size,
            ScriptKind::Sub | ScriptKind::Sup => (font_size * 0.72).max(5.5),
        };
        for ch in run.chars() {
            w += char_width_em(ch, bold) * scale;
        }
    }
    w
}

fn escape_pdf_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() + 8);
    for &b in bytes {
        match b {
            b'(' => out.push_str("\\("),
            b')' => out.push_str("\\)"),
            b'\\' => out.push_str("\\\\"),
            0x20..=0x7E => out.push(b as char),
            _ => {
                let _ = write!(out, "\\{:03o}", b);
            }
        }
    }
    out
}

fn wrap_rich_text(text: &str, max_width: f64, font_size: f64, bold: bool) -> Vec<String> {
    let normalized = normalize_math_symbols(text);
    let safe_w = max_width.max(16.0);
    let mut lines = Vec::new();
    for para in normalized.split('\n') {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for raw_word in trimmed.split_whitespace() {
            for word in split_oversized_token(raw_word, safe_w, font_size, bold) {
                let candidate = if current.is_empty() {
                    word.clone()
                } else {
                    format!("{current} {word}")
                };
                if measure_rich_text(&candidate, font_size, bold) <= safe_w {
                    current = candidate;
                } else {
                    if !current.is_empty() {
                        lines.push(current);
                    }
                    current = word;
                }
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
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

fn blend_with_bg(fg: (f64, f64, f64), bg: (f64, f64, f64), alpha: f64) -> (f64, f64, f64) {
    (
        fg.0 * alpha + bg.0 * (1.0 - alpha),
        fg.1 * alpha + bg.1 * (1.0 - alpha),
        fg.2 * alpha + bg.2 * (1.0 - alpha),
    )
}

fn heatmap_rgb(fs: f64) -> (f64, f64, f64) {
    let c = GeoEstabilTheme::heatmap_color(fs);
    (c.r() as f64 / 255.0, c.g() as f64 / 255.0, c.b() as f64 / 255.0)
}

pub struct PdfDocument {
    pages: Vec<String>,
    current: String,
    cursor_y: f64,
    lang: Language,
}

impl PdfDocument {
    pub fn new(lang: Language) -> Self {
        let mut doc = Self {
            pages: Vec::new(),
            current: String::with_capacity(16_384),
            cursor_y: PAGE_H - MARGIN_TOP,
            lang,
        };
        doc.begin_page_decor();
        doc
    }

    fn begin_page_decor(&mut self) {
        self.cursor_y = PAGE_H - MARGIN_TOP;
        let pt = self.lang == Language::PtBr;
        self.set_stroke(0.80, 0.84, 0.89);
        self.set_line_width(0.6);
        self.draw_line(MARGIN_X, PAGE_H - 28.0, PAGE_W - MARGIN_X, PAGE_H - 28.0);
        let hdr = if pt {
            "GeoEstabil — Memorial Técnico de Cálculo Geotécnico e Estrutural (ABNT)"
        } else {
            "GeoEstabil — Technical Geotechnical & Structural Calculation Report (ABNT)"
        };
        self.draw_text_rich(
            MARGIN_X,
            PAGE_H - 22.0,
            7.8,
            false,
            (0.42, 0.48, 0.56),
            hdr,
        );
    }

    fn push_page_break(&mut self) {
        let finished = std::mem::replace(&mut self.current, String::with_capacity(16_384));
        self.pages.push(finished);
        self.begin_page_decor();
    }

    fn ensure_space(&mut self, needed_h: f64) {
        if self.cursor_y - needed_h < MARGIN_BOTTOM {
            self.push_page_break();
        }
    }

    fn set_fill(&mut self, r: f64, g: f64, b: f64) {
        let _ = writeln!(self.current, "{r:.3} {g:.3} {b:.3} rg");
    }

    fn set_stroke(&mut self, r: f64, g: f64, b: f64) {
        let _ = writeln!(self.current, "{r:.3} {g:.3} {b:.3} RG");
    }

    fn set_line_width(&mut self, w: f64) {
        let _ = writeln!(self.current, "{w:.2} w");
    }

    fn fill_rect(&mut self, x: f64, y_bot: f64, w: f64, h: f64, rgb: (f64, f64, f64)) {
        self.set_fill(rgb.0, rgb.1, rgb.2);
        let _ = writeln!(self.current, "{x:.2} {y_bot:.2} {w:.2} {h:.2} re f");
    }

    fn fill_and_stroke_rect(
        &mut self,
        x: f64,
        y_bot: f64,
        w: f64,
        h: f64,
        fill: (f64, f64, f64),
        stroke: (f64, f64, f64),
        lw: f64,
    ) {
        self.set_fill(fill.0, fill.1, fill.2);
        self.set_stroke(stroke.0, stroke.1, stroke.2);
        self.set_line_width(lw);
        let _ = writeln!(self.current, "{x:.2} {y_bot:.2} {w:.2} {h:.2} re B");
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let _ = writeln!(self.current, "{x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S");
    }

    fn fill_polygon(&mut self, pts: &[(f64, f64)], fill: (f64, f64, f64)) {
        if pts.len() < 3 {
            return;
        }
        self.set_fill(fill.0, fill.1, fill.2);
        let _ = writeln!(self.current, "{:.2} {:.2} m", pts[0].0, pts[0].1);
        for &(x, y) in &pts[1..] {
            let _ = writeln!(self.current, "{x:.2} {y:.2} l");
        }
        self.current.push_str("h f\n");
    }

    fn fill_and_stroke_polygon(
        &mut self,
        pts: &[(f64, f64)],
        fill: (f64, f64, f64),
        stroke: (f64, f64, f64),
        lw: f64,
    ) {
        if pts.len() < 3 {
            return;
        }
        self.set_fill(fill.0, fill.1, fill.2);
        self.set_stroke(stroke.0, stroke.1, stroke.2);
        self.set_line_width(lw);
        let _ = writeln!(self.current, "{:.2} {:.2} m", pts[0].0, pts[0].1);
        for &(x, y) in &pts[1..] {
            let _ = writeln!(self.current, "{x:.2} {y:.2} l");
        }
        self.current.push_str("h B\n");
    }

    fn draw_polyline(&mut self, pts: &[(f64, f64)], stroke: (f64, f64, f64), lw: f64) {
        if pts.len() < 2 {
            return;
        }
        self.set_stroke(stroke.0, stroke.1, stroke.2);
        self.set_line_width(lw);
        let _ = writeln!(self.current, "{:.2} {:.2} m", pts[0].0, pts[0].1);
        for &(x, y) in &pts[1..] {
            let _ = writeln!(self.current, "{x:.2} {y:.2} l");
        }
        self.current.push_str("S\n");
    }

    fn fill_circle(&mut self, cx: f64, cy: f64, r: f64, fill: (f64, f64, f64)) {
        let mut pts = Vec::with_capacity(16);
        for i in 0..16 {
            let a = (i as f64) * std::f64::consts::TAU / 16.0;
            pts.push((cx + r * a.cos(), cy + r * a.sin()));
        }
        self.fill_polygon(&pts, fill);
    }

    fn draw_text_rich(
        &mut self,
        x: f64,
        y: f64,
        size: f64,
        bold: bool,
        rgb: (f64, f64, f64),
        text: &str,
    ) {
        self.draw_text_rich_styled(x, y, size, bold, false, rgb, text);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_text_rich_styled(
        &mut self,
        x: f64,
        y: f64,
        size: f64,
        bold: bool,
        italic: bool,
        rgb: (f64, f64, f64),
        text: &str,
    ) {
        let normalized = normalize_math_symbols(text);
        let base_font = if bold {
            PdfFont::HelveticaBold
        } else if italic {
            PdfFont::HelveticaOblique
        } else {
            PdfFont::Helvetica
        };

        self.set_fill(rgb.0, rgb.1, rgb.2);
        let _ = writeln!(self.current, "BT 1 0 0 1 {x:.2} {y:.2} Tm");

        for (run, kind) in split_math_runs(&normalized) {
            let (run_size, rise) = match kind {
                ScriptKind::Normal => (size, 0.0),
                ScriptKind::Sub => ((size * 0.72).max(5.2), -size * 0.24),
                ScriptKind::Sup => ((size * 0.72).max(5.2), size * 0.34),
            };
            let _ = writeln!(self.current, "{rise:.2} Ts");

            // Agrupa caracteres consecutivos por fonte (/F1,/F2,/F3,/F4)
            let mut cur_font: Option<PdfFont> = None;
            let mut buf: Vec<u8> = Vec::new();
            for ch in run.chars() {
                let (font, byte) = map_char_pdf(ch, base_font);
                if cur_font != Some(font) {
                    if !buf.is_empty() {
                        if let Some(cf) = cur_font {
                            let _ = writeln!(
                                self.current,
                                "{} {run_size:.2} Tf ({}) Tj",
                                cf.tag(),
                                escape_pdf_bytes(&buf)
                            );
                        }
                        buf.clear();
                    }
                    cur_font = Some(font);
                }
                buf.push(byte);
            }
            if !buf.is_empty() {
                if let Some(cf) = cur_font {
                    let _ = writeln!(
                        self.current,
                        "{} {run_size:.2} Tf ({}) Tj",
                        cf.tag(),
                        escape_pdf_bytes(&buf)
                    );
                }
            }
        }
        self.current.push_str("0 Ts ET\n");
    }

    fn draw_text_centered(
        &mut self,
        cx: f64,
        y: f64,
        size: f64,
        bold: bool,
        rgb: (f64, f64, f64),
        text: &str,
    ) {
        let w = measure_rich_text(text, size, bold);
        self.draw_text_rich(cx - w * 0.5, y, size, bold, rgb, text);
    }

    fn draw_text_right(
        &mut self,
        rx: f64,
        y: f64,
        size: f64,
        bold: bool,
        rgb: (f64, f64, f64),
        text: &str,
    ) {
        let w = measure_rich_text(text, size, bold);
        self.draw_text_rich(rx - w, y, size, bold, rgb, text);
    }

    pub fn draw_banner(&mut self, title: &str, subtitle: &str) {
        let title_lines = wrap_rich_text(title, USABLE_W - 28.0, 14.0, true);
        let sub_lines = wrap_rich_text(subtitle, USABLE_W - 28.0, 9.0, false);
        let title_h = (title_lines.len() as f64) * 17.5;
        let sub_h = (sub_lines.len() as f64) * 12.0;
        let banner_h = (title_h + sub_h + 22.0).max(54.0);
        self.ensure_space(banner_h + 12.0);
        let bot = self.cursor_y - banner_h;
        self.fill_rect(MARGIN_X, bot, USABLE_W, banner_h, (0.106, 0.212, 0.365));
        self.fill_rect(MARGIN_X, bot, 6.0, banner_h, (0.145, 0.388, 0.922));
        let mut ty = self.cursor_y - 20.0;
        for tl in &title_lines {
            self.draw_text_rich(MARGIN_X + 16.0, ty, 14.0, true, (1.0, 1.0, 1.0), tl);
            ty -= 17.5;
        }
        let mut sy = ty - 1.0;
        for sl in &sub_lines {
            self.draw_text_rich(MARGIN_X + 16.0, sy, 9.0, false, (0.84, 0.90, 0.97), sl);
            sy -= 12.0;
        }
        self.cursor_y = bot - 14.0;
    }

    pub fn draw_info_card(&mut self, items: &[(&str, &str)]) {
        if items.is_empty() {
            return;
        }
        let inner_w = USABLE_W - 24.0;
        let single_col = items.len() == 1;
        let col_w = if single_col { inner_w } else { (inner_w - 10.0) * 0.5 };

        // Empacota (k, v) dentro da largura exata da coluna, preservando chave em negrito na 1ª linha
        let wrap_kv = |k: &str, v: &str, max_w: f64| -> Vec<(String, String)> {
            let kw = measure_rich_text(k, 8.6, true);
            if kw + measure_rich_text(v, 8.6, false) + 3.0 <= max_w {
                return vec![(k.to_string(), v.to_string())];
            }
            let first_avail = (max_w - kw - 3.0).max(30.0);
            let full_avail = max_w.max(30.0);
            let v_norm = normalize_math_symbols(v);
            let mut result: Vec<(String, String)> = Vec::new();
            let mut cur_line = String::new();
            let mut cur_max = first_avail;
            let mut cur_k = k.to_string();

            for raw_word in v_norm.split_whitespace() {
                for word in split_oversized_token(raw_word, cur_max, 8.6, false) {
                    let cand = if cur_line.is_empty() {
                        word.clone()
                    } else {
                        format!("{cur_line} {word}")
                    };
                    if measure_rich_text(&cand, 8.6, false) <= cur_max {
                        cur_line = cand;
                    } else {
                        if !cur_line.is_empty() || !cur_k.is_empty() {
                            result.push((std::mem::take(&mut cur_k), cur_line));
                        }
                        cur_max = full_avail;
                        cur_line = word;
                    }
                }
            }
            if !cur_line.is_empty() || !cur_k.is_empty() || result.is_empty() {
                result.push((cur_k, cur_line));
            }
            result
        };

        let mut row_pairs: Vec<(Vec<(String, String)>, Vec<(String, String)>)> = Vec::new();
        if single_col {
            row_pairs.push((wrap_kv(items[0].0, items[0].1, col_w), Vec::new()));
        } else {
            for chunk in items.chunks(2) {
                let left = wrap_kv(chunk[0].0, chunk[0].1, col_w);
                let right = chunk
                    .get(1)
                    .map(|(rk, rv)| wrap_kv(rk, rv, col_w))
                    .unwrap_or_default();
                row_pairs.push((left, right));
            }
        }

        let line_h = 12.5_f64;
        let card_h: f64 = 16.0
            + row_pairs
                .iter()
                .map(|(l, r)| (l.len().max(r.len()).max(1) as f64) * line_h + 3.0)
                .sum::<f64>();
        self.ensure_space(card_h + 10.0);
        let bot = self.cursor_y - card_h;
        self.fill_and_stroke_rect(
            MARGIN_X,
            bot,
            USABLE_W,
            card_h,
            (0.968, 0.980, 0.992),
            (0.80, 0.85, 0.90),
            0.7,
        );
        self.fill_rect(MARGIN_X, bot, 3.5, card_h, (0.145, 0.388, 0.922));

        let mut ry = self.cursor_y - 16.0;
        for (left, right) in &row_pairs {
            let max_lines = left.len().max(right.len()).max(1);
            for (li, (lk, lv)) in left.iter().enumerate() {
                let y = ry - (li as f64) * line_h;
                let x = MARGIN_X + 14.0;
                if !lk.is_empty() {
                    self.draw_text_rich(x, y, 8.6, true, (0.28, 0.34, 0.42), lk);
                    let kw = measure_rich_text(lk, 8.6, true) + 3.0;
                    self.draw_text_rich(x + kw, y, 8.6, false, (0.09, 0.14, 0.22), lv);
                } else {
                    self.draw_text_rich(x, y, 8.6, false, (0.09, 0.14, 0.22), lv);
                }
            }
            for (li, (rk, rv)) in right.iter().enumerate() {
                let y = ry - (li as f64) * line_h;
                let x = MARGIN_X + 14.0 + col_w + 10.0;
                if !rk.is_empty() {
                    self.draw_text_rich(x, y, 8.6, true, (0.28, 0.34, 0.42), rk);
                    let kw = measure_rich_text(rk, 8.6, true) + 3.0;
                    self.draw_text_rich(x + kw, y, 8.6, false, (0.09, 0.14, 0.22), rv);
                } else {
                    self.draw_text_rich(x, y, 8.6, false, (0.09, 0.14, 0.22), rv);
                }
            }
            ry -= (max_lines as f64) * line_h + 3.0;
        }
        self.cursor_y = bot - 12.0;
    }

    pub fn draw_section_heading(&mut self, title: &str) {
        let lines = wrap_rich_text(title, USABLE_W - 4.0, 11.8, true);
        let needed = (lines.len() as f64) * 15.0 + 18.0;
        self.ensure_space(needed);
        self.cursor_y -= 6.0;
        let mut y = self.cursor_y - 13.0;
        for l in &lines {
            self.draw_text_rich(MARGIN_X, y, 11.8, true, (0.106, 0.212, 0.365), l);
            y -= 14.5;
        }
        let line_y = y + 9.5;
        self.set_stroke(0.145, 0.388, 0.922);
        self.set_line_width(1.4);
        self.draw_line(MARGIN_X, line_y, PAGE_W - MARGIN_X, line_y);
        self.cursor_y = line_y - 9.0;
    }

    pub fn draw_sub_heading(&mut self, title: &str) {
        let lines = wrap_rich_text(title, USABLE_W - 4.0, 9.8, true);
        let needed = (lines.len() as f64) * 13.0 + 10.0;
        self.ensure_space(needed);
        let mut y = self.cursor_y - 11.0;
        for l in &lines {
            self.draw_text_rich(MARGIN_X, y, 9.8, true, (0.145, 0.388, 0.922), l);
            y -= 13.0;
        }
        self.cursor_y = y - 6.0;
    }

    pub fn draw_paragraph(&mut self, text: &str) {
        let lines = wrap_rich_text(text, USABLE_W - 8.0, 8.8, false);
        let needed = (lines.len() as f64) * 12.5 + 4.0;
        self.ensure_space(needed);
        for l in lines {
            self.draw_text_rich(MARGIN_X + 2.0, self.cursor_y - 9.5, 8.8, false, (0.12, 0.16, 0.23), &l);
            self.cursor_y -= 12.5;
        }
        self.cursor_y -= 3.0;
    }

    /// Cartão de equação normativa com suporte a múltiplas frações empilhadas (`\frac{num}{den}`),
    /// quebra automática dentro das margens e tipografia grega/subscrita com `/ToUnicode`.
    pub fn draw_equation_box(&mut self, tag: &str, expr: &str, note: Option<&str>) {
        let tag_w = if tag.is_empty() {
            0.0
        } else {
            (measure_rich_text(tag, 7.6, true) + 12.0).max(40.0)
        };
        let first_line_w = (USABLE_W - 24.0 - if tag.is_empty() { 0.0 } else { tag_w + 10.0 }).max(120.0);
        let full_line_w = USABLE_W - 24.0;

        enum FlowAtom {
            Text(String, f64),
            Fraction { num: String, den: String, bar_w: f64 },
        }

        let build_flow_lines = |raw: &str, font_sz: f64, frac_sz: f64, bold: bool, first_w: f64, next_w: f64| -> Vec<(Vec<FlowAtom>, f64)> {
            let norm = normalize_math_symbols(raw);
            let segs = split_equation_segments(&norm);
            let mut atoms: Vec<FlowAtom> = Vec::new();
            for seg in segs {
                match seg {
                    EqSegment::Fraction { num, den } => {
                        let w_num = measure_rich_text(&num, frac_sz, false);
                        let w_den = measure_rich_text(&den, frac_sz, false);
                        let bar_w = w_num.max(w_den).max(20.0) + 10.0;
                        atoms.push(FlowAtom::Fraction { num, den, bar_w });
                    }
                    EqSegment::Text(t) => {
                        // Mantém espaçamento natural dividindo por palavras para quebra elegante nas margens
                        let lead_sp = t.starts_with(' ');
                        let trail_sp = t.ends_with(' ');
                        let words: Vec<&str> = t.split_whitespace().collect();
                        if words.is_empty() {
                            if !t.is_empty() {
                                let w = measure_rich_text(" ", font_sz, bold);
                                atoms.push(FlowAtom::Text(" ".into(), w));
                            }
                        } else {
                            for (wi, wd) in words.iter().enumerate() {
                                let mut s = String::new();
                                if (wi == 0 && lead_sp) || wi > 0 {
                                    s.push(' ');
                                }
                                s.push_str(wd);
                                if wi + 1 == words.len() && trail_sp {
                                    s.push(' ');
                                }
                                let limit = if wi == 0 { first_w } else { next_w };
                                for part in split_oversized_token(&s, limit, font_sz, bold) {
                                    let pw = measure_rich_text(&part, font_sz, bold);
                                    atoms.push(FlowAtom::Text(part, pw));
                                }
                            }
                        }
                    }
                }
            }

            let mut lines: Vec<(Vec<FlowAtom>, f64)> = Vec::new();
            let mut cur_row: Vec<FlowAtom> = Vec::new();
            let mut cur_w = 0.0_f64;
            let mut cur_has_frac = false;

            for atom in atoms {
                let atom_w = match &atom {
                    FlowAtom::Text(_, w) => *w,
                    FlowAtom::Fraction { bar_w, .. } => *bar_w + 4.0,
                };
                let is_frac = matches!(atom, FlowAtom::Fraction { .. });
                let row_limit = if lines.is_empty() { first_w } else { next_w };

                if cur_w + atom_w > row_limit && !cur_row.is_empty() {
                    let row_h = if cur_has_frac { 27.0 } else { font_sz + 5.5 };
                    lines.push((std::mem::take(&mut cur_row), row_h));
                    cur_w = 0.0;
                    cur_has_frac = false;
                }
                if is_frac {
                    cur_has_frac = true;
                }
                cur_w += atom_w;
                cur_row.push(atom);
            }
            if !cur_row.is_empty() || lines.is_empty() {
                let row_h = if cur_has_frac { 27.0 } else { font_sz + 5.5 };
                lines.push((cur_row, row_h));
            }
            lines
        };

        let eq_lines = build_flow_lines(expr, 9.6, 8.4, true, first_line_w, full_line_w);
        let note_lines = note
            .map(|n| build_flow_lines(n, 7.9, 7.2, false, full_line_w, full_line_w))
            .unwrap_or_default();

        let eq_total_h: f64 = eq_lines.iter().map(|(_, h)| *h).sum::<f64>();
        let note_total_h: f64 = note_lines.iter().map(|(_, h)| *h).sum::<f64>();
        let box_h = 12.0 + eq_total_h + if note_lines.is_empty() { 0.0 } else { 4.0 + note_total_h };
        self.ensure_space(box_h + 10.0);

        let bot = self.cursor_y - box_h;
        self.fill_and_stroke_rect(
            MARGIN_X,
            bot,
            USABLE_W,
            box_h,
            (0.955, 0.972, 0.996),
            (0.75, 0.84, 0.95),
            0.7,
        );
        self.fill_rect(MARGIN_X, bot, 4.0, box_h, (0.145, 0.388, 0.922));

        let x0 = MARGIN_X + 12.0;
        if !tag.is_empty() {
            let ty = self.cursor_y - 16.5;
            self.fill_rect(PAGE_W - MARGIN_X - tag_w - 8.0, ty - 2.0, tag_w, 13.0, (0.86, 0.91, 0.99));
            self.draw_text_centered(
                PAGE_W - MARGIN_X - tag_w * 0.5 - 8.0,
                ty + 2.0,
                7.6,
                true,
                (0.106, 0.212, 0.365),
                tag,
            );
        }

        let mut row_top = self.cursor_y - 6.0;
        for (atoms, row_h) in &eq_lines {
            let has_frac = atoms.iter().any(|a| matches!(a, FlowAtom::Fraction { .. }));
            let baseline_y = if has_frac { row_top - 16.5 } else { row_top - 10.5 };
            let mut cx = x0;
            for atom in atoms {
                match atom {
                    FlowAtom::Text(txt, w) => {
                        let clean = if cx == x0 { txt.trim_start() } else { txt.as_str() };
                        self.draw_text_rich(cx, baseline_y, 9.6, true, (0.106, 0.212, 0.365), clean);
                        cx += if cx == x0 {
                            measure_rich_text(clean, 9.6, true)
                        } else {
                            *w
                        };
                    }
                    FlowAtom::Fraction { num, den, bar_w } => {
                        let fcx = cx + *bar_w * 0.5;
                        self.draw_text_centered(fcx, baseline_y + 7.2, 8.4, false, (0.09, 0.14, 0.22), num);
                        self.set_stroke(0.106, 0.212, 0.365);
                        self.set_line_width(0.85);
                        self.draw_line(cx + 1.5, baseline_y + 3.2, cx + *bar_w - 1.5, baseline_y + 3.2);
                        self.draw_text_centered(fcx, baseline_y - 6.8, 8.4, false, (0.09, 0.14, 0.22), den);
                        cx += *bar_w + 4.0;
                    }
                }
            }
            row_top -= *row_h;
        }

        if !note_lines.is_empty() {
            row_top -= 4.0;
            for (atoms, row_h) in &note_lines {
                let has_frac = atoms.iter().any(|a| matches!(a, FlowAtom::Fraction { .. }));
                let baseline_y = if has_frac { row_top - 16.0 } else { row_top - 9.0 };
                let mut cx = x0;
                for atom in atoms {
                    match atom {
                        FlowAtom::Text(txt, w) => {
                            let clean = if cx == x0 { txt.trim_start() } else { txt.as_str() };
                            self.draw_text_rich_styled(cx, baseline_y, 7.9, false, true, (0.33, 0.40, 0.49), clean);
                            cx += if cx == x0 {
                                measure_rich_text(clean, 7.9, false)
                            } else {
                                *w
                            };
                        }
                        FlowAtom::Fraction { num, den, bar_w } => {
                            let fcx = cx + *bar_w * 0.5;
                            self.draw_text_centered(fcx, baseline_y + 6.5, 7.2, false, (0.28, 0.35, 0.44), num);
                            self.set_stroke(0.33, 0.40, 0.49);
                            self.set_line_width(0.75);
                            self.draw_line(cx + 1.5, baseline_y + 2.8, cx + *bar_w - 1.5, baseline_y + 2.8);
                            self.draw_text_centered(fcx, baseline_y - 5.8, 7.2, false, (0.28, 0.35, 0.44), den);
                            cx += *bar_w + 4.0;
                        }
                    }
                }
                row_top -= *row_h;
            }
        }
        self.cursor_y = bot - 8.0;
    }

    pub fn draw_table(&mut self, headers: &[&str], rows: &[Vec<String>], col_fracs: &[f64]) {
        let sum: f64 = col_fracs.iter().sum::<f64>().max(0.01);
        let col_widths: Vec<f64> = col_fracs.iter().map(|f| USABLE_W * (*f / sum)).collect();

        let wrapped_headers: Vec<Vec<String>> = headers
            .iter()
            .zip(col_widths.iter())
            .map(|(h, w)| wrap_rich_text(h, (*w - 8.0).max(16.0), 8.0, true))
            .collect();
        let max_hdr_lines = wrapped_headers.iter().map(|v| v.len()).max().unwrap_or(1).max(1);
        let hdr_h = (max_hdr_lines as f64) * 10.5 + 8.0;

        let render_header = |pdf: &mut PdfDocument| {
            pdf.ensure_space(hdr_h + 18.0);
            let bot = pdf.cursor_y - hdr_h;
            pdf.fill_rect(MARGIN_X, bot, USABLE_W, hdr_h, (0.106, 0.212, 0.365));
            let mut cx = MARGIN_X;
            for (hlines, w) in wrapped_headers.iter().zip(col_widths.iter()) {
                for (li, hline) in hlines.iter().enumerate() {
                    let hy = pdf.cursor_y - 11.5 - (li as f64) * 10.5;
                    pdf.draw_text_rich(cx + 4.0, hy, 8.0, true, (1.0, 1.0, 1.0), hline);
                }
                cx += *w;
            }
            pdf.cursor_y = bot;
        };

        render_header(self);
        for (r_idx, row) in rows.iter().enumerate() {
            let mut wrapped_cols = Vec::with_capacity(col_widths.len());
            let mut max_lines = 1usize;
            for (c_idx, w) in col_widths.iter().enumerate() {
                let txt = row.get(c_idx).map(|s| s.as_str()).unwrap_or("");
                let lines = wrap_rich_text(txt, (*w - 8.0).max(16.0), 7.9, false);
                max_lines = max_lines.max(lines.len());
                wrapped_cols.push(lines);
            }
            let row_h = (max_lines as f64) * 11.0 + 7.0;
            if self.cursor_y - row_h < MARGIN_BOTTOM {
                self.push_page_break();
                render_header(self);
            }
            let bot = self.cursor_y - row_h;
            let bg = if r_idx % 2 == 1 {
                (0.968, 0.980, 0.992)
            } else {
                (1.0, 1.0, 1.0)
            };
            self.fill_and_stroke_rect(MARGIN_X, bot, USABLE_W, row_h, bg, (0.84, 0.88, 0.92), 0.4);
            let mut cx = MARGIN_X;
            for (lines, w) in wrapped_cols.iter().zip(col_widths.iter()) {
                for (li, line) in lines.iter().enumerate() {
                    let ly = self.cursor_y - 12.0 - (li as f64) * 11.0;
                    let is_bold = line.starts_with("APROVADO")
                        || line.starts_with("APPROVED")
                        || line.starts_with("REPROVADO")
                        || line.starts_with("FAILED")
                        || line.starts_with("ATENDE")
                        || line.starts_with("EXCEEDS")
                        || line.starts_with("ACIMA")
                        || line.starts_with("ESTÁVEL")
                        || line.starts_with("STABLE");
                    let col = if line.contains("REPROVADO")
                        || line.contains("FAILED")
                        || line.contains("EXCEEDS")
                        || line.contains("ACIMA")
                        || line.contains("INSTÁVEL")
                        || line.contains("UNSTABLE")
                    {
                        (0.80, 0.14, 0.14)
                    } else if line.contains("APROVADO")
                        || line.contains("APPROVED")
                        || line.contains("ATENDE")
                        || line.contains("OK")
                        || line.contains("ESTÁVEL")
                        || line.contains("STABLE")
                    {
                        (0.08, 0.52, 0.24)
                    } else {
                        (0.11, 0.16, 0.24)
                    };
                    self.draw_text_rich(cx + 4.0, ly, 7.9, is_bold, col, line);
                }
                cx += *w;
            }
            self.cursor_y = bot;
        }
        self.cursor_y -= 10.0;
    }

    pub fn draw_bullet_list(&mut self, items: &[String]) {
        for item in items {
            let lines = wrap_rich_text(item, USABLE_W - 18.0, 8.5, false);
            let needed = (lines.len() as f64) * 12.0 + 3.0;
            self.ensure_space(needed);
            self.fill_circle(MARGIN_X + 5.5, self.cursor_y - 6.8, 1.8, (0.145, 0.388, 0.922));
            for line in lines {
                self.draw_text_rich(
                    MARGIN_X + 13.0,
                    self.cursor_y - 9.5,
                    8.5,
                    false,
                    (0.12, 0.16, 0.24),
                    &line,
                );
                self.cursor_y -= 12.0;
            }
            self.cursor_y -= 2.0;
        }
        self.cursor_y -= 6.0;
    }

    /// Figura vetorial completa da Aba 2 — Estabilidade de Taludes:
    /// camadas estratigráficas preenchidas, lençol freático (`N.A.`/`W.T.`), sobrecargas `q`,
    /// Heatmap completo dos centros de FS + escala colorida, fatias verticais, arco crítico e legendas.
    pub fn draw_slope_vector_figure(
        &mut self,
        project: &GeoProject,
        report: Option<&SlopeAnalysisReport>,
        caption: &str,
    ) {
        let fig_h = 250.0;
        self.ensure_space(fig_h + 24.0);
        let fig_bot = self.cursor_y - fig_h;
        let pt = self.lang == Language::PtBr;

        self.fill_and_stroke_rect(
            MARGIN_X,
            fig_bot,
            USABLE_W,
            fig_h,
            (1.0, 1.0, 1.0),
            (0.79, 0.84, 0.89),
            0.7,
        );

        let plot_x = MARGIN_X + 26.0;
        let plot_y = fig_bot + 26.0;
        let plot_w = USABLE_W - 36.0;
        let plot_h = fig_h - 48.0;
        let plot_bg = (0.973, 0.980, 0.988);
        self.fill_and_stroke_rect(plot_x, plot_y, plot_w, plot_h, plot_bg, (0.82, 0.86, 0.91), 0.5);

        let title = if pt {
            "Aba 2 — Seção Geotécnica, Heatmap FS, Fatias e Superfície Crítica"
        } else {
            "Tab 2 — Geotechnical Profile, FS Heatmap, Slices & Critical Slip Circle"
        };
        self.draw_text_rich(plot_x + 2.0, fig_bot + fig_h - 15.0, 8.8, true, (0.106, 0.212, 0.365), title);

        let slope = &project.slope_model;
        let terrain = &slope.terrain_surface;
        let mut x_min = terrain.x_min().min(0.0);
        let mut x_max = terrain.x_max().max(40.0);
        let mut y_min = terrain
            .y_min()
            .min(slope.layers.iter().map(|l| l.top_profile.y_min()).fold(0.0_f64, f64::min) - 3.5);
        let mut y_max = terrain.y_max().max(12.0) + 3.0;
        if let Some(rep) = report {
            y_max = y_max.max(rep.critical_circle.center_y + 2.5);
            y_min = y_min.min(rep.critical_circle.center_y - rep.critical_circle.radius - 1.0);
            x_min = x_min.min(rep.critical_circle.x_entry - 2.0);
            x_max = x_max.max(rep.critical_circle.x_exit + 2.0);
        }

        let sx = plot_w / (x_max - x_min).max(1.0);
        let sy = plot_h / (y_max - y_min).max(1.0);
        let scale = sx.min(sy);
        let cw = (x_max - x_min) * scale;
        let ch = (y_max - y_min) * scale;
        let pad_x = plot_x + (plot_w - cw) * 0.5;
        let pad_y = plot_y + (plot_h - ch) * 0.5;
        let to_p = |wx: f64, wy: f64| -> (f64, f64) { (pad_x + (wx - x_min) * scale, pad_y + (wy - y_min) * scale) };

        // Clipping no retângulo do gráfico
        let _ = writeln!(self.current, "q {plot_x:.2} {plot_y:.2} {plot_w:.2} {plot_h:.2} re W n");

        // Grade métrica
        let step_x = nice_grid_step(x_max - x_min);
        let step_y = nice_grid_step(y_max - y_min);
        self.set_stroke(0.89, 0.92, 0.95);
        self.set_line_width(0.4);
        let mut gx = (x_min / step_x).floor() * step_x;
        while gx <= x_max + 0.001 {
            let (px, _) = to_p(gx, y_min);
            self.draw_line(px, plot_y, px, plot_y + plot_h);
            gx += step_x;
        }
        let mut gy = (y_min / step_y).floor() * step_y;
        while gy <= y_max + 0.001 {
            let (_, py) = to_p(x_min, gy);
            self.draw_line(plot_x, py, plot_x + plot_w, py);
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
            let upper = if idx == 0 { terrain } else { &layer.top_profile };
            let lower_opt = slope.layers.get(idx + 1).map(|l| &l.top_profile);
            let mut xs = vec![x_start, x_end];
            xs.extend(terrain.points.iter().map(|p| p.x));
            xs.extend(upper.points.iter().map(|p| p.x));
            if let Some(lp) = lower_opt {
                xs.extend(lp.points.iter().map(|p| p.x));
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            xs.dedup_by(|a, b| (*a - *b).abs() < 1e-4);

            let l_rgb = (
                layer.color_rgba[0] as f64 / 255.0,
                layer.color_rgba[1] as f64 / 255.0,
                layer.color_rgba[2] as f64 / 255.0,
            );
            let fill_rgb = blend_with_bg(l_rgb, plot_bg, 0.30);
            for pair in xs.windows(2) {
                let (xa, xb) = (pair[0], pair[1]);
                if xb - xa < 1e-4 {
                    continue;
                }
                let yt_a = terrain.interpolate_y(xa).unwrap_or(0.0).min(upper.interpolate_y(xa).unwrap_or(f64::INFINITY));
                let yt_b = terrain.interpolate_y(xb).unwrap_or(0.0).min(upper.interpolate_y(xb).unwrap_or(f64::INFINITY));
                let yb_a = lower_opt.and_then(|lp| lp.interpolate_y(xa)).unwrap_or(deep_floor_y).min(yt_a);
                let yb_b = lower_opt.and_then(|lp| lp.interpolate_y(xb)).unwrap_or(deep_floor_y).min(yt_b);
                if yt_a > yb_a + 1e-3 || yt_b > yb_b + 1e-3 {
                    self.fill_polygon(
                        &[to_p(xa, yt_a), to_p(xb, yt_b), to_p(xb, yb_b), to_p(xa, yb_a)],
                        fill_rgb,
                    );
                }
            }
            if layer.top_profile.points.len() >= 2 {
                let pts: Vec<(f64, f64)> = layer.top_profile.points.iter().map(|p| to_p(p.x, p.y)).collect();
                self.draw_polyline(&pts, l_rgb, 1.0);
            }
        }

        // Lençol freático
        if slope.water_table.enabled && slope.water_table.phreatic_line.points.len() >= 2 {
            let wt = &slope.water_table.phreatic_line;
            let pts: Vec<(f64, f64)> = wt.points.iter().map(|p| to_p(p.x, p.y)).collect();
            self.current.push_str("[5 3] 0 d\n");
            self.draw_polyline(&pts, (0.01, 0.52, 0.78), 1.3);
            self.current.push_str("[] 0 d\n");
            let mx = 0.5 * (wt.x_min() + wt.x_max());
            if let Some(my) = wt.interpolate_y(mx) {
                let na = to_p(mx, my);
                self.fill_polygon(
                    &[(na.0 - 4.0, na.1 + 6.0), (na.0 + 4.0, na.1 + 6.0), (na.0, na.1)],
                    (0.01, 0.52, 0.78),
                );
                self.draw_text_rich(
                    na.0 + 6.0,
                    na.1 + 1.5,
                    7.2,
                    true,
                    (0.01, 0.52, 0.78),
                    if pt { "N.A." } else { "W.T." },
                );
            }
        }

        // Perfil do Terreno + Vértices
        if terrain.points.len() >= 2 {
            let pts: Vec<(f64, f64)> = terrain.points.iter().map(|p| to_p(p.x, p.y)).collect();
            self.draw_polyline(&pts, (0.12, 0.16, 0.23), 1.8);
            for (vi, &(vx, vy)) in pts.iter().enumerate() {
                self.fill_circle(vx, vy, 2.2, (0.06, 0.09, 0.16));
                if pts.len() <= 10 {
                    self.draw_text_rich(vx + 3.0, vy + 4.0, 6.8, true, (0.20, 0.25, 0.33), &format!("P{}", vi + 1));
                }
            }
        }

        // Sobrecargas
        if slope.surcharge.enabled {
            let sur = &slope.surcharge;
            let xa = sur.x_start.min(sur.x_end);
            let xb = sur.x_start.max(sur.x_end);
            let span = (xb - xa).max(0.2);
            let arrows = 5usize;
            let mut cap = Vec::new();
            for i in 0..=arrows {
                let wx = xa + span * (i as f64 / arrows as f64);
                let wy = terrain.interpolate_y(wx).unwrap_or(0.0);
                let pt_top = to_p(wx, wy + 1.2);
                let pt_bot = to_p(wx, wy);
                cap.push(pt_top);
                self.set_stroke(0.86, 0.15, 0.15);
                self.set_line_width(1.0);
                self.draw_line(pt_top.0, pt_top.1, pt_bot.0, pt_bot.1 + 2.0);
                self.fill_polygon(
                    &[pt_bot, (pt_bot.0 - 2.3, pt_bot.1 + 4.2), (pt_bot.0 + 2.3, pt_bot.1 + 4.2)],
                    (0.86, 0.15, 0.15),
                );
            }
            if let (Some(&f), Some(&l)) = (cap.first(), cap.last()) {
                self.set_stroke(0.86, 0.15, 0.15);
                self.set_line_width(1.2);
                self.draw_line(f.0, f.1, l.0, l.1);
            }
            let mid = to_p((xa + xb) * 0.5, terrain.interpolate_y((xa + xb) * 0.5).unwrap_or(0.0) + 1.45);
            self.draw_text_centered(
                mid.0,
                mid.1 + 2.0,
                7.0,
                true,
                (0.75, 0.12, 0.12),
                &format!("q = {:.1} kPa ({})", sur.q_kpa, sur.name),
            );
        }

        // Heatmap, Fatias e Arco Crítico
        if let Some(rep) = report {
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
                let cw_hm = (dx_w * scale * 0.88).clamp(3.2, 13.0);
                let ch_hm = (dy_w * scale * 0.88).clamp(3.2, 13.0);
                for pt_hm in &hm.points {
                    if pt_hm.min_fs < 50.0 {
                        let sc = to_p(pt_hm.center_x, pt_hm.center_y);
                        let raw_rgb = heatmap_rgb(pt_hm.min_fs);
                        let soft = blend_with_bg(raw_rgb, plot_bg, 0.48);
                        self.fill_rect(sc.0 - cw_hm * 0.5, sc.1 - ch_hm * 0.5, cw_hm, ch_hm, soft);
                        self.fill_circle(sc.0, sc.1, 1.3, raw_rgb);
                    }
                }
            }

            let circ = &rep.critical_circle;
            let slice_fill = blend_with_bg((0.23, 0.51, 0.96), plot_bg, 0.16);
            for sl in &circ.slices {
                let hw = sl.width * 0.5;
                let y_tl = terrain.interpolate_y(sl.x_mid - hw).unwrap_or(sl.y_top);
                let y_tr = terrain.interpolate_y(sl.x_mid + hw).unwrap_or(sl.y_top);
                let dy_l = circ.radius.powi(2) - (sl.x_mid - hw - circ.center_x).powi(2);
                let dy_r = circ.radius.powi(2) - (sl.x_mid + hw - circ.center_x).powi(2);
                let y_bl = if dy_l >= 0.0 { (circ.center_y - dy_l.sqrt()).min(y_tl) } else { sl.y_base };
                let y_br = if dy_r >= 0.0 { (circ.center_y - dy_r.sqrt()).min(y_tr) } else { sl.y_base };
                if y_tl >= y_bl && y_tr >= y_br {
                    self.fill_and_stroke_polygon(
                        &[
                            to_p(sl.x_mid - hw, y_tl),
                            to_p(sl.x_mid + hw, y_tr),
                            to_p(sl.x_mid + hw, y_br),
                            to_p(sl.x_mid - hw, y_bl),
                        ],
                        slice_fill,
                        (0.58, 0.64, 0.72),
                        0.45,
                    );
                }
            }

            let cs = to_p(circ.center_x, circ.center_y);
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
            let arc_pts: Vec<(f64, f64)> = (0..=64)
                .map(|i| {
                    let th = ang_start + sweep * (i as f64 / 64.0);
                    to_p(circ.center_x + circ.radius * th.cos(), circ.center_y + circ.radius * th.sin())
                })
                .collect();
            let arc_rgb = if rep.is_stable { (0.09, 0.64, 0.29) } else { (0.86, 0.15, 0.15) };
            let pe = to_p(circ.x_entry, y_entry);
            let px = to_p(circ.x_exit, y_exit);
            self.set_stroke(arc_rgb.0, arc_rgb.1, arc_rgb.2);
            self.set_line_width(0.7);
            self.draw_line(cs.0, cs.1, pe.0, pe.1);
            self.draw_line(cs.0, cs.1, px.0, px.1);
            self.draw_polyline(&arc_pts, arc_rgb, 2.1);
            self.fill_circle(pe.0, pe.1, 2.4, arc_rgb);
            self.fill_circle(px.0, px.1, 2.4, arc_rgb);
            self.set_stroke(arc_rgb.0, arc_rgb.1, arc_rgb.2);
            self.set_line_width(1.3);
            self.draw_line(cs.0 - 4.5, cs.1, cs.0 + 4.5, cs.1);
            self.draw_line(cs.0, cs.1 - 4.5, cs.0, cs.1 + 4.5);
        }

        // Fim do clipping
        self.current.push_str("Q\n");

        // Rótulos de eixo fora do clip
        let mut tx = (x_min / step_x).ceil() * step_x;
        while tx <= x_max + 0.001 {
            let (px, _) = to_p(tx, y_min);
            if px >= plot_x + 10.0 && px <= plot_x + plot_w - 10.0 {
                self.draw_text_centered(px, plot_y - 9.0, 6.8, false, (0.39, 0.45, 0.55), &format!("{tx:.0} m"));
            }
            tx += step_x;
        }
        let mut ty = (y_min / step_y).ceil() * step_y;
        while ty <= y_max + 0.001 {
            let (_, py) = to_p(x_min, ty);
            if py >= plot_y + 6.0 && py <= plot_y + plot_h - 6.0 {
                self.draw_text_right(plot_x - 3.0, py - 2.0, 6.8, false, (0.39, 0.45, 0.55), &format!("{ty:.0} m"));
            }
            ty += step_y;
        }

        // Legenda de Estratigrafia (Superior Esquerdo)
        if !slope.layers.is_empty() {
            let rows = slope.layers.len().min(4);
            let lw = 196.0;
            let lh = 16.0 + (rows as f64) * 10.5;
            let lx = plot_x + 6.0;
            let lbot = plot_y + plot_h - lh - 6.0;
            self.fill_and_stroke_rect(lx, lbot, lw, lh, (1.0, 1.0, 1.0), (0.80, 0.84, 0.89), 0.5);
            self.draw_text_rich(
                lx + 5.0,
                lbot + lh - 10.0,
                7.0,
                true,
                (0.06, 0.09, 0.16),
                if pt { "Estratigrafia (Mohr-Coulomb):" } else { "Stratigraphy (Mohr-Coulomb):" },
            );
            for (i, layer) in slope.layers.iter().take(4).enumerate() {
                let ry = lbot + lh - 20.0 - (i as f64) * 10.5;
                let l_rgb = (
                    layer.color_rgba[0] as f64 / 255.0,
                    layer.color_rgba[1] as f64 / 255.0,
                    layer.color_rgba[2] as f64 / 255.0,
                );
                self.fill_rect(lx + 5.0, ry - 0.5, 6.0, 6.0, l_rgb);
                let lbl_txt = fit_rich_text_ellipsis(
                    &format!(
                        "{}: c'={:.0} kPa, φ'={:.0}°, γ={:.1} kN/m³",
                        layer.name, layer.cohesion, layer.friction_angle_deg, layer.gamma
                    ),
                    lw - 20.0,
                    6.5,
                    false,
                );
                self.draw_text_rich(lx + 14.0, ry, 6.5, false, (0.20, 0.25, 0.33), &lbl_txt);
            }
        }

        // Legenda Heatmap + FS (Superior Direito)
        let hm_w = 122.0;
        let hm_h = 45.0;
        let hx = plot_x + plot_w - hm_w - 6.0;
        let hbot = plot_y + plot_h - hm_h - 6.0;
        self.fill_and_stroke_rect(hx, hbot, hm_w, hm_h, (1.0, 1.0, 1.0), (0.80, 0.84, 0.89), 0.5);
        self.draw_text_rich(
            hx + 5.0,
            hbot + hm_h - 10.0,
            7.0,
            true,
            (0.06, 0.09, 0.16),
            if pt { "Heatmap FS (Centros)" } else { "FS Heatmap (Centers)" },
        );
        let bar_w = hm_w - 10.0;
        let segs = 20usize;
        for i in 0..segs {
            let fs_s = 0.8 + (i as f64 / (segs - 1) as f64) * 1.4;
            self.fill_rect(
                hx + 5.0 + (i as f64) * (bar_w / segs as f64),
                hbot + 24.5,
                bar_w / segs as f64 + 0.3,
                6.0,
                heatmap_rgb(fs_s),
            );
        }
        self.draw_text_rich(hx + 5.0, hbot + 16.5, 6.1, false, (0.28, 0.34, 0.42), "≤1.0      1.3      1.5      ≥2.0");
        if let Some(rep) = report {
            let col = if rep.is_stable { (0.09, 0.64, 0.29) } else { (0.86, 0.15, 0.15) };
            self.draw_text_rich(
                hx + 5.0,
                hbot + 5.5,
                7.2,
                true,
                col,
                &format!(
                    "{} = {:.3} ({:.1} ms)",
                    if pt { "FS_{mín}" } else { "FS_{min}" },
                    rep.critical_circle.fs_bishop,
                    rep.elapsed_ms
                ),
            );
        } else {
            self.draw_text_rich(
                hx + 5.0,
                hbot + 5.5,
                6.5,
                true,
                (0.85, 0.47, 0.02),
                if pt { "Executar varredura Aba 2" } else { "Run Tab 2 sweep" },
            );
        }

        let cap_lines = wrap_rich_text(caption, USABLE_W - 14.0, 7.6, false);
        let cap_top = fig_bot + 5.5 + ((cap_lines.len().saturating_sub(1)) as f64) * 9.5;
        for (ci, cl) in cap_lines.iter().take(2).enumerate() {
            self.draw_text_centered(
                PAGE_W * 0.5,
                cap_top - (ci as f64) * 9.5,
                7.6,
                false,
                (0.32, 0.38, 0.48),
                cl,
            );
        }
        self.cursor_y = fig_bot - 12.0;
    }

    /// Figura vetorial completa da Aba 3 — Muro de Contenção:
    /// fundação, embutimento frontal, reaterro β, água H_w, sobrecarga q, corpo + dente de ancoragem,
    /// diagramas de empuxo lateral & hidrostático + vetores E_{a,h} e E_p, tensões na base σ_{ponta}/σ_{calcanhar},
    /// posição da resultante x_R vs B/6, cotas H/B e HUD de verificações normativas.
    pub fn draw_wall_vector_figure(&mut self, project: &GeoProject, caption: &str) {
        let fig_h = 250.0;
        self.ensure_space(fig_h + 24.0);
        let fig_bot = self.cursor_y - fig_h;
        let pt = self.lang == Language::PtBr;

        self.fill_and_stroke_rect(
            MARGIN_X,
            fig_bot,
            USABLE_W,
            fig_h,
            (1.0, 1.0, 1.0),
            (0.79, 0.84, 0.89),
            0.7,
        );

        let plot_x = MARGIN_X + 26.0;
        let plot_y = fig_bot + 26.0;
        let plot_w = USABLE_W - 36.0;
        let plot_h = fig_h - 48.0;
        let plot_bg = (0.973, 0.980, 0.988);
        self.fill_and_stroke_rect(plot_x, plot_y, plot_w, plot_h, plot_bg, (0.82, 0.86, 0.91), 0.5);

        let model = &project.wall_model;
        let title = if pt {
            format!("Aba 3 — Muro de Contenção ({}), Empuxos e Tensões na Base", model.wall_type.name(self.lang))
        } else {
            format!("Tab 3 — Retaining Wall ({}), Lateral Pressures & Base Bearing Stress", model.wall_type.name(self.lang))
        };
        self.draw_text_rich(plot_x + 2.0, fig_bot + fig_h - 15.0, 8.8, true, (0.106, 0.212, 0.365), &title);

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

        let scale = (plot_w / (x_max - x_min)).min(plot_h / (y_max - y_min));
        let cw = (x_max - x_min) * scale;
        let ch = (y_max - y_min) * scale;
        let pad_x = plot_x + (plot_w - cw) * 0.5;
        let pad_y = plot_y + (plot_h - ch) * 0.5;
        let to_p = |wx: f64, wy: f64| -> (f64, f64) { (pad_x + (wx - x_min) * scale, pad_y + (wy - y_min) * scale) };

        let _ = writeln!(self.current, "q {plot_x:.2} {plot_y:.2} {plot_w:.2} {plot_h:.2} re W n");

        let step_x = nice_grid_step(x_max - x_min);
        let step_y = nice_grid_step(y_max - y_min);
        self.set_stroke(0.89, 0.92, 0.95);
        self.set_line_width(0.4);
        let mut gx = (x_min / step_x).floor() * step_x;
        while gx <= x_max + 0.001 {
            let (px, _) = to_p(gx, y_min);
            self.draw_line(px, plot_y, px, plot_y + plot_h);
            gx += step_x;
        }
        let mut gy = (y_min / step_y).floor() * step_y;
        while gy <= y_max + 0.001 {
            let (_, py) = to_p(x_min, gy);
            self.draw_line(plot_x, py, plot_x + plot_w, py);
            gy += step_y;
        }

        // 1. Solo de fundação
        self.fill_polygon(
            &[
                to_p(-1.5, 0.0),
                to_p(b_base + slope_ext + 1.1, 0.0),
                to_p(b_base + slope_ext + 1.1, -1.6),
                to_p(-1.5, -1.6),
            ],
            blend_with_bg((0.47, 0.31, 0.20), plot_bg, 0.24),
        );
        self.draw_polyline(&[to_p(-1.5, 0.0), to_p(b_base + slope_ext + 1.1, 0.0)], (0.63, 0.38, 0.03), 1.0);

        // 2. Embutimento frontal
        if !is_gravity && h_base > 0.01 && b_toe > 0.01 {
            self.fill_polygon(
                &[to_p(-1.5, h_base), to_p(b_toe, h_base), to_p(b_toe, 0.0), to_p(-1.5, 0.0)],
                blend_with_bg((0.63, 0.38, 0.03), plot_bg, 0.28),
            );
            self.draw_polyline(&[to_p(-1.5, h_base), to_p(b_toe, h_base)], (0.63, 0.38, 0.03), 1.2);
        }

        // 3. Reaterro no tardoz + talude β
        let x_back_bot = if is_gravity { b_base } else { b_toe + b_stem_base };
        let x_back_top = if is_gravity { b_stem_top } else { b_toe + b_stem_top };
        let backfill_rgb = blend_with_bg((0.85, 0.62, 0.36), plot_bg, 0.30);
        self.fill_polygon(
            &[
                to_p(x_back_top, h_total),
                to_p(b_base, h_total),
                to_p(b_base, h_base),
                to_p(x_back_bot, h_base),
            ],
            backfill_rgb,
        );
        self.fill_polygon(
            &[
                to_p(b_base, h_total),
                to_p(b_base + slope_ext, h_total + slope_rise),
                to_p(b_base + slope_ext, 0.0),
                to_p(b_base, 0.0),
            ],
            backfill_rgb,
        );
        self.draw_polyline(
            &[
                to_p(x_back_top, h_total),
                to_p(b_base, h_total),
                to_p(b_base + slope_ext, h_total + slope_rise),
            ],
            (0.63, 0.38, 0.03),
            1.4,
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
                x_back_bot + t * (x_back_top - x_back_bot)
            };
            self.fill_polygon(
                &[
                    to_p(x_wl, hw),
                    to_p(b_base + slope_ext, hw),
                    to_p(b_base + slope_ext, 0.0),
                    to_p(b_base, 0.0),
                ],
                blend_with_bg((0.22, 0.74, 0.97), plot_bg, 0.26),
            );
            self.current.push_str("[5 3] 0 d\n");
            self.draw_polyline(&[to_p(x_wl, hw), to_p(b_base + slope_ext, hw)], (0.01, 0.52, 0.78), 1.3);
            self.current.push_str("[] 0 d\n");
        }

        // 5. Sobrecarga q
        if model.surcharge_q > 0.1 {
            let arrows = 5usize;
            let mut caps = Vec::new();
            for i in 0..=arrows {
                let dx = (slope_ext - 0.3) * (i as f64 / arrows as f64) + 0.15;
                let gx_w = b_base + dx;
                let gy_w = h_total + dx * slope_rad.tan();
                let pt_top = to_p(gx_w, gy_w + 0.55);
                let pt_bot = to_p(gx_w, gy_w);
                caps.push(pt_top);
                self.set_stroke(0.85, 0.47, 0.02);
                self.set_line_width(1.0);
                self.draw_line(pt_top.0, pt_top.1, pt_bot.0, pt_bot.1 + 1.5);
                self.fill_polygon(
                    &[pt_bot, (pt_bot.0 - 2.0, pt_bot.1 + 3.8), (pt_bot.0 + 2.0, pt_bot.1 + 3.8)],
                    (0.85, 0.47, 0.02),
                );
            }
            if let (Some(&f), Some(&l)) = (caps.first(), caps.last()) {
                self.set_stroke(0.85, 0.47, 0.02);
                self.set_line_width(1.2);
                self.draw_line(f.0, f.1, l.0, l.1);
            }
            let pl = to_p(b_base + slope_ext * 0.5, h_total + slope_rise * 0.5 + 0.65);
            self.draw_text_centered(
                pl.0,
                pl.1 + 2.0,
                7.2,
                true,
                (0.71, 0.33, 0.04),
                &format!("q = {:.1} kPa", model.surcharge_q),
            );
        }

        // 6. Corpo do muro + Dente de ancoragem
        if is_gravity {
            self.fill_and_stroke_polygon(
                &[to_p(0.0, 0.0), to_p(b_base, 0.0), to_p(b_stem_top, h_total), to_p(0.0, h_total)],
                (0.80, 0.84, 0.88),
                (0.06, 0.09, 0.16),
                1.5,
            );
        } else {
            if h_base > 0.01 {
                let (s1, s2) = (to_p(0.0, 0.0), to_p(b_base, h_base));
                self.fill_and_stroke_rect(
                    s1.0,
                    s1.1,
                    s2.0 - s1.0,
                    s2.1 - s1.1,
                    (0.80, 0.84, 0.88),
                    (0.06, 0.09, 0.16),
                    1.5,
                );
            }
            self.fill_and_stroke_polygon(
                &[
                    to_p(b_toe, h_base),
                    to_p(b_toe + b_stem_base, h_base),
                    to_p(b_toe + b_stem_top, h_total),
                    to_p(b_toe, h_total),
                ],
                (0.80, 0.84, 0.88),
                (0.06, 0.09, 0.16),
                1.5,
            );
        }
        if g.shear_key_enabled && g.shear_key_depth > 0.02 && g.shear_key_width > 0.02 {
            let kx = if is_gravity { b_base * 0.5 } else { b_toe + b_stem_base * 0.5 };
            let (k1, k2) = (to_p(kx - g.shear_key_width * 0.5, -g.shear_key_depth), to_p(kx + g.shear_key_width * 0.5, 0.0));
            self.fill_and_stroke_rect(
                k1.0,
                k1.1,
                k2.0 - k1.0,
                k2.1 - k1.1,
                (0.58, 0.64, 0.72),
                (0.06, 0.09, 0.16),
                1.3,
            );
        }

        // 7. Diagramas de empuxo lateral & hidrostático
        let ka = pressures.ka;
        let sigma_q = model.surcharge_q * ka;
        let sigma_soil = model.backfill_soil.gamma * h_total * ka;
        let sigma_max = (sigma_q + sigma_soil + 9.81 * hw).max(1.0);
        let d_scale = 1.45 / sigma_max;
        let x_ref = b_base + 0.15;
        let w_top = sigma_q * d_scale;
        let w_bot = ((sigma_q + sigma_soil) * d_scale).max(0.05);
        self.fill_and_stroke_polygon(
            &[
                to_p(x_ref, h_total),
                to_p(x_ref + w_top, h_total),
                to_p(x_ref + w_bot, 0.0),
                to_p(x_ref, 0.0),
            ],
            blend_with_bg((0.94, 0.27, 0.27), plot_bg, 0.24),
            (0.86, 0.15, 0.15),
            1.0,
        );
        if hw > 0.05 {
            let ww = (9.81 * hw * d_scale).max(0.05);
            self.fill_and_stroke_polygon(
                &[to_p(x_ref + w_bot, hw), to_p(x_ref + w_bot + ww, 0.0), to_p(x_ref + w_bot, 0.0)],
                blend_with_bg((0.22, 0.74, 0.97), plot_bg, 0.36),
                (0.01, 0.52, 0.78),
                0.9,
            );
        }

        // Vetor resultante Ea,h
        let pt_tgt = to_p(b_base, pressures.ea_arm_y);
        let pt_org = to_p(b_base + 2.15, pressures.ea_arm_y);
        self.set_stroke(0.86, 0.15, 0.15);
        self.set_line_width(1.7);
        self.draw_line(pt_org.0, pt_org.1, pt_tgt.0 + 2.0, pt_tgt.1);
        self.fill_polygon(
            &[pt_tgt, (pt_tgt.0 + 5.8, pt_tgt.1 + 2.7), (pt_tgt.0 + 5.8, pt_tgt.1 - 2.7)],
            (0.86, 0.15, 0.15),
        );
        let eah_txt = format!("E_{{a,h}} = {:.1} kN/m", pressures.ea_horizontal);
        let eah_w = measure_rich_text(&eah_txt, 7.2, true);
        let eah_x = (pt_org.0 + 4.0).clamp(plot_x + 6.0, (plot_x + plot_w - eah_w - 8.0).max(plot_x + 6.0));
        // Evita sobreposição vertical com o card HUD no canto superior direito
        let hud_bot_limit = plot_y + plot_h - 78.0;
        let eah_y = if pt_org.1 >= hud_bot_limit && eah_x + eah_w >= plot_x + plot_w - 178.0 {
            hud_bot_limit - 12.0
        } else {
            pt_org.1 + 2.5
        };
        self.draw_text_rich(eah_x, eah_y, 7.2, true, (0.73, 0.11, 0.11), &eah_txt);
        self.draw_text_rich(eah_x, eah_y - 9.5, 6.5, false, (0.73, 0.11, 0.11), &format!("(y = {:.2} m)", pressures.ea_arm_y));

        // Vetor passivo Ep
        if pressures.ep_horizontal > 0.5 && h_base > 0.05 {
            let ep_s = to_p(-1.05, h_base / 3.0);
            let ep_e = to_p(0.0, h_base / 3.0);
            self.set_stroke(0.09, 0.64, 0.29);
            self.set_line_width(1.5);
            self.draw_line(ep_s.0, ep_s.1, ep_e.0 - 2.0, ep_e.1);
            self.fill_polygon(
                &[ep_e, (ep_e.0 - 5.2, ep_e.1 + 2.4), (ep_e.0 - 5.2, ep_e.1 - 2.4)],
                (0.09, 0.64, 0.29),
            );
            let ep_lbl = format!("E_p = {:.1} kN/m", pressures.ep_horizontal);
            let ep_w = measure_rich_text(&ep_lbl, 6.8, true);
            let ep_x = (ep_s.0 - 3.0 - ep_w).max(plot_x + 4.0);
            self.draw_text_rich(ep_x, ep_s.1 + 4.5, 6.8, true, (0.08, 0.50, 0.24), &ep_lbl);
        }

        // 8. Tensões na base + marcador xR
        let max_sig = stability.sigma_toe_kpa.max(stability.sigma_heel_kpa).max(1.0);
        let depth_max = 1.05;
        let d_toe = (stability.sigma_toe_kpa / max_sig).clamp(0.05, 1.0) * depth_max;
        let d_heel = (stability.sigma_heel_kpa / max_sig).clamp(0.0, 1.0) * depth_max;
        let sig_rgb = if stability.is_bearing_stress_ok { (0.15, 0.39, 0.92) } else { (0.86, 0.15, 0.15) };
        let (pb0, pbb, pst, psh) = (to_p(0.0, 0.0), to_p(b_base, 0.0), to_p(0.0, -d_toe), to_p(b_base, -d_heel));
        let sig_fill = blend_with_bg(sig_rgb, plot_bg, 0.24);
        if d_heel > 0.01 {
            self.fill_and_stroke_polygon(&[pb0, pbb, psh, pst], sig_fill, sig_rgb, 1.2);
        } else {
            let x_c = (3.0 * stability.resultant_position_xr).clamp(0.05, b_base);
            self.fill_and_stroke_polygon(&[pb0, to_p(x_c, 0.0), pst], sig_fill, sig_rgb, 1.2);
        }
        let toe_lbl = format!("{} = {:.1} kPa", if pt { "σ_{ponta}" } else { "σ_{toe}" }, stability.sigma_toe_kpa);
        let toe_w = measure_rich_text(&toe_lbl, 6.8, true);
        let toe_x = (pst.0 - 3.0 - toe_w).max(plot_x + 4.0);
        self.draw_text_rich(toe_x, pst.1 - 8.5, 6.8, true, sig_rgb, &toe_lbl);

        let heel_lbl = format!("{} = {:.1} kPa", if pt { "σ_{calcanhar}" } else { "σ_{heel}" }, stability.sigma_heel_kpa);
        let heel_w = measure_rich_text(&heel_lbl, 6.8, true);
        let heel_x = (psh.0 + 3.0).min((plot_x + plot_w - heel_w - 4.0).max(plot_x + 4.0));
        self.draw_text_rich(heel_x, psh.1 - 8.5, 6.8, true, sig_rgb, &heel_lbl);

        let xr_pt = to_p(stability.resultant_position_xr, 0.0);
        let xr_rgb = if stability.is_in_middle_third { (0.09, 0.64, 0.29) } else { (0.85, 0.47, 0.02) };
        self.fill_circle(xr_pt.0, xr_pt.1, 2.7, xr_rgb);
        let xr_lbl = to_p(b_base * 0.5, -(depth_max + 0.22));
        let xr_txt = format!(
            "x_R = {:.2} m | e = {:.3} m (lim B/6 = {:.3} m)",
            stability.resultant_position_xr, stability.eccentricity, stability.max_allowed_eccentricity
        );
        let xr_fit = fit_rich_text_ellipsis(&xr_txt, plot_w - 12.0, 6.8, true);
        self.draw_text_centered(
            xr_lbl.0.clamp(plot_x + 80.0, plot_x + plot_w - 80.0),
            (xr_lbl.1 - 6.5).max(plot_y + 3.5),
            6.8,
            true,
            xr_rgb,
            &xr_fit,
        );

        // 9. Cotas dimensionais H e B
        let (hd0, hd1) = (to_p(-0.48, 0.0), to_p(-0.48, h_total));
        self.set_stroke(0.39, 0.45, 0.55);
        self.set_line_width(0.8);
        self.draw_line(hd0.0, hd0.1, hd1.0, hd1.1);
        self.draw_text_right(hd0.0 - 3.5, (hd0.1 + hd1.1) * 0.5 - 2.0, 7.0, true, (0.28, 0.34, 0.42), &format!("H = {:.2} m", h_total));
        let (bd0, bd1) = (to_p(0.0, h_total + 0.25), to_p(b_base, h_total + 0.25));
        self.set_stroke(0.39, 0.45, 0.55);
        self.set_line_width(0.8);
        self.draw_line(bd0.0, bd0.1, bd1.0, bd1.1);
        self.draw_text_centered((bd0.0 + bd1.0) * 0.5, bd0.1 + 3.5, 7.0, true, (0.28, 0.34, 0.42), &format!("B = {:.2} m", b_base));

        // Fim do clipping
        self.current.push_str("Q\n");

        // Rótulos dos eixos
        let mut tx = (x_min / step_x).ceil() * step_x;
        while tx <= x_max + 0.001 {
            let (px, _) = to_p(tx, y_min);
            if px >= plot_x + 10.0 && px <= plot_x + plot_w - 10.0 {
                self.draw_text_centered(px, plot_y - 9.0, 6.8, false, (0.39, 0.45, 0.55), &format!("{tx:.0} m"));
            }
            tx += step_x;
        }
        let mut ty = (y_min / step_y).ceil() * step_y;
        while ty <= y_max + 0.001 {
            let (_, py) = to_p(x_min, ty);
            if py >= plot_y + 6.0 && py <= plot_y + plot_h - 6.0 {
                self.draw_text_right(plot_x - 3.0, py - 2.0, 6.8, false, (0.39, 0.45, 0.55), &format!("{ty:.0} m"));
            }
            ty += step_y;
        }

        // HUD Card de Verificações Normativas (Superior Direito)
        let card_w = 166.0;
        let card_h = 64.0;
        let cx = plot_x + plot_w - card_w - 6.0;
        let cbot = plot_y + plot_h - card_h - 6.0;
        self.fill_and_stroke_rect(cx, cbot, card_w, card_h, (1.0, 1.0, 1.0), (0.80, 0.84, 0.89), 0.5);
        self.draw_text_rich(
            cx + 6.0,
            cbot + card_h - 11.0,
            7.2,
            true,
            (0.06, 0.09, 0.16),
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
            let cy_row = cbot + card_h - 23.0 - (idx as f64) * 11.2;
            let col = if *ok { (0.09, 0.64, 0.29) } else { (0.86, 0.15, 0.15) };
            // Ícone vetorial check / cross
            self.set_stroke(col.0, col.1, col.2);
            self.set_line_width(1.1);
            if *ok {
                self.draw_polyline(
                    &[(cx + 6.5, cy_row + 2.5), (cx + 8.5, cy_row + 0.5), (cx + 12.0, cy_row + 5.0)],
                    col,
                    1.1,
                );
            } else {
                self.draw_line(cx + 6.5, cy_row + 0.5, cx + 11.5, cy_row + 5.0);
                self.draw_line(cx + 6.5, cy_row + 5.0, cx + 11.5, cy_row + 0.5);
            }
            self.draw_text_rich(cx + 16.0, cy_row, 6.8, true, col, lbl);
        }

        let cap_lines = wrap_rich_text(caption, USABLE_W - 14.0, 7.6, false);
        let cap_top = fig_bot + 5.5 + ((cap_lines.len().saturating_sub(1)) as f64) * 9.5;
        for (ci, cl) in cap_lines.iter().take(2).enumerate() {
            self.draw_text_centered(
                PAGE_W * 0.5,
                cap_top - (ci as f64) * 9.5,
                7.6,
                false,
                (0.32, 0.38, 0.48),
                cl,
            );
        }
        self.cursor_y = fig_bot - 12.0;
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.pages.push(self.current);
        let num_pages = self.pages.len();
        let pt = self.lang == Language::PtBr;

        for (idx, page) in self.pages.iter_mut().enumerate() {
            let footer_txt = if pt {
                format!("GeoEstabil — A4 ABNT | Página {} de {}", idx + 1, num_pages)
            } else {
                format!("GeoEstabil — A4 Engineering | Page {} of {}", idx + 1, num_pages)
            };
            let _ = writeln!(page, "0.80 0.84 0.89 RG 0.5 w {MARGIN_X:.1} 30.0 m {:.1} 30.0 l S", PAGE_W - MARGIN_X);
            let fw = measure_rich_text(&footer_txt, 7.6, false);
            let fx = PAGE_W * 0.5 - fw * 0.5;
            let winansi_bytes: Vec<u8> = footer_txt
                .chars()
                .map(|c| map_char_pdf(c, PdfFont::Helvetica).1)
                .collect();
            let _ = writeln!(
                page,
                "0.42 0.48 0.56 rg BT /F1 7.6 Tf 1 0 0 1 {fx:.2} 18.0 Tm ({}) Tj ET",
                escape_pdf_bytes(&winansi_bytes)
            );
        }

        let mut pdf = Vec::<u8>::with_capacity(64_000);
        pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

        // 1: Catalog, 2: Pages, 3: F1 Helvetica, 4: F2 Helvetica-Bold,
        // 5: F3 Symbol (+ ToUnicode 6), 6: ToUnicode CMap stream, 7: F4 Helvetica-Oblique
        let total_objs = 7 + num_pages * 2;
        let mut offsets = vec![0usize; total_objs + 1];

        let write_obj = |out: &mut Vec<u8>, id: usize, body: &[u8], offs: &mut [usize]| {
            offs[id] = out.len();
            out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
            out.extend_from_slice(body);
            out.extend_from_slice(b"\nendobj\n");
        };

        write_obj(&mut pdf, 1, b"<< /Type /Catalog /Pages 2 0 R >>", &mut offsets);

        let mut kids = String::new();
        for i in 0..num_pages {
            let page_obj = 8 + i * 2;
            let _ = write!(kids, "{page_obj} 0 R ");
        }
        let pages_body = format!("<< /Type /Pages /Kids [{kids}] /Count {num_pages} >>");
        write_obj(&mut pdf, 2, pages_body.as_bytes(), &mut offsets);

        write_obj(
            &mut pdf,
            3,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
            &mut offsets,
        );
        write_obj(
            &mut pdf,
            4,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>",
            &mut offsets,
        );
        write_obj(
            &mut pdf,
            5,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Symbol /ToUnicode 6 0 R >>",
            &mut offsets,
        );
        let cmap_bytes = build_symbol_tounicode_cmap();
        let mut cmap_stream = format!("<< /Length {} >>\nstream\n", cmap_bytes.len()).into_bytes();
        cmap_stream.extend_from_slice(&cmap_bytes);
        cmap_stream.extend_from_slice(b"\nendstream");
        write_obj(&mut pdf, 6, &cmap_stream, &mut offsets);

        write_obj(
            &mut pdf,
            7,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Oblique /Encoding /WinAnsiEncoding >>",
            &mut offsets,
        );

        for (i, content) in self.pages.iter().enumerate() {
            let page_obj = 8 + i * 2;
            let stream_obj = 9 + i * 2;
            let page_dict = format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W:.2} {PAGE_H:.2}] /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 7 0 R >> >> /Contents {stream_obj} 0 R >>"
            );
            write_obj(&mut pdf, page_obj, page_dict.as_bytes(), &mut offsets);

            let stream_bytes = content.as_bytes();
            let mut full_stream = format!("<< /Length {} >>\nstream\n", stream_bytes.len()).into_bytes();
            full_stream.extend_from_slice(stream_bytes);
            full_stream.extend_from_slice(b"\nendstream");
            write_obj(&mut pdf, stream_obj, &full_stream, &mut offsets);
        }

        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", total_objs + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for off in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n", total_objs + 1).as_bytes(),
        );
        pdf
    }
}
