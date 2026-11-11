use geoestabil_rs::core::project::{ActiveProjectMode, GeoProject};
use geoestabil_rs::engine::slope::grid_search::run_slope_stability_analysis;
use geoestabil_rs::locale::translations::{Language, Localizer};
use geoestabil_rs::reports::MemorialGenerator;
use geoestabil_rs::ui::app::GeoEstabilApp;
use geoestabil_rs::ui::canvas_2d::CanvasState;
use geoestabil_rs::ui::dialogs::doc_renderer::{parse_doc_blocks, try_extract_fraction, DocBlock};
use geoestabil_rs::ui::dialogs::modeless::{
    compute_child_window_rect, ctx_zorder_raise, ctx_zorder_snapshot, WindowZOrderManager,
    ROOT_WINDOW_ID,
};
use geoestabil_rs::ui::dialogs::{AboutDialog, ManualDialog, SettingsDialog, SoilCatalogDialog};
use geoestabil_rs::utils::export_dxf_svg::VectorExporter;
use geoestabil_rs::utils::paths::{
    load_language_code_from_file, resolve_startup_project_path, resolver_caminho_asset_em_bases,
    save_language_code_to_file,
};

fn make_lightweight_app() -> GeoEstabilApp {
    let mut project = GeoProject::default();
    project.slope_model.grid_x_steps = 6;
    project.slope_model.grid_y_steps = 6;
    let slope_report = run_slope_stability_analysis(&project.slope_model);
    GeoEstabilApp {
        project,
        active_tab: 0,
        slope_report: Some(slope_report),
        slope_canvas_state: CanvasState::default(),
        wall_canvas_state: CanvasState::default(),
        dark_mode: true,
        localizer: Localizer::new(Language::PtBr),
        db: None,
        about_dialog: AboutDialog::default(),
        manual_dialog: ManualDialog::default(),
        soil_catalog_dialog: SoilCatalogDialog::default(),
        settings_dialog: SettingsDialog::default(),
    }
}

fn collect_texts_from_output(out: &egui::FullOutput) -> Vec<String> {
    let mut texts = Vec::new();
    for cs in &out.shapes {
        collect_shape_texts(&cs.shape, &mut texts);
    }
    texts
}

fn collect_shape_texts(shape: &egui::Shape, out: &mut Vec<String>) {
    match shape {
        egui::Shape::Text(t) => {
            let txt = t.galley.text().trim();
            if !txt.is_empty() {
                out.push(txt.to_string());
            }
        }
        egui::Shape::Vec(vec) => {
            for s in vec {
                collect_shape_texts(s, out);
            }
        }
        _ => {}
    }
}

fn render_with_viewport_outputs(
    ctx: &egui::Context,
    app: &mut GeoEstabilApp,
    raw_input: egui::RawInput,
) -> Vec<String> {
    let mut all_texts = Vec::new();
    let mut out = ctx.run_ui(raw_input, |ui| {
        app.render_ui(ui);
    });
    all_texts.extend(collect_texts_from_output(&out));
    out.textures_delta.clear();
    all_texts
}

#[test]
fn test_default_engineer_name_empty_and_geoestabil_fallback_all_formats() {
    let mut project = GeoProject::default();
    assert!(
        project.engineer_name.is_empty(),
        "Default GeoProject::engineer_name must be blank instead of hardcoded personal author"
    );
    assert!(
        !project.engineer_name.contains("Fernando"),
        "Default GeoProject::engineer_name must not pre-fill Fernando Nillsson Cidade"
    );
    assert_eq!(project.effective_engineer_name(), "GeoEstabil");

    project.engineer_name = "   ".to_string();
    assert_eq!(project.effective_engineer_name(), "GeoEstabil");

    project.active_mode = ActiveProjectMode::RetainingWall;

    for lang in [Language::PtBr, Language::EnUs] {
        let html = MemorialGenerator::generate_html_lang(&project, None, lang);
        assert!(
            html.contains("GeoEstabil"),
            "HTML must report GeoEstabil fallback when engineer_name is blank"
        );
        assert!(
            !html.contains("Fernando Nillsson Cidade"),
            "HTML engineer field must not inject hardcoded personal name when blank"
        );

        let md = MemorialGenerator::generate_markdown_lang(&project, None, lang);
        assert!(
            md.contains("GeoEstabil"),
            "Markdown must include GeoEstabil sign-off fallback"
        );
        assert!(
            !md.contains("Fernando Nillsson Cidade"),
            "Markdown must not report personal name when blank"
        );

        let pdf_bytes = MemorialGenerator::generate_pdf_lang(&project, None, lang);
        let pdf_text = String::from_utf8_lossy(&pdf_bytes);
        assert!(
            pdf_text.contains("(GeoEstabil"),
            "PDF must emit GeoEstabil engineer fallback in text stream"
        );
        assert!(
            !pdf_text.contains("Fernando Nillsson Cidade"),
            "PDF must not emit personal name when engineer_name is blank"
        );
    }

    // Tab 5 GUI preview + placeholder check
    let ctx = egui::Context::default();
    let mut app = make_lightweight_app();
    app.active_tab = 4;
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1280.0, 820.0),
        )),
        ..Default::default()
    };
    let texts = render_with_viewport_outputs(&ctx, &mut app, raw);
    let joined = texts.join("\n");
    assert!(
        !joined.contains("Fernando Nillsson Cidade"),
        "Tab 5 main GUI preview must not inject personal name when blank:\n{joined}"
    );
    assert!(
        joined.contains("GeoEstabil"),
        "Tab 5 GUI preview must display GeoEstabil fallback"
    );

    // Custom professional name is honored when entered
    project.engineer_name = "Eng. Alice Mendes".to_string();
    assert_eq!(project.effective_engineer_name(), "Eng. Alice Mendes");
    let html_custom = MemorialGenerator::generate_html_lang(&project, None, Language::EnUs);
    let md_custom = MemorialGenerator::generate_markdown_lang(&project, None, Language::EnUs);
    let pdf_custom = MemorialGenerator::generate_pdf_lang(&project, None, Language::EnUs);
    assert!(html_custom.contains("Eng. Alice Mendes"));
    assert!(md_custom.contains("Eng. Alice Mendes"));
    assert!(String::from_utf8_lossy(&pdf_custom).contains("Eng. Alice Mendes"));
}

#[test]
fn test_pdf_tounicode_cmap_multiple_fractions_and_no_raw_latex() {
    let mut project = GeoProject::default();
    project.active_mode = ActiveProjectMode::RetainingWall;

    for lang in [Language::PtBr, Language::EnUs] {
        // 1. PDF CMap & equations
        let pdf_bytes = MemorialGenerator::generate_pdf_lang(&project, None, lang);
        let pdf_text = String::from_utf8_lossy(&pdf_bytes);

        assert!(
            pdf_text.contains("/BaseFont /Symbol") && pdf_text.contains("/ToUnicode 6 0 R"),
            "PDF Symbol font (/F3) must bind /ToUnicode CMap for proper Unicode copy/extraction"
        );
        assert!(
            pdf_text.contains("<73> <03C3>") && pdf_text.contains("<66> <03C6>"),
            "PDF ToUnicode CMap must map Symbol glyph 0x73 -> U+03C3 (σ) and 0x66 -> U+03C6 (φ)"
        );
        assert!(
            !pdf_text.contains("\\frac"),
            "PDF output must never leave raw LaTeX \\frac markup unrendered"
        );
        assert!(
            !pdf_text.contains("sponta") && !pdf_text.contains("stoe"),
            "PDF must never output malformed 'sponta'/'stoe' text without σ mapping"
        );

        // 2. HTML stacked dual fractions & formatted A_s
        let html = MemorialGenerator::generate_html_lang(&project, None, lang);
        assert!(
            !html.contains("\\frac"),
            "HTML output must convert every \\frac into stacked HTML fractions"
        );
        assert!(
            !html.contains("sponta"),
            "HTML must contain properly formatted Greek σ instead of 'sponta'"
        );
        assert!(
            html.contains("A<sub>s</sub>"),
            "HTML must format rebar area A_s with HTML subscript tag A<sub>s</sub>"
        );
        // Base bearing stress formula uses TWO stacked fractions (\frac{N}{B} and \frac{6 · e}{B})
        let base_eq_marker = if lang == Language::PtBr {
            "ponta,calcanhar"
        } else {
            "toe,heel"
        };
        if let Some(pos) = html.find(base_eq_marker) {
            let slice_end = (pos + 900).min(html.len());
            let eq_snippet = &html[pos..slice_end];
            let frac_count = eq_snippet.matches("<span class=\"frac\">").count();
            assert!(
                frac_count >= 2,
                "Base bearing stress equation in HTML ({lang:?}) must render both stacked fractions, found {frac_count}:\n{eq_snippet}"
            );
        }

        // 3. Markdown Tab 5 equation parser handles multiple fractions recursively
        let md = MemorialGenerator::generate_markdown_lang(&project, None, lang);
        let blocks = parse_doc_blocks(&md);
        let eq_blocks: Vec<&str> = blocks
            .iter()
            .filter_map(|b| match b {
                DocBlock::Equation(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        let bearing_eq = eq_blocks
            .iter()
            .find(|s| s.contains(base_eq_marker) || s.contains("3 \\cdot x_R"))
            .expect("Markdown preview must include base bearing pressure equation block");
        if bearing_eq.contains("6 \\cdot e") {
            let f1 = try_extract_fraction(bearing_eq).expect("First \\frac must extract");
            let f2 = try_extract_fraction(&f1.suffix).expect("Second \\frac in suffix must also extract");
            assert!(f2.numerator.contains("6") && f2.denominator.trim() == "B");
        }
    }
}

#[test]
fn test_extreme_long_inputs_wrap_cleanly_within_margins_and_canvas_bounds() {
    let mut project = GeoProject {
        title: "Estudo Geotecnico Avancado de Contencao de Encosta Urbana em Trecho Critico Rodoviario Km 142+850 com Variacao Estratigrafica Complexa e Sobrecarga Permanente Elevada".to_string(),
        client: "Consorcio Infraestrutura Metropolitana do Sul S/A - Divisao Especial de Obras Geotecnicas e Fundacoes Profundas".to_string(),
        site_location: "Trevo Viario Norte - Rodovia Estadual RS-118, Intersecao Km 22, Pista Marginal Esquerda, Municipio de Gravatai/RS".to_string(),
        engineer_name: "Eng. Responsavel com Nome Extremamente Extenso para Verificacao de Quebra Automatica em Cards e Assinatura".to_string(),
        ..Default::default()
    };

    if let Some(layer) = project.slope_model.layers.first_mut() {
        layer.name = "Argila Silto-Arenosa Residual Muito Heterogenea com Pedregulhos e Blocos de Rocha Alterada".to_string();
    }

    let slope_report = run_slope_stability_analysis(&project.slope_model);

    for mode in [ActiveProjectMode::SlopeStability, ActiveProjectMode::RetainingWall] {
        project.active_mode = mode;
        for lang in [Language::PtBr, Language::EnUs] {
            let pdf_bytes = MemorialGenerator::generate_pdf_lang(&project, Some(&slope_report), lang);
            let pdf_str = String::from_utf8_lossy(&pdf_bytes);
            assert!(pdf_bytes.starts_with(b"%PDF-1.4\n"));
            assert!(pdf_str.contains("%%EOF"));

            // Parse every PDF text matrix `BT 1 0 0 1 X Y Tm` to confirm coordinates stay within A4 margins
            let mut tm_count = 0usize;
            for line in pdf_str.lines() {
                if let Some(rest) = line.strip_prefix("BT 1 0 0 1 ") {
                    if let Some(coords) = rest.strip_suffix(" Tm") {
                        let parts: Vec<&str> = coords.split_whitespace().collect();
                        if parts.len() == 2 {
                            let x: f64 = parts[0].parse().unwrap();
                            let y: f64 = parts[1].parse().unwrap();
                            assert!(
                                (20.0..=580.0).contains(&x),
                                "PDF text X={x} escaped A4 safe bounds [20, 580] in line: {line}"
                            );
                            assert!(
                                (15.0..=830.0).contains(&y),
                                "PDF text Y={y} escaped A4 page height [15, 830] in line: {line}"
                            );
                            tm_count += 1;
                        }
                    }
                }
            }
            assert!(tm_count > 40, "Expected rich PDF text content, found {tm_count} Tm lines");

            // HTML responsive wrapping & SVG containment CSS
            let html = MemorialGenerator::generate_html_lang(&project, Some(&slope_report), lang);
            assert!(html.contains("box-sizing: border-box"));
            assert!(html.contains("overflow-wrap: anywhere"));
            assert!(html.contains(".svg-figure svg { max-width: 100% !important; height: auto !important;"));

            // SVG legend truncation & viewBox containment
            let svg = VectorExporter::export_active_svg_lang(&project, Some(&slope_report), lang);
            assert!(svg.contains("<svg") && svg.contains("</svg>"));
            if mode == ActiveProjectMode::SlopeStability {
                assert!(
                    svg.contains("..."),
                    "Long soil layer name in Slope SVG legend must truncate cleanly with ellipsis"
                );
            }
        }
    }
}

#[test]
fn test_bilingual_full_export_parity_slope_and_wall_and_cli() {
    let mut project = GeoProject::default();
    let slope_rep = run_slope_stability_analysis(&project.slope_model);

    // Slope stability EN vs PT exports
    project.active_mode = ActiveProjectMode::SlopeStability;
    let pt_html = MemorialGenerator::generate_html_lang(&project, Some(&slope_rep), Language::PtBr);
    let en_html = MemorialGenerator::generate_html_lang(&project, Some(&slope_rep), Language::EnUs);
    assert!(pt_html.contains("Estabilidade de Taludes") && pt_html.contains("Bishop Simplificado"));
    assert!(!pt_html.contains("Simplified Bishop"));
    assert!(en_html.contains("Slope Stability") && en_html.contains("Simplified Bishop"));
    assert!(!en_html.contains("Estabilidade de Taludes"));

    // Retaining wall EN vs PT exports
    project.active_mode = ActiveProjectMode::RetainingWall;
    let pt_md = MemorialGenerator::generate_markdown_lang(&project, None, Language::PtBr);
    let en_md = MemorialGenerator::generate_markdown_lang(&project, None, Language::EnUs);
    assert!(pt_md.contains("Muro de Contenção") && pt_md.contains("Tombamento"));
    assert!(!pt_md.contains("Retaining Wall Structural Design"));
    assert!(en_md.contains("Retaining Wall") && en_md.contains("Overturning"));
    assert!(!en_md.contains("Muro de Contenção"));

    // CLI `--lang en` and `--lang pt` verification using compiled geoestabil-cli binary
    let tmp_dir = std::env::temp_dir().join(format!("geoestabil_cli_i18n_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let en_pdf_path = tmp_dir.join("preset4_en.pdf");
    let pt_html_path = tmp_dir.join("preset1_pt.html");

    let out_en = std::process::Command::new(env!("CARGO_BIN_EXE_geoestabil-cli"))
        .args([
            "run-preset",
            "4",
            "-o",
            en_pdf_path.to_str().unwrap(),
            "--lang",
            "en",
        ])
        .output()
        .expect("execute geoestabil-cli --lang en");
    assert!(
        out_en.status.success(),
        "CLI EN failed: {}",
        String::from_utf8_lossy(&out_en.stderr)
    );
    let en_stdout = String::from_utf8_lossy(&out_en.stdout);
    assert!(
        en_stdout.contains("Running Preset 4")
            && en_stdout.contains("Calculating Retaining Wall")
            && en_stdout.contains("FS_ovt =")
            && !en_stdout.contains("FS_tomb")
            && en_stdout.contains("Report saved to:"),
        "CLI stdout with --lang en must be localized to English:\n{en_stdout}"
    );
    let en_pdf_bytes = std::fs::read(&en_pdf_path).unwrap();
    assert!(en_pdf_bytes.starts_with(b"%PDF-1.4\n"));
    let en_pdf_text = String::from_utf8_lossy(&en_pdf_bytes);
    assert!(en_pdf_text.contains("Calculation Memorial") || en_pdf_text.contains("Retaining Wall"));
    assert!(!en_pdf_text.contains("Muro de Conten"));

    let out_pt = std::process::Command::new(env!("CARGO_BIN_EXE_geoestabil-cli"))
        .args([
            "run-preset",
            "1",
            "-o",
            pt_html_path.to_str().unwrap(),
            "--lang",
            "pt",
        ])
        .output()
        .expect("execute geoestabil-cli --lang pt");
    assert!(
        out_pt.status.success(),
        "CLI PT failed: {}",
        String::from_utf8_lossy(&out_pt.stderr)
    );
    let pt_stdout = String::from_utf8_lossy(&out_pt.stdout);
    assert!(
        pt_stdout.contains("Executando Preset 1") && pt_stdout.contains("Relatório salvo em:"),
        "CLI stdout with --lang pt must be localized to Portuguese:\n{pt_stdout}"
    );
    let pt_html_cli = std::fs::read_to_string(&pt_html_path).unwrap();
    assert!(
        pt_html_cli.contains("MEMORIAL DESCRITIVO E DE CÁLCULO")
            && pt_html_cli.contains("Estabilidade de Taludes")
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_zorder_manager_modeless_lifecycle_and_click_to_raise() {
    // 1. Unit verification of WindowZOrderManager stack semantics
    let mut mgr = WindowZOrderManager::default();
    assert_eq!(mgr.top(), Some(ROOT_WINDOW_ID));
    assert_eq!(mgr.depth_index(ROOT_WINDOW_ID), Some(0));

    mgr.raise("geoestabil_user_manual");
    mgr.raise("geoestabil_about_dialog");
    mgr.raise("geoestabil_soil_catalog");
    assert_eq!(mgr.top(), Some("geoestabil_soil_catalog"));
    assert_eq!(mgr.depth_index("geoestabil_user_manual"), Some(1));
    assert_eq!(mgr.depth_index("geoestabil_about_dialog"), Some(2));
    assert_eq!(mgr.depth_index("geoestabil_soil_catalog"), Some(3));
    assert_eq!(mgr.child_cascade_slot("geoestabil_user_manual"), 0);
    assert_eq!(mgr.child_cascade_slot("geoestabil_soil_catalog"), 2);

    // ensure_present must NOT steal Z-order top from an already active top window
    mgr.ensure_present("geoestabil_user_manual");
    assert_eq!(mgr.top(), Some("geoestabil_soil_catalog"));

    // Clicking earlier window lifts it to the absolute top
    mgr.raise("geoestabil_user_manual");
    assert_eq!(mgr.top(), Some("geoestabil_user_manual"));

    // Closing top falls back cleanly without removing ROOT_WINDOW_ID
    mgr.close("geoestabil_user_manual");
    assert_eq!(mgr.top(), Some("geoestabil_soil_catalog"));
    mgr.close(ROOT_WINDOW_ID);
    assert_eq!(mgr.depth_index(ROOT_WINDOW_ID), Some(0));

    // 2. End-to-end GeoEstabilApp modeless non-blocking open + click-to-raise
    let ctx = egui::Context::default();
    let mut app = make_lightweight_app();
    app.manual_dialog.open();
    app.about_dialog.open();
    app.soil_catalog_dialog.open();
    app.settings_dialog.open();

    assert!(app.manual_dialog.modeless.is_open);
    assert!(app.about_dialog.modeless.is_open);
    assert!(app.soil_catalog_dialog.is_open());
    assert!(app.settings_dialog.is_open());

    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1440.0, 900.0),
        )),
        ..Default::default()
    };
    let _ = render_with_viewport_outputs(&ctx, &mut app, raw.clone());
    let texts = render_with_viewport_outputs(&ctx, &mut app, raw);
    let joined = texts.join("\n");
    assert!(joined.contains("Manual (F1)"), "Missing Manual pill in:\n{joined}");
    assert!(joined.contains("Sobre"), "Missing About pill in:\n{joined}");
    assert!(joined.contains("Solos SPT"), "Missing Soils pill in:\n{joined}");
    assert!(joined.contains("Preferências"), "Missing Settings pill in:\n{joined}");
    assert!(
        joined.contains("▲ "),
        "Status bar pill must mark top window in Z-order with ▲"
    );

    // Raising another child dialog changes top without blocking any open modeless window
    ctx_zorder_raise(&ctx, "geoestabil_manual_custom");
    assert_eq!(
        ctx_zorder_snapshot(&ctx).top(),
        Some("geoestabil_manual_custom")
    );

    // Clicking root viewport restores ROOT_WINDOW_ID to top of Z-Order
    let click_root = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1280.0, 800.0),
        )),
        events: vec![egui::Event::PointerButton {
            pos: egui::pos2(600.0, 400.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = render_with_viewport_outputs(&ctx, &mut app, click_root);
    assert_eq!(ctx_zorder_snapshot(&ctx).top(), Some(ROOT_WINDOW_ID));
    assert!(app.manual_dialog.modeless.is_open);
    assert!(app.settings_dialog.is_open());
}

#[test]
fn test_child_window_same_monitor_placement_multimonitor_and_clamp() {
    // 1. Primary Monitor [0..1920 x 0..1040]
    let mon1 = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1920.0, 1040.0));
    let parent1 = egui::Rect::from_min_size(egui::pos2(200.0, 100.0), egui::vec2(1280.0, 800.0));
    let (pos1, size1) = compute_child_window_rect(parent1, Some(mon1), [760.0, 520.0], 0);
    assert!(pos1[0] >= mon1.min.x && pos1[0] + size1[0] <= mon1.max.x);
    assert!(pos1[1] >= mon1.min.y && pos1[1] + size1[1] <= mon1.max.y);

    // 2. Secondary Monitor to the RIGHT [1920..3840 x 0..1080]
    let mon2_right =
        egui::Rect::from_min_max(egui::pos2(1920.0, 0.0), egui::pos2(3840.0, 1080.0));
    let parent_right =
        egui::Rect::from_min_size(egui::pos2(2200.0, 120.0), egui::vec2(1200.0, 760.0));
    let (pos_r, size_r) =
        compute_child_window_rect(parent_right, Some(mon2_right), [980.0, 680.0], 1);
    assert!(
        pos_r[0] >= mon2_right.min.x && pos_r[0] + size_r[0] <= mon2_right.max.x,
        "Child opened at x={} w={} must remain on secondary right monitor [1920..3840]",
        pos_r[0],
        size_r[0]
    );
    assert!(pos_r[1] >= mon2_right.min.y && pos_r[1] + size_r[1] <= mon2_right.max.y);

    // 3. Secondary Monitor to the LEFT [-1920..0 x 0..1080] (negative X desktop topology)
    let mon2_left =
        egui::Rect::from_min_max(egui::pos2(-1920.0, 0.0), egui::pos2(0.0, 1080.0));
    let parent_left =
        egui::Rect::from_min_size(egui::pos2(-1600.0, 140.0), egui::vec2(1200.0, 760.0));
    let (pos_l, size_l) =
        compute_child_window_rect(parent_left, Some(mon2_left), [760.0, 520.0], 2);
    assert!(
        pos_l[0] >= mon2_left.min.x && pos_l[0] + size_l[0] <= mon2_left.max.x,
        "Child opened at x={} w={} must remain on secondary left monitor [-1920..0]",
        pos_l[0],
        size_l[0]
    );

    // 4. Compact monitor work area clamps oversized dialog safely inside same monitor
    let mon_small =
        egui::Rect::from_min_max(egui::pos2(1920.0, 0.0), egui::pos2(2720.0, 600.0));
    let parent_corner =
        egui::Rect::from_min_size(egui::pos2(2300.0, 300.0), egui::vec2(700.0, 500.0));
    let (pos_s, size_s) =
        compute_child_window_rect(parent_corner, Some(mon_small), [1080.0, 760.0], 0);
    assert!(size_s[0] <= mon_small.width() - 24.0 + 0.1);
    assert!(size_s[1] <= mon_small.height() - 42.0 + 0.1);
    assert!(pos_s[0] >= mon_small.min.x && pos_s[0] + size_s[0] <= mon_small.max.x);
    assert!(pos_s[1] >= mon_small.min.y && pos_s[1] + size_s[1] <= mon_small.max.y);
}

#[test]
fn test_inno_setup_portable_asset_resolution_any_install_path() {
    let base = std::env::temp_dir().join(format!("geoestabil_inno_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);

    // 1. Standard Inno Setup `{app}` containing `src\assets\...` while CWD & manifest point elsewhere
    let app_dir = base.join("CustomDriveProgramFiles").join("GeoEstabil");
    let app_about = app_dir.join("src").join("assets").join("ABOUT");
    let app_icons = app_dir.join("src").join("assets").join("icons");
    let dev_repo_fake = base.join("DevRepo");
    let dev_about = dev_repo_fake.join("src").join("assets").join("ABOUT");
    let unrelated_cwd = base.join("RandomClientProject");

    std::fs::create_dir_all(&app_about).unwrap();
    std::fs::create_dir_all(&app_icons).unwrap();
    std::fs::create_dir_all(&dev_about).unwrap();
    std::fs::create_dir_all(&unrelated_cwd).unwrap();

    let installed_about = app_about.join("ABOUT_en_US.txt");
    let dev_about_file = dev_about.join("ABOUT_en_US.txt");
    let installed_ico = app_icons.join("geoestabil.ico");
    std::fs::write(&installed_about, "Installed ABOUT").unwrap();
    std::fs::write(&dev_about_file, "Dev Repo ABOUT").unwrap();
    std::fs::write(&installed_ico, b"ICO_BYTES").unwrap();

    // Exact relative, `assets/...`, `src/assets/...`, and backslash variants must resolve to `{app}\src\assets` FIRST
    for query in [
        "ABOUT/ABOUT_en_US.txt",
        "assets/ABOUT/ABOUT_en_US.txt",
        "src/assets/ABOUT/ABOUT_en_US.txt",
        "src\\assets\\ABOUT\\ABOUT_en_US.txt",
    ] {
        let resolved = resolver_caminho_asset_em_bases(
            query,
            Some(&app_dir),
            Some(&unrelated_cwd),
            Some(&dev_repo_fake),
            None,
            &[],
        )
        .expect("Installed Inno Setup src\\assets file must resolve from arbitrary CWD");
        assert_eq!(resolved, installed_about);
    }

    let resolved_ico = resolver_caminho_asset_em_bases(
        "icons/geoestabil.ico",
        Some(&app_dir),
        Some(&unrelated_cwd),
        Some(&dev_repo_fake),
        None,
        &[],
    )
    .expect("Installed icon must resolve");
    assert_eq!(resolved_ico, installed_ico);

    // 2. Exe inside `{app}\bin` resolving `{app}\src\assets\...` in parent directory
    let bin_dir = app_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let resolved_from_bin = resolver_caminho_asset_em_bases(
        "ABOUT/ABOUT_en_US.txt",
        Some(&bin_dir),
        Some(&unrelated_cwd),
        None,
        None,
        &[],
    )
    .expect("Exe inside {app}\\bin must resolve parent {app}\\src\\assets");
    assert_eq!(resolved_from_bin, installed_about);

    // 3. Relocated `{app}\assets\...` directly beside executable without `src\`
    let flat_app = base.join("FlatApp");
    let flat_rel = flat_app.join("assets").join("NOTICES");
    std::fs::create_dir_all(&flat_rel).unwrap();
    let flat_notice = flat_rel.join("NOTICE_pt_BR.txt");
    std::fs::write(&flat_notice, "Aviso").unwrap();
    let resolved_flat = resolver_caminho_asset_em_bases(
        "NOTICES/NOTICE_pt_BR.txt",
        Some(&flat_app),
        Some(&unrelated_cwd),
        None,
        None,
        &[],
    )
    .expect("{app}\\assets\\... must also resolve");
    assert_eq!(resolved_flat, flat_notice);

    // 4. `%LOCALAPPDATA%\GeoEstabil\src\assets` or `%ProgramData%\GeoEstabil\src\assets`
    let empty_app = base.join("ExeOnly");
    std::fs::create_dir_all(&empty_app).unwrap();
    let prog_data = base.join("ProgramData");
    let prog_asset_dir = prog_data
        .join("GeoEstabil")
        .join("src")
        .join("assets")
        .join("EULA");
    std::fs::create_dir_all(&prog_asset_dir).unwrap();
    let prog_eula = prog_asset_dir.join("EULA_en_US - GeoEstabil.txt");
    std::fs::write(&prog_eula, "EULA").unwrap();
    let resolved_prog_data = resolver_caminho_asset_em_bases(
        "EULA/EULA_en_US - GeoEstabil.txt",
        Some(&empty_app),
        Some(&unrelated_cwd),
        None,
        None,
        &[prog_data],
    )
    .expect("%ProgramData%\\GeoEstabil\\src\\assets must resolve");
    assert_eq!(resolved_prog_data, prog_eula);

    // 5. Explicit GEOESTABIL_ASSETS_DIR override takes top precedence
    let custom_override = base.join("OverrideAssets");
    let custom_about = custom_override.join("ABOUT");
    std::fs::create_dir_all(&custom_about).unwrap();
    let override_file = custom_about.join("ABOUT_en_US.txt");
    std::fs::write(&override_file, "Override").unwrap();
    let resolved_override = resolver_caminho_asset_em_bases(
        "ABOUT/ABOUT_en_US.txt",
        Some(&app_dir),
        Some(&unrelated_cwd),
        Some(&dev_repo_fake),
        Some(&custom_override),
        &[],
    )
    .expect("GEOESTABIL_ASSETS_DIR override must resolve first");
    assert_eq!(resolved_override, override_file);

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_language_config_json_roundtrip_and_code_parsing() {
    let base = std::env::temp_dir().join(format!("geoestabil_lang_cfg_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cfg_path = base.join("GeoEstabil").join("language_config.json");

    save_language_code_to_file(&cfg_path, Language::EnUs.code()).expect("save en_US");
    let loaded_en = load_language_code_from_file(&cfg_path).expect("load en_US");
    assert_eq!(loaded_en, "en_US");
    assert_eq!(
        Language::from_config_code(&loaded_en),
        Some(Language::EnUs)
    );

    save_language_code_to_file(&cfg_path, Language::PtBr.code()).expect("save pt_BR");
    let loaded_pt = load_language_code_from_file(&cfg_path).expect("load pt_BR");
    assert_eq!(loaded_pt, "pt_BR");
    assert_eq!(
        Language::from_config_code(&loaded_pt),
        Some(Language::PtBr)
    );

    assert_eq!(Language::from_config_code("  EN_US "), Some(Language::EnUs));
    assert_eq!(Language::from_config_code("invalid"), None);
    assert_eq!(
        load_language_code_from_file(&base.join("missing.json")),
        None
    );

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_startup_geoestabil_project_arg_resolution_and_app_loading() {
    let base = std::env::temp_dir().join(format!("geoestabil_cli_open_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();

    let proj_path = base.join("Talude Associado.geoestabil");
    let mut proj = GeoProject::default();
    proj.title = "Projeto Aberto pelo Windows Explorer".to_string();
    proj.active_mode = ActiveProjectMode::RetainingWall;
    proj.save_to_file(&proj_path).expect("save .geoestabil");

    let args = vec![
        "C:\\Program Files\\GeoEstabil\\geoestabil.exe".to_string(),
        "--some-flag".to_string(),
        proj_path.to_string_lossy().to_string(),
    ];
    let resolved = resolve_startup_project_path(args).expect("resolve .geoestabil arg");
    assert_eq!(resolved, proj_path);

    let no_proj_args = vec![
        "geoestabil.exe".to_string(),
        "--help".to_string(),
    ];
    assert_eq!(resolve_startup_project_path(no_proj_args), None);

    let app = GeoEstabilApp::with_startup_file(Some(&resolved));
    assert_eq!(app.project.title, "Projeto Aberto pelo Windows Explorer");
    assert_eq!(app.project.active_mode, ActiveProjectMode::RetainingWall);
    assert_eq!(app.active_tab, 2);

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_inno_setup_os_integration_invariants() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let iss_path = manifest_dir.join("repo").join("GeoEstabil.iss");
    let content = std::fs::read_to_string(&iss_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", iss_path.display(), e));

    // Zero stale template names from other apps
    for forbidden in ["Huginn", "Eolo", "eolo.exe", "eolo-cli.exe", "EoloPlatform"] {
        assert!(
            !content.contains(forbidden),
            "Forbidden stale string `{}` found in GeoEstabil.iss",
            forbidden
        );
    }

    // Core identity, dual privileges, OS shell notifications, clean distribution & registry integration
    for required in [
        r#"#define MyAppName "GeoEstabil""#,
        r#"#define MyAppExeName "geoestabil.exe""#,
        r#"#define MyAppCliName "geoestabil-cli.exe""#,
        r#"#define MyAppUserModelID "fernandoncidade.GeoEstabil""#,
        r#"#define MyAppAssocExt ".geoestabil""#,
        r#"#define MyAppAssocKey "GeoEstabil.ProjectFile""#,
        "AppId={{7C8A6122-2026-1110-9F4B-6E0E57AB1168}",
        "OutputBaseFilename=GeoEstabil_v{#MyAppVersion}",
        "MinVersion=10.0",
        "PrivilegesRequired=lowest",
        "PrivilegesRequiredOverridesAllowed=dialog commandline",
        "CloseApplications=yes",
        "CloseApplicationsFilter={#MyAppExeName},{#MyAppCliName}",
        "ChangesEnvironment=yes",
        "ChangesAssociations=yes",
        r#"AppUserModelID: "{#MyAppUserModelID}""#,
        "Root: HKA;",
        r#"Software\Classes\{#MyAppAssocExt}"#,
        r#"Software\Classes\{#MyAppAssocKey}\shell\open\command"#,
        r#"Software\Classes\Applications\{#MyAppExeName}"#,
        r#"Software\Microsoft\Windows\CurrentVersion\App Paths\{#MyAppExeName}"#,
        r#"Software\Microsoft\Windows\CurrentVersion\App Paths\{#MyAppCliName}"#,
        "src\\assets\\*",
        "AddPathToEnvironment",
        "RemovePathFromEnvironment",
        "CreateLanguageConfigJSON",
        "language_config.json",
    ] {
        assert!(
            content.contains(required),
            "Missing required OS-integration directive `{}` in GeoEstabil.iss",
            required
        );
    }
}
