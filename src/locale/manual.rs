use crate::locale::translations::Language;

/// Seção estruturada do Manual de Instruções interativo do GeoEstabil.
#[derive(Debug, Clone)]
pub struct ManualSection {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub content: &'static str,
}

pub fn get_manual_sections(lang: Language) -> Vec<ManualSection> {
    match lang {
        Language::PtBr => vec![
            ManualSection {
                id: "cap1",
                title: "1. Visão Geral & Interface Windows 11",
                subtitle: "Navegação por abas, barra superior, atalhos e janelas não-modais (Modeless Z-Order)",
                content: r#"==== VISÃO GERAL DA PLATAFORMA GEOESTABIL ====

O GeoEstabil reúne em ambiente desktop nativo (Rust + egui/wgpu) ferramentas auxiliares para estudo de Estabilidade de Taludes (ref. ABNT NBR 11682:2009), Muros de Contenção (ref. ABNT NBR 6122:2022 e NBR 6118:2023), catálogo de solos SPT e exportação de Memoriais de Cálculo de estudo.

## Fluxo das 5 Abas Principais
• Aba 1 — Estratigrafia & Solos: Gerencie metadados do projeto, perfis estratigráficos, camadas de solo (c', φ', γ, γ_sat) e o Catálogo SPT em SQLite.
• Aba 2 — Estabilidade de Taludes: Configure vértices do terreno, nível freático, sobrecarga acidental e dispare a varredura paralela Rayon com exibição do Heatmap do Fator de Segurança (FS) e círculo crítico.
• Aba 3 — Muro de Contenção: Dimensione muros de gravidade (concreto ciclópico), gabiões escalonados e muros em balanço de concreto armado (fuste + sapata corrida com ponta e calcanhar).
• Aba 4 — Fundações & Estrutural (NBR 6118): Verifique capacidade de carga (Meyerhof/Hansen), tensões na base, armadura longitudinal de flexão, cortante V_Rd1 >= V_d e fissuração ELS-W.
• Aba 5 — Memorial de Cálculo: Gere o resumo numérico em Markdown, HTML ou PDF para estudo acadêmico e conferência manual externa.

## Janelas Não-Modais e Atalhos Globais
• F1 ou Ctrl+M: Abre ou traz para frente o Manual de Instruções.
• Ctrl+I: Abre a janela Sobre com documentação completa, EULA, CLC, Privacidade, Referências Bibliográficas ABNT (aba Terceiros / Notices) e Licenças OSS.
• Na barra de status inferior, os botões (pills) alternam o foco Z-Order entre janelas abertas lado a lado."#,
            },
            ManualSection {
                id: "cap2",
                title: "2. Estabilidade de Taludes (ABNT NBR 11682)",
                subtitle: "Modelagem de perfil, lençol freático, varredura paralela Rayon, Fellenius, Bishop e Janbu",
                content: r#"==== ESTABILIDADE DE TALUDES E ENCOSTAS ====

## Passo a Passo de Modelagem
1. Perfil Topográfico e Camadas de Solo: Defina os vértices `(X, Y)` do terreno e as cotas das camadas estratigráficas. Para cada horizonte informe peso específico natural (γ), peso específico saturado (γ_sat), coesão efetiva (c') e ângulo de atrito efetivo (φ').
2. Lençol Freático e Sobrecarga: Ajuste a polilinha piezométrica do lençol freático. O motor calcula automaticamente a subpressão hidrostática `u = γ_w · h_w` na base de cada fatia submersa, reduzindo a tensão normal efetiva `N'`. Aplique sobrecarga distribuída `q` (kPa) na crista ou berma quando aplicável.
3. Grau de Segurança Requerido (NBR 11682 Tabela 1): Escolha o nível de segurança ao deslizamento em função da perda potencial de vidas humanas e danos materiais/ambientais (Alto FS >= 1,50; Médio FS >= 1,30; Baixo FS >= 1,20).
4. Varredura Paralela Rayon e Mapa de Calor: O algoritmo distribui milhares de círculos candidatos `(Xc, Yc, R)` pelos núcleos do processador. No Canvas 2D:
   - Use a roda do mouse para Zoom centrado no cursor;
   - Arraste com o botão esquerdo para deslocar (Pan);
   - Observe os centros coloridos no Heatmap (vermelho = crítico, verde = seguro) e o arco crítico com suas fatias verticais.

## Critério Numérico dos Métodos Implementados
• Fellenius (Ordinário): Explícito e conservador; útil como estimativa inicial.
• Bishop Simplificado (Método Padrão da Norma): Iterativo sobre o Fator de Segurança até convergência, com guarda numérica `m_α >= 0,20` para evitar divisão instável em bases muito inclinadas junto ao pé do talude.
• Janbu Simplificado: Método baseado no equilíbrio de forças horizontais corrigido pelo fator empírico de forma `f0` conforme a razão profundidade/comprimento (d/L)."#,
            },
            ManualSection {
                id: "cap3",
                title: "3. Muros de Arrimo — Gravidade & Flexão (NBR 6122 / 6118)",
                subtitle: "Rankine, Coulomb, tombamento, deslizamento, capacidade de carga e dimensionamento CA",
                content: r#"==== PROJETO E VERIFICAÇÃO DE MUROS DE ARRIMO ====

## Tipologias Disponíveis
• Gravidade — Concreto Ciclópico / Alvenaria de Pedra: Estabilidade dominada pelo peso próprio do maciço trapézico ou escalonado (`γ_conc ~ 22 a 24 kN/m³`).
• Muro de Gabião Escalonado: Gaiolas em malha metálica preenchidas por rachão (`γ_gabião ~ 17 a 19 kN/m³`), permeável (reduz empuxo hidrostático quando drenado).
• Flexão em Concreto Armado (Fuste + Ponta + Calcanhar): O peso do prisma de terra sobre o calcanhar colabora para estabilização ao tombamento e deslizamento.

## Empuxos Ativo e Passivo
• Teoria de Rankine: Adequada para paramento interno vertical liso ou solo sobre o calcanhar (`δ = β`), fornecendo coeficientes `Ka` e `Kp`.
• Teoria de Coulomb: Incorpora o atrito solo-muro `δ` (em geral `1/2 φ'` a `2/3 φ'`) e a inclinação `ψ` do paramento interno, inclinando a resultante do empuxo e gerando componente vertical favorável de atrito na face posterior.

## Verificações Geotécnicas Externas (ABNT NBR 6122)
• Segurança ao Tombamento: `FS_tomb = M_estabilizante / M_tombamento >= 1,50`.
• Segurança ao Deslizamento: `FS_desl >= 1,50` (com controle de utilização parcial ou nula do empuxo passivo na ponta em função de eventual escavação futura).
• Excentricidade na Base e Distribuição de Tensões: Caso `e <= B/6`, o diagrama sob a sapata permanece inteiramente comprimido (trapezoidal). Se `B/6 < e < B/3`, o motor ativa a redistribuição triangular em zona comprimida `x_R = 3 · (B/2 - e)` com `σ_máx = 2·N / (3·x_R)`.
• Capacidade de Carga da Fundação: Avaliação pelos fatores `Nc`, `Nq`, `Nγ` de Hansen/Meyerhof com fator de segurança global contra ruptura da fundação.

## Dimensionamento Estrutural em Concreto Armado (ABNT NBR 6118)
• Majoração de solicitações (`γ_f = 1,40`), minoração de materiais (`γ_c = 1,40`, `γ_s = 1,15`) e cobrimento nominal segundo Classe de Agressividade Ambiental (CAA I a IV).
• Verificação de ductilidade na flexão (`x/d <= 0,45` para `f_ck <= 50 MPa`), armadura mínima (`0,15% · b_w · h`), força cortante resistente sem estribos (`V_Rd1 >= V_d`) e abertura característica de fissuras em Estado Limite de Serviço (`ELS-W: w_k <= 0,30 mm`)."#,
            },
            ManualSection {
                id: "cap4",
                title: "4. Catálogo de Solos SPT, SQLite & CLI",
                subtitle: "Correlações NBR 6484/6122, persistência local e automação via geoestabil-cli",
                content: r#"==== CATÁLOGO GEOTÉCNICO E AUTOMAÇÃO CLI ====

## Catálogo de Solos em SQLite (`geoestabil.db`)
O GeoEstabil armazena em banco SQLite local (`%APPDATA%\GeoEstabil\geoestabil.db`) uma biblioteca inicial de solos brasileiros típicos (areias fofas a compactas, siltes e argilas moles a rijas) com correlações referenciais entre índice de resistência à penetração (`N_SPT`, ABNT NBR 6484), peso específico `γ`, coesão efetiva `c'`, ângulo de atrito `φ'` e tensão admissível estimada. Os registros podem ser consultados e aplicados diretamente às camadas do talude ou reaterro/fundação do muro.

⚠️ Importante: Correlações empíricas `N_SPT` são referências preliminares. Obras definitivas, taludes urbanos com risco à vida ou solos colapsíveis/expansivos exigem ensaios laboratoriais específicos (cisalhamento direto, triaxial ou adensamento).

## Uso do Binário de Linha de Comando (`geoestabil-cli.exe`)
Ideal para verificação em lote e pipelines de engenharia:
• `geoestabil-cli presets` — Lista os 4 casos pré-configurados.
• `geoestabil-cli run-preset 1 --output memorial.html` — Roda o preset #1 e salva relatório HTML.
• `geoestabil-cli slope -i obra.geoestabil -o relatorio.html` — Analisa arquivo de talude via Rayon.
• `geoestabil-cli wall -i muro.geoestabil -o relatorio.html` — Verifica muro e armaduras."#,
            },
            ManualSection {
                id: "cap5",
                title: "5. Memoriais de Cálculo & Uso Acadêmico",
                subtitle: "Exportação Markdown/HTML e limites de uso (conferência externa obrigatória)",
                content: r#"==== MEMORIAIS DE ESTUDO E LIMITES DE USO ====

## Exportação do Memorial (Aba 5 / Menu Arquivo)
O módulo `MemorialGenerator` organiza as hipóteses geométricas, parâmetros geotécnicos adotados por camada, resultados dos métodos de Fellenius, Bishop e Janbu, coordenadas e raio do círculo crítico, verificações externas de muros e sugestões de armadura baseadas nas formulações de referência ABNT NBR 11682:2009, NBR 6122:2022 e NBR 6118:2023.

O relatório de estudo pode ser salvo em `.md`, `.html` ou `.pdf`.

## Aviso de Uso Acadêmico e Isenção ("AS IS")
O GeoEstabil destina-se exclusivamente a estudo acadêmico, ensino e experimentação computacional, podendo conter falhas de cálculo ou aproximações numéricas. NÃO deve ser utilizado como software definitivo em produção, obras ou projetos reais sem:
• Conferência manual completa de todas as equações e resultados;
• Validação cruzada em outras ferramentas consolidadas de mercado e ensaios locais de campo;
• Supervisão técnica e eventual emissão de ART/RRT por profissional legalmente habilitado, ficando o autor integralmente isento por qualquer uso indevido ou aplicação produtiva."#,
            },
            ManualSection {
                id: "cap6",
                title: "6. Duplo Licenciamento, Privacidade & Suporte por E-mail",
                subtitle: "GNU AGPLv3 no GitHub vs. EULA Comercial na Microsoft Store e Atendimento por Disponibilidade",
                content: r#"==== DUPLO LICENCIAMENTO, PRIVACIDADE E SUPORTE ====

## Duplo Licenciamento (Dual-Licensing)
1. Código-Fonte Público (GitHub): Licenciado sob **GNU AGPLv3** (`AGPL-3.0-only`), permitindo livre uso, estudo, modificação e compartilhamento sob a mesma licença, com proteção nominal de nome e marca `"GeoEstabil"` (Seção 7(e) AGPLv3).
2. Edição Comercial (Microsoft Store): Os pacotes oficiais são regidos pelo **EULA Comercial e CLC** (disponíveis integralmente na janela **Sobre**), sem obrigação de abertura de código da GNU AGPLv3 ao adquirente comercial.

## Privacidade Local (100% Offline)
• Execução **100% local e offline**: O GeoEstabil NÃO coleta telemetria, métricas de uso, projetos ou dados pessoais em servidores externos.
• Arquivos `.geoestabil`, banco `geoestabil.db` e memoriais permanecem exclusivamente no computador do usuário.

## Canal Único de Suporte (Sem Urgência / AS IS)
• Todo eventual suporte ocorre **exclusivamente por e-mail (`linceu_lighthouse@outlook.com`)**, **conforme a disponibilidade do autor e sem obrigação de urgência, prazo ou SLA** (sem telefone, acesso remoto ou consultoria).
• Software fornecido no estado em que se encontra ("AS IS"), com isenção integral por uso indevido."#,
            },
        ],
        Language::EnUs => vec![
            ManualSection {
                id: "cap1",
                title: "1. Overview & Windows 11 Workspace",
                subtitle: "Tabbed workflow, toolbar, global shortcuts, and Modeless Z-Order windows",
                content: r#"==== GEOESTABIL OVERVIEW ====

GeoEstabil unifies Rayon-parallelized Slope Stability analysis (ABNT NBR 11682:2009), Retaining Wall Geotechnical & Reinforced Concrete Design (ABNT NBR 6122:2022 & NBR 6118:2023), a local SQLite SPT soil catalog, and automated Markdown/HTML Calculation Memorial generation in a native Windows 11 Rust + egui/wgpu application.

## Five Main Workspace Tabs
• Tab 1 — Stratigraphy & Soils: Manage project metadata, soil layers, and the local SQLite SPT catalog.
• Tab 2 — Slope Stability: Configure topographic coordinates, phreatic water line, surcharge loads, and execute multi-threaded Rayon circle grid searches with a live Factor of Safety Heatmap.
• Tab 3 — Retaining Wall: Design Cyclopean Gravity walls, Stepped Gabion walls, and Reinforced Concrete Cantilever walls with continuous footings.
• Tab 4 — Foundations & Structural Design (NBR 6118): Check bearing capacity, base stresses, flexural/shear steel, and ELS-W crack width limits.
• Tab 5 — Calculation Memorial: Export traceable engineering verification reports in Markdown or HTML.

## Modeless Z-Order Windows & Shortcuts
• F1 or Ctrl+M: Open / focus the User Manual.
• Ctrl+I: Open the 9-tab About window (EULA, CLC, Privacy Policy, Copyright, ABNT Bibliographic References under Third-Party Notices, and OSS Licenses).
• Bottom status-bar window pills bring open modeless windows cleanly to the foreground."#,
            },
            ManualSection {
                id: "cap2",
                title: "2. Slope Stability Analysis (ABNT NBR 11682)",
                subtitle: "Profile geometry, water table, Rayon multi-threaded sweep, Fellenius, Bishop & Janbu",
                content: r#"==== SLOPE STABILITY SOLVER GUIDE ====

## Modeling Steps
1. Topography & Soil Stratigraphy: Enter surface polyline points `(X, Y)` and layer elevations. For each soil stratum define unit weight `γ`, saturated weight `γ_sat`, effective cohesion `c'`, and friction angle `φ'`.
2. Phreatic Surface & Surcharge: Define the water table polyline. Hydrostatic pore pressure `u = γ_w · h_w` reduces effective normal stress at each submerged slice base. Apply optional uniform surface load `q` (kPa).
3. Required Factor of Safety (NBR 11682 Table 1): Select required safety level by failure consequence severity (High FS >= 1.50; Medium FS >= 1.30; Low FS >= 1.20).
4. Parallel Rayon Sweep & 2D Canvas: Candidate `(Xc, Yc, R)` circles are evaluated across CPU cores. Scroll mouse wheel to zoom toward the cursor and drag to pan across the FS Heatmap and critical failure arc.

## Limit Equilibrium Formulations
• Ordinary Method (Fellenius): Closed-form conservative baseline.
• Simplified Bishop (Primary Standard): Iterative fixed-point solution with `m_α >= 0.20` guard preventing numerical blow-up near steep toe slices.
• Simplified Janbu: Horizontal force equilibrium corrected by empirical shape factor `f0(d/L)`."#,
            },
            ManualSection {
                id: "cap3",
                title: "3. Retaining Walls — Gravity & RC Design (NBR 6122 / 6118)",
                subtitle: "Rankine, Coulomb, overturning, sliding, bearing capacity & RC detailing",
                content: r#"==== RETAINING WALL ANALYSIS & DESIGN ====

## Supported Wall Typologies
• Gravity Cyclopean Concrete: Stability derived from trapezoidal concrete self-weight.
• Stepped Gabion Wall: Free-draining wire-basket stone mass reducing hydrostatic thrust.
• Reinforced Concrete Cantilever Wall: Stem + Toe + Heel continuous footing leveraging backfill weight over the heel.

## Earth Pressure Theories
• Rankine: Smooth vertical virtual face (`δ = β`) computing active/passive `Ka` and `Kp`.
• Coulomb: Accounts for wall-soil interface friction `δ` and backface inclination `ψ`, tilting the resultant active thrust.

## External Checks (ABNT NBR 6122:2022)
• Overturning (`FS_overturning >= 1.50`) and Base Sliding (`FS_sliding >= 1.50`).
• Base Kern Eccentricity (`e <= B/6`) with triangular bearing stress redistribution `σ_max = 2·N / (3·x_R)` when partial base uplift occurs.
• Foundation Ultimate & Allowable Bearing Capacity via Hansen/Meyerhof `Nc`, `Nq`, `Nγ`.

## RC Structural Checks (ABNT NBR 6118:2023)
• Partial load/resistance factors `γ_f = 1.40`, `γ_c = 1.40`, `γ_s = 1.15`, CAA environmental cover.
• Ductility limit (`x/d <= 0.45`), minimum reinforcement (`0.15%`), shear without stirrups (`V_Rd1 >= V_d`), and ELS-W crack width check (`w_k <= 0.30 mm`)."#,
            },
            ManualSection {
                id: "cap4",
                title: "4. SPT Soil Catalog, SQLite & CLI",
                subtitle: "NBR 6484/6122 reference correlations and geoestabil-cli headless automation",
                content: r#"==== SOIL LIBRARY & HEADLESS CLI ====

• Local SQLite Catalog (`%APPDATA%\GeoEstabil\geoestabil.db`) provides editable SPT (`N_SPT`, NBR 6484) soil reference correlations (`γ`, `c'`, `φ'`, `σ_adm`). Empirical correlations must always be verified with laboratory shear tests on critical slopes.
• `geoestabil-cli` Commands:
  - `presets`: List built-in engineering presets;
  - `run-preset <1..4> -o report.html`: Run preset and output HTML memorial;
  - `slope -i project.geoestabil -o report.html`: Run Rayon slope stability sweep;
  - `wall -i project.geoestabil -o report.html`: Run retaining wall stability and RC design;
  - `soils`: Print NBR 6122 SPT soil catalog."#,
            },
            ManualSection {
                id: "cap5",
                title: "5. Calculation Memorials & Academic Scope",
                subtitle: "Markdown/HTML/PDF export and verification limits (external check required)",
                content: r#"==== CALCULATION MEMORIALS & DISCLAIMER ====

Tab 5 exports study calculation memorials summarizing input stratigraphy, search grid settings, Fellenius/Bishop/Janbu outputs, retaining wall stability ratios, rebar sizing suggestions, and reference comparisons under ABNT NBR 11682, NBR 6122, and NBR 6118.

GeoEstabil is intended strictly for academic study, teaching, and experimentation, and may contain calculation bugs or approximations. It MUST NOT be relied upon as definitive software in production or civil works without full manual verification, cross-checking against established commercial market tools, field investigations, and formal oversight (e.g., ART/RRT) by a licensed engineer. Provided strictly "AS IS", wholly exempting the author from improper or production misuse."#,
            },
            ManualSection {
                id: "cap6",
                title: "6. Dual Licensing, Local Privacy & Email Support",
                subtitle: "GitHub GNU AGPLv3 vs. Microsoft Store EULA/CLC & Availability-Based Support",
                content: r#"==== DUAL LICENSING, PRIVACY & SUPPORT ====

• Open-Source GitHub Codebase: Licensed under **GNU AGPLv3** (`AGPL-3.0-only`), permitting free use, study, modification, and sharing under the same license while protecting the `"GeoEstabil"` name/mark under Section 7(e).
• Microsoft Store Commercial Builds: Governed by our **Commercial EULA & CLC** (readable in the About dialog), exempting Store purchasers from GNU AGPLv3 copyleft obligations.
• Zero Telemetry: 100% local/offline processing on the user's computer with zero network transmission.
• Email-Only Support: Assistance is provided exclusively via email (`linceu_lighthouse@outlook.com`), strictly according to the author's availability and with no obligation of urgency or SLA."#,
            },
        ],
    }
}
