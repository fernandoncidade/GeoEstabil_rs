use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    PtBr,
    EnUs,
}

impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::PtBr => "Português (BR)",
            Language::EnUs => "English (US)",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::PtBr => "pt_BR",
            Language::EnUs => "en_US",
        }
    }

    /// Converte uma string de código (`"pt_BR"`, `"en_US"`, `"en"`, `"pt"`) para [`Language`].
    pub fn from_config_code(code: &str) -> Option<Self> {
        let lower = code.trim().to_ascii_lowercase();
        if lower.starts_with("en") {
            Some(Language::EnUs)
        } else if lower.starts_with("pt") {
            Some(Language::PtBr)
        } else {
            None
        }
    }
}

/// Conjunto tipado legado mantido por compatibilidade com módulos antigos.
pub struct Translations {
    pub app_title: &'static str,
    pub tab_project: &'static str,
    pub tab_slope: &'static str,
    pub tab_wall: &'static str,
    pub tab_memorial: &'static str,
    pub btn_calculate: &'static str,
    pub btn_export: &'static str,
    pub stable: &'static str,
    pub unstable: &'static str,
}

pub fn get_translations(lang: Language) -> Translations {
    match lang {
        Language::PtBr => Translations {
            app_title: "GeoEstabil — Estabilidade de Taludes & Muros de Arrimo (ABNT)",
            tab_project: "1. Projeto & Solos",
            tab_slope: "2. Estabilidade de Taludes",
            tab_wall: "3. Muro de Contenção",
            tab_memorial: "4. Memorial de Cálculo",
            btn_calculate: "Calcular & Analisar",
            btn_export: "Exportar Memorial",
            stable: "ESTÁVEL (REF. CRIT.)",
            unstable: "ABAIXO DA REF.",
        },
        Language::EnUs => Translations {
            app_title: "GeoEstabil — Slope Stability & Retaining Wall Design (ABNT)",
            tab_project: "1. Project & Soils",
            tab_slope: "2. Slope Stability",
            tab_wall: "3. Retaining Wall",
            tab_memorial: "4. Calculation Memorial",
            btn_calculate: "Compute & Analyze",
            btn_export: "Export Memorial",
            stable: "STABLE (REF. CHECK)",
            unstable: "UNSTABLE (BELOW REF.)",
        },
    }
}

/// Motor de localização dinâmica bilíngue (`pt-BR` / `en-US`) em tempo real.
#[derive(Debug, Clone)]
pub struct Localizer {
    pub language: Language,
    pt_map: HashMap<&'static str, &'static str>,
    en_map: HashMap<&'static str, &'static str>,
}

impl Default for Localizer {
    fn default() -> Self {
        Self::new(Language::PtBr)
    }
}

impl Localizer {
    pub fn new(language: Language) -> Self {
        let mut pt_map = HashMap::new();
        let mut en_map = HashMap::new();

        let entries: &[(&'static str, &'static str, &'static str)] = &[
            ("app.title", "GeoEstabil — Estabilidade de Taludes & Muros de Arrimo (ABNT)", "GeoEstabil — Slope Stability & Retaining Walls (ABNT)"),

            // Barra de Menus
            ("menu.file", "Arquivo", "File"),
            ("menu.file.new", "Novo Projeto", "New Project"),
            ("menu.file.open", "Abrir Projeto (.geoestabil)...", "Open Project (.geoestabil)..."),
            ("menu.file.save", "Salvar Projeto", "Save Project"),
            ("menu.file.save_as", "Salvar Como...", "Save As..."),
            ("menu.file.export_md", "Exportar Memorial Markdown (.md)...", "Export Markdown Memorial (.md)..."),
            ("menu.file.export_html", "Exportar Memorial HTML5 / PDF (A4)...", "Export HTML5 / PDF Memorial (A4)..."),
            ("menu.file.export_svg", "Exportar Perfil Vetorial (.svg)...", "Export Vector Profile (.svg)..."),
            ("menu.file.exit", "Sair", "Exit"),
            ("menu.presets", "Predefinições", "Presets"),
            ("menu.tools", "Ferramentas", "Tools"),
            ("menu.tools.soil_catalog", "Catálogo de Solos SPT (NBR 6122 / 6484)...", "SPT Soil Catalog (NBR 6122 / 6484)..."),
            ("menu.tools.settings", "Preferências do Software...", "Software Preferences..."),
            ("menu.tools.lang", "Idioma / Language", "Language / Idioma"),
            ("menu.tools.to_light", "Alternar para Modo Claro", "Switch to Light Mode"),
            ("menu.tools.to_dark", "Alternar para Modo Escuro", "Switch to Dark Mode"),
            ("menu.help", "Ajuda", "Help"),
            ("menu.help.manual", "Manual de Instruções... (F1)", "User Manual... (F1)"),
            ("menu.help.about", "Sobre o GeoEstabil... (Ctrl+I)", "About GeoEstabil... (Ctrl+I)"),

            // Barra de Ferramentas
            ("toolbar.new", "Novo", "New"),
            ("toolbar.open", "Abrir", "Open"),
            ("toolbar.save", "Salvar", "Save"),
            ("toolbar.active_mode", "Modo Ativo:", "Active Mode:"),
            ("toolbar.mode.slope", "Estabilidade de Taludes", "Slope Stability"),
            ("toolbar.mode.wall", "Muros de Contenção", "Retaining Walls"),
            ("toolbar.awaiting_calc", "Aguardando Cálculo", "Awaiting Calculation"),
            ("toolbar.theme_light", "Claro", "Light"),
            ("toolbar.theme_dark", "Escuro", "Dark"),

            // Abas Principais
            ("tab.project", "1. Estratigrafia & Solos", "1. Stratigraphy & Soils"),
            ("tab.slope", "2. Estabilidade de Taludes", "2. Slope Stability"),
            ("tab.wall", "3. Muro de Contenção", "3. Retaining Wall"),
            ("tab.structural", "4. Fundações & Estrutural (NBR 6118)", "4. Foundations & Structural (NBR 6118)"),
            ("tab.memorial", "5. Memorial de Cálculo", "5. Calculation Memorial"),

            // Barra de Status
            ("status.project", "Projeto", "Project"),
            ("status.mode", "Modo", "Mode"),
            ("status.unsaved", "Alterações não salvas", "Unsaved changes"),
            ("status.saved", "Salvo", "Saved"),
            ("status.windows", "Janelas:", "Windows:"),
            ("status.manual_pill", "Manual (F1)", "Manual (F1)"),
            ("status.about_pill", "Sobre", "About"),
            ("status.soils_pill", "Solos SPT", "SPT Soils"),
            ("status.settings_pill", "Preferências", "Settings"),

            // Diálogos Gerais
            ("dialog.close", "Fechar", "Close"),
            ("dialog.copy", "Copiar Texto", "Copy Text"),
            ("dialog.copied", "Copiado para a área de transferência!", "Copied to clipboard!"),
            ("dialog.export_md", "Exportar (.md)", "Export (.md)"),
            ("dialog.search", "Buscar:", "Search:"),
            ("manual.title", "Manual de Instruções — GeoEstabil", "User Manual — GeoEstabil"),
            ("manual.toc", "Capítulos do Manual", "Manual Chapters"),
            ("manual.search_hint", "Buscar palavra-chave no manual...", "Search keyword in manual..."),
            ("about.title", "Sobre o GeoEstabil", "About GeoEstabil"),
            ("about.header", "GeoEstabil — Estabilidade de Taludes & Muros de Arrimo", "GeoEstabil — Slope Stability & Retaining Walls"),
            ("about.tab.overview", "Visão Geral", "Overview"),
            ("about.tab.author", "Autor & Suporte", "Author & Support"),
            ("about.tab.history", "Histórico", "History"),
            ("about.tab.eula", "EULA (Comercial)", "EULA (Commercial)"),
            ("about.tab.clc", "CLC", "CLC"),
            ("about.tab.copyright", "Copyright & Marcas", "Copyright & Marks"),
            ("about.tab.privacy", "Privacidade (LGPD)", "Privacy (LGPD)"),
            ("about.tab.notices", "Terceiros (Notices)", "Third-Party Notices"),
            ("about.tab.licenses", "Licenças (AGPLv3 + OSS)", "Licenses (AGPLv3 + OSS)"),
            ("soils.title", "🗄 Catálogo de Solos Padronizados (ABNT NBR 6122 / SPT)", "🗄 Standard Soil Catalog (ABNT NBR 6122 / SPT)"),
            ("soils.intro", "Selecione um tipo de solo da literatura técnica brasileira para inserir na estratigrafia do projeto:", "Select a reference soil type from geotechnical practice to insert into the project stratigraphy:"),
            ("soils.insert", "➕ Inserir no Projeto", "➕ Insert into Project"),

            // Aba 1 — Estratigrafia & Solos
            ("tab1.title", "1. Estratigrafia Geotécnica, Camadas de Solo e Nível Freático", "1. Geotechnical Stratigraphy, Soil Layers & Water Table"),
            ("tab1.subtitle", "Defina as camadas estratigráficas do subsolo, parâmetros de cisalhamento (Mohr-Coulomb), lençol freático e sobrecargas.", "Define subsoil stratigraphic layers, shear strength parameters (Mohr-Coulomb), water table, and surcharges."),
            ("tab1.soil_bank", "Banco de Solos Padronizados (ABNT NBR 6122 / SPT):", "Standard Soil Library (ABNT NBR 6122 / SPT):"),
            ("tab1.layers_title", "Camadas de Solo do Modelo", "Model Soil Layers"),
            ("tab1.add_layer", "➕ Adicionar Camada Personalizada", "➕ Add Custom Layer"),
            ("tab1.water_title", "Lençol Freático (Nível d'Água)", "Water Table (Phreatic Surface)"),
            ("tab1.water_enable", "Considerar Nível Freático Ativo", "Enable Active Water Table"),
            ("tab1.water_gamma", "Peso Específico da Água γw [kN/m³]:", "Unit Weight of Water γw [kN/m³]:"),
            ("tab1.water_points", "Pontos da Linha Freática (x; y):", "Phreatic Line Points (x; y):"),
            ("tab1.surcharge_title", "Sobrecarga Superficial Acidental", "Surface Surcharge Load"),
            ("tab1.surcharge_enable", "Aplicar Sobrecarga na Crista / Terreno", "Apply Crest / Surface Surcharge"),
            ("tab1.surcharge_q", "Carga q [kPa = kN/m²]:", "Load q [kPa = kN/m²]:"),
            ("tab1.surcharge_start", "Início X [m]:", "Start X [m]:"),
            ("tab1.surcharge_end", "Fim X [m]:", "End X [m]:"),
            ("tab1.safety_title", "Enquadramento Normativo da Encosta (ABNT NBR 11682:2009)", "Slope Safety Classification (ABNT NBR 11682:2009)"),
            ("tab1.safety_label", "Nível de Segurança Exigido:", "Required Safety Level:"),
            ("tab1.safety_high", "Alto (FS ≥ 1.50)", "High (FS ≥ 1.50)"),
            ("tab1.safety_medium", "Médio (FS ≥ 1.30)", "Medium (FS ≥ 1.30)"),
            ("tab1.safety_low", "Baixo (FS ≥ 1.20)", "Low (FS ≥ 1.20)"),

            // Aba 2 — Estabilidade de Taludes
            ("tab2.title", "2. Estabilidade de Taludes", "2. Slope Stability"),
            ("tab2.subtitle", "Métodos de Fatias & Varredura Multithreaded Rayon", "Method of Slices & Multi-Threaded Rayon Grid Search"),
            ("tab2.compute", "⚡ CALCULAR ESTABILIDADE (RAYON)", "⚡ COMPUTE STABILITY (RAYON)"),
            ("tab2.stable", "✔ TALUDE ESTÁVEL (REF. NBR 11682)", "✔ STABLE SLOPE (REF. NBR 11682)"),
            ("tab2.unstable", "✖ TALUDE INSTÁVEL (ABAIXO REF.)", "✖ UNSTABLE SLOPE (BELOW REF.)"),
            ("tab2.method_title", "Método de Análise Primário:", "Primary Analysis Method:"),
            ("tab2.method_bishop", "Bishop Simplificado (Iterativo)", "Simplified Bishop (Iterative)"),
            ("tab2.method_fellenius", "Fellenius (Ordinário / Sueco)", "Fellenius (Ordinary / Swedish)"),
            ("tab2.method_janbu", "Janbu Simplificado", "Simplified Janbu"),
            ("tab2.grid_title", "Malha de Centros & Raios (Grid Search):", "Center & Radius Grid Search:"),
            ("tab2.xc_minmax", "Xc mín/máx [m]:", "Xc min/max [m]:"),
            ("tab2.xc_steps", "Passos Xc:", "Xc steps:"),
            ("tab2.yc_minmax", "Yc mín/máx [m]:", "Yc min/max [m]:"),
            ("tab2.yc_steps", "Passos Yc:", "Yc steps:"),
            ("tab2.r_minmax", "Raio mín/máx [m]:", "Radius min/max [m]:"),
            ("tab2.r_steps", "Passos Raio:", "Radius steps:"),
            ("tab2.num_slices", "Nº de Fatias:", "Num. of Slices:"),
            ("tab2.terrain_title", "Perfil do Terreno (Pontos):", "Terrain Profile (Points):"),
            ("tab2.add_point", "+ Ponto", "+ Point"),
            ("tab2.show_heatmap", "🔥 Exibir Mapa de Calor (Heatmap FS)", "🔥 Show FS Heatmap"),
            ("tab2.show_slices", "📏 Exibir Fatias", "📏 Show Slices"),
            ("tab2.show_grid", "📐 Grade Métrica", "📐 Metric Grid"),
            ("tab2.fit_view", "⛶ Enquadrar", "⛶ Fit View"),
            ("tab2.reset_view", "↺ Reset Vista", "↺ Reset View"),
            ("tab2.zoom_pan", "Zoom: Roda do mouse | Pan: Arrastar", "Zoom: Mouse wheel | Pan: Drag"),

            // Aba 3 — Muro de Contenção
            ("tab3.title", "3. Muro de Contenção", "3. Retaining Wall"),
            ("tab3.subtitle", "Geometria, Empuxos de Terra & Sobrecarga", "Geometry, Earth Pressures & Surcharge"),
            ("tab3.wall_type", "Tipologia do Muro de Contenção:", "Retaining Wall Type:"),
            ("tab3.type_cantilever", "Muro de Flexão em Concreto Armado (L/T)", "Reinforced Concrete Cantilever Wall (L/T)"),
            ("tab3.type_gravity", "Muro de Gravidade (Concreto Ciclópico/Gabião)", "Gravity Wall (Cyclopean Concrete/Gabion)"),
            ("tab3.theory", "Teoria de Empuxo:", "Earth Pressure Theory:"),
            ("tab3.theory_rankine", "Rankine (Solo-muro δ = 0)", "Rankine (Wall friction δ = 0)"),
            ("tab3.theory_coulomb", "Coulomb (com atrito solo-muro δ)", "Coulomb (with wall friction δ)"),
            ("tab3.dimensions", "Dimensões Geométricas [m]:", "Geometric Dimensions [m]:"),
            ("tab3.stem_height", "Altura Livre H:", "Clear Stem Height H:"),
            ("tab3.stem_top", "Espessura Crista b_{top}:", "Stem Top Width b_{top}:"),
            ("tab3.stem_base", "Espessura Base Fuste:", "Stem Base Width:"),
            ("tab3.base_width", "Largura da Base B:", "Footing Base Width B:"),
            ("tab3.base_thickness", "Espessura Sapata h_{s}:", "Footing Thickness h_{s}:"),
            ("tab3.toe_length", "Ponta (Aba Ant.) B_{toe}:", "Toe Length B_{toe}:"),
            ("tab3.shear_key", "Dente de Ancoragem (Chave Cisalhamento)", "Base Shear Key"),
            ("tab3.depth", "Profundidade:", "Depth:"),
            ("tab3.width", "Largura:", "Width:"),
            ("tab3.loads_water", "Sobrecarga e Nível d'Água:", "Surcharge & Water Level:"),
            ("tab3.surcharge_q", "Sobrecarga q [kPa]:", "Surcharge q [kPa]:"),
            ("tab3.water_hw", "Nível d'Água h_{w} [m]:", "Water Height h_{w} [m]:"),
            ("tab3.backfill_slope", "Inclinação Aterro β [°]:", "Backfill Slope β [°]:"),
            ("tab3.backfill_soil", "Solo de Reaterro (Tardoz):", "Backfill Soil (Behind Wall):"),
            ("tab3.foundation_soil", "Solo de Fundação (Apoio da Base):", "Foundation Soil (Bearing Base):"),
            ("tab3.gamma", "Peso Específico γ [kN/m³]:", "Unit Weight γ [kN/m³]:"),
            ("tab3.phi", "Ângulo de Atrito φ' [°]:", "Friction Angle φ' [°]:"),
            ("tab3.cohesion", "Coesão c' [kPa]:", "Cohesion c' [kPa]:"),
            ("tab3.sigma_adm", "Tensão Admissível σ_{adm}:", "Allowable Bearing σ_{adm}:"),
            ("tab3.pressures_hdr", "📊 Empuxos:", "📊 Pressures:"),
            ("tab3.view_title", "Visualização Gráfica do Muro & Diagramas de Pressão:", "Wall Graphic View & Pressure Diagrams:"),
            ("tab3.show_diagrams", "📊 Diagramas", "📊 Diagrams"),
            ("tab3.show_dimensions", "📏 Cotas", "📏 Dimensions"),
            ("tab3.show_grid", "📐 Grade", "📐 Grid"),

            // Aba 4 — Fundações & Estrutural
            ("tab4.title", "4. Fundações & Dimensionamento Estrutural em Concreto Armado", "4. Foundations & Reinforced Concrete Design"),
            ("tab4.subtitle", "Verificação de Estabilidade Geotécnica Externa (NBR 11682 / NBR 6122) e Dimensionamento da Armadura (NBR 6118:2023).", "External Geotechnical Stability Checks (NBR 11682 / NBR 6122) and Reinforcement Design (NBR 6118:2023)."),
            ("tab4.sec_a", "A. Verificações de Estabilidade Externa da Fundação", "A. External Foundation Stability Checks"),
            ("tab4.overturning", "Tombamento", "Overturning"),
            ("tab4.sliding", "Deslizamento", "Sliding"),
            ("tab4.eccentricity", "Excentricidade (e)", "Eccentricity (e)"),
            ("tab4.max_stress", "Tensão Máx. (σ_{toe})", "Max Bearing (σ_{toe})"),
            ("tab4.bearing", "Capacidade (Hansen)", "Bearing (Hansen)"),
            ("tab4.req_150", "Exigido: ≥ 1.50", "Required: ≥ 1.50"),
            ("tab4.pass", "✔ APROVADO", "✔ PASS"),
            ("tab4.fail", "✖ REPROVADO", "✖ FAIL"),
            ("tab4.middle_third", "✔ Terço Médio", "✔ Middle Third"),
            ("tab4.uplift", "✖ Descolamento", "✖ Uplift"),
            ("tab4.stress_ok", "✔ σ ≤ σ_{adm}", "✔ σ ≤ σ_{adm}"),
            ("tab4.overstress", "✖ Sobretensão", "✖ Overstress"),
            ("tab4.bearing_ok", "✔ FS ≥ 2.0", "✔ FS ≥ 2.0"),
            ("tab4.bearing_fail", "✖ Ruptura", "✖ Failure"),
            ("tab4.sec_b", "B. Parâmetros dos Materiais Estruturais (ABNT NBR 6118:2023)", "B. Structural Material Properties (ABNT NBR 6118:2023)"),
            ("tab4.fck", "f_{ck} do Concreto [MPa]:", "Concrete f_{ck} [MPa]:"),
            ("tab4.fyk", "f_{yk} do Aço [MPa]:", "Steel f_{yk} [MPa]:"),
            ("tab4.cover", "Recobrimento c_{nom} [mm]:", "Cover c_{nom} [mm]:"),
            ("tab4.gamma_f", "γ_{f} (Majoração):", "γ_{f} (Load Factor):"),
            ("tab4.sec_c", "C. Dimensionamento e Detalhamento da Armadura de Flexão", "C. Flexural Reinforcement Design & Detailing"),
            ("tab4.ductile", "Domínio 3 Dúctil", "Domain 3 Ductile"),
            ("tab4.brittle", "Frágil", "Brittle"),
            ("tab4.shear_ok", "✔ DISPENSADA ARM. TRANSVERSAL", "✔ NO STIRRUPS REQUIRED"),
            ("tab4.shear_fail", "✖ EXIGE ESTRIBOS", "✖ STIRRUPS REQUIRED"),
            ("tab4.horiz_title", "Armadura Horizontal de Distribuição e Retração Térmica (Fuste)", "Horizontal Distribution & Shrinkage Reinforcement (Stem)"),

            // Aba 5 — Memorial de Cálculo
            ("tab5.title", "5. Memorial de Cálculo e Relatórios de Estudo", "5. Technical Calculation Memorial & Engineering Reports"),
            ("tab5.subtitle", "Emissão de Memorial de Estudo Geotécnico e Estrutural com referências ABNT NBR 11682:2009, NBR 6122:2022 e NBR 6118:2023 (uso acadêmico; exige conferência externa).", "Generation of Geotechnical & Structural Study Memorial referencing ABNT NBR 11682:2009, NBR 6122:2022, and NBR 6118:2023 (academic study; external verification required)."),
            ("tab5.metadata", "Dados do Projeto e Responsabilidade Técnica", "Project Data & Engineering Responsibility"),
            ("tab5.field_title", "Título:", "Title:"),
            ("tab5.field_client", "Cliente:", "Client:"),
            ("tab5.field_site", "Local da Obra:", "Site Location:"),
            ("tab5.field_date", "Data:", "Date:"),
            ("tab5.field_engineer", "Responsável Técnico:", "Lead Engineer:"),
            ("tab5.field_crea", "CREA/CAU:", "License / Reg.:"),
            ("tab5.field_art", "ART/RRT:", "ART/RRT / Permit:"),
            ("tab5.field_notes", "Observações:", "Notes:"),
            ("tab5.export_html", "📄 Exportar Memorial HTML5 / PDF (A4)", "📄 Export HTML5 / PDF Memorial (A4)"),
            ("tab5.export_md", "📝 Exportar Markdown (.md)", "📝 Export Markdown (.md)"),
            ("tab5.export_svg", "📐 Exportar Perfil SVG", "📐 Export SVG Profile"),
            ("tab5.copy_text", "📋 Copiar Texto do Memorial", "📋 Copy Memorial Text"),
            ("tab5.preview", "Pré-visualização do Memorial Técnico Formatado:", "Formatted Technical Memorial Preview:"),

            // Canvas 2D
            ("canvas.wt", "N.A.", "W.T."),
            ("canvas.stable", "ESTÁVEL", "STABLE"),
            ("canvas.unstable", "INSTÁVEL", "UNSTABLE"),
            ("canvas.stratigraphy", "Estratigrafia (Mohr-Coulomb):", "Stratigraphy (Mohr-Coulomb):"),
            ("canvas.heatmap", "Heatmap FS (Centros)", "FS Heatmap (Centers)"),
            ("canvas.calc_hint", "Clique em ⚡ CALCULAR ESTABILIDADE\npara gerar o Heatmap e Círculo Crítico", "Click ⚡ COMPUTE STABILITY\nto generate Heatmap & Critical Arc"),
            ("canvas.wall_checks", "Verificações (NBR 11682 / 6122)", "Checks (NBR 11682 / 6122)"),
            ("canvas.hover_hint", "Passe o cursor sobre o gráfico para ler coordenadas", "Hover cursor over plot to read coordinates"),
            ("canvas.autofit_on", "Auto-Enquadramento: ON", "Auto-Fit: ON"),
            ("canvas.autofit_manual", "Manual (Duplo clique: Auto)", "Manual (Double-click: Auto)"),
        ];

        for &(k, pt, en) in entries {
            pt_map.insert(k, pt);
            en_map.insert(k, en);
        }

        Self {
            language,
            pt_map,
            en_map,
        }
    }

    pub fn set_language(&mut self, lang: Language) {
        self.language = lang;
    }

    /// Inicializa o localizador lendo a preferência persistida em `%APPDATA%\GeoEstabil\language_config.json`, usando Português (BR) por padrão.
    pub fn from_saved_preference() -> Self {
        let lang = crate::utils::paths::load_saved_language_code()
            .as_deref()
            .and_then(Language::from_config_code)
            .unwrap_or_default();
        Self::new(lang)
    }

    pub fn is_pt(&self) -> bool {
        self.language == Language::PtBr
    }

    pub fn pick<'a>(&self, pt: &'a str, en: &'a str) -> &'a str {
        match self.language {
            Language::PtBr => pt,
            Language::EnUs => en,
        }
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        let map = match self.language {
            Language::PtBr => &self.pt_map,
            Language::EnUs => &self.en_map,
        };
        map.get(key).copied().unwrap_or(key)
    }
}
