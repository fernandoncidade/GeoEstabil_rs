use crate::locale::translations::Language;
use crate::ui::theme::GeoEstabilTheme;
use eframe::egui::{
    self, Align, Color32, CornerRadius, FontFamily, FontId, Frame, Grid, Margin, ScrollArea,
    Stroke, TextFormat, Ui, text::LayoutJob,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocBlock {
    Banner(String),
    Heading2(String),
    Heading3(String),
    Heading4(String),
    Equation(String),
    Bullet(String),
    SubBullet(String),
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Diagram(String),
    Separator,
    Paragraph(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathFractionParts {
    pub prefix: String,
    pub numerator: String,
    pub denominator: String,
    pub suffix: String,
}

fn is_table_separator_row(trimmed: &str) -> bool {
    if !trimmed.starts_with('|') {
        return false;
    }
    trimmed
        .trim_matches('|')
        .split('|')
        .all(|cell| {
            let c = cell.trim();
            !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')
        })
}

fn split_table_cells(trimmed: &str) -> Vec<String> {
    let mut s = trimmed.trim();
    if s.starts_with('|') {
        s = &s[1..];
    }
    if s.ends_with('|') {
        s = &s[..s.len() - 1];
    }
    s.split('|').map(|c| c.trim().to_string()).collect()
}

pub fn parse_doc_blocks(raw: &str) -> Vec<DocBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = raw.lines().collect();
    let mut i = 0usize;
    let mut para_buf: Vec<String> = Vec::new();

    let flush_para = |buf: &mut Vec<String>, out: &mut Vec<DocBlock>| {
        if !buf.is_empty() {
            out.push(DocBlock::Paragraph(buf.join(" ")));
            buf.clear();
        }
    };

    while i < lines.len() {
        let line_raw = lines[i];
        let trimmed = line_raw.trim();

        if trimmed.is_empty() {
            flush_para(&mut para_buf, &mut blocks);
            i += 1;
            continue;
        }

        // Banner multilinha (==== ... ====) ou linha única (==== TÍTULO ====)
        if trimmed == "====" || trimmed.chars().all(|c| c == '=') && trimmed.len() >= 4 {
            flush_para(&mut para_buf, &mut blocks);
            i += 1;
            let mut banner_lines = Vec::new();
            while i < lines.len() {
                let bt = lines[i].trim();
                if bt == "====" || (bt.len() >= 4 && bt.chars().all(|c| c == '=')) {
                    i += 1;
                    break;
                }
                if !bt.is_empty() {
                    banner_lines.push(bt.to_string());
                }
                i += 1;
            }
            if !banner_lines.is_empty() {
                blocks.push(DocBlock::Banner(banner_lines.join("\n")));
            }
            continue;
        }

        if trimmed.starts_with("====") && trimmed.ends_with("====") && trimmed.len() > 8 {
            flush_para(&mut para_buf, &mut blocks);
            let title = trimmed.trim_matches('=').trim().to_string();
            blocks.push(DocBlock::Banner(title));
            i += 1;
            continue;
        }

        // Diagrama vetorial [[DIAGRAM:CODE]]
        if trimmed.starts_with("[[DIAGRAM:") && trimmed.ends_with("]]") {
            flush_para(&mut para_buf, &mut blocks);
            let code = trimmed
                .trim_start_matches("[[DIAGRAM:")
                .trim_end_matches("]]")
                .trim()
                .to_string();
            blocks.push(DocBlock::Diagram(code));
            i += 1;
            continue;
        }

        // Separador horizontal (---- / ---)
        if (trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-'))
            || (trimmed.len() >= 4 && trimmed.chars().all(|c| c == '—' || c == '─'))
        {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(DocBlock::Separator);
            i += 1;
            continue;
        }

        // Cabeçalhos H4, H3, H2
        if let Some(h4) = trimmed.strip_prefix("#### ") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(DocBlock::Heading4(h4.trim().to_string()));
            i += 1;
            continue;
        }
        if let Some(h3) = trimmed.strip_prefix("### ") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(DocBlock::Heading3(h3.trim().to_string()));
            i += 1;
            continue;
        }
        if let Some(h2) = trimmed.strip_prefix("## ") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(DocBlock::Heading2(h2.trim().to_string()));
            i += 1;
            continue;
        }
        if let Some(h1) = trimmed.strip_prefix("# ") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(DocBlock::Banner(h1.trim().to_string()));
            i += 1;
            continue;
        }

        // Cartão de equação [EQ] ou $$ ... $$
        if let Some(eq) = trimmed.strip_prefix("[EQ]") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(DocBlock::Equation(normalize_inline_latex(eq.trim())));
            i += 1;
            continue;
        }
        if trimmed.starts_with("$$") {
            flush_para(&mut para_buf, &mut blocks);
            if trimmed.len() > 4 && trimmed.ends_with("$$") {
                let inner = &trimmed[2..trimmed.len() - 2];
                blocks.push(DocBlock::Equation(normalize_inline_latex(inner.trim())));
                i += 1;
                continue;
            }
            let mut eq_buf = String::from(trimmed.trim_start_matches("$$").trim());
            i += 1;
            while i < lines.len() {
                let l_cur = lines[i].trim();
                if let Some(before_end) = l_cur.strip_suffix("$$") {
                    if !before_end.trim().is_empty() {
                        if !eq_buf.is_empty() {
                            eq_buf.push(' ');
                        }
                        eq_buf.push_str(before_end.trim());
                    }
                    i += 1;
                    break;
                }
                if !l_cur.is_empty() {
                    if !eq_buf.is_empty() {
                        eq_buf.push(' ');
                    }
                    eq_buf.push_str(l_cur);
                }
                i += 1;
            }
            blocks.push(DocBlock::Equation(normalize_inline_latex(eq_buf.trim())));
            continue;
        }

        // Tabelas Markdown | col1 | col2 |
        if trimmed.starts_with('|') && trimmed.contains('|') {
            flush_para(&mut para_buf, &mut blocks);
            let mut headers = Vec::new();
            let mut rows = Vec::new();
            while i < lines.len() {
                let tl = lines[i].trim();
                if !tl.starts_with('|') {
                    break;
                }
                if is_table_separator_row(tl) {
                    i += 1;
                    continue;
                }
                let cells = split_table_cells(tl);
                if headers.is_empty() {
                    headers = cells;
                } else {
                    rows.push(cells);
                }
                i += 1;
            }
            if !headers.is_empty() {
                blocks.push(DocBlock::Table { headers, rows });
            }
            continue;
        }

        // Sub-tópicos indentados (`  - ` ou `  • ` ou `\t- `)
        let has_leading_indent = line_raw.starts_with("  ") || line_raw.starts_with('\t');
        if has_leading_indent {
            if let Some(sb) = trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("• "))
            {
                flush_para(&mut para_buf, &mut blocks);
                blocks.push(DocBlock::SubBullet(sb.trim().to_string()));
                i += 1;
                continue;
            }
        }

        // Marcadores principais (`• ` ou `- `)
        if let Some(b) = trimmed
            .strip_prefix("• ")
            .or_else(|| trimmed.strip_prefix("- "))
        {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(DocBlock::Bullet(b.trim().to_string()));
            i += 1;
            continue;
        }

        para_buf.push(trimmed.to_string());
        i += 1;
    }

    flush_para(&mut para_buf, &mut blocks);
    blocks
}

fn strip_inline_markup_for_search(s: &str) -> String {
    s.replace("**", "")
}

pub fn count_matches_ci(haystack: &str, query: &str) -> usize {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return 0;
    }
    strip_inline_markup_for_search(haystack)
        .to_lowercase()
        .matches(&q)
        .count()
}

fn is_unicode_subscript(c: char) -> bool {
    matches!(
        c,
        '₀'..='₉'
            | 'ₐ'
            | 'ₑ'
            | 'ₒ'
            | 'ₓ'
            | 'ₕ'
            | 'ₖ'
            | 'ₗ'
            | 'ₘ'
            | 'ₙ'
            | 'ₚ'
            | 'ₛ'
            | 'ₜ'
            | 'ᵢ'
            | 'ⱼ'
            | 'ᵣ'
            | 'ᵤ'
            | 'ᵥ'
            | '₊'
            | '₋'
            | '₌'
            | '₍'
            | '₎'
    )
}

fn is_unicode_superscript(c: char) -> bool {
    matches!(
        c,
        '⁰' | '¹' | '²' | '³' | '⁴'..='⁹' | '⁺' | '⁻' | '⁼' | '⁽' | '⁾' | 'ⁿ' | 'ⁱ'
    )
}

fn is_math_operator(c: char) -> bool {
    matches!(
        c,
        '=' | '≤' | '≥' | '≠' | '≈' | '<' | '>' | '+' | '−' | '×' | '·' | '÷' | '±' | '∞' | '∂'
    )
}

fn find_char_match_spans(text: &str, query: &str) -> Vec<(usize, usize)> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lower_buf = String::with_capacity(text.len());
    let mut byte_to_char: Vec<usize> = Vec::new();
    for (ci, &ch) in chars.iter().enumerate() {
        for lc in ch.to_lowercase() {
            let mut b = [0u8; 4];
            let s = lc.encode_utf8(&mut b);
            for _ in 0..s.len() {
                byte_to_char.push(ci);
            }
            lower_buf.push_str(s);
        }
    }
    if lower_buf.is_empty() {
        return Vec::new();
    }
    let mut spans = Vec::new();
    let mut pos = 0usize;
    while let Some(found) = lower_buf[pos..].find(&q) {
        let abs_start = pos + found;
        let abs_end = abs_start + q.len();
        if abs_start < byte_to_char.len() && abs_end > 0 && abs_end - 1 < byte_to_char.len() {
            let c_start = byte_to_char[abs_start];
            let c_end = byte_to_char[abs_end - 1] + 1;
            spans.push((c_start, c_end));
        }
        pos = abs_start + q.len().max(1);
    }
    spans
}

fn span_is_highlighted(
    char_idx: usize,
    spans: &[(usize, usize)],
    match_counter_base: usize,
    active_match_index: usize,
    query_non_empty: bool,
    hit_active: &mut bool,
) -> (bool, bool) {
    if !query_non_empty {
        return (false, false);
    }
    for (k, &(s, e)) in spans.iter().enumerate() {
        if char_idx >= s && char_idx < e {
            let is_act = match_counter_base + k == active_match_index;
            if is_act {
                *hit_active = true;
            }
            return (true, is_act);
        }
    }
    (false, false)
}

fn needs_math_fallback_font(c: char, family: &FontFamily) -> bool {
    if !cfg!(target_os = "windows") {
        return false;
    }
    // Ubuntu-Light.ttf e Hack-Regular.ttf não contêm diversos símbolos matemáticos/setas/subscritos
    matches!(
        c,
        '←' | '→'
            | '↺'
            | '∅'
            | '∈'
            | '≅'
            | '≫'
            | '─'
            | '▪'
            | '▶'
            | '◀'
            | '●'
            | '′'
            | '″'
            | '\u{0304}'
            | '\u{0302}'
            | 'ⁿ'
            | 'ᵃ'
            | 'ᶜ'
            | 'ᵢ'
            | 'ᵣ'
            | 'ᵤ'
            | 'ᵥ'
            | 'ⁱ'
            | 'ⱼ'
            | '⁺'
            | '⁻'
            | '⁼'
            | '⁽'
            | '⁾'
            | '₊'
            | '₋'
            | '₌'
            | '₍'
            | '₎'
            | 'ₐ'
            | 'ₑ'
            | 'ₒ'
            | 'ₓ'
            | 'ₕ'
            | 'ₖ'
            | 'ₗ'
            | 'ₘ'
            | 'ₙ'
            | 'ₚ'
            | 'ₛ'
            | 'ₜ'
    ) || (*family == FontFamily::Monospace
        && (c == 'ℓ' || is_unicode_subscript(c) || is_unicode_superscript(c)))
}

fn find_balanced_group(chars: &[char], open_idx: usize) -> Option<usize> {
    if open_idx >= chars.len() {
        return None;
    }
    let open = chars[open_idx];
    let close = match open {
        '{' => '}',
        '(' => ')',
        _ => return None,
    };
    let mut depth = 0i32;
    for (idx, &ch) in chars.iter().enumerate().skip(open_idx) {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn is_non_math_underscore_token(chars: &[char], underscore_idx: usize) -> bool {
    let is_token_char = |ch: char| {
        !ch.is_whitespace() && !matches!(ch, '`' | '*' | '"' | '(' | ')' | '[' | ']' | '<' | '>' | ';')
    };
    let mut start = underscore_idx;
    while start > 0 && is_token_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = underscore_idx + 1;
    while end < chars.len() && is_token_char(chars[end]) {
        end += 1;
    }
    let token: String = chars[start..end].iter().collect();
    if token.contains("://") || token.contains('@') || token.contains("www.") {
        return true;
    }
    let low = token.to_lowercase();
    if [
        ".rs", ".txt", ".md", ".db", ".html", ".svg", ".pdf", ".toml", ".json", ".iss", ".exe",
    ]
    .iter()
    .any(|ext| low.ends_with(ext))
    {
        return true;
    }
    let before: String = chars[start..underscore_idx].iter().collect();
    let after: String = chars[underscore_idx + 1..end].iter().collect();
    if !after.starts_with('{')
        && !after.starts_with('(')
        && before.chars().all(|c| c.is_ascii_lowercase())
        && before.len() >= 5
    {
        // Identificadores snake_case de código/arquivo (ex: linceu_lighthouse, geoestabil_cli)
        return true;
    }
    before.len() >= 3
        && after.len() >= 3
        && before.chars().all(|c| c.is_ascii_uppercase())
        && after.chars().all(|c| c.is_ascii_uppercase())
}

fn append_fragment_with_math(
    job: &mut LayoutJob,
    fragment: &str,
    font_size: f32,
    family: &FontFamily,
    base_color: Color32,
    emphasis: bool,
    dark_mode: bool,
    spans: &[(usize, usize)],
    char_offset: usize,
    match_counter_base: usize,
    active_match_index: usize,
    hit_active: &mut bool,
) {
    let chars: Vec<char> = fragment.chars().collect();
    let n = chars.len();
    let query_non_empty = !spans.is_empty();

    let sub_color = if dark_mode {
        Color32::from_rgb(135, 205, 255)
    } else {
        Color32::from_rgb(12, 80, 175)
    };
    let sup_color = GeoEstabilTheme::warning_amber(dark_mode);
    let op_color = if dark_mode {
        Color32::from_rgb(115, 175, 250)
    } else {
        Color32::from_rgb(28, 92, 185)
    };
    let hi_bg = if dark_mode {
        Color32::from_rgb(180, 130, 18)
    } else {
        Color32::from_rgb(253, 224, 71)
    };
    let act_bg = if dark_mode {
        Color32::from_rgb(234, 88, 12)
    } else {
        Color32::from_rgb(251, 146, 60)
    };
    let hi_fg = if dark_mode {
        Color32::WHITE
    } else {
        Color32::from_rgb(15, 23, 42)
    };

    let make_fmt = |sz: f32,
                    col: Color32,
                    valign: Align,
                    italics: bool,
                    abs_ci: usize,
                    hit_flag: &mut bool|
     -> TextFormatWrapper {
        let (is_hl, is_act) = span_is_highlighted(
            abs_ci,
            spans,
            match_counter_base,
            active_match_index,
            query_non_empty,
            hit_flag,
        );
        TextFormatWrapper {
            font_id: FontId::new(sz, family.clone()),
            color: if is_hl { hi_fg } else { col },
            background: if is_act {
                act_bg
            } else if is_hl {
                hi_bg
            } else {
                Color32::TRANSPARENT
            },
            valign,
            italics,
        }
    };

    let append_script_span = |job: &mut LayoutJob,
                              start: usize,
                              end: usize,
                              script_sz: f32,
                              script_col: Color32,
                              script_valign: Align,
                              hit_active: &mut bool| {
        let mut k = start;
        while k < end {
            let ch = chars[k];
            if (ch == '_' || ch == '^') && k + 1 < end {
                let inner_col = if ch == '_' { sub_color } else { sup_color };
                let inner_valign = if ch == '_' { Align::Max } else { Align::Min };
                let inner_sz = script_sz * 0.85;
                if chars[k + 1] == '{' || chars[k + 1] == '(' {
                    if let Some(inner_end) = find_balanced_group(&chars, k + 1) {
                        for m in (k + 2)..inner_end.min(end) {
                            let ic = chars[m];
                            if matches!(ic, '{' | '}' | '_' | '^') {
                                continue;
                            }
                            let mut w = make_fmt(inner_sz, inner_col, inner_valign, false, char_offset + m, hit_active);
                            if needs_math_fallback_font(ic, family) {
                                w.font_id = FontId::new(inner_sz, FontFamily::Name("math_sym".into()));
                            }
                            job.append(&ic.to_string(), 0.0, w.into());
                        }
                        k = inner_end.min(end) + 1;
                        continue;
                    }
                } else {
                    k += 1;
                    while k < end && (chars[k].is_alphanumeric() || chars[k] == '\'' || chars[k] == '′') {
                        let ic = chars[k];
                        let mut w = make_fmt(inner_sz, inner_col, inner_valign, false, char_offset + k, hit_active);
                        if needs_math_fallback_font(ic, family) {
                            w.font_id = FontId::new(inner_sz, FontFamily::Name("math_sym".into()));
                        }
                        job.append(&ic.to_string(), 0.0, w.into());
                        k += 1;
                    }
                    continue;
                }
            }
            if ch == '{' || ch == '}' {
                k += 1;
                continue;
            }
            let mut w = make_fmt(script_sz, script_col, script_valign, false, char_offset + k, hit_active);
            if needs_math_fallback_font(ch, family) {
                w.font_id = FontId::new(script_sz, FontFamily::Name("math_sym".into()));
            }
            job.append(&ch.to_string(), 0.0, w.into());
            k += 1;
        }
    };

    let mut i = 0usize;
    while i < n {
        let c = chars[i];
        let abs_i = char_offset + i;

        // 1. Subscrito explícito `_` (com suporte a chaves aninhadas e índices não-braceados como V_base)
        if c == '_' && i + 1 < n && !is_non_math_underscore_token(&chars, i) {
            if chars[i + 1] == '{' || chars[i + 1] == '(' {
                if let Some(end) = find_balanced_group(&chars, i + 1) {
                    let start = i + 2;
                    if end > start {
                        append_script_span(job, start, end, font_size * 0.74, sub_color, Align::Max, hit_active);
                        i = end + 1;
                        continue;
                    }
                }
            } else {
                let start = i + 1;
                let mut end = start;
                while end < n {
                    let ch = chars[end];
                    if ch.is_alphanumeric() || ch == 'ℓ' || ch == '\'' || ch == '′' {
                        end += 1;
                    } else if (ch == ',' || ch == '_')
                        && end + 1 < n
                        && (chars[end + 1].is_alphanumeric() || chars[end + 1] == 'ℓ')
                    {
                        end += 1;
                    } else {
                        break;
                    }
                }
                if end > start {
                    append_script_span(job, start, end, font_size * 0.74, sub_color, Align::Max, hit_active);
                    i = end;
                    continue;
                }
            }
        }

        // 2. Sobrescrito / Expoente explícito `^` (com suporte a chaves/parênteses aninhados)
        if c == '^' && i + 1 < n {
            if chars[i + 1] == '{' || chars[i + 1] == '(' {
                if let Some(end) = find_balanced_group(&chars, i + 1) {
                    let start = i + 2;
                    if end > start {
                        append_script_span(job, start, end, font_size * 0.74, sup_color, Align::Min, hit_active);
                        i = end + 1;
                        continue;
                    }
                }
            } else {
                let start = i + 1;
                let mut end = start;
                while end < n {
                    let ch = chars[end];
                    if ch.is_alphanumeric() || ch == '−' || ch == '+' || ch == '\'' || ch == '′' {
                        end += 1;
                    } else if ch == '-' && end == start {
                        end += 1;
                    } else if (ch == '.' || ch == ',')
                        && end + 1 < n
                        && chars[end + 1].is_alphanumeric()
                    {
                        end += 1;
                    } else {
                        break;
                    }
                }
                if end > start {
                    for k in start..end {
                        let ch = chars[k];
                        let mut w = make_fmt(
                            font_size * 0.74,
                            sup_color,
                            Align::Min,
                            false,
                            char_offset + k,
                            hit_active,
                        );
                        if needs_math_fallback_font(ch, family) {
                            w.font_id = FontId::new(
                                font_size * 0.74,
                                FontFamily::Name("math_sym".into()),
                            );
                        }
                        job.append(&ch.to_string(), 0.0, w.into());
                    }
                    i = end;
                    continue;
                }
            }
        }

        // 3. Subscrito Unicode
        if is_unicode_subscript(c) {
            let mut w = make_fmt(
                font_size * 0.78,
                sub_color,
                Align::Max,
                false,
                abs_i,
                hit_active,
            );
            if needs_math_fallback_font(c, family) {
                w.font_id = FontId::new(font_size * 0.78, FontFamily::Name("math_sym".into()));
            }
            job.append(&c.to_string(), 0.0, w.into());
            i += 1;
            continue;
        }

        // 4. Sobrescrito Unicode
        if is_unicode_superscript(c) {
            let mut w = make_fmt(
                font_size * 0.78,
                sup_color,
                Align::Min,
                false,
                abs_i,
                hit_active,
            );
            if needs_math_fallback_font(c, family) {
                w.font_id = FontId::new(font_size * 0.78, FontFamily::Name("math_sym".into()));
            }
            job.append(&c.to_string(), 0.0, w.into());
            i += 1;
            continue;
        }

        // 5. Símbolos grandes (∑, ∫, √)
        if c == '∑' || c == '∫' || c == '∏' {
            let w = make_fmt(
                font_size * 1.28,
                sup_color,
                Align::Center,
                false,
                abs_i,
                hit_active,
            );
            job.append(&c.to_string(), 0.0, w.into());
            i += 1;
            continue;
        }
        if c == '√' {
            let w = make_fmt(
                font_size * 1.20,
                sub_color,
                Align::Center,
                false,
                abs_i,
                hit_active,
            );
            job.append(&c.to_string(), 0.0, w.into());
            i += 1;
            continue;
        }

        // 6. Operadores matemáticos
        if is_math_operator(c) {
            let sz = if emphasis { font_size + 0.3 } else { font_size };
            let w = make_fmt(sz, op_color, Align::Center, false, abs_i, hit_active);
            job.append(&c.to_string(), 0.0, w.into());
            i += 1;
            continue;
        }

        // 7. Caractere comum (inclui fallback math_sym para ∅, ∈, ′, ″, ℓ mono, etc.)
        let sz = if emphasis { font_size + 0.35 } else { font_size };
        let mut w = make_fmt(sz, base_color, Align::Center, false, abs_i, hit_active);
        if needs_math_fallback_font(c, family) {
            w.font_id = FontId::new(sz, FontFamily::Name("math_sym".into()));
        }
        job.append(&c.to_string(), 0.0, w.into());
        i += 1;
    }
}

struct TextFormatWrapper {
    font_id: FontId,
    color: Color32,
    background: Color32,
    valign: Align,
    italics: bool,
}

impl From<TextFormatWrapper> for TextFormat {
    fn from(w: TextFormatWrapper) -> Self {
        egui::TextFormat {
            font_id: w.font_id,
            color: w.color,
            background: w.background,
            valign: w.valign,
            italics: w.italics,
            ..Default::default()
        }
    }
}

fn normalize_inline_latex(raw: &str) -> String {
    let mut s = crate::reports::pdf_writer::normalize_math_symbols(raw);
    if !s.contains('$') {
        return s;
    }
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;
    while i < n {
        if chars[i] == '$' && i + 1 < n && !chars[i + 1].is_whitespace() {
            // Não remover cifrão monetário (R$ 100, US$ 50)
            let prev_letter = i > 0 && chars[i - 1].is_alphabetic();
            if !prev_letter {
                if let Some(rel_end) = chars[i + 1..].iter().position(|&c| c == '$' || c == '\n') {
                    let close_idx = i + 1 + rel_end;
                    if chars[close_idx] == '$' && close_idx > i + 1 && !chars[close_idx - 1].is_whitespace() {
                        let inner: String = chars[i + 1..close_idx].iter().collect();
                        let looks_math = inner.chars().any(|c| {
                            c.is_alphabetic()
                                || matches!(
                                    c,
                                    '_' | '^' | '{' | '}' | '=' | '≥' | '≤' | '≠' | '≈' | '·' | '×' | '\'' | '′'
                                )
                        });
                        if looks_math {
                            out.push_str(&inner);
                            i = close_idx + 1;
                            continue;
                        }
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    s = out;
    s
}

pub fn build_markdown_job_full(
    text: &str,
    font_size: f32,
    default_bold: bool,
    monospace: bool,
    dark_mode: bool,
    override_color: Option<Color32>,
    query: &str,
    match_counter: &mut usize,
    active_match_index: usize,
) -> (LayoutJob, bool) {
    let cleaned_owned;
    let text = if text.contains('$') || text.contains('\\') {
        cleaned_owned = normalize_inline_latex(text);
        cleaned_owned.as_str()
    } else {
        text
    };
    let mut job = LayoutJob::default();
    let family = if monospace {
        FontFamily::Monospace
    } else {
        FontFamily::Proportional
    };

    let pri_color = override_color.unwrap_or_else(|| GeoEstabilTheme::text_primary(dark_mode));
    let bold_color = override_color.unwrap_or_else(|| {
        if dark_mode {
            Color32::from_rgb(248, 250, 252)
        } else {
            Color32::from_rgb(12, 24, 52)
        }
    });
    let code_color = GeoEstabilTheme::accent_blue(dark_mode);

    let stripped = strip_inline_markup_for_search(text);
    let spans = find_char_match_spans(&stripped, query);
    let base_counter = *match_counter;
    *match_counter += spans.len();
    let mut hit_active = false;

    // Caminhamos mantendo o offset de caracteres no texto sem `**` para destacar corretamente buscas
    let bytes = text.as_bytes();
    let mut byte_idx = 0usize;
    let mut stripped_char_pos = 0usize;

    let count_stripped_chars = |s: &str| s.replace("**", "").chars().count();

    while byte_idx < bytes.len() {
        let rest = &text[byte_idx..];

        if let Some(after_bold) = rest.strip_prefix("**") {
            if let Some(close_rel) = after_bold.find("**") {
                let inner = &after_bold[..close_rel];
                append_fragment_with_math(
                    &mut job,
                    inner,
                    font_size,
                    &family,
                    bold_color,
                    true,
                    dark_mode,
                    &spans,
                    stripped_char_pos,
                    base_counter,
                    active_match_index,
                    &mut hit_active,
                );
                stripped_char_pos += inner.chars().count();
                byte_idx += 2 + close_rel + 2;
                continue;
            }
        }

        if let Some(after_code) = rest.strip_prefix('`') {
            if let Some(close_rel) = after_code.find('`') {
                let inner = &after_code[..close_rel];
                let inner_clean = strip_inline_markup_for_search(inner);
                let char_len = inner.chars().count();
                // Account for backtick char itself in `stripped`
                let backtick_off = stripped_char_pos + 1;
                append_fragment_with_math(
                    &mut job,
                    &inner_clean,
                    font_size - 0.5,
                    &FontFamily::Monospace,
                    code_color,
                    default_bold,
                    dark_mode,
                    &spans,
                    backtick_off,
                    base_counter,
                    active_match_index,
                    &mut hit_active,
                );
                stripped_char_pos += 1 + char_len + 1;
                byte_idx += 1 + close_rel + 1;
                continue;
            }
        }

        // Chunk normal até próximo `**` ou `` ` ``
        let first_len = rest.chars().next().map(|ch| ch.len_utf8()).unwrap_or(1);
        let mut next_special = rest.len();
        if let Some(p) = rest[first_len..].find("**") {
            next_special = next_special.min(first_len + p);
        }
        if let Some(p) = rest[first_len..].find('`') {
            next_special = next_special.min(first_len + p);
        }
        let plain = &rest[..next_special];
        let col = if default_bold { bold_color } else { pri_color };
        append_fragment_with_math(
            &mut job,
            plain,
            font_size,
            &family,
            col,
            default_bold,
            dark_mode,
            &spans,
            stripped_char_pos,
            base_counter,
            active_match_index,
            &mut hit_active,
        );
        stripped_char_pos += count_stripped_chars(plain);
        byte_idx += next_special;
    }

    (job, hit_active)
}

pub fn build_math_job_for_painter(
    text: &str,
    font_size: f32,
    default_bold: bool,
    monospace: bool,
    dark_mode: bool,
    override_color: Option<Color32>,
) -> LayoutJob {
    let mut dummy = 0usize;
    let (job, _) = build_markdown_job_full(
        text,
        font_size,
        default_bold,
        monospace,
        dark_mode,
        override_color,
        "",
        &mut dummy,
        0,
    );
    job
}

/// Renderiza um rótulo em UI com formatação automática de subscritos (`_`), expoentes (`^`) e símbolos matemáticos.
pub fn rich_math_label(
    ui: &mut Ui,
    text: &str,
    font_size: f32,
    bold: bool,
    dark_mode: bool,
    override_color: Option<Color32>,
) -> egui::Response {
    let mut job = build_math_job_for_painter(text, font_size, bold, false, dark_mode, override_color);
    job.wrap.max_width = f32::INFINITY;
    ui.label(job)
}

pub fn build_highlighted_job(
    text: &str,
    font_id: FontId,
    base_color: Color32,
    bold: bool,
    query: &str,
    match_counter: &mut usize,
    active_match_index: usize,
    dark_mode: bool,
) -> (LayoutJob, bool) {
    let monospace = font_id.family == FontFamily::Monospace;
    build_markdown_job_full(
        text,
        font_id.size,
        bold,
        monospace,
        dark_mode,
        Some(base_color),
        query,
        match_counter,
        active_match_index,
    )
}

fn find_matching_delim(bytes: &[u8], open_idx: usize, open_ch: u8, close_ch: u8) -> Option<usize> {
    if open_idx >= bytes.len() || bytes[open_idx] != open_ch {
        return None;
    }
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(open_idx) {
        if b == open_ch {
            depth += 1;
        } else if b == close_ch {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn delimiters_are_balanced(s: &str) -> bool {
    let mut sq = 0i32;
    let mut par = 0i32;
    for ch in s.chars() {
        match ch {
            '[' => sq += 1,
            ']' => {
                sq -= 1;
                if sq < 0 {
                    return false;
                }
            }
            '(' => par += 1,
            ')' => {
                par -= 1;
                if par < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    sq == 0 && par == 0
}

pub fn try_extract_fraction(expr: &str) -> Option<MathFractionParts> {
    if expr.contains("\\frac{") {
        let norm = crate::reports::pdf_writer::normalize_math_symbols(expr);
        if let Some((prefix, numerator, denominator, suffix)) =
            crate::reports::pdf_writer::extract_fraction_components(&norm)
        {
            return Some(MathFractionParts {
                prefix,
                numerator,
                denominator,
                suffix,
            });
        }
    }
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;
    while i < len {
        if bytes[i] == b'/' {
            let before = &expr[..i];
            let after = &expr[i + 1..];
            let before_trim = before.trim_end();
            let after_trim = after.trim_start();

            // Caso 1: [NUM] / [DEN]
            if before_trim.ends_with(']') && after_trim.starts_with('[') {
                if let Some(open_num) = {
                    let bt = before_trim.as_bytes();
                    let mut d = 0i32;
                    let mut found = None;
                    let mut j = bt.len();
                    while j > 0 {
                        j -= 1;
                        if bt[j] == b']' {
                            d += 1;
                        } else if bt[j] == b'[' {
                            d -= 1;
                            if d == 0 {
                                found = Some(j);
                                break;
                            }
                        }
                    }
                    found
                } {
                    if let Some(close_den) = find_matching_delim(after_trim.as_bytes(), 0, b'[', b']') {
                        return Some(MathFractionParts {
                            prefix: before_trim[..open_num].trim_end().to_string(),
                            numerator: before_trim[open_num + 1..before_trim.len() - 1]
                                .trim()
                                .to_string(),
                            denominator: after_trim[1..close_den].trim().to_string(),
                            suffix: after_trim[close_den + 1..].trim_start().to_string(),
                        });
                    }
                }
            }

            // Caso 2: (NUM) / (DEN)
            if before_trim.ends_with(')') && after_trim.starts_with('(') {
                if let Some(open_num) = {
                    let bt = before_trim.as_bytes();
                    let mut d = 0i32;
                    let mut found = None;
                    let mut j = bt.len();
                    while j > 0 {
                        j -= 1;
                        if bt[j] == b')' {
                            d += 1;
                        } else if bt[j] == b'(' {
                            d -= 1;
                            if d == 0 {
                                found = Some(j);
                                break;
                            }
                        }
                    }
                    found
                } {
                    if let Some(close_den) = find_matching_delim(after_trim.as_bytes(), 0, b'(', b')') {
                        return Some(MathFractionParts {
                            prefix: before_trim[..open_num].trim_end().to_string(),
                            numerator: before_trim[open_num + 1..before_trim.len() - 1]
                                .trim()
                                .to_string(),
                            denominator: after_trim[1..close_den].trim().to_string(),
                            suffix: after_trim[close_den + 1..].trim_start().to_string(),
                        });
                    }
                }
            }

            // Caso 3: LHS = [NUM] / DEN simples ou LHS = NUM / [DEN]
            if let Some(eq_idx) = before_trim.rfind(" = ") {
                let lhs = before_trim[..eq_idx + 3].trim_end().to_string();
                let mut num = before_trim[eq_idx + 3..].trim();
                let den_end = after_trim
                    .find("  ")
                    .or_else(|| after_trim.find(" | "))
                    .unwrap_or(after_trim.len());
                let mut den = after_trim[..den_end].trim();
                let suffix = after_trim[den_end..].trim_start().to_string();
                if num.starts_with('[') && num.ends_with(']') {
                    num = num[1..num.len() - 1].trim();
                } else if num.starts_with('(') && num.ends_with(')') {
                    if find_matching_delim(num.as_bytes(), 0, b'(', b')') == Some(num.len() - 1) {
                        num = num[1..num.len() - 1].trim();
                    }
                }
                if den.starts_with('[') && den.ends_with(']') {
                    den = den[1..den.len() - 1].trim();
                } else if den.starts_with('(') && den.ends_with(')') {
                    if find_matching_delim(den.as_bytes(), 0, b'(', b')') == Some(den.len() - 1) {
                        den = den[1..den.len() - 1].trim();
                    }
                }
                if !num.is_empty()
                    && !den.is_empty()
                    && !num.contains('/')
                    && !num.contains(';')
                    && !num.contains(',')
                    && !den.contains('/')
                    && !den.contains(';')
                    && delimiters_are_balanced(num)
                    && delimiters_are_balanced(den)
                {
                    return Some(MathFractionParts {
                        prefix: lhs,
                        numerator: num.to_string(),
                        denominator: den.to_string(),
                        suffix,
                    });
                }
            }
        }
        i += 1;
    }
    None
}

fn split_equation_tag_and_note(raw_eq: &str) -> (String, Option<String>, Option<String>) {
    let mut main = raw_eq.trim().to_string();
    let mut eq_tag: Option<String> = None;
    let mut note: Option<String> = None;

    // Extrai sufixo de nota entre parênteses longos explicativos ou `(Nota: ...)` / `(Note: ...)`
    if let Some(idx) = main.rfind("  (").or_else(|| main.rfind(" (")) {
        if main.ends_with(')') {
            let candidate = main[idx..].trim();
            let low = candidate.to_lowercase();
            let is_eq_tag = low.starts_with("(eq.")
                || low.starts_with("(nbr")
                || (candidate.len() <= 14
                    && candidate
                        .trim_matches(|c| c == '(' || c == ')')
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '.'));
            let is_note = low.starts_with("(nota")
                || low.starts_with("(note")
                || low.starts_with("(onde")
                || low.starts_with("(where")
                || low.starts_with("(para ")
                || low.starts_with("(for ")
                || low.starts_with("(com ")
                || low.starts_with("(with ")
                || low.starts_with("(se ")
                || low.starts_with("(if ")
                || low.starts_with("(ver ")
                || low.starts_with("(see ");
            if is_eq_tag {
                eq_tag = Some(
                    candidate
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .trim()
                        .to_string(),
                );
                main = main[..idx].trim_end().to_string();
            } else if is_note {
                note = Some(
                    candidate
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .trim()
                        .to_string(),
                );
                main = main[..idx].trim_end().to_string();
            }
        }
    }

    // Se ainda houver etiqueta numérica antes da nota ou vice-versa
    if note.is_none() {
        if let Some(sep) = main.find(" — Nota:").or_else(|| main.find(" — Note:")) {
            note = Some(main[sep + " — ".len()..].trim().to_string());
            main = main[..sep].trim_end().to_string();
        }
    }

    (main, eq_tag, note)
}

fn split_condition_prefix(expr: &str) -> (Option<String>, String) {
    if let Some(colon) = expr.find(": ") {
        let cond = expr[..colon].trim();
        let rhs = expr[colon + 2..].trim();
        if !cond.is_empty()
            && !rhs.is_empty()
            && !cond.contains('(')
            && !cond.contains('[')
            && !cond.starts_with("http")
            && !cond.starts_with("NBR")
        {
            return (Some(cond.to_string()), rhs.to_string());
        }
    }
    (None, expr.to_string())
}

pub fn render_doc_blocks(
    ui: &mut Ui,
    blocks: &[DocBlock],
    query: &str,
    active_match_index: usize,
    scroll_to_active: bool,
    dark_mode: bool,
    diagram_renderer: Option<&dyn Fn(&mut Ui, &str, bool)>,
) {
    render_doc_blocks_localized(
        ui,
        blocks,
        query,
        active_match_index,
        scroll_to_active,
        dark_mode,
        Language::PtBr,
        diagram_renderer,
    );
}

pub fn render_doc_blocks_localized(
    ui: &mut Ui,
    blocks: &[DocBlock],
    query: &str,
    active_match_index: usize,
    scroll_to_active: bool,
    dark_mode: bool,
    lang: Language,
    diagram_renderer: Option<&dyn Fn(&mut Ui, &str, bool)>,
) {
    let mut match_counter = 0usize;
    let text_pri = GeoEstabilTheme::text_primary(dark_mode);
    let text_sec = GeoEstabilTheme::text_secondary(dark_mode);
    let accent = GeoEstabilTheme::accent_blue(dark_mode);
    let amber = GeoEstabilTheme::warning_amber(dark_mode);
    let green = GeoEstabilTheme::success_green(dark_mode);
    let max_w = (ui.available_width() - 4.0).max(60.0);

    for (idx, block) in blocks.iter().enumerate() {
        match block {
            DocBlock::Banner(title) => {
                let banner_frame = Frame::NONE
                    .fill(if dark_mode {
                        Color32::from_rgb(28, 35, 48)
                    } else {
                        Color32::from_rgb(225, 237, 255)
                    })
                    .stroke(Stroke::new(1.5, accent))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(14, 10));

                let banner_inner_w = (max_w - 28.0).max(60.0);
                banner_frame.show(ui, |ui| {
                    ui.set_width(banner_inner_w);
                    ui.set_max_width(banner_inner_w);
                    for line in title.lines() {
                        let (mut job, is_active) = build_markdown_job_full(
                            line,
                            14.5,
                            true,
                            false,
                            dark_mode,
                            Some(accent),
                            query,
                            &mut match_counter,
                            active_match_index,
                        );
                        job.wrap.max_width = (ui.available_width() - 4.0).max(60.0);
                        let resp = ui.label(job);
                        if scroll_to_active && is_active {
                            resp.scroll_to_me(Some(Align::Center));
                        }
                    }
                });
                ui.add_space(6.0);
            }
            DocBlock::Heading2(h) => {
                ui.add_space(10.0);
                let (mut job, is_active) = build_markdown_job_full(
                    h,
                    16.0,
                    true,
                    false,
                    dark_mode,
                    Some(accent),
                    query,
                    &mut match_counter,
                    active_match_index,
                );
                job.wrap.max_width = max_w;
                let resp = ui.label(job);
                if scroll_to_active && is_active {
                    resp.scroll_to_me(Some(Align::Center));
                }
                ui.separator();
                ui.add_space(3.0);
            }
            DocBlock::Heading3(h) => {
                ui.add_space(7.0);
                let (mut job, is_active) = build_markdown_job_full(
                    h,
                    14.2,
                    true,
                    false,
                    dark_mode,
                    Some(GeoEstabilTheme::secondary_blue(dark_mode)),
                    query,
                    &mut match_counter,
                    active_match_index,
                );
                job.wrap.max_width = max_w;
                let resp = ui.label(job);
                if scroll_to_active && is_active {
                    resp.scroll_to_me(Some(Align::Center));
                }
                ui.add_space(2.0);
            }
            DocBlock::Heading4(h) => {
                ui.add_space(4.0);
                let (mut job, is_active) = build_markdown_job_full(
                    h,
                    13.2,
                    true,
                    false,
                    dark_mode,
                    Some(amber),
                    query,
                    &mut match_counter,
                    active_match_index,
                );
                job.wrap.max_width = max_w;
                let resp = ui.label(job);
                if scroll_to_active && is_active {
                    resp.scroll_to_me(Some(Align::Center));
                }
                ui.add_space(1.0);
            }
            DocBlock::Equation(raw_eq) => {
                render_equation_card(
                    ui,
                    raw_eq,
                    query,
                    &mut match_counter,
                    active_match_index,
                    scroll_to_active,
                    dark_mode,
                    lang,
                );
                ui.add_space(5.0);
            }
            DocBlock::Bullet(b) => {
                render_doc_bullet(
                    ui,
                    b,
                    max_w,
                    BulletStyle {
                        indent: 8.0,
                        glyph: "•",
                        glyph_size: 13.5,
                        text_size: 13.0,
                        glyph_color: accent,
                        text_color: text_pri,
                    },
                    query,
                    &mut match_counter,
                    active_match_index,
                    scroll_to_active,
                    dark_mode,
                );
                ui.add_space(2.0);
            }
            DocBlock::SubBullet(sb) => {
                render_doc_bullet(
                    ui,
                    sb,
                    max_w,
                    BulletStyle {
                        indent: 24.0,
                        glyph: "▪",
                        glyph_size: 12.0,
                        text_size: 12.6,
                        glyph_color: green,
                        text_color: text_pri,
                    },
                    query,
                    &mut match_counter,
                    active_match_index,
                    scroll_to_active,
                    dark_mode,
                );
                ui.add_space(2.0);
            }
            DocBlock::Table { headers, rows } => {
                let tbl_frame = Frame::NONE
                    .fill(if dark_mode {
                        Color32::from_rgb(24, 28, 36)
                    } else {
                        Color32::from_rgb(246, 249, 255)
                    })
                    .stroke(Stroke::new(
                        1.0,
                        if dark_mode {
                            Color32::from_rgb(51, 65, 85)
                        } else {
                            Color32::from_rgb(203, 213, 225)
                        },
                    ))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(8));

                let avail_tbl_w = ui.available_width().max(260.0);
                let col_cnt = headers.len().max(1) as f32;
                let cell_wrap_w = ((avail_tbl_w - (col_cnt - 1.0) * 14.0 - 20.0) / col_cnt.max(1.0) * 1.45)
                    .clamp(110.0, 340.0);

                tbl_frame.show(ui, |ui| {
                    ScrollArea::horizontal()
                        .id_salt(("doc_tbl_scroll_h", idx))
                        .show(ui, |ui| {
                            Grid::new(("doc_renderer_grid", idx))
                                .striped(true)
                                .spacing([14.0, 6.0])
                                .show(ui, |ui| {
                                    for h in headers {
                                        let (mut job, is_active) = build_markdown_job_full(
                                            h,
                                            12.4,
                                            true,
                                            false,
                                            dark_mode,
                                            Some(accent),
                                            query,
                                            &mut match_counter,
                                            active_match_index,
                                        );
                                        job.wrap.max_width = cell_wrap_w;
                                        let resp = ui.label(job);
                                        if scroll_to_active && is_active {
                                            resp.scroll_to_me(Some(Align::Center));
                                        }
                                    }
                                    ui.end_row();
                                    for row in rows {
                                        for col_idx in 0..headers.len() {
                                            let cell = row
                                                .get(col_idx)
                                                .map(|s| s.as_str())
                                                .unwrap_or("");
                                            let (mut job, is_active) = build_markdown_job_full(
                                                cell,
                                                12.2,
                                                false,
                                                false,
                                                dark_mode,
                                                Some(text_pri),
                                                query,
                                                &mut match_counter,
                                                active_match_index,
                                            );
                                            job.wrap.max_width = cell_wrap_w;
                                            let resp = ui.label(job);
                                            if scroll_to_active && is_active {
                                                resp.scroll_to_me(Some(Align::Center));
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                });
                ui.add_space(6.0);
            }
            DocBlock::Diagram(code) => {
                if let Some(renderer) = diagram_renderer {
                    renderer(ui, code, dark_mode);
                    ui.add_space(6.0);
                } else {
                    ui.label(
                        egui::RichText::new(format!("[Diagram: {}]", code))
                            .monospace()
                            .size(11.5)
                            .color(text_sec),
                    );
                }
            }
            DocBlock::Separator => {
                ui.add_space(3.0);
                ui.separator();
                ui.add_space(3.0);
            }
            DocBlock::Paragraph(p) => {
                let (mut job, is_active) = build_markdown_job_full(
                    p,
                    13.2,
                    false,
                    false,
                    dark_mode,
                    Some(text_pri),
                    query,
                    &mut match_counter,
                    active_match_index,
                );
                job.wrap.max_width = max_w;
                let resp = ui.label(job);
                if scroll_to_active && is_active {
                    resp.scroll_to_me(Some(Align::Center));
                }
                ui.add_space(4.0);
            }
        }
    }
}

struct BulletStyle {
    indent: f32,
    glyph: &'static str,
    glyph_size: f32,
    text_size: f32,
    glyph_color: Color32,
    text_color: Color32,
}

#[allow(clippy::too_many_arguments)]
fn render_doc_bullet(
    ui: &mut Ui,
    text: &str,
    max_w: f32,
    style: BulletStyle,
    query: &str,
    match_counter: &mut usize,
    active_match_index: usize,
    scroll_to_active: bool,
    dark_mode: bool,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(max_w, 0.0),
        egui::Layout::left_to_right(Align::TOP),
        |ui| {
            ui.set_max_width(max_w);
            ui.add_space(style.indent);
            ui.label(
                egui::RichText::new(style.glyph)
                    .strong()
                    .size(style.glyph_size)
                    .color(style.glyph_color),
            );
            let wrap_w = (ui.available_width() - 6.0).max(40.0);
            let (mut job, is_active) = build_markdown_job_full(
                text,
                style.text_size,
                false,
                false,
                dark_mode,
                Some(style.text_color),
                query,
                match_counter,
                active_match_index,
            );
            job.wrap.max_width = wrap_w;
            let resp = ui.label(job);
            if scroll_to_active && is_active {
                resp.scroll_to_me(Some(Align::Center));
            }
        },
    );
}

fn render_math_sub_expr(
    ui: &mut Ui,
    expr: &str,
    font_size: f32,
    base_color: Color32,
    query: &str,
    match_counter: &mut usize,
    active_match_index: usize,
    scroll_to_active: bool,
    dark_mode: bool,
) {
    let (mut job, is_active) = build_markdown_job_full(
        expr,
        font_size,
        true,
        false,
        dark_mode,
        Some(base_color),
        query,
        match_counter,
        active_match_index,
    );
    job.wrap.max_width = ui.available_width().max(180.0);
    let resp = ui.label(job);
    if scroll_to_active && is_active {
        resp.scroll_to_me(Some(Align::Center));
    }
}

#[allow(clippy::too_many_arguments)]
fn render_math_expr_with_fractions(
    ui: &mut Ui,
    expr: &str,
    math_fg: Color32,
    eq_border: Color32,
    query: &str,
    match_counter: &mut usize,
    active_match_index: usize,
    scroll_to_active: bool,
    dark_mode: bool,
    depth: usize,
) {
    if depth >= 4 {
        render_math_sub_expr(
            ui,
            expr,
            14.5,
            math_fg,
            query,
            match_counter,
            active_match_index,
            scroll_to_active,
            dark_mode,
        );
        return;
    }

    if let Some(frac) = try_extract_fraction(expr) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if !frac.prefix.is_empty() {
                render_math_sub_expr(
                    ui,
                    &frac.prefix,
                    14.5,
                    math_fg,
                    query,
                    match_counter,
                    active_match_index,
                    scroll_to_active,
                    dark_mode,
                );
            }
            let (num_job, num_active) = build_markdown_job_full(
                &frac.numerator,
                13.5,
                true,
                false,
                dark_mode,
                Some(math_fg),
                query,
                match_counter,
                active_match_index,
            );
            let (den_job, den_active) = build_markdown_job_full(
                &frac.denominator,
                13.5,
                true,
                false,
                dark_mode,
                Some(math_fg),
                query,
                match_counter,
                active_match_index,
            );
            let num_galley = ui.ctx().fonts_mut(|f| f.layout_job(num_job));
            let den_galley = ui.ctx().fonts_mut(|f| f.layout_job(den_job));
            let bar_w = num_galley
                .size()
                .x
                .max(den_galley.size().x)
                .clamp(28.0, 520.0)
                + 14.0;

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                let (num_rect, num_resp) = ui.allocate_exact_size(
                    egui::vec2(bar_w, num_galley.size().y),
                    egui::Sense::hover(),
                );
                ui.painter().galley(
                    egui::pos2(
                        num_rect.center().x - num_galley.size().x * 0.5,
                        num_rect.top(),
                    ),
                    num_galley,
                    math_fg,
                );
                if scroll_to_active && num_active {
                    num_resp.scroll_to_me(Some(Align::Center));
                }

                let (bar_rect, _) = ui.allocate_exact_size(
                    egui::vec2(bar_w, 2.5),
                    egui::Sense::hover(),
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(bar_rect.left(), bar_rect.center().y),
                        egui::pos2(bar_rect.right(), bar_rect.center().y),
                    ],
                    Stroke::new(1.5, eq_border),
                );

                let (den_rect, den_resp) = ui.allocate_exact_size(
                    egui::vec2(bar_w, den_galley.size().y),
                    egui::Sense::hover(),
                );
                ui.painter().galley(
                    egui::pos2(
                        den_rect.center().x - den_galley.size().x * 0.5,
                        den_rect.top(),
                    ),
                    den_galley,
                    math_fg,
                );
                if scroll_to_active && den_active {
                    den_resp.scroll_to_me(Some(Align::Center));
                }
            });
            if !frac.suffix.is_empty() {
                render_math_expr_with_fractions(
                    ui,
                    &frac.suffix,
                    math_fg,
                    eq_border,
                    query,
                    match_counter,
                    active_match_index,
                    scroll_to_active,
                    dark_mode,
                    depth + 1,
                );
            }
        });
    } else {
        render_math_sub_expr(
            ui,
            expr,
            14.5,
            math_fg,
            query,
            match_counter,
            active_match_index,
            scroll_to_active,
            dark_mode,
        );
    }
}

fn render_equation_card(
    ui: &mut Ui,
    raw_eq: &str,
    query: &str,
    match_counter: &mut usize,
    active_match_index: usize,
    scroll_to_active: bool,
    dark_mode: bool,
    lang: Language,
) {
    let eq_bg = if dark_mode {
        Color32::from_rgb(22, 27, 36)
    } else {
        Color32::from_rgb(239, 246, 255)
    };
    let eq_border = GeoEstabilTheme::accent_blue(dark_mode);
    let math_fg = if dark_mode {
        Color32::from_rgb(222, 236, 255)
    } else {
        Color32::from_rgb(18, 42, 84)
    };
    let amber = GeoEstabilTheme::warning_amber(dark_mode);
    let muted = GeoEstabilTheme::text_secondary(dark_mode);

    let (main_expr, eq_tag, note_opt) = split_equation_tag_and_note(raw_eq);

    let frame = Frame::NONE
        .fill(eq_bg)
        .stroke(Stroke::new(1.2, eq_border))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(12, 9));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let hdr_txt = match lang {
                Language::PtBr => "EQUAÇÃO NORMATIVA",
                Language::EnUs => "NORMATIVE EQUATION",
            };
            ui.label(
                egui::RichText::new(hdr_txt)
                    .size(10.5)
                    .strong()
                    .color(eq_border),
            );
            if let Some(tag) = &eq_tag {
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("({})", tag))
                            .monospace()
                            .size(11.0)
                            .strong()
                            .color(amber),
                    );
                });
            }
        });
        ui.add_space(3.0);

        ScrollArea::horizontal()
            .id_salt(("eq_scroll_h", raw_eq.len(), raw_eq))
            .show(ui, |ui| {
                for sub_raw in main_expr.split(" | ") {
                    let sub_trim = sub_raw.trim();
                    if sub_trim.is_empty() {
                        continue;
                    }
                    let (cond_opt, math_body) = split_condition_prefix(sub_trim);
                    if let Some(cond) = cond_opt {
                        render_math_sub_expr(
                            ui,
                            &format!("{}:", cond),
                            11.5,
                            amber,
                            query,
                            match_counter,
                            active_match_index,
                            scroll_to_active,
                            dark_mode,
                        );
                    }

                    render_math_expr_with_fractions(
                        ui,
                        &math_body,
                        math_fg,
                        eq_border,
                        query,
                        match_counter,
                        active_match_index,
                        scroll_to_active,
                        dark_mode,
                        0,
                    );
                    ui.add_space(2.0);
                }
            });

        if let Some(note) = note_opt {
            ui.add_space(2.0);
            render_math_sub_expr(
                ui,
                &note,
                11.4,
                muted,
                query,
                match_counter,
                active_match_index,
                scroll_to_active,
                dark_mode,
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doc_blocks_rich_elements() {
        let md = r#"====
ABNT NBR 11682:2009
Estabilidade de Encostas
====

## 1 Escopo
### 1.1 Requisitos
#### 1.1.1 Notas
Texto introdutório com **α_d** e `f_{cd} = f_{ck}/γ_c`.

[EQ] M_{Sd,tot} = [α_b · M_{1d,A}] / [1 - λ^2 / (120 · κ / ν)]  (Eq. 15.1)

• Tópico principal
  - Subtópico indentado

| Classe | f_{ck} | γ_c |
| :--- | :---: | ---: |
| C25 | 25 MPa | 1,40 |

[[DIAGRAM:SLOPE_BISHOP]]
----
Fim."#;
        let blocks = parse_doc_blocks(md);
        assert!(matches!(&blocks[0], DocBlock::Banner(b) if b.contains("Estabilidade")));
        assert!(matches!(&blocks[1], DocBlock::Heading2(s) if s == "1 Escopo"));
        assert!(matches!(&blocks[2], DocBlock::Heading3(s) if s == "1.1 Requisitos"));
        assert!(matches!(&blocks[3], DocBlock::Heading4(s) if s == "1.1.1 Notas"));
        assert!(matches!(&blocks[4], DocBlock::Paragraph(_)));
        assert!(matches!(&blocks[5], DocBlock::Equation(_)));
        assert!(matches!(&blocks[6], DocBlock::Bullet(s) if s == "Tópico principal"));
        assert!(matches!(&blocks[7], DocBlock::SubBullet(s) if s == "Subtópico indentado"));
        assert!(matches!(&blocks[8], DocBlock::Table { headers, rows } if headers.len() == 3 && rows.len() == 1));
        assert!(matches!(&blocks[9], DocBlock::Diagram(d) if d == "SLOPE_BISHOP"));
        assert!(matches!(&blocks[10], DocBlock::Separator));
    }

    #[test]
    fn test_try_extract_fraction_variants() {
        let f1 = try_extract_fraction("FS = [Σ R_i] / [Σ S_i]").expect("bracket frac");
        assert_eq!(f1.prefix, "FS =");
        assert_eq!(f1.numerator, "Σ R_i");
        assert_eq!(f1.denominator, "Σ S_i");

        let f2 = try_extract_fraction("f_cd = f_ck / γ_c").expect("lhs simple frac");
        assert_eq!(f2.prefix, "f_cd =");
        assert_eq!(f2.numerator, "f_ck");
        assert_eq!(f2.denominator, "γ_c");

        let f3 = try_extract_fraction("Δ_{r} = [P_{r} · L] / [A · E] + [D] / [30]").expect("first stacked term");
        assert_eq!(f3.numerator, "P_{r} · L");
        assert_eq!(f3.denominator, "A · E");
        let f3_next = try_extract_fraction(&f3.suffix).expect("second stacked term");
        assert_eq!(f3_next.prefix, "+");
        assert_eq!(f3_next.numerator, "D");
        assert_eq!(f3_next.denominator, "30");

        assert!(
            try_extract_fraction("R_{k} = mín [ (R_{se})_{méd} / ξ_{1} ; (R_{se})_{mín} / ξ_{2} ]").is_none(),
            "Must not produce malformed outer fraction across semicolon inside brackets"
        );
    }

    #[test]
    fn test_math_subscripts_exponents_and_braces_no_raw_markup() {
        let job = build_math_job_for_painter(
            "V_base + E_{a,h} = 10^{-5} · (40/f_{ck})^{1/3} · e^{-(μ_{1}·Σα + kx)}",
            14.0,
            false,
            false,
            false,
            None,
        );
        assert!(
            !job.text.contains('_') && !job.text.contains('{') && !job.text.contains('}') && !job.text.contains('^'),
            "Rendered LayoutJob text must not expose raw markup characters: {}",
            job.text
        );
        assert!(job.text.starts_with("Vbase + Ea,h = 10-5"));

        let base_start = job.text.find("base").expect("subscript 'base'");
        let base_end = base_start + "base".len();
        let has_base_subscript = job.sections.iter().any(|s| {
            s.byte_range.start >= egui::text::ByteIndex(base_start)
                && s.byte_range.end <= egui::text::ByteIndex(base_end)
                && s.format.valign == Align::Max
                && s.format.font_id.size < 12.0
        });
        assert!(has_base_subscript, "Expected 'base' in 'V_base' to use Align::Max subscript format");

        let exp_start = job.text.find("-5").expect("exponent '-5'");
        let has_sup_format = job.sections.iter().any(|s| {
            s.byte_range.start >= egui::text::ByteIndex(exp_start)
                && s.byte_range.end <= egui::text::ByteIndex(exp_start + "-5".len())
                && s.format.valign == Align::Min
                && s.format.font_id.size < 12.0
        });
        assert!(has_sup_format, "Expected '-5' in '10^{{-5}}' to use Align::Min superscript format");

        // Emails e macros em caixa alta não devem ser corrompidos
        let email_job = build_math_job_for_painter(
            "Contato: linceu_lighthouse@outlook.com e PRIVACY_POLICY",
            13.0,
            false,
            false,
            false,
            None,
        );
        assert!(email_job.text.contains("linceu_lighthouse@outlook.com"));
        assert!(email_job.text.contains("PRIVACY_POLICY"));
    }
}
