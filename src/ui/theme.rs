use egui::{Color32, Context, CornerRadius, Stroke, Style, Visuals};

/// Tema visual profissional e ergonômico do GeoEstabil
/// Especializado em Engenharia Geotécnica, Encostas e Concreto Estrutural.
pub struct GeoEstabilTheme;

impl GeoEstabilTheme {
    // =========================================================================
    // Cores Dinâmicas Sensíveis ao Tema (Modo Escuro vs. Modo Claro)
    // =========================================================================

    pub fn accent_blue(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(100, 181, 246) // Azul celeste claro (padrão EoloPlatform Dark)
        } else {
            Color32::from_rgb(29, 78, 216) // Azul marinho técnico
        }
    }

    pub fn primary_button_fill(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(25, 118, 210)
        } else {
            Color32::from_rgb(29, 78, 216)
        }
    }

    pub fn secondary_blue(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(129, 212, 250)
        } else {
            Color32::from_rgb(37, 99, 235)
        }
    }

    pub fn soil_brown(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(217, 158, 93)
        } else {
            Color32::from_rgb(161, 98, 7)
        }
    }

    pub fn water_blue(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(56, 189, 248)
        } else {
            Color32::from_rgb(2, 132, 199)
        }
    }

    pub fn slip_circle_red(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(248, 113, 113)
        } else {
            Color32::from_rgb(220, 38, 38)
        }
    }

    pub fn success_green(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(102, 187, 106)
        } else {
            Color32::from_rgb(22, 163, 74)
        }
    }

    pub fn warning_amber(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(251, 191, 36)
        } else {
            Color32::from_rgb(217, 119, 6)
        }
    }

    pub fn concrete_gray(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(156, 163, 175)
        } else {
            Color32::from_rgb(75, 85, 99)
        }
    }

    pub fn rebar_orange(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(251, 146, 60)
        } else {
            Color32::from_rgb(234, 88, 12)
        }
    }

    pub fn text_primary(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(235, 240, 245)
        } else {
            Color32::from_rgb(15, 23, 42)
        }
    }

    pub fn text_secondary(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(165, 175, 190)
        } else {
            Color32::from_rgb(100, 116, 139)
        }
    }

    pub fn card_bg(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(28, 33, 42)
        } else {
            Color32::from_rgb(248, 250, 252)
        }
    }

    pub fn panel_border(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(58, 68, 84)
        } else {
            Color32::from_rgb(226, 232, 240)
        }
    }

    pub fn canvas_bg(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(14, 17, 23)
        } else {
            Color32::from_rgb(255, 255, 255)
        }
    }

    pub fn grid_lines(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgba_premultiplied(255, 255, 255, 18)
        } else {
            Color32::from_rgba_premultiplied(0, 0, 0, 15)
        }
    }

    /// Gradiente contínuo de calor (Heatmap) para Fatores de Segurança:
    /// FS < 1.0 -> Vermelho intenso (Ruptura iminente)
    /// FS 1.0 a 1.3 -> Laranja (Instável / Alerta)
    /// FS 1.3 a 1.5 -> Amarelo / Âmbar (Atenção / Risco Médio)
    /// FS ≥ 1.5 -> Verde vívido (Seguro conforme NBR 11682)
    pub fn heatmap_color(fs: f64) -> Color32 {
        if fs <= 1.0 {
            Color32::from_rgba_premultiplied(239, 68, 68, 210) // Red
        } else if fs <= 1.30 {
            let t = (fs - 1.0) / 0.30;
            let r = (239.0 + t * (249.0 - 239.0)) as u8;
            let g = (68.0 + t * (115.0 - 68.0)) as u8;
            let b = (68.0 + t * (22.0 - 68.0)) as u8;
            Color32::from_rgba_premultiplied(r, g, b, 210) // Red to Orange
        } else if fs <= 1.50 {
            let t = (fs - 1.30) / 0.20;
            let r = (249.0 + t * (234.0 - 249.0)) as u8;
            let g = (115.0 + t * (179.0 - 115.0)) as u8;
            let b = (22.0 + t * (8.0 - 22.0)) as u8;
            Color32::from_rgba_premultiplied(r, g, b, 210) // Orange to Yellow
        } else {
            let t = ((fs - 1.50) / 0.50).clamp(0.0, 1.0);
            let r = (234.0 + t * (34.0 - 234.0)) as u8;
            let g = (179.0 + t * (197.0 - 179.0)) as u8;
            let b = (8.0 + t * (94.0 - 8.0)) as u8;
            Color32::from_rgba_premultiplied(r, g, b, 220) // Yellow to Green
        }
    }

    /// Constrói os visuais completos e contrastados para modo escuro ou claro.
    /// Define explicitamente `bg_fill`, `weak_bg_fill`, `bg_stroke` e `fg_stroke`
    /// para que checkboxes, radio buttons, botões e campos de texto mantenham
    /// contraste nítido em ambos os temas sobre `card_bg` e `panel_fill`.
    pub fn build_visuals(dark_mode: bool) -> Visuals {
        if dark_mode {
            let mut visuals = Visuals::dark();
            visuals.override_text_color = None;
            visuals.window_corner_radius = CornerRadius::same(6);
            visuals.panel_fill = Color32::from_rgb(20, 23, 29);
            visuals.window_fill = Color32::from_rgb(25, 29, 36);
            visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(55, 65, 82));
            visuals.extreme_bg_color = Color32::from_rgb(14, 16, 21);
            visuals.faint_bg_color = Color32::from_rgb(28, 33, 42);
            visuals.code_bg_color = Color32::from_rgb(30, 35, 44);
            visuals.selection.bg_fill = Color32::from_rgb(25, 118, 210);
            visuals.selection.stroke = Stroke::new(1.2, Color32::WHITE);

            visuals.widgets.noninteractive.weak_bg_fill = Color32::from_rgb(26, 30, 38);
            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 30, 38);
            visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(52, 60, 74));
            visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(228, 233, 240));
            visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

            visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(36, 42, 54);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 23, 29);
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.2, Color32::from_rgb(96, 110, 132));
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.6, Color32::from_rgb(235, 240, 245));
            visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

            visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(48, 58, 76);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 58, 76);
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.4, Color32::from_rgb(100, 181, 246));
            visuals.widgets.hovered.fg_stroke = Stroke::new(1.8, Color32::WHITE);
            visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

            visuals.widgets.active.weak_bg_fill = Color32::from_rgb(25, 118, 210);
            visuals.widgets.active.bg_fill = Color32::from_rgb(25, 118, 210);
            visuals.widgets.active.bg_stroke = Stroke::new(1.5, Color32::from_rgb(144, 202, 249));
            visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
            visuals.widgets.active.corner_radius = CornerRadius::same(4);

            visuals.widgets.open.weak_bg_fill = Color32::from_rgb(42, 50, 66);
            visuals.widgets.open.bg_fill = Color32::from_rgb(42, 50, 66);
            visuals.widgets.open.bg_stroke = Stroke::new(1.2, Color32::from_rgb(100, 181, 246));
            visuals.widgets.open.fg_stroke = Stroke::new(1.5, Color32::WHITE);
            visuals.widgets.open.corner_radius = CornerRadius::same(4);
            visuals
        } else {
            let mut visuals = Visuals::light();
            visuals.window_corner_radius = CornerRadius::same(6);
            visuals.panel_fill = Color32::from_rgb(241, 245, 249);
            visuals.window_fill = Color32::from_rgb(255, 255, 255);
            visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(203, 213, 225));
            visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
            visuals.faint_bg_color = Color32::from_rgb(241, 245, 249);
            visuals.selection.bg_fill = Color32::from_rgb(191, 219, 254);
            visuals.selection.stroke = Stroke::new(1.2, Color32::from_rgb(29, 78, 216));

            visuals.widgets.noninteractive.weak_bg_fill = Color32::from_rgb(248, 250, 252);
            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
            visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(203, 213, 225));
            visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(15, 23, 42));
            visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

            visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(226, 232, 240);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(255, 255, 255);
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.2, Color32::from_rgb(100, 116, 139));
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.6, Color32::from_rgb(15, 23, 42));
            visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

            visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(203, 213, 225);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(219, 234, 254);
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.4, Color32::from_rgb(29, 78, 216));
            visuals.widgets.hovered.fg_stroke = Stroke::new(1.8, Color32::from_rgb(15, 23, 42));
            visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

            visuals.widgets.active.weak_bg_fill = Color32::from_rgb(191, 219, 254);
            visuals.widgets.active.bg_fill = Color32::from_rgb(191, 219, 254);
            visuals.widgets.active.bg_stroke = Stroke::new(1.5, Color32::from_rgb(29, 78, 216));
            visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::from_rgb(15, 23, 42));
            visuals.widgets.active.corner_radius = CornerRadius::same(4);

            visuals.widgets.open.weak_bg_fill = Color32::from_rgb(219, 234, 254);
            visuals.widgets.open.bg_fill = Color32::from_rgb(255, 255, 255);
            visuals.widgets.open.bg_stroke = Stroke::new(1.2, Color32::from_rgb(29, 78, 216));
            visuals.widgets.open.fg_stroke = Stroke::new(1.5, Color32::from_rgb(15, 23, 42));
            visuals.widgets.open.corner_radius = CornerRadius::same(4);
            visuals
        }
    }

    /// Registra fontes de fallback Unicode/Matemática (Segoe UI / Segoe UI Symbol no Windows)
    /// uma única vez por contexto egui, mantendo as fontes padrão do egui como primárias.
    pub fn ensure_math_fonts(ctx: &Context) {
        #[derive(Clone)]
        struct MathFontsInstalled;

        let already = ctx
            .data(|d| d.get_temp::<MathFontsInstalled>(egui::Id::NULL))
            .is_some();
        if already {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, MathFontsInstalled));

        #[cfg(target_os = "windows")]
        {
            let mut fonts = egui::FontDefinitions::default();
            let mut math_family: Vec<String> = Vec::new();
            let candidates = [
                ("segoe_ui_fallback", r"C:\Windows\Fonts\segoeui.ttf"),
                ("segoe_sym_fallback", r"C:\Windows\Fonts\seguisym.ttf"),
            ];
            for (name, path) in candidates {
                if let Ok(bytes) = std::fs::read(path) {
                    fonts.font_data.insert(
                        name.to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                    );
                    math_family.push(name.to_owned());
                }
            }
            if !math_family.is_empty() {
                for fam in [
                    egui::FontFamily::Proportional,
                    egui::FontFamily::Monospace,
                ] {
                    if let Some(list) = fonts.families.get_mut(&fam) {
                        for name in &math_family {
                            if !list.contains(name) {
                                list.push(name.clone());
                            }
                        }
                    }
                }
                math_family.push("Ubuntu-Light".to_owned());
                fonts
                    .families
                    .insert(egui::FontFamily::Name("math_sym".into()), math_family);
                ctx.set_fonts(fonts);
            }
        }
    }

    /// Aplica as preferências de estilo e tema ativo diretamente ao Context do egui
    pub fn apply(ctx: &Context, dark_mode: bool) {
        Self::ensure_math_fonts(ctx);
        ctx.set_theme(egui::Theme::from_dark_mode(dark_mode));

        let configure_style = |s: &mut Style, is_dark: bool| {
            s.visuals = Self::build_visuals(is_dark);
            s.spacing.item_spacing = egui::vec2(8.0, 6.0);
            s.spacing.button_padding = egui::vec2(8.0, 4.0);
        };
        ctx.style_mut_of(egui::Theme::Dark, |s| configure_style(s, true));
        ctx.style_mut_of(egui::Theme::Light, |s| configure_style(s, false));
    }
}
