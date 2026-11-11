use geoestabil_rs::core::presets::get_preset_cases;
use geoestabil_rs::core::project::GeoProject;
use geoestabil_rs::engine::slope::grid_search::run_slope_stability_analysis;
use geoestabil_rs::engine::soils::get_standard_nbr6122_soils;
use geoestabil_rs::engine::wall::bearing_capacity::calculate_bearing_capacity;
use geoestabil_rs::engine::wall::concrete_design::design_concrete_reinforcement;
use geoestabil_rs::engine::wall::earth_pressure::calculate_earth_pressures;
use geoestabil_rs::engine::wall::stability::check_wall_external_stability;
use geoestabil_rs::locale::translations::Language;
use geoestabil_rs::reports::memorial_generator::MemorialGenerator;
use geoestabil_rs::utils::export_dxf_svg::VectorExporter;
use std::env;
use std::path::Path;

fn parse_cli_language(args: &[String]) -> Language {
    if let Some(pos) = args.iter().position(|a| a == "--lang" || a == "-l") {
        if let Some(val) = args.get(pos + 1) {
            if let Some(lang) = Language::from_config_code(val) {
                return lang;
            }
        }
    }
    geoestabil_rs::utils::paths::load_saved_language_code()
        .as_deref()
        .and_then(Language::from_config_code)
        .unwrap_or(Language::PtBr)
}

fn write_cli_report(
    out_path: &str,
    project: &GeoProject,
    slope_report: Option<&geoestabil_rs::engine::slope::results::SlopeAnalysisReport>,
    lang: Language,
) {
    let p = Path::new(out_path);
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("html")
        .to_ascii_lowercase();
    let res = match ext.as_str() {
        "pdf" => std::fs::write(p, MemorialGenerator::generate_pdf_lang(project, slope_report, lang)),
        "md" => std::fs::write(
            p,
            MemorialGenerator::generate_markdown_with_diagrams_lang(project, slope_report, lang).as_bytes(),
        ),
        "svg" => std::fs::write(p, VectorExporter::export_active_svg_lang(project, slope_report, lang).as_bytes()),
        _ => std::fs::write(p, MemorialGenerator::generate_html_lang(project, slope_report, lang).as_bytes()),
    };
    let pt = lang == Language::PtBr;
    match res {
        Ok(()) => println!(
            "{}: {}",
            if pt { "Relatório salvo em" } else { "Report saved to" },
            out_path
        ),
        Err(e) => eprintln!(
            "{} '{}': {}",
            if pt { "Erro ao salvar relatório" } else { "Failed to save report" },
            out_path,
            e
        ),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let lang = parse_cli_language(&args);
    let pt = lang == Language::PtBr;

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" || args[1] == "help" {
        print_help(lang);
        return;
    }

    match args[1].as_str() {
        "--version" | "-v" | "version" => {
            println!("GeoEstabil CLI v{}", env!("CARGO_PKG_VERSION"));
            println!(
                "{}",
                if pt {
                    "Ferramenta Auxiliar de Estudo — Estabilidade de Taludes e Muros de Arrimo"
                } else {
                    "Auxiliary Study Tool — Slope Stability & Retaining Wall Analysis"
                }
            );
            println!(
                "{}",
                if pt {
                    "Referências: ABNT NBR 11682:2009, NBR 6122:2022 e NBR 6118:2023 (Uso acadêmico — AS IS)"
                } else {
                    "References: ABNT NBR 11682:2009, NBR 6122:2022 & NBR 6118:2023 (Academic study — AS IS)"
                }
            );
            println!("Suporte / Support (e-mail apenas / email only, sem urgência / no urgency): Fernando Nillsson Cidade <linceu_lighthouse@outlook.com>");
        }
        "presets" => {
            println!("================================================================================");
            println!(
                "{}",
                if pt {
                    "Casos Pré-configurados de Engenharia (GeoEstabil Presets):"
                } else {
                    "Pre-configured Engineering Benchmark Cases (GeoEstabil Presets):"
                }
            );
            println!("================================================================================");
            let cases = get_preset_cases();
            for (idx, c) in cases.iter().enumerate() {
                println!("{}. {}", idx + 1, c.localized_name(lang));
                println!(
                    "   {}: {}",
                    if pt { "Descrição" } else { "Description" },
                    c.localized_description(lang)
                );
                println!("   {}: {}", if pt { "Modo" } else { "Mode" }, c.mode.name(lang));
                println!("--------------------------------------------------------------------------------");
            }
        }
        "run-preset" => {
            if args.len() < 3 {
                eprintln!(
                    "{}",
                    if pt {
                        "Erro: Especifique o índice do preset (1 a 4). Ex: geoestabil-cli run-preset 1"
                    } else {
                        "Error: Specify preset index (1 to 4). Ex: geoestabil-cli run-preset 1 --lang en"
                    }
                );
                return;
            }
            let idx: usize = args[2].parse().unwrap_or(1);
            let mut cases = get_preset_cases();
            if idx == 0 || idx > cases.len() {
                eprintln!(
                    "{} 1..={}.",
                    if pt { "Erro: Índice inválido. Escolha de" } else { "Error: Invalid preset index. Choose" },
                    cases.len()
                );
                return;
            }

            let case = cases.swap_remove(idx - 1);
            let case_mode = case.mode;
            println!(
                "{} {}: {}",
                if pt { "Executando Preset" } else { "Running Preset" },
                idx,
                case.localized_name(lang)
            );

            let project = case.project_for_lang(lang);
            let mut slope_report = None;

            if matches!(case_mode, geoestabil_rs::core::project::ActiveProjectMode::SlopeStability) {
                println!(
                    "{}",
                    if pt {
                        "Iniciando varredura multithreaded de superfícies de ruptura com Rayon..."
                    } else {
                        "Running multithreaded circular slip surface grid search with Rayon..."
                    }
                );
                let report = run_slope_stability_analysis(&project.slope_model);
                println!("{}", if pt { "Varredura concluída!" } else { "Sweep completed!" });
                println!("- {}: {}", if pt { "Círculos avaliados" } else { "Evaluated circles" }, report.total_circles_evaluated);
                println!("- {}: {:.2} ms", if pt { "Tempo decorrido" } else { "Elapsed time" }, report.elapsed_ms);
                println!("- {}: X_c={:.2}m, Y_c={:.2}m, R={:.2}m", if pt { "Círculo crítico" } else { "Critical circle" }, report.critical_circle.center_x, report.critical_circle.center_y, report.critical_circle.radius);
                println!(
                    "- {}: {:.3} ({} ≥ {:.2})",
                    if pt { "Fator de Segurança (Bishop)" } else { "Factor of Safety (Bishop)" },
                    report.critical_circle.fs_bishop,
                    if pt { "Exigido" } else { "Required" },
                    report.required_min_fs
                );
                println!(
                    "- Status: {}",
                    match (report.is_stable, pt) {
                        (true, true) => "ESTÁVEL (REF. CRIT.)",
                        (false, true) => "ABAIXO DA REF. / INSTÁVEL",
                        (true, false) => "STABLE (REF. CHECK)",
                        (false, false) => "BELOW REF. / UNSTABLE",
                    }
                );
                slope_report = Some(report);
            } else {
                println!(
                    "{}",
                    if pt {
                        "Calculando Muro de Contenção e Fundações..."
                    } else {
                        "Calculating Retaining Wall & Foundations..."
                    }
                );
                let pressures = calculate_earth_pressures(&project.wall_model);
                let stability = check_wall_external_stability(&project.wall_model);
                let bearing = calculate_bearing_capacity(&project.wall_model, &stability);
                let reinf = design_concrete_reinforcement(&project.wall_model, &stability);

                println!("- E_a,h: {:.2} kN/m", pressures.ea_horizontal);
                println!(
                    "- {} = {:.2} (≥ 1.50) → {}",
                    if pt { "FS_tomb" } else { "FS_ovt" },
                    stability.fs_overturning,
                    if stability.is_overturning_ok { "OK" } else if pt { "FALHA" } else { "FAIL" }
                );
                println!(
                    "- {} = {:.2} (≥ 1.50) → {}",
                    if pt { "FS_desl" } else { "FS_sld" },
                    stability.fs_sliding,
                    if stability.is_sliding_ok { "OK" } else if pt { "FALHA" } else { "FAIL" }
                );
                println!(
                    "- {}: e = {:.3} m ({} B/6 = {:.3} m)",
                    if pt { "Excentricidade na Base" } else { "Base Eccentricity" },
                    stability.eccentricity,
                    if pt { "Limite" } else { "Limit" },
                    stability.max_allowed_eccentricity
                );
                println!(
                    "- {}: σ = {:.1} kPa (σ_adm = {:.1} kPa)",
                    if pt { "Tensão Máx. Solo" } else { "Max Bearing Stress" },
                    stability.sigma_toe_kpa,
                    stability.sigma_adm_kpa
                );
                println!("- Hansen FS = {:.2} (≥ 2.00)", bearing.fs_bearing);
                println!(
                    "- {}: A_s,req = {:.2} cm²/m → {}",
                    if pt { "Armadura Fuste" } else { "Stem Rebar" },
                    reinf.stem_section.as_required_cm2_m,
                    reinf.stem_section.recommendation.localized_description(lang)
                );
            }

            if let Some(out_pos) = args.iter().position(|a| a == "--output" || a == "-o") {
                if out_pos + 1 < args.len() {
                    write_cli_report(&args[out_pos + 1], &project, slope_report.as_ref(), lang);
                }
            }
        }
        "soils" => {
            println!("==========================================================================================");
            println!(
                "{}",
                if pt {
                    "Catálogo de Solos Típicos Brasileiros (ABNT NBR 6122 / Correlações SPT):"
                } else {
                    "Catalog of Typical Soils (ABNT NBR 6122 / SPT Correlations):"
                }
            );
            println!("==========================================================================================");
            println!(
                "{:<32} | {:<12} | {:<6} | {:<6} | {:<8} | {:<8}",
                if pt { "Nome do Solo" } else { "Soil Name" },
                "SPT",
                "γ kN/m³",
                "φ' °",
                "c' kPa",
                "σ_adm kPa"
            );
            println!("------------------------------------------------------------------------------------------");
            for s in get_standard_nbr6122_soils() {
                println!(
                    "{:<32} | {:<12} | {:<6.1} | {:<6.1} | {:<8.1} | {:<8.1}",
                    s.localized_name(lang),
                    s.localized_spt_range(lang),
                    s.gamma,
                    s.friction_angle_deg,
                    s.cohesion,
                    s.sigma_adm
                );
            }
        }
        "slope" => {
            if let Some(in_pos) = args.iter().position(|a| a == "--input" || a == "-i") {
                if in_pos + 1 < args.len() {
                    let in_path = &args[in_pos + 1];
                    match GeoProject::load_from_file(in_path) {
                        Ok(mut project) => {
                            if !pt {
                                project.translate_defaults(Language::PtBr, lang);
                            }
                            println!("{}: {}", if pt { "Carregado projeto" } else { "Loaded project" }, project.title);
                            let report = run_slope_stability_analysis(&project.slope_model);
                            println!("FS Bishop: {:.3} | FS Fellenius: {:.3} | FS Janbu: {:.3}", report.critical_circle.fs_bishop, report.critical_circle.fs_fellenius, report.critical_circle.fs_janbu);
                            println!("{}: {:.2} ms | {}: {}", if pt { "Tempo" } else { "Elapsed" }, report.elapsed_ms, if pt { "Círculos" } else { "Circles" }, report.total_circles_evaluated);

                            if let Some(out_pos) = args.iter().position(|a| a == "--output" || a == "-o") {
                                if out_pos + 1 < args.len() {
                                    write_cli_report(&args[out_pos + 1], &project, Some(&report), lang);
                                }
                            }
                        }
                        Err(e) => eprintln!("{}: {}", if pt { "Erro ao carregar arquivo de projeto" } else { "Error loading project file" }, e),
                    }
                }
            } else {
                eprintln!("{}", if pt { "Erro: Parâmetro --input <arquivo.geoestabil> obrigatório." } else { "Error: Parameter --input <file.geoestabil> required." });
            }
        }
        "wall" => {
            if let Some(in_pos) = args.iter().position(|a| a == "--input" || a == "-i") {
                if in_pos + 1 < args.len() {
                    let in_path = &args[in_pos + 1];
                    match GeoProject::load_from_file(in_path) {
                        Ok(mut project) => {
                            if !pt {
                                project.translate_defaults(Language::PtBr, lang);
                            }
                            let stability = check_wall_external_stability(&project.wall_model);
                            let reinf = design_concrete_reinforcement(&project.wall_model, &stability);
                            println!(
                                "{}: {:.2} | {}: {:.2}",
                                if pt { "FS_tomb" } else { "FS_ovt" },
                                stability.fs_overturning,
                                if pt { "FS_desl" } else { "FS_sld" },
                                stability.fs_sliding
                            );
                            println!("{}: {}", if pt { "Armadura Fuste" } else { "Stem Rebar" }, reinf.stem_section.recommendation.localized_description(lang));

                            if let Some(out_pos) = args.iter().position(|a| a == "--output" || a == "-o") {
                                if out_pos + 1 < args.len() {
                                    write_cli_report(&args[out_pos + 1], &project, None, lang);
                                }
                            }
                        }
                        Err(e) => eprintln!("{}: {}", if pt { "Erro ao carregar projeto" } else { "Error loading project" }, e),
                    }
                }
            } else {
                eprintln!("{}", if pt { "Erro: Parâmetro --input <arquivo.geoestabil> obrigatório." } else { "Error: Parameter --input <file.geoestabil> required." });
            }
        }
        _ => {
            eprintln!("{}: '{}'", if pt { "Comando desconhecido" } else { "Unknown command" }, args[1]);
            print_help(lang);
        }
    }
}

fn print_help(lang: Language) {
    let pt = lang == Language::PtBr;
    println!("================================================================================");
    println!(
        "{}",
        if pt {
            "GeoEstabil CLI — Estabilidade de Taludes e Muros de Arrimo (ABNT)"
        } else {
            "GeoEstabil CLI — Slope Stability & Retaining Walls (ABNT Standards)"
        }
    );
    println!("================================================================================");
    if pt {
        println!("Uso: geoestabil-cli <comando> [opções]");
        println!();
        println!("Comandos disponíveis:");
        println!("  presets                               Lista os casos pré-configurados.");
        println!("  run-preset <1..4> [-o out] [--lang]   Executa caso de referência e exporta (.pdf/.html/.md/.svg).");
        println!("  slope -i proj.geoestabil [-o out]     Executa análise de talude via Rayon.");
        println!("  wall  -i proj.geoestabil [-o out]     Executa dimensionamento de muro e fundação.");
        println!("  soils [--lang pt|en]                  Exibe o catálogo de solos da NBR 6122.");
        println!("  version                               Exibe a versão e dados técnicos.");
        println!("  help                                  Exibe esta mensagem de ajuda.");
        println!();
        println!("Exemplos:");
        println!("  geoestabil-cli run-preset 1 --output memorial.pdf --lang pt");
        println!("  geoestabil-cli run-preset 1 --output report.pdf --lang en");
    } else {
        println!("Usage: geoestabil-cli <command> [options]");
        println!();
        println!("Available commands:");
        println!("  presets                               List pre-configured benchmark cases.");
        println!("  run-preset <1..4> [-o out] [--lang]   Run preset case and export (.pdf/.html/.md/.svg).");
        println!("  slope -i proj.geoestabil [-o out]     Run Rayon parallel slope stability analysis.");
        println!("  wall  -i proj.geoestabil [-o out]     Run retaining wall & RC structural design.");
        println!("  soils [--lang pt|en]                  Print NBR 6122 soil catalog.");
        println!("  version                               Print version and technical info.");
        println!("  help                                  Show this help message.");
        println!();
        println!("Examples:");
        println!("  geoestabil-cli run-preset 1 --output report.pdf --lang en");
        println!("  geoestabil-cli slope --input project.geoestabil --output report.html --lang en");
    }
    println!("================================================================================");
}
