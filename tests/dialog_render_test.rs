use eframe::egui;
use geoestabil_rs::locale::manual::get_manual_sections;
use geoestabil_rs::locale::translations::{Language, Localizer};
use geoestabil_rs::ui::dialogs::{AboutDialog, ManualDialog};
use geoestabil_rs::utils::icons::{carregar_icone_janela, obter_caminho_icone};
use geoestabil_rs::utils::paths::resolver_caminho_asset;

fn run_headless_frame(ctx: &egui::Context, mut ui_fn: impl FnMut(&egui::Context)) -> usize {
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 840.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |ui| {
        ui_fn(ui.ctx());
    });
    let shapes = output.shapes.len();
    output.textures_delta.clear();
    shapes
}

#[test]
fn test_manual_dialog_bilingual_render_and_search() {
    let ctx = egui::Context::default();
    let mut manual = ManualDialog::default();
    manual.open();
    manual.search_query = "Bishop".to_string();

    for lang in [Language::PtBr, Language::EnUs] {
        let localizer = Localizer::new(lang);
        let sections = get_manual_sections(lang);
        assert_eq!(sections.len(), 6);
        for idx in 0..sections.len() {
            manual.selected_section = idx;
            let shapes = run_headless_frame(&ctx, |c| manual.show(c, &localizer, true));
            assert!(shapes > 0, "Manual section {} failed to emit UI shapes", idx);
        }
    }
}

#[test]
fn test_manual_dialog_dynamic_resize_wrap_and_framed_columns() {
    let ctx = egui::Context::default();
    let mut manual = ManualDialog::default();
    manual.open();
    let vp_id = egui::ViewportId::from_hash_of("geoestabil_user_manual");

    fn collect_text_and_frames(
        shape: &egui::epaint::Shape,
        text_boxes: &mut Vec<(egui::Rect, String, usize)>,
        framed_rects: &mut Vec<egui::Rect>,
    ) {
        match shape {
            egui::epaint::Shape::Vec(children) => {
                for ch in children {
                    collect_text_and_frames(ch, text_boxes, framed_rects);
                }
            }
            egui::epaint::Shape::Text(ts) => {
                let rect = ts.galley.rect.translate(ts.pos.to_vec2());
                text_boxes.push((rect, ts.galley.text().to_owned(), ts.galley.rows.len()));
            }
            egui::epaint::Shape::Rect(rs)
                if rs.stroke.width > 0.0
                    && rs.rect.width() >= 140.0
                    && rs.rect.height() >= 200.0 =>
            {
                framed_rects.push(rs.rect);
            }
            _ => {}
        }
    }

    for (lang, dark) in [(Language::PtBr, true), (Language::EnUs, false)] {
        let localizer = Localizer::new(lang);
        let mut wide_max_right = 0.0_f32;
        let mut wide_rows = 0usize;
        let mut narrow_max_right = 0.0_f32;
        let mut narrow_rows = 0usize;

        for (win_w, win_h, track_wide, track_narrow) in [
            (1260.0_f32, 820.0_f32, true, false),
            (780.0_f32, 540.0_f32, false, true),
            (640.0_f32, 500.0_f32, false, false),
            (1180.0_f32, 760.0_f32, false, false),
        ] {
            for sec_idx in 0..6 {
                manual.selected_section = sec_idx;
                let mut text_boxes = Vec::new();
                let mut framed_rects = Vec::new();

                // Run 2 frames so ScrollArea/viewport geometry settles at the new manually resized dimensions
                for _ in 0..2 {
                    text_boxes.clear();
                    framed_rects.clear();
                    let mut raw_input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(win_w, win_h),
                        )),
                        ..Default::default()
                    };
                    raw_input.viewports.insert(
                        vp_id,
                        egui::ViewportInfo {
                            inner_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(win_w, win_h),
                            )),
                            ..Default::default()
                        },
                    );
                    let mut out =
                        ctx.run_ui(raw_input, |ui| manual.show(ui.ctx(), &localizer, dark));
                    for cs in &out.shapes {
                        collect_text_and_frames(&cs.shape, &mut text_boxes, &mut framed_rects);
                    }
                    out.textures_delta.clear();
                }

                // Must paint two distinct framed column cards (left TOC moldura + right content moldura)
                assert!(
                    framed_rects.len() >= 2,
                    "Expected framed borders for both columns at {}x{} sec {} ({:?})",
                    win_w,
                    win_h,
                    sec_idx,
                    lang
                );

                let left_frame_rect = framed_rects
                    .iter()
                    .min_by(|a, b| a.center().x.total_cmp(&b.center().x))
                    .expect("left column frame");
                let content_frame_rect = framed_rects
                    .iter()
                    .max_by(|a, b| a.center().x.total_cmp(&b.center().x))
                    .expect("right column frame");
                assert!(
                    content_frame_rect.max.x <= win_w + 1.0,
                    "Right column frame {} exceeded window width {}",
                    content_frame_rect.max.x,
                    win_w
                );
                assert!(
                    left_frame_rect.max.x < content_frame_rect.min.x,
                    "Left column frame {} must sit cleanly left of right column frame {}",
                    left_frame_rect.max.x,
                    content_frame_rect.min.x
                );

                for (r, txt, rows) in &text_boxes {
                    assert!(
                        r.max.x <= win_w + 2.0,
                        "Text '{}' overflowed window width {} on sec {} (rect.max.x={}) for {:?}",
                        txt,
                        win_w,
                        sec_idx,
                        r.max.x,
                        lang
                    );
                    // Text belonging inside Column 1 moldura stays strictly inside left_frame_rect
                    if r.min.y >= left_frame_rect.min.y - 1.0
                        && r.min.x < left_frame_rect.max.x
                    {
                        assert!(
                            r.max.x <= left_frame_rect.max.x + 1.0,
                            "Column 1 text '{}' overflowed moldura {} (rect.max.x={}) at {}x{}",
                            txt,
                            left_frame_rect.max.x,
                            r.max.x,
                            win_w,
                            win_h
                        );
                    }
                    // Text belonging inside Column 2 moldura stays strictly inside content_frame_rect
                    if r.min.y >= content_frame_rect.min.y - 1.0
                        && r.min.x >= content_frame_rect.min.x - 1.0
                    {
                        assert!(
                            r.max.x <= content_frame_rect.max.x + 1.0,
                            "Column 2 text '{}' overflowed moldura {} (rect.max.x={}) on sec {} at {}x{} for {:?}",
                            txt,
                            content_frame_rect.max.x,
                            r.max.x,
                            sec_idx,
                            win_w,
                            win_h,
                            lang
                        );
                    }
                    if sec_idx == 1 && (txt.contains("Fellenius") || txt.contains("Ordinary Method"))
                    {
                        if track_wide {
                            wide_max_right = wide_max_right.max(r.max.x);
                            wide_rows = wide_rows.max(*rows);
                        }
                        if track_narrow {
                            narrow_max_right = narrow_max_right.max(r.max.x);
                            narrow_rows = narrow_rows.max(*rows);
                        }
                    }
                }
            }
        }

        assert!(
            narrow_max_right <= 780.0 + 2.0 && narrow_max_right < wide_max_right,
            "Manual Column 2 text must dynamically re-wrap to narrower width on manual resize (narrow={}, wide={})",
            narrow_max_right,
            wide_max_right
        );
        assert!(
            narrow_rows >= wide_rows,
            "Narrower Manual window should wrap text onto >= rows ({} vs {})",
            narrow_rows,
            wide_rows
        );
    }
}

#[test]
fn test_about_dialog_all_9_tabs_and_assets_loaded() {
    let ctx = egui::Context::default();
    let mut about = AboutDialog::default();
    about.open();
    about.search_query = "Fernando".to_string();

    for lang in [Language::PtBr, Language::EnUs] {
        let localizer = Localizer::new(lang);
        for tab in 0..=8 {
            about.selected_tab = tab;
            if tab == 2 {
                about.selected_history_sub = 1; // Release Notes sub-view
            }
            if tab == 8 {
                about.selected_license_idx = 0; // GNU AGPLv3
            }
            let shapes = run_headless_frame(&ctx, |c| about.show(c, &localizer, tab % 2 == 0));
            assert!(shapes > 0, "About tab {} rendered 0 shapes", tab);
        }
    }
}

#[test]
fn test_about_notices_bilingual_abnt_bibliographic_references_and_copyright() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ctx = egui::Context::default();

    fn gather_texts(s: &egui::epaint::Shape, dst: &mut Vec<String>) {
        match s {
            egui::epaint::Shape::Vec(v) => v.iter().for_each(|c| gather_texts(c, dst)),
            egui::epaint::Shape::Text(t) => dst.push(t.galley.text().to_owned()),
            _ => {}
        }
    }

    for (lang, notice_rel, header_marker, reserved_marker, tab_label) in [
        (
            Language::PtBr,
            "src/assets/NOTICES/NOTICE_pt_BR.txt",
            "3. REFERÊNCIAS BIBLIOGRÁFICAS DAS NORMAS TÉCNICAS (ABNT)",
            "Todos os direitos reservados",
            "Terceiros (Notices)",
        ),
        (
            Language::EnUs,
            "src/assets/NOTICES/NOTICE_en_US.txt",
            "3. BIBLIOGRAPHIC REFERENCES OF TECHNICAL STANDARDS (ABNT)",
            "All rights reserved",
            "Third-Party Notices",
        ),
    ] {
        let notice_text = std::fs::read_to_string(manifest_dir.join(notice_rel))
            .unwrap_or_else(|e| panic!("Missing {}: {}", notice_rel, e));
        assert!(notice_text.contains(header_marker));
        assert!(notice_text.contains(reserved_marker));
        for required_ref in [
            "ABNT NBR 11682:2009",
            "ABNT NBR 6122:2022",
            "ABNT NBR 6118:2023",
            "ABNT NBR 6484:2020",
            "ABNT NBR 6502:2022",
            "ABNT NBR 8036:1983",
            "ABNT NBR 6120:2019",
            "ABNT NBR 7480:2022",
            "ABNT NBR 8681:2003",
            "ABNT NBR 8953:2015",
            "ABNT NBR 12655:2022",
            "ABNT NBR 13133:2021",
            "ABNT NBR 14931:2023",
            "https://www.abntcatalogo.com.br",
        ] {
            assert!(
                notice_text.contains(required_ref),
                "Missing bibliographic reference '{}' in {}",
                required_ref,
                notice_rel
            );
        }

        // Verify AboutDialog renders tab 7 ("Terceiros (Notices)" / "Third-Party Notices") cleanly in both themes
        let localizer = Localizer::new(lang);
        let mut about = AboutDialog::default();
        about.open();
        about.selected_tab = 7;
        about.search_query = "ABNT".to_string();

        let mut emitted = Vec::new();
        for dark in [true, false] {
            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280.0, 900.0),
                )),
                ..Default::default()
            };
            let mut out = ctx.run_ui(raw_input, |ui| about.show(ui.ctx(), &localizer, dark));
            assert!(!out.shapes.is_empty(), "Notices tab emitted 0 shapes ({:?})", lang);
            for cs in &out.shapes {
                gather_texts(&cs.shape, &mut emitted);
            }
            out.textures_delta.clear();
        }
        let joined = emitted.join("\n");
        assert!(
            joined.contains(tab_label),
            "AboutDialog missing tab label '{}' for {:?}",
            tab_label,
            lang
        );

        // Verify Manual sections no longer expose F3 / Compêndio shortcuts and point to Ctrl+I Notices
        let manual_secs = get_manual_sections(lang);
        let manual_all = manual_secs
            .iter()
            .map(|s| s.content)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!manual_all.contains("F3"));
        assert!(!manual_all.contains("Ctrl+N"));
        assert!(!manual_all.contains("Compêndio"));
        assert!(!manual_all.contains("Compendium"));
        assert!(manual_all.contains("Ctrl+I"));
    }
}

#[test]
fn test_memorial_html_formatting_and_abnt_citations() {
    let mut proj = geoestabil_rs::core::project::GeoProject::default();
    for lang in [Language::PtBr, Language::EnUs] {
        proj.translate_defaults(Language::PtBr, lang);
        let html = geoestabil_rs::reports::MemorialGenerator::generate_html_lang(&proj, None, lang);
        match lang {
            Language::PtBr => {
                assert!(html.contains("M<sub>est</sub>") && html.contains("F<sub>resist</sub>"));
                assert!(!html.contains("M_est") && !html.contains("F_resist"));
            }
            Language::EnUs => {
                assert!(html.contains("M<sub>res</sub>") && html.contains("F<sub>res</sub>"));
                assert!(!html.contains("M_res") && !html.contains("F_res"));
            }
        }
        assert!(html.contains("ABNT NBR 6122:2022") && !html.contains("ABNT NBR 6122:2019"));
        assert!(!html.contains("GeoEstabil Platform"), "Stale brand name in HTML report");
        assert!(html.contains("<span class=\"frac\">"), "Expected stacked HTML fraction equations");
        assert_eq!(
            html.matches("<svg ").count(),
            2,
            "HTML memorial must embed both Tab 2 and Tab 3 SVG diagrams even when slope_report is None"
        );
    }
}

#[test]
fn test_memorial_pdf_html_md_and_svg_export_integrity() {
    use geoestabil_rs::core::project::{ActiveProjectMode, GeoProject};
    use geoestabil_rs::engine::slope::grid_search::run_slope_stability_analysis;
    use geoestabil_rs::reports::MemorialGenerator;
    use geoestabil_rs::ui::dialogs::doc_renderer::{parse_doc_blocks, DocBlock};
    use geoestabil_rs::utils::export_dxf_svg::VectorExporter;

    let mut proj = GeoProject {
        title: "Encosta & Muro <Teste>".to_string(),
        ..Default::default()
    };
    let mut slope_model = proj.slope_model.clone();
    slope_model.grid_x_steps = 8;
    slope_model.grid_y_steps = 8;
    slope_model.radius_steps = 8;
    slope_model.num_slices = 20;
    let slope_rep = run_slope_stability_analysis(&slope_model);

    let tmp_dir = std::env::temp_dir().join(format!("geoestabil_export_test_{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let forbidden_raw_tokens = [
        "Ea_h", "Ea_solo", "Ea_soil", "Ea_q", "FS_tomb", "FS_desl", "FS_ovt", "FS_sld",
        "sigma_toe", "sigma_heel", "sigma_adm", "sigma_max", "V_Rd1", "A_s,req", "A_s,min",
        "A_s,dist", "gamma_sat", "gamma_f", "c_nom", "GeoEstabil Platform",
    ];

    for lang in [Language::PtBr, Language::EnUs] {
        let tag = if lang == Language::PtBr { "pt" } else { "en" };

        // 1. PDF bytes direct generation + rich fonts + vector equations & both Tab 2/3 figures
        let pdf_bytes = MemorialGenerator::generate_pdf_lang(&proj, Some(&slope_rep), lang);
        let pdf_text = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_bytes.starts_with(b"%PDF-1.4\n"));
        assert!(pdf_text.contains("\nxref\n"));
        assert!(pdf_text.contains("\nstartxref\n"));
        assert!(pdf_text.trim_end().ends_with("%%EOF"));
        assert!(pdf_text.contains("/BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding"));
        assert!(pdf_text.contains("/BaseFont /Symbol"), "PDF missing Base-14 /Symbol Greek font");
        assert!(pdf_text.contains("/BaseFont /Helvetica-Oblique /Encoding /WinAnsiEncoding"));
        assert!(pdf_text.contains(" Ts\n") && pdf_text.contains("/F3 "), "PDF must emit Ts shifts and /F3 Symbol runs");
        assert!(!pdf_text.contains("sigma_toe") && !pdf_text.contains("gamma_sat") && !pdf_text.contains("V_Rd1"));
        assert!(!pdf_text.contains("GeoEstabil Platform"));

        // 2. write_html_or_pdf dispatches .pdf -> PDF binary, .html -> HTML5, and extensionless -> .pdf
        let pdf_out = MemorialGenerator::write_html_or_pdf(
            &tmp_dir.join(format!("mem_{tag}.pdf")),
            &proj,
            Some(&slope_rep),
            lang,
        )
        .unwrap();
        assert_eq!(pdf_out.extension().and_then(|e| e.to_str()), Some("pdf"));
        assert!(std::fs::read(&pdf_out).unwrap().starts_with(b"%PDF-1.4"));

        let no_ext_out = MemorialGenerator::write_html_or_pdf(
            &tmp_dir.join(format!("mem_noext_{tag}")),
            &proj,
            Some(&slope_rep),
            lang,
        )
        .unwrap();
        assert_eq!(no_ext_out.extension().and_then(|e| e.to_str()), Some("pdf"));
        assert!(std::fs::read(&no_ext_out).unwrap().starts_with(b"%PDF-1.4"));

        let html_out = MemorialGenerator::write_html_or_pdf(
            &tmp_dir.join(format!("mem_{tag}.html")),
            &proj,
            Some(&slope_rep),
            lang,
        )
        .unwrap();
        assert_eq!(html_out.extension().and_then(|e| e.to_str()), Some("html"));
        let html_str = std::fs::read_to_string(&html_out).unwrap();
        assert!(html_str.contains("<!DOCTYPE html>"));
        assert_eq!(html_str.matches("<svg ").count(), 2, "HTML file must contain both Tab 2 and Tab 3 SVGs");
        assert!(html_str.contains("Heatmap FS") || html_str.contains("FS Heatmap"));
        assert!(html_str.contains("<tspan") && html_str.contains("a,h</tspan>"));
        for raw in forbidden_raw_tokens {
            assert!(!html_str.contains(raw), "Unformatted token `{}` leaked in HTML ({tag})", raw);
        }

        // 3. Preview Markdown + write_markdown (.md file embeds both SVGs & resolves LaTeX blocks)
        let preview_md = MemorialGenerator::generate_markdown_lang(&proj, Some(&slope_rep), lang);
        let blocks = parse_doc_blocks(&preview_md);
        assert!(blocks.iter().any(|b| matches!(b, DocBlock::Equation(_))), "Markdown preview missing $$ equations");
        assert!(blocks.iter().any(|b| matches!(b, DocBlock::Diagram(d) if d == "SLOPE_TAB2")));
        assert!(blocks.iter().any(|b| matches!(b, DocBlock::Diagram(d) if d == "WALL_TAB3")));

        let md_out = MemorialGenerator::write_markdown(
            &tmp_dir.join(format!("mem_{tag}")),
            &proj,
            Some(&slope_rep),
            lang,
        )
        .unwrap();
        assert_eq!(md_out.extension().and_then(|e| e.to_str()), Some("md"));
        let md_str = std::fs::read_to_string(&md_out).unwrap();
        assert!(md_str.contains("ABNT NBR 11682:2009"));
        assert_eq!(md_str.matches("<svg ").count(), 2, "Exported .md file must embed both Tab 2 and Tab 3 SVG diagrams");
        assert!(!md_str.contains("[[DIAGRAM:"), "Exported .md must replace diagram placeholders");
        assert!(!md_str.contains("GeoEstabil Platform"));

        // 4. VectorExporter complete dual-panel SVG (Tab 2 Slope+Heatmap+Slices AND Tab 3 Wall+Thrusts+Stress)
        proj.active_mode = ActiveProjectMode::SlopeStability;
        let slope_svg = VectorExporter::export_active_svg_lang(&proj, Some(&slope_rep), lang);
        assert!(slope_svg.contains("<svg") && slope_svg.contains("&lt;Teste&gt;") && slope_svg.contains("<circle"));
        assert!(slope_svg.contains("Heatmap FS") || slope_svg.contains("FS Heatmap"));

        proj.active_mode = ActiveProjectMode::RetainingWall;
        let wall_svg = VectorExporter::export_active_svg_lang(&proj, Some(&slope_rep), lang);
        assert!(wall_svg.contains("<svg") && wall_svg.contains("<polygon") && wall_svg.contains("a,h</tspan>"));
        for raw in ["Ea_h", "Ea_solo", "FS_tomb", "FS_desl", "x_R", "GeoEstabil Platform"] {
            assert!(!wall_svg.contains(raw), "Unformatted raw token `{}` leaked in SVG", raw);
        }
    }

    // 5. GeoProject round-trip .geoestabil save & load
    let proj_file = tmp_dir.join("roundtrip.geoestabil");
    proj.save_to_file(&proj_file).unwrap();
    assert!(!proj.is_dirty);
    let reloaded = GeoProject::load_from_file(&proj_file).unwrap();
    assert_eq!(reloaded.title, proj.title);

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_portable_assets_and_store_icons_present() {
    assert!(carregar_icone_janela().is_some(), "Window icon must decode cleanly");
    for icon_name in [
        "geoestabil.ico",
        "geoestabil.png",
        "geoestabil_71-71.png",
        "geoestabil_150-150.png",
        "geoestabil_300-300.png",
        "geoestabil_584-800.png",
        "geoestabil_1080-1080.png",
        "geoestabil_1440-2160.png",
        "geoestabil_1919-1079.png",
        "geoestabil_1920-1080.png",
        "geoestabil_2160-2160.png",
        "geoestabil_3840-2160.png",
    ] {
        assert!(
            obter_caminho_icone(icon_name).is_some(),
            "Missing Store icon asset: {}",
            icon_name
        );
    }

    for rel_doc in [
        "ABOUT/ABOUT_pt_BR.txt",
        "ABOUT/ABOUT_en_US.txt",
        "CLC/CLC_pt_BR - GeoEstabil.txt",
        "CLC/CLC_en_US - GeoEstabil.txt",
        "EULA/EULA_pt_BR - GeoEstabil.txt",
        "EULA/EULA_en_US - GeoEstabil.txt",
        "PRIVACY_POLICY/Privacy_Policy_pt_BR.txt",
        "PRIVACY_POLICY/Privacy_Policy_en_US.txt",
        "NOTICES/NOTICE_pt_BR.txt",
        "NOTICES/NOTICE_en_US.txt",
        "RELEASE/RELEASE NOTES_pt_BR.txt",
        "RELEASE/RELEASE NOTES_en_US.txt",
        "COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt",
        "COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt",
        "LICENSES/GNU AFFERO GENERAL PUBLIC LICENSE Version3.txt",
    ] {
        let path = resolver_caminho_asset(rel_doc);
        assert!(path.is_some(), "Missing asset doc: {}", rel_doc);
    }
}

#[test]
fn test_dual_licensing_and_microsoft_compliance_invariants() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 1. Validate Local-First Privacy, Email-Only Support (no urgency) & Dual-Licensing clauses across core assets
    let privacy_pt = std::fs::read_to_string(manifest_dir.join("src/assets/PRIVACY_POLICY/Privacy_Policy_pt_BR.txt"))
        .expect("Failed to read Privacy_Policy_pt_BR.txt");
    let privacy_en = std::fs::read_to_string(manifest_dir.join("src/assets/PRIVACY_POLICY/Privacy_Policy_en_US.txt"))
        .expect("Failed to read Privacy_Policy_en_US.txt");
    for privacy in [&privacy_pt, &privacy_en] {
        assert!(privacy.contains("https://www.microsoft.com/pt-br/privacy/privacystatement"));
        assert!(privacy.contains("linceu_lighthouse@outlook.com"));
        assert!(privacy.contains("geoestabil.db"));
    }

    let eula_pt = std::fs::read_to_string(manifest_dir.join("src/assets/EULA/EULA_pt_BR - GeoEstabil.txt"))
        .expect("Failed to read EULA_pt_BR");
    let eula_en = std::fs::read_to_string(manifest_dir.join("src/assets/EULA/EULA_en_US - GeoEstabil.txt"))
        .expect("Failed to read EULA_en_US");
    let clc_pt = std::fs::read_to_string(manifest_dir.join("src/assets/CLC/CLC_pt_BR - GeoEstabil.txt"))
        .expect("Failed to read CLC_pt_BR");
    let clc_en = std::fs::read_to_string(manifest_dir.join("src/assets/CLC/CLC_en_US - GeoEstabil.txt"))
        .expect("Failed to read CLC_en_US");

    for commercial_doc in [&eula_pt, &eula_en, &clc_pt, &clc_en] {
        assert!(commercial_doc.contains("AGPLv3"));
        assert!(commercial_doc.contains("linceu_lighthouse@outlook.com"));
        assert!(commercial_doc.contains("https://www.microsoft.com/pt-br/privacy/privacystatement"));
        assert!(!commercial_doc.contains("SALT"));
    }
    assert!(eula_pt.to_lowercase().contains("disponibilidade") && eula_pt.to_lowercase().contains("urgência"));
    assert!(eula_en.to_lowercase().contains("availability") && eula_en.to_lowercase().contains("urgency"));
    assert!(clc_pt.to_lowercase().contains("disponibilidade") && clc_pt.to_lowercase().contains("urgente"));
    assert!(clc_en.to_lowercase().contains("availability") && clc_en.to_lowercase().contains("urgent"));

    // 1b. Verify all 7 src/assets legal/documentation directories declare academic-study purpose,
    // potential errors, non-definitive production warning, manual + established market-tool check,
    // email-only support (no urgency), and liability exemptions
    for (rel, is_pt) in [
        ("src/assets/ABOUT/ABOUT_pt_BR.txt", true),
        ("src/assets/ABOUT/ABOUT_en_US.txt", false),
        ("src/assets/ABOUT/History_APP_pt_BR.txt", true),
        ("src/assets/ABOUT/History_APP_en_US.txt", false),
        ("src/assets/CLC/CLC_pt_BR - GeoEstabil.txt", true),
        ("src/assets/CLC/CLC_en_US - GeoEstabil.txt", false),
        ("src/assets/COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt", true),
        ("src/assets/COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt", false),
        ("src/assets/EULA/EULA_pt_BR - GeoEstabil.txt", true),
        ("src/assets/EULA/EULA_en_US - GeoEstabil.txt", false),
        ("src/assets/NOTICES/NOTICE_pt_BR.txt", true),
        ("src/assets/NOTICES/NOTICE_en_US.txt", false),
        ("src/assets/PRIVACY_POLICY/Privacy_Policy_pt_BR.txt", true),
        ("src/assets/PRIVACY_POLICY/Privacy_Policy_en_US.txt", false),
        ("src/assets/RELEASE/RELEASE NOTES_pt_BR.txt", true),
        ("src/assets/RELEASE/RELEASE NOTES_en_US.txt", false),
    ] {
        let txt = std::fs::read_to_string(manifest_dir.join(rel))
            .unwrap_or_else(|e| panic!("Missing asset {}: {}", rel, e));
        let lower = txt.to_lowercase();
        assert!(txt.contains("linceu_lighthouse@outlook.com"), "Missing official support email in {rel}");
        if is_pt {
            assert!(lower.contains("estudo") && lower.contains("acadêmic"), "Missing academic study clause in {rel}");
            assert!(lower.contains("produção") && lower.contains("consolidad"), "Missing production/market verification clause in {rel}");
            assert!(lower.contains("manual"), "Missing manual verification clause in {rel}");
            assert!(lower.contains("as-is") || lower.contains("as is") || lower.contains("agplv3"), "Missing legal disclaimer clause in {rel}");
            assert!(lower.contains("disponibilidade") && (lower.contains("urgên") || lower.contains("urgen")), "Missing availability/no-urgency support notice in {rel}");
        } else {
            assert!(lower.contains("academic") && lower.contains("study"), "Missing academic study clause in {rel}");
            assert!(lower.contains("production") && lower.contains("established"), "Missing production/market verification clause in {rel}");
            assert!(lower.contains("manual"), "Missing manual verification clause in {rel}");
            assert!(lower.contains("as-is") || lower.contains("as is") || lower.contains("agplv3"), "Missing legal disclaimer clause in {rel}");
            assert!(lower.contains("availability") && lower.contains("urgen"), "Missing availability/no-urgency support notice in {rel}");
        }
    }

    // 2. Validate dedicated docs exist, removed complex docs stay absent, and zero unrelated template names remain
    assert!(!manifest_dir.join("docs/CLA.md").exists(), "Removed complex CLA doc must not reappear");
    assert!(!manifest_dir.join("docs/microsoft-store-msix.md").exists(), "Removed complex MSIX guide must not reappear");
    for doc_rel in [
        "docs/licensing.md",
        "docs/security-model.md",
        "docs/compliance.md",
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/ISSUE_TEMPLATE/bug_report.md",
        ".github/ISSUE_TEMPLATE/config.yml",
        "repo/GeoEstabil.iss",
    ] {
        let content = std::fs::read_to_string(manifest_dir.join(doc_rel))
            .unwrap_or_else(|e| panic!("Missing doc {}: {}", doc_rel, e));
        assert!(!content.contains("Huginn"), "Stale reference in {}", doc_rel);
        assert!(!content.contains("EoloPlatform"), "Stale reference in {}", doc_rel);
        assert!(!content.contains("eolo.exe"), "Stale reference in {}", doc_rel);
        assert!(!content.contains("48 horas") && !content.contains("48-hour"), "Stale SLA promise in {}", doc_rel);
    }
}

#[test]
fn test_bilingual_documentation_exact_line_count_parity() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 1. All 8 paired txt files in src/assets must have identical line count and aligned paragraph breaks
    for (pt_rel, en_rel) in [
        ("src/assets/ABOUT/ABOUT_pt_BR.txt", "src/assets/ABOUT/ABOUT_en_US.txt"),
        ("src/assets/ABOUT/History_APP_pt_BR.txt", "src/assets/ABOUT/History_APP_en_US.txt"),
        ("src/assets/CLC/CLC_pt_BR - GeoEstabil.txt", "src/assets/CLC/CLC_en_US - GeoEstabil.txt"),
        (
            "src/assets/COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt",
            "src/assets/COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt",
        ),
        ("src/assets/EULA/EULA_pt_BR - GeoEstabil.txt", "src/assets/EULA/EULA_en_US - GeoEstabil.txt"),
        ("src/assets/NOTICES/NOTICE_pt_BR.txt", "src/assets/NOTICES/NOTICE_en_US.txt"),
        ("src/assets/PRIVACY_POLICY/Privacy_Policy_pt_BR.txt", "src/assets/PRIVACY_POLICY/Privacy_Policy_en_US.txt"),
        ("src/assets/RELEASE/RELEASE NOTES_pt_BR.txt", "src/assets/RELEASE/RELEASE NOTES_en_US.txt"),
    ] {
        let pt_text = std::fs::read_to_string(manifest_dir.join(pt_rel)).unwrap();
        let en_text = std::fs::read_to_string(manifest_dir.join(en_rel)).unwrap();
        let pt_lines: Vec<&str> = pt_text.lines().collect();
        let en_lines: Vec<&str> = en_text.lines().collect();
        assert_eq!(
            pt_lines.len(),
            en_lines.len(),
            "Line count mismatch between {} ({}) and {} ({})",
            pt_rel,
            pt_lines.len(),
            en_rel,
            en_lines.len()
        );
        for (idx, (p, e)) in pt_lines.iter().zip(en_lines.iter()).enumerate() {
            assert_eq!(
                p.trim().is_empty(),
                e.trim().is_empty(),
                "Blank line alignment mismatch at line {} between {} and {}",
                idx + 1,
                pt_rel,
                en_rel
            );
        }
    }

    // 2. All bilingual Markdown docs with PT-BR / EN-US <details> blocks must match line-for-line
    for md_rel in [
        "README.md",
        "CONTRIBUTING.md",
        "CODE_OF_CONDUCT.md",
        "SECURITY.md",
        "MANUAL.md",
        "OBSERVACAO.md",
        "docs/architecture.md",
        "docs/cli-reference.md",
        "docs/compliance.md",
        "docs/licensing.md",
        "docs/MIGRATION_REPORT.md",
        "docs/release-v2026.11.12.0.md",
        "docs/security-model.md",
        "docs/suggested-issues.md",
        "docs/user-guide.md",
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/ISSUE_TEMPLATE/bug_report.md",
        ".github/ISSUE_TEMPLATE/feature_request.md",
        ".github/ISSUE_TEMPLATE/question.md",
    ] {
        let text = std::fs::read_to_string(manifest_dir.join(md_rel)).unwrap();
        let lines: Vec<&str> = text.lines().collect();

        let extract_details = |start_after_idx: usize| -> Vec<&str> {
            let mut started = false;
            let mut out = Vec::new();
            for &l in &lines[start_after_idx..] {
                if !started {
                    if l.trim_start().starts_with("<details") {
                        started = true;
                    }
                } else {
                    if l.trim() == "</details>" {
                        break;
                    }
                    if !l.trim_start().starts_with("<summary>") {
                        out.push(l);
                    }
                }
            }
            out
        };

        let pt_pos = lines.iter().position(|l| l.contains("<details")).unwrap_or(0);
        let pt_block = extract_details(pt_pos);
        // Find second <details block
        let en_pos = lines[pt_pos + 1..]
            .iter()
            .position(|l| l.contains("<details"))
            .map(|rel| pt_pos + 1 + rel)
            .unwrap_or_else(|| panic!("Missing second <details> EN block in {}", md_rel));
        let en_block = extract_details(en_pos);

        assert_eq!(
            pt_block.len(),
            en_block.len(),
            "PT-BR ({}) vs EN-US ({}) line count mismatch in {}",
            pt_block.len(),
            en_block.len(),
            md_rel
        );
        for (idx, (p, e)) in pt_block.iter().zip(en_block.iter()).enumerate() {
            assert_eq!(
                p.trim().is_empty(),
                e.trim().is_empty(),
                "PT-BR vs EN-US blank line mismatch inside {} at block line {}",
                md_rel,
                idx + 1
            );
        }
    }

    // 3. README.md must embed localized screenshots from repo/LISTAGEM/pt_BR & repo/LISTAGEM/en_US
    let readme = std::fs::read_to_string(manifest_dir.join("README.md")).unwrap();
    let en_anchor = readme.find("## <a id=\"enus\"></a>").expect("Missing EN anchor in README.md");
    let (pt_section, en_section) = readme.split_at(en_anchor);

    for img in [
        "Captura de tela 2026-11-11 002316.png",
        "Captura de tela 2026-11-11 002325.png",
        "Captura de tela 2026-11-11 002335.png",
        "Captura de tela 2026-11-11 002344.png",
        "Captura de tela 2026-11-11 002354.png",
    ] {
        let disk_path = manifest_dir.join("repo").join("LISTAGEM").join("pt_BR").join(img);
        assert!(disk_path.is_file(), "Missing screenshot {}", disk_path.display());
        let encoded = format!("repo/LISTAGEM/pt_BR/{}", img.replace(' ', "%20"));
        assert!(pt_section.contains(&encoded), "PT section missing {}", encoded);
        assert!(!en_section.contains(&encoded), "EN section must not contain PT screenshot {}", encoded);
    }

    for img in [
        "Captura de tela 2026-11-11 002418.png",
        "Captura de tela 2026-11-11 002447.png",
        "Captura de tela 2026-11-11 002513.png",
        "Captura de tela 2026-11-11 002525.png",
        "Captura de tela 2026-11-11 002540.png",
    ] {
        let disk_path = manifest_dir.join("repo").join("LISTAGEM").join("en_US").join(img);
        assert!(disk_path.is_file(), "Missing screenshot {}", disk_path.display());
        let encoded = format!("repo/LISTAGEM/en_US/{}", img.replace(' ', "%20"));
        assert!(en_section.contains(&encoded), "EN section missing {}", encoded);
        assert!(!pt_section.contains(&encoded), "PT section must not contain EN screenshot {}", encoded);
    }
}
