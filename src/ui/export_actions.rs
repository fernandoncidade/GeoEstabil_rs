use crate::core::project::GeoProject;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::locale::translations::Localizer;
use crate::reports::memorial_generator::MemorialGenerator;
use crate::utils::export_dxf_svg::VectorExporter;
use std::path::PathBuf;

pub fn open_project_dialog(localizer: &Localizer) -> Result<Option<GeoProject>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter(
            localizer.pick("Projeto GeoEstabil", "GeoEstabil Project"),
            &["geoestabil", "json"],
        )
        .pick_file()
    else {
        return Ok(None);
    };
    GeoProject::load_from_file(path).map(Some)
}

pub fn save_project_dialog(
    project: &mut GeoProject,
    localizer: &Localizer,
    force_save_as: bool,
) -> Result<Option<PathBuf>, String> {
    if !force_save_as {
        if let Some(ref existing) = project.file_path.clone() {
            project.save_to_file(existing)?;
            return Ok(Some(existing.clone()));
        }
    }

    let default_name = localizer.pick(
        "Projeto_GeoEstabil.geoestabil",
        "GeoEstabil_Project.geoestabil",
    );
    let Some(mut path) = rfd::FileDialog::new()
        .set_file_name(default_name)
        .add_filter(
            localizer.pick("Projeto GeoEstabil", "GeoEstabil Project"),
            &["geoestabil", "json"],
        )
        .save_file()
    else {
        return Ok(None);
    };

    if path.extension().is_none() {
        path.set_extension("geoestabil");
    }
    project.save_to_file(&path)?;
    Ok(Some(path))
}

pub fn export_memorial_html_or_pdf_dialog(
    project: &GeoProject,
    slope_report: Option<&SlopeAnalysisReport>,
    localizer: &Localizer,
) -> Result<Option<PathBuf>, String> {
    let stem = localizer.pick(
        "Memorial_Calculo_GeoEstabil",
        "Calculation_Memorial_GeoEstabil",
    );
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(stem)
        .add_filter(
            localizer.pick("Documento PDF (A4)", "PDF Document (A4)"),
            &["pdf"],
        )
        .add_filter(
            localizer.pick("Documento HTML5", "HTML5 Document"),
            &["html", "htm"],
        )
        .add_filter(
            localizer.pick("Memorial PDF ou HTML5", "PDF or HTML5 Memorial"),
            &["pdf", "html", "htm"],
        )
        .save_file()
    else {
        return Ok(None);
    };

    let written = MemorialGenerator::write_html_or_pdf(&path, project, slope_report, localizer.language)?;
    Ok(Some(written))
}

pub fn export_memorial_markdown_dialog(
    project: &GeoProject,
    slope_report: Option<&SlopeAnalysisReport>,
    localizer: &Localizer,
) -> Result<Option<PathBuf>, String> {
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(localizer.pick(
            "Memorial_Calculo_GeoEstabil.md",
            "Calculation_Memorial_GeoEstabil.md",
        ))
        .add_filter(
            localizer.pick("Documento Markdown", "Markdown Document"),
            &["md"],
        )
        .save_file()
    else {
        return Ok(None);
    };

    let written = MemorialGenerator::write_markdown(&path, project, slope_report, localizer.language)?;
    Ok(Some(written))
}

pub fn export_profile_svg_dialog(
    project: &GeoProject,
    slope_report: Option<&SlopeAnalysisReport>,
    localizer: &Localizer,
) -> Result<Option<PathBuf>, String> {
    let default_name = localizer.pick(
        "Prancha_Diagramas_Abas2_e_3_GeoEstabil.svg",
        "Slope_and_Wall_Diagrams_GeoEstabil.svg",
    );
    let Some(mut path) = rfd::FileDialog::new()
        .set_file_name(default_name)
        .add_filter("Scalable Vector Graphics", &["svg"])
        .save_file()
    else {
        return Ok(None);
    };

    if path.extension().is_none() {
        path.set_extension("svg");
    }
    let svg_content = VectorExporter::export_active_svg_lang(project, slope_report, localizer.language);
    std::fs::write(&path, svg_content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(Some(path))
}
