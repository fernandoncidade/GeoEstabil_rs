use eframe::egui;
use eframe::egui::epaint::Shape;
use geoestabil_rs::core::presets::get_preset_cases;
use geoestabil_rs::core::project::{ActiveProjectMode, GeoProject};
use geoestabil_rs::data::database::GeoDatabase;
use geoestabil_rs::engine::slope::grid_search::run_slope_stability_analysis;
use geoestabil_rs::locale::translations::{Language, Localizer};
use geoestabil_rs::reports::memorial_generator::MemorialGenerator;
use geoestabil_rs::ui::app::GeoEstabilApp;
use geoestabil_rs::ui::canvas_2d::CanvasState;
use geoestabil_rs::ui::dialogs::{
    AboutDialog, ManualDialog, SettingsDialog, SoilCatalogDialog,
};
use geoestabil_rs::ui::theme::GeoEstabilTheme;
use geoestabil_rs::utils::export_dxf_svg::VectorExporter;

fn make_test_app() -> GeoEstabilApp {
    let mut project = GeoProject::default();
    project.slope_model.grid_x_steps = 8;
    project.slope_model.grid_y_steps = 8;
    project.slope_model.radius_steps = 8;
    project.slope_model.num_slices = 20;
    let initial_report = run_slope_stability_analysis(&project.slope_model);

    GeoEstabilApp {
        project,
        active_tab: 0,
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
    }
}

fn collect_shapes_recursive(shape: &Shape, texts: &mut Vec<String>, rects: &mut Vec<egui::epaint::RectShape>, paths: &mut Vec<egui::epaint::PathShape>) {
    match shape {
        Shape::Vec(children) => {
            for child in children {
                collect_shapes_recursive(child, texts, rects, paths);
            }
        }
        Shape::Text(text_shape) => {
            texts.push(text_shape.galley.text().to_owned());
        }
        Shape::Rect(rect_shape) => {
            rects.push(rect_shape.clone());
        }
        Shape::Path(path_shape) => {
            paths.push(path_shape.clone());
        }
        _ => {}
    }
}

struct FrameSnapshot {
    texts: Vec<String>,
    rects: Vec<egui::epaint::RectShape>,
    paths: Vec<egui::epaint::PathShape>,
    root_title: Option<String>,
}

fn render_app_frame(ctx: &egui::Context, app: &mut GeoEstabilApp) -> FrameSnapshot {
    let mut texts = Vec::new();
    let mut rects = Vec::new();
    let mut paths = Vec::new();
    let mut root_title = None;

    for _ in 0..2 {
        texts.clear();
        rects.clear();
        paths.clear();

        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            ..Default::default()
        };
        let mut output = ctx.run_ui(raw_input, |ui| app.render_ui(ui));
        for clipped in &output.shapes {
            collect_shapes_recursive(&clipped.shape, &mut texts, &mut rects, &mut paths);
        }
        if let Some(vp) = output.viewport_output.get(&egui::ViewportId::ROOT) {
            for cmd in &vp.commands {
                if let egui::ViewportCommand::Title(t) = cmd {
                    root_title = Some(t.clone());
                }
            }
        }
        output.textures_delta.clear();
    }

    FrameSnapshot {
        texts,
        rects,
        paths,
        root_title,
    }
}

#[test]
fn test_dynamic_runtime_translation_all_tabs_and_dialogs() {
    let ctx = egui::Context::default();
    let mut app = make_test_app();
    app.soil_catalog_dialog.open();
    app.settings_dialog.open();

    // 1. Inicia em Português (BR) e valida textos principais
    app.active_tab = 0;
    let pt_tab0 = render_app_frame(&ctx, &mut app);
    let pt_joined0 = pt_tab0.texts.join("\n");
    assert!(pt_joined0.contains("1. Estratigrafia Geotécnica, Camadas de Solo e Nível Freático"));
    assert!(pt_joined0.contains("Catálogo de Solos Padronizados"));
    assert!(pt_joined0.contains("Preferências do Sistema"));
    assert!(pt_joined0.contains("Modo Escuro"));
    for removed in [
        "Normas ABNT",
        "Compêndio Normativo ABNT",
        "Normas ABNT (F3)",
        "ABNT Standards",
        "ABNT Standards Compendium",
        "ABNT Norms (F3)",
        "Exibir",
    ] {
        assert!(
            !pt_joined0.contains(removed),
            "Removed menu/compendium text '{}' must not appear in PT UI",
            removed
        );
    }
    assert!(pt_joined0.contains("Ferramentas"));
    assert!(Localizer::new(Language::PtBr).t("menu.tools.to_light") == "Alternar para Modo Claro");
    assert!(Localizer::new(Language::EnUs).t("menu.tools.to_light") == "Switch to Light Mode");
    assert!(Localizer::new(Language::PtBr).t("menu.tools.to_dark") == "Alternar para Modo Escuro");
    assert!(Localizer::new(Language::EnUs).t("menu.tools.to_dark") == "Switch to Dark Mode");

    // Abre o menu "Ferramentas" via clique real e aciona "Alternar para Modo Claro" (e em seguida em inglês "Switch to Dark Mode")
    let find_text_center = |app: &mut GeoEstabilApp, target: &str| -> Option<egui::Pos2> {
        let mut found = None;
        for _ in 0..2 {
            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1400.0, 900.0),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(raw_input, |ui| app.render_ui(ui));
            fn scan(s: &Shape, target: &str, found: &mut Option<egui::Pos2>) {
                match s {
                    Shape::Vec(v) => v.iter().for_each(|c| scan(c, target, found)),
                    Shape::Text(t) if t.galley.text().contains(target) => {
                        *found = Some(t.galley.rect.translate(t.pos.to_vec2()).center());
                    }
                    _ => {}
                }
            }
            for cs in &output.shapes {
                scan(&cs.shape, target, &mut found);
            }
            output.textures_delta.clear();
        }
        found
    };
    let click_at = |app: &mut GeoEstabilApp, pos: egui::Pos2| -> Vec<String> {
        let mut texts = Vec::new();
        for pressed in [Some(true), Some(false), None] {
            texts.clear();
            let mut events = vec![egui::Event::PointerMoved(pos)];
            if let Some(p) = pressed {
                events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: p,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1400.0, 900.0),
                )),
                events,
                ..Default::default()
            };
            let mut rects = Vec::new();
            let mut paths = Vec::new();
            let mut output = ctx.run_ui(raw_input, |ui| app.render_ui(ui));
            for cs in &output.shapes {
                collect_shapes_recursive(&cs.shape, &mut texts, &mut rects, &mut paths);
            }
            output.textures_delta.clear();
        }
        texts
    };

    assert!(app.dark_mode);
    let tools_pt_pos = find_text_center(&mut app, "Ferramentas").expect("Ferramentas menu button");
    let tools_pt_popup = click_at(&mut app, tools_pt_pos).join("\n");
    assert!(tools_pt_popup.contains("Alternar para Modo Claro"));
    let to_light_pos = find_text_center(&mut app, "Alternar para Modo Claro").expect("to_light item");
    click_at(&mut app, to_light_pos);
    assert!(!app.dark_mode, "Ferramentas > Alternar para Modo Claro must switch to light mode");

    // Alterna para Inglês US e verifica "Tools > Switch to Dark Mode" restaurando Dark Mode
    let prev_tmp = app.localizer.language;
    app.localizer.set_language(Language::EnUs);
    app.project.translate_defaults(prev_tmp, Language::EnUs);
    let tools_en_pos = find_text_center(&mut app, "Tools").expect("Tools menu button");
    let tools_en_popup = click_at(&mut app, tools_en_pos).join("\n");
    assert!(tools_en_popup.contains("Switch to Dark Mode"));
    let to_dark_pos = find_text_center(&mut app, "Switch to Dark Mode").expect("to_dark item");
    click_at(&mut app, to_dark_pos);
    assert!(app.dark_mode, "Tools > Switch to Dark Mode must restore dark mode");
    let prev_tmp2 = app.localizer.language;
    app.localizer.set_language(Language::PtBr);
    app.project.translate_defaults(prev_tmp2, Language::PtBr);

    // 2. Alterna idioma dinamicamente em tempo de execução para Inglês (US)
    let prev = app.localizer.language;
    app.localizer.set_language(Language::EnUs);
    app.project.translate_defaults(prev, Language::EnUs);

    let pt_forbidden_markers = [
        "Estratigrafia & Solos",
        "Estratigrafia Geotécnica",
        "Estabilidade de Taludes",
        "Muro de Contenção",
        "Fundações & Estrutural",
        "Memorial de Cálculo",
        "Alterações não salvas",
        "Salvo",
        "Aguardando Cálculo",
        "Exibir Mapa de Calor",
        "Exibir Fatias",
        "Grade Métrica",
        "Dente de Ancoragem",
        "Tombamento",
        "Deslizamento",
        "Excentricidade",
        "Fuste (Base",
        "Ponta da Sapata",
        "Calcanhar da Sapata",
        "Pré-visualização do Memorial",
        "Catálogo de Solos Padronizados",
        "Inserir no Projeto",
        "Preferências do Sistema",
        "Modo Escuro",
        "Modo Claro",
        "Auto-Enquadramento",
        "Calcular Estabilidade",
        "Adicionar Camada Personalizada",
        "Considerar Nível Freático Ativo",
        "Aplicar Sobrecarga na Crista",
        "Normas ABNT",
        "ABNT Standards",
        "Compêndio Normativo ABNT",
        "ABNT Standards Compendium",
        "ABNT Norms (F3)",
    ];

    // Verifica TODAS as 5 abas + diálogos modais em Inglês (US)
    for tab in 0..=4 {
        app.active_tab = tab;
        if tab == 2 {
            app.project.active_mode = ActiveProjectMode::RetainingWall;
        } else if tab == 1 {
            app.project.active_mode = ActiveProjectMode::SlopeStability;
        }
        let snap = render_app_frame(&ctx, &mut app);
        let joined = snap.texts.join("\n");

        assert_eq!(
            snap.root_title.as_deref(),
            Some("GeoEstabil — Slope Stability & Retaining Walls (ABNT)")
        );
        assert!(joined.contains("1. Stratigraphy & Soils"), "Tab bar missing EN tab 1 on tab {tab}");
        assert!(joined.contains("Tools"), "Menu bar missing Tools on tab {tab}");
        assert!(!joined.contains("\nView\n") && !joined.lines().any(|l| l.trim() == "View"), "Removed top menu 'View' leaked on tab {tab}");
        assert!(joined.contains("2. Slope Stability"), "Tab bar missing EN tab 2 on tab {tab}");
        assert!(joined.contains("3. Retaining Wall"), "Tab bar missing EN tab 3 on tab {tab}");
        assert!(joined.contains("4. Foundations & Structural (NBR 6118)"), "Tab bar missing EN tab 4 on tab {tab}");
        assert!(joined.contains("5. Calculation Memorial"), "Tab bar missing EN tab 5 on tab {tab}");

        for forbidden in pt_forbidden_markers {
            assert!(
                !joined.contains(forbidden),
                "Untranslated PT-BR UI text '{}' leaked into EN-US on tab {}:\n{}",
                forbidden,
                tab,
                joined
            );
        }

        if tab >= 2 {
            for raw_math in [
                "_{", "^{", "Ea_h", "FS_tomb", "FS_desl", "FS_ovt", "FS_sld",
                "σ_toe", "σ_heel", "σ_adm", "b_top", "B_toe", "q_ult", "N_SPT", "A_s",
            ] {
                assert!(
                    !joined.contains(raw_math),
                    "Unformatted raw math token '{}' leaked on tab {}:\n{}",
                    raw_math,
                    tab,
                    joined
                );
            }
        }

        match tab {
            0 => {
                assert!(joined.contains("1. Geotechnical Stratigraphy, Soil Layers & Water Table"));
                assert!(joined.contains("Enable Active Water Table"));
                assert!(joined.contains("Apply Crest / Surface Surcharge"));
                assert!(joined.contains("Standard Soil Catalog (ABNT NBR 6122 / SPT)"));
                assert!(joined.contains("Insert into Project"));
                assert!(joined.contains("System Preferences"));
                assert!(joined.contains("Settings"));
            }
            1 => {
                assert!(joined.contains("COMPUTE STABILITY (RAYON)"));
                assert!(joined.contains("Show FS Heatmap"));
                assert!(joined.contains("Show Slices"));
                assert!(joined.contains("Metric Grid"));
                assert!(joined.contains("Stratigraphy (Mohr-Coulomb):"));
                assert!(joined.contains("FS Heatmap (Centers)"));
                assert!(joined.contains("Auto-Fit: ON"));
            }
            2 => {
                assert!(joined.contains("Retaining Wall Type:"));
                assert!(joined.contains("Base Shear Key"));
                assert!(joined.contains("Diagrams"));
                assert!(joined.contains("Dimensions"));
                assert!(joined.contains("Checks (NBR 11682 / 6122)"));
            }
            3 => {
                assert!(joined.contains("4. Foundations & Reinforced Concrete Design"));
                assert!(joined.contains("Overturning"));
                assert!(joined.contains("Sliding"));
                assert!(joined.contains("Stem (Wall Base at Footing)"));
                assert!(joined.contains("Footing Toe (Front Cantilever)"));
                assert!(joined.contains("Footing Heel (Back Cantilever)"));
            }
            4 => {
                assert!(joined.contains("5. Technical Calculation Memorial & Engineering Reports"));
                assert!(joined.contains("Formatted Technical Memorial Preview:"));
                assert!(joined.contains("Slope Stability"));
            }
            _ => unreachable!(),
        }
    }

    // 3. Alterna de volta para Português (BR) dinamicamente e confirma retorno integral
    let prev = app.localizer.language;
    app.localizer.set_language(Language::PtBr);
    app.project.translate_defaults(prev, Language::PtBr);
    app.active_tab = 1;
    app.project.active_mode = ActiveProjectMode::SlopeStability;
    let pt_back = render_app_frame(&ctx, &mut app);
    let pt_back_joined = pt_back.texts.join("\n");
    assert!(pt_back_joined.contains("CALCULAR ESTABILIDADE (RAYON)"));
    assert!(pt_back_joined.contains("Exibir Mapa de Calor"));
    assert!(pt_back_joined.contains("Auto-Enquadramento: ON"));
}

#[test]
fn test_project_translate_defaults_preserves_custom_text() {
    let mut p = GeoProject {
        title: "Custom Highway Slope 42".to_string(),
        ..Default::default()
    };
    p.translate_defaults(Language::PtBr, Language::EnUs);

    assert_eq!(p.title, "Custom Highway Slope 42");
    assert_eq!(p.client, "Client / Developer / Contractor");
    assert_eq!(p.slope_model.layers[0].name, "Residual / Colluvial Soil");

    p.translate_defaults(Language::EnUs, Language::PtBr);
    assert_eq!(p.title, "Custom Highway Slope 42");
    assert_eq!(p.client, "Cliente / Incorporadora / Construtora");
    assert_eq!(p.slope_model.layers[0].name, "Solo Residual / Colúvio");

    // Verifica presets e geração bilíngue de memoriais e SVG
    for preset in get_preset_cases() {
        assert!(!preset.localized_name(Language::EnUs).contains("Caso "));
        let en_proj = preset.project_for_lang(Language::EnUs);
        let md_en = MemorialGenerator::generate_markdown_lang(&en_proj, None, Language::EnUs);
        let html_en = MemorialGenerator::generate_html_lang(&en_proj, None, Language::EnUs);
        let svg_en = VectorExporter::export_slope_svg_lang(&en_proj, None, Language::EnUs);
        assert!(md_en.contains("Retaining Wall & External Stability"));
        assert!(html_en.contains("<html lang=\"en-US\">"));
        assert!(svg_en.contains("GeoEstabil —"));
    }
}

#[test]
fn test_dynamic_runtime_theme_switching_and_checkbox_adaptation() {
    let ctx = egui::Context::default();
    let mut app = make_test_app();
    app.active_tab = 1; // Tab 2 possui 3 checkboxes sobre card_bg
    app.settings_dialog.open();

    // 1. Renderiza no Modo Escuro (tonalidade grafite alinhada ao EoloPlatform_rs)
    app.dark_mode = true;
    let dark_snap = render_app_frame(&ctx, &mut app);
    assert_eq!(ctx.theme(), egui::Theme::Dark);
    assert!(ctx.global_style().visuals.dark_mode);
    assert_eq!(
        ctx.global_style().visuals.panel_fill,
        egui::Color32::from_rgb(20, 23, 29)
    );
    assert_eq!(
        ctx.global_style().visuals.window_fill,
        egui::Color32::from_rgb(25, 29, 36)
    );
    assert_eq!(
        ctx.global_style().visuals.extreme_bg_color,
        egui::Color32::from_rgb(14, 16, 21)
    );
    assert_eq!(GeoEstabilTheme::card_bg(true), egui::Color32::from_rgb(28, 33, 42));

    let dark_box_fill = egui::Color32::from_rgb(20, 23, 29);
    let dark_box_border = egui::Color32::from_rgb(96, 110, 132);
    let dark_check_color = egui::Color32::from_rgb(235, 240, 245);
    assert_ne!(dark_box_fill, GeoEstabilTheme::card_bg(true));

    let has_dark_checkbox_box = dark_snap.rects.iter().any(|r| {
        r.fill == dark_box_fill
            && r.stroke.color == dark_box_border
            && (r.rect.width() - r.rect.height()).abs() < 2.0
            && r.rect.width() <= 24.0
    });
    assert!(
        has_dark_checkbox_box,
        "Checkbox box should use dark theme fill & crisp border"
    );
    let has_dark_checkmark = dark_snap.paths.iter().any(|p| {
        p.points.len() == 3
            && p.stroke.color == eframe::epaint::ColorMode::Solid(dark_check_color)
    });
    assert!(
        has_dark_checkmark,
        "Checked checkbox checkmark should use bright dark-mode fg_stroke"
    );

    // 2. Alterna dinamicamente em tempo de execução para o Modo Claro
    app.dark_mode = false;
    let light_snap = render_app_frame(&ctx, &mut app);
    assert_eq!(ctx.theme(), egui::Theme::Light);
    assert!(!ctx.global_style().visuals.dark_mode);

    let light_box_fill = egui::Color32::from_rgb(255, 255, 255);
    let light_box_border = egui::Color32::from_rgb(100, 116, 139);
    let light_check_color = egui::Color32::from_rgb(15, 23, 42);
    assert_ne!(light_box_fill, GeoEstabilTheme::card_bg(false));

    let has_light_checkbox_box = light_snap.rects.iter().any(|r| {
        r.fill == light_box_fill
            && r.stroke.color == light_box_border
            && (r.rect.width() - r.rect.height()).abs() < 2.0
            && r.rect.width() <= 24.0
    });
    assert!(
        has_light_checkbox_box,
        "Checkbox box MUST adapt dynamically to light mode fill & border"
    );
    let has_light_checkmark = light_snap.paths.iter().any(|p| {
        p.points.len() == 3
            && p.stroke.color == eframe::epaint::ColorMode::Solid(light_check_color)
    });
    assert!(
        has_light_checkmark,
        "Checked checkbox checkmark MUST adapt dynamically to dark ink in light mode"
    );

    // Garante que não sobrou nenhum checkbox com fill escuro antigo no modo claro
    let stale_dark_checkbox = light_snap.rects.iter().any(|r| {
        r.fill == dark_box_fill
            && r.stroke.color == dark_box_border
            && (r.rect.width() - r.rect.height()).abs() < 2.0
            && r.rect.width() <= 24.0
    });
    assert!(
        !stale_dark_checkbox,
        "No checkbox should retain dark-mode box fill after switching to light mode"
    );

    // 3. Alterna de volta para Modo Escuro e verifica Tab 1 e Tab 3 também
    app.dark_mode = true;
    for tab in [0_usize, 1, 2] {
        app.active_tab = tab;
        let back_dark = render_app_frame(&ctx, &mut app);
        assert_eq!(ctx.theme(), egui::Theme::Dark);
        let has_box = back_dark.rects.iter().any(|r| {
            r.fill == dark_box_fill
                && r.stroke.color == dark_box_border
                && (r.rect.width() - r.rect.height()).abs() < 2.0
                && r.rect.width() <= 24.0
        });
        assert!(has_box, "Tab {tab} checkboxes must adapt back to dark mode");
    }

    // 4. Verifica que Tab 1 e Tab 3 também adaptam todos os checkboxes para Modo Claro
    app.dark_mode = false;
    for tab in [0_usize, 2] {
        app.active_tab = tab;
        let tab_light = render_app_frame(&ctx, &mut app);
        assert_eq!(ctx.theme(), egui::Theme::Light);
        let has_box = tab_light.rects.iter().any(|r| {
            r.fill == light_box_fill
                && r.stroke.color == light_box_border
                && (r.rect.width() - r.rect.height()).abs() < 2.0
                && r.rect.width() <= 24.0
        });
        assert!(has_box, "Tab {tab} checkboxes must adapt to light mode");
    }
}

#[test]
fn test_modeless_dialogs_adapt_theme_and_language_dynamically() {
    let ctx = egui::Context::default();
    let mut manual = ManualDialog::default();
    let mut about = AboutDialog::default();
    manual.open();
    about.open();

    for (dark, lang) in [
        (true, Language::PtBr),
        (false, Language::EnUs),
        (true, Language::EnUs),
        (false, Language::PtBr),
    ] {
        let loc = Localizer::new(lang);
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 840.0),
            )),
            ..Default::default()
        };
        let mut out = ctx.run_ui(raw_input, |ui| {
            let c = ui.ctx().clone();
            manual.show(&c, &loc, dark);
            about.show(&c, &loc, dark);
        });
        assert_eq!(ctx.theme(), egui::Theme::from_dark_mode(dark));
        assert_eq!(ctx.global_style().visuals.dark_mode, dark);
        out.textures_delta.clear();
    }
}
