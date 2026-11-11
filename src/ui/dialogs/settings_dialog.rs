use crate::locale::translations::{Language, Localizer};
use crate::ui::dialogs::modeless::{show_modeless_dialog, ModelessState};
use crate::ui::theme::GeoEstabilTheme;
use eframe::egui;

#[derive(Default)]
pub struct SettingsDialog {
    pub modeless: ModelessState,
}

impl SettingsDialog {
    pub fn open(&mut self) {
        self.modeless.open();
    }

    pub fn close(&mut self) {
        self.modeless.close();
    }

    pub fn is_open(&self) -> bool {
        self.modeless.is_open
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        dark_mode: &mut bool,
        localizer: &mut Localizer,
    ) {
        let mut modeless_state = self.modeless.clone();
        let pt = localizer.is_pt();
        let title = if pt {
            "⚙ Preferências do Sistema — GeoEstabil"
        } else {
            "⚙ System Preferences — GeoEstabil"
        };

        show_modeless_dialog(
            ctx,
            "geoestabil_settings_dialog",
            title,
            [500.0, 390.0],
            [400.0, 320.0],
            *dark_mode,
            &mut modeless_state,
            |ui, close_clicked| {
                let c = ui.ctx().clone();
                ui.heading(if pt { "Idioma / Language" } else { "Language / Idioma" });
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(localizer.language == Language::PtBr, "🇧🇷 Português (BR)")
                        .clicked()
                    {
                        localizer.set_language(Language::PtBr);
                    }
                    if ui
                        .selectable_label(localizer.language == Language::EnUs, "🇺🇸 English (US)")
                        .clicked()
                    {
                        localizer.set_language(Language::EnUs);
                    }
                });

                ui.separator();
                ui.heading(if pt { "Aparência e Tema" } else { "Appearance & Theme" });
                ui.horizontal(|ui| {
                    ui.label(if pt { "Modo de Exibição:" } else { "Theme Mode:" });
                    if ui
                        .radio_value(dark_mode, true, if pt { "🌙 Modo Escuro" } else { "🌙 Dark Mode" })
                        .clicked()
                    {
                        GeoEstabilTheme::apply(&c, true);
                    }
                    if ui
                        .radio_value(dark_mode, false, if pt { "☀ Modo Claro" } else { "☀ Light Mode" })
                        .clicked()
                    {
                        GeoEstabilTheme::apply(&c, false);
                    }
                });

                ui.separator();
                ui.heading(if pt { "Unidades de Medida" } else { "Engineering Units" });
                ui.label(if pt {
                    "Sistema de Unidades: SI Técnico Oficial (ABNT)"
                } else {
                    "Unit System: Technical Metric SI (ABNT)"
                });
                ui.label("• m | kN | kN/m | kPa (kN/m²) | ° | MPa | cm²/m");

                ui.separator();
                ui.heading(if pt {
                    "Multithreading e Paralelismo"
                } else {
                    "Multithreading & Parallelism"
                });
                ui.label(format!(
                    "{}: {}",
                    if pt {
                        "Processadores lógicos disponíveis"
                    } else {
                        "Available logical CPU cores"
                    },
                    num_cpus()
                ));
                ui.label(if pt {
                    "Motor Work-Stealing: Rayon"
                } else {
                    "Work-Stealing Engine: Rayon"
                });

                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(localizer.t("dialog.close")).clicked() {
                            *close_clicked = true;
                        }
                    });
                });
            },
        );

        self.modeless = modeless_state;
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
