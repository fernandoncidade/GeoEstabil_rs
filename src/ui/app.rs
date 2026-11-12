use eframe::App;
use egui::{CentralPanel, CornerRadius, Frame, Margin, Panel, RichText, Stroke, Ui};

use crate::core::project::{ActiveProjectMode, GeoProject};
use crate::data::database::GeoDatabase;
use crate::engine::slope::grid_search::run_slope_stability_analysis;
use crate::engine::slope::results::SlopeAnalysisReport;
use crate::locale::translations::{Language, Localizer};
use crate::ui::canvas_2d::CanvasState;
use crate::ui::dialogs::modeless::{
    ctx_zorder_raise, ctx_zorder_snapshot, ROOT_WINDOW_ID,
};
use crate::ui::dialogs::{AboutDialog, ManualDialog, SettingsDialog, SoilCatalogDialog};
use crate::ui::menu_bar::MenuBar;
use crate::ui::tabs::*;
use crate::ui::theme::GeoEstabilTheme;
use crate::ui::toolbar::ToolBar;
use crate::utils::paths::{get_database_path, save_language_code};
use std::path::Path;

pub struct GeoEstabilApp {
    pub project: GeoProject,
    pub active_tab: usize,
    pub slope_report: Option<SlopeAnalysisReport>,
    pub slope_canvas_state: CanvasState,
    pub wall_canvas_state: CanvasState,
    pub dark_mode: bool,
    pub localizer: Localizer,
    pub db: Option<GeoDatabase>,
    pub about_dialog: AboutDialog,
    pub manual_dialog: ManualDialog,
    pub soil_catalog_dialog: SoilCatalogDialog,
    pub settings_dialog: SettingsDialog,
}

impl Default for GeoEstabilApp {
    fn default() -> Self {
        Self::with_startup_file(None)
    }
}

impl GeoEstabilApp {
    /// Inicializa o aplicativo carregando o idioma salvo (`language_config.json`) e, opcionalmente, um arquivo `.geoestabil` passado via linha de comando.
    pub fn with_startup_file(startup_file: Option<&Path>) -> Self {
        let db = GeoDatabase::open_or_create(get_database_path())
            .or_else(|_| GeoDatabase::open_in_memory())
            .ok();

        let localizer = Localizer::from_saved_preference();
        let mut project = GeoProject::default();
        if localizer.language != Language::PtBr {
            project.translate_defaults(Language::PtBr, localizer.language);
        }

        let mut active_tab = 1; // Começa na aba de Estabilidade de Taludes
        if let Some(path) = startup_file {
            if let Ok(loaded) = GeoProject::load_from_file(path) {
                active_tab = match loaded.active_mode {
                    ActiveProjectMode::SlopeStability => 1,
                    ActiveProjectMode::RetainingWall => 2,
                };
                project = loaded;
            }
        }

        let initial_report = run_slope_stability_analysis(&project.slope_model);

        Self {
            project,
            active_tab,
            slope_report: Some(initial_report),
            slope_canvas_state: CanvasState::default(),
            wall_canvas_state: CanvasState::default(),
            dark_mode: true,
            localizer,
            db,
            about_dialog: AboutDialog::default(),
            manual_dialog: ManualDialog::default(),
            soil_catalog_dialog: SoilCatalogDialog::default(),
            settings_dialog: SettingsDialog::default(),
        }
    }
}

impl App for GeoEstabilApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.render_ui(ui);
    }
}

impl GeoEstabilApp {
    pub fn render_ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();

        // Atualiza a pilha de Z-Order quando a janela principal recebe clique ou foco OS
        let root_activated = ctx.input(|i| {
            i.raw.viewport_id == egui::ViewportId::ROOT
                && (i.pointer.any_pressed()
                    || i.events
                        .iter()
                        .any(|e| matches!(e, egui::Event::WindowFocused(true))))
        });
        if root_activated {
            ctx_zorder_raise(&ctx, ROOT_WINDOW_ID);
        }

        // 0. Atalhos globais para janelas Modeless de Documentação & Sobre
        let (open_manual, open_about) = ctx.input(|i| {
            let ctrl = i.modifiers.command || i.modifiers.ctrl;
            (
                i.key_pressed(egui::Key::F1) || (ctrl && i.key_pressed(egui::Key::M)),
                ctrl && i.key_pressed(egui::Key::I),
            )
        });
        if open_manual {
            self.manual_dialog.open();
        }
        if open_about {
            self.about_dialog.open();
        }

        // 1. Aplica o tema visual dinâmico em tempo de execução e sincroniza o root Ui
        GeoEstabilTheme::apply(&ctx, self.dark_mode);
        *ui.style_mut() = (*ctx.global_style()).clone();

        let dark_before_top = self.dark_mode;
        let lang_before_top = self.localizer.language;

        // 2. Painel Superior: Barra de Menus + Toolbar + Barra de Abas
        Panel::top("geoestabil_top_panel").show(ui, |ui| {
            MenuBar::show(
                ui,
                &mut self.project,
                &mut self.active_tab,
                &mut self.slope_report,
                &mut self.slope_canvas_state,
                &mut self.wall_canvas_state,
                &mut self.about_dialog,
                &mut self.manual_dialog,
                &mut self.soil_catalog_dialog,
                &mut self.settings_dialog,
                &mut self.localizer,
                &mut self.dark_mode,
            );

            ui.add_space(3.0);

            ToolBar::show(
                ui,
                &mut self.project,
                &mut self.active_tab,
                &mut self.slope_report,
                &mut self.slope_canvas_state,
                &mut self.wall_canvas_state,
                &self.localizer,
                &mut self.dark_mode,
            );

            ui.add_space(3.0);

            // Se o tema foi alternado no MenuBar ou ToolBar, sincroniza antes da Barra de Abas
            if self.dark_mode != dark_before_top {
                GeoEstabilTheme::apply(&ctx, self.dark_mode);
                *ui.style_mut() = (*ctx.global_style()).clone();
            }
            let dark_tabs = self.dark_mode;

            // Barra de Abas dentro de moldura fina
            let tab_frame = Frame::NONE
                .fill(ui.visuals().widgets.noninteractive.bg_fill)
                .stroke(Stroke::new(
                    1.0,
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                ))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(8, 4));

            tab_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    let tab_names = [
                        ("🌍", self.localizer.t("tab.project")),
                        ("⛰", self.localizer.t("tab.slope")),
                        ("🧱", self.localizer.t("tab.wall")),
                        ("📐", self.localizer.t("tab.structural")),
                        ("📄", self.localizer.t("tab.memorial")),
                    ];

                    for (idx, (icon, name)) in tab_names.iter().enumerate() {
                        let is_sel = self.active_tab == idx;
                        let text = if is_sel {
                            RichText::new(format!("{} {}", icon, name))
                                .size(13.0)
                                .color(GeoEstabilTheme::accent_blue(dark_tabs))
                        } else {
                            RichText::new(format!("{} {}", icon, name))
                                .size(13.0)
                                .color(GeoEstabilTheme::text_primary(dark_tabs))
                        };

                        if ui.selectable_label(is_sel, text).clicked() {
                            self.active_tab = idx;
                            if idx == 1 {
                                self.project.active_mode = ActiveProjectMode::SlopeStability;
                            } else if idx == 2 {
                                self.project.active_mode = ActiveProjectMode::RetainingWall;
                            }
                        }
                        if idx < 4 {
                            ui.separator();
                        }
                    }
                });
            });
            ui.add_space(2.0);
        });

        // Se o tema mudou no painel superior, propaga imediatamente para o root Ui e painéis seguintes
        if self.dark_mode != dark_before_top {
            GeoEstabilTheme::apply(&ctx, self.dark_mode);
            *ui.style_mut() = (*ctx.global_style()).clone();
        }
        let dark = self.dark_mode;
        let lang = self.localizer.language;

        // Atualiza dinamicamente o título da janela principal no viewport
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            self.localizer.t("app.title").to_string(),
        ));

        // 3. Painel Inferior: Barra de Status responsiva
        Panel::bottom("geoestabil_bottom_panel").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!("GeoEstabil v{}", env!("CARGO_PKG_VERSION"))).size(11.5),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        self.localizer.t("status.project"),
                        self.project.title
                    ))
                    .size(11.5),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        self.localizer.t("status.mode"),
                        self.project.active_mode.name(lang)
                    ))
                    .size(11.5),
                );
                ui.separator();
                if self.project.is_dirty {
                    ui.label(
                        RichText::new(format!("● {}", self.localizer.t("status.unsaved")))
                            .size(11.5)
                            .color(GeoEstabilTheme::warning_amber(dark)),
                    );
                } else {
                    ui.label(
                        RichText::new(format!("✔ {}", self.localizer.t("status.saved")))
                            .size(11.5)
                            .color(GeoEstabilTheme::success_green(dark)),
                    );
                }
                ui.separator();
                ui.label(
                    RichText::new("🔒 Offline • NBR 11682 / 6122 / 6118 ✓")
                        .size(11.5)
                        .color(GeoEstabilTheme::accent_blue(dark))
                        .strong(),
                );
                ui.separator();
                ui.label(RichText::new(self.localizer.t("status.windows")).size(11.2));
                let zorder = ctx_zorder_snapshot(&ctx);
                let top_id = zorder.top().unwrap_or(ROOT_WINDOW_ID);
                let pill_label = |icon: &str, text: &str, win_id: &str| {
                    let prefix = if top_id == win_id { "▲ " } else { "" };
                    RichText::new(format!("{prefix}{icon} {text}")).size(11.2)
                };

                if ui
                    .selectable_label(
                        self.manual_dialog.modeless.is_open,
                        pill_label(
                            "📖",
                            self.localizer.t("status.manual_pill"),
                            "geoestabil_user_manual",
                        ),
                    )
                    .clicked()
                {
                    self.manual_dialog.open();
                }
                if ui
                    .selectable_label(
                        self.about_dialog.modeless.is_open,
                        pill_label(
                            "ℹ",
                            self.localizer.t("status.about_pill"),
                            "geoestabil_about_dialog",
                        ),
                    )
                    .clicked()
                {
                    self.about_dialog.open();
                }
                if ui
                    .selectable_label(
                        self.soil_catalog_dialog.is_open(),
                        pill_label(
                            "🗄",
                            self.localizer.t("status.soils_pill"),
                            "geoestabil_soil_catalog",
                        ),
                    )
                    .clicked()
                {
                    self.soil_catalog_dialog.open();
                }
                if ui
                    .selectable_label(
                        self.settings_dialog.is_open(),
                        pill_label(
                            "⚙",
                            self.localizer.t("status.settings_pill"),
                            "geoestabil_settings_dialog",
                        ),
                    )
                    .clicked()
                {
                    self.settings_dialog.open();
                }
            });
        });

        // 4. Painel Central da Aba Ativa
        CentralPanel::default()
            .frame(Frame::central_panel(ui.style()).inner_margin(Margin::same(4)))
            .show(ui, |ui| match self.active_tab {
                0 => Tab1Geotechnical::show(ui, &mut self.project, &self.localizer, dark),
                1 => Tab2SlopeStability::show(
                    ui,
                    &mut self.project,
                    &mut self.slope_report,
                    &mut self.slope_canvas_state,
                    &self.localizer,
                    dark,
                ),
                2 => Tab3RetainingWall::show(
                    ui,
                    &mut self.project,
                    &mut self.wall_canvas_state,
                    &self.localizer,
                    dark,
                ),
                3 => Tab4StructuralDesign::show(ui, &mut self.project, &self.localizer, dark),
                4 => Tab5Memorial::show(
                    ui,
                    &mut self.project,
                    self.slope_report.as_ref(),
                    &self.localizer,
                    dark,
                ),
                _ => {}
            });

        // 5. Diálogos Modeless e Modais
        let dark_before_dialogs = self.dark_mode;
        let lang_before_dialogs = self.localizer.language;

        self.about_dialog
            .show(&ctx, &self.localizer, dark);
        self.manual_dialog
            .show(&ctx, &self.localizer, dark);
        self.soil_catalog_dialog
            .show(&ctx, &mut self.project, &self.localizer, dark);
        self.settings_dialog
            .show(&ctx, &mut self.dark_mode, &mut self.localizer);

        if self.localizer.language != lang_before_dialogs {
            self.project
                .translate_defaults(lang_before_dialogs, self.localizer.language);
        }

        if self.dark_mode != dark_before_dialogs {
            GeoEstabilTheme::apply(&ctx, self.dark_mode);
        }

        if self.db.is_some()
            && (self.localizer.language != lang_before_top
                || self.localizer.language != lang_before_dialogs)
        {
            save_language_code(self.localizer.language.code());
        }

        // Garante atualização imediata em todos os viewports caso tema ou idioma tenha mudado neste frame
        if self.dark_mode != dark_before_top
            || self.dark_mode != dark_before_dialogs
            || self.localizer.language != lang_before_top
            || self.localizer.language != lang_before_dialogs
        {
            ctx.request_repaint();
            for id_str in [
                "geoestabil_user_manual",
                "geoestabil_about_dialog",
                "geoestabil_soil_catalog",
                "geoestabil_settings_dialog",
            ] {
                ctx.request_repaint_of(egui::ViewportId::from_hash_of(id_str));
            }
        }
    }
}
