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
                subtitle: "Navegação por abas, menus, barra de ferramentas, atalhos e janelas modeless Z-Order",
                content: r#"==== VISÃO GERAL DA PLATAFORMA GEOESTABIL ====

O GeoEstabil reúne em ambiente desktop nativo (Rust + egui/eframe) com renderização gráfica leve em Glow (OpenGL 3.3/WGL) ferramentas técnicas para estudo de Estabilidade de Taludes (ref. ABNT NBR 11682:2009), Muros de Contenção (ref. ABNT NBR 6122:2022 e NBR 6118:2023), catálogo de solos SPT em SQLite e exportação de Memoriais de Cálculo de estudo.

## Fluxo de Trabalho das 5 Abas Principais
• Aba 1 — Estratigrafia & Solos: Gerencie metadados do projeto, perfis estratigráficos, camadas de solo (c', φ', γ, γ_sat), parâmetros de fundação (σ_adm), sobrecarga na crista e o Catálogo SPT em SQLite.
• Aba 2 — Estabilidade de Taludes: Configure vértices (X, Y) do terreno, nível freático, sobrecarga acidental e dispare a varredura paralela Rayon com exibição no Canvas 2D do Heatmap do Fator de Segurança (FS), fatias verticais e arco crítico.
• Aba 3 — Muro de Contenção: Dimensione muros em balanço de concreto armado (fuste + sapata corrida com ponta e calcanhar), muros de gravidade (concreto ciclópico) e gabiões escalonados, avaliando empuxos ativos/passivos por Rankine ou Coulomb.
• Aba 4 — Fundações & Estrutural (NBR 6118): Verifique os 5 critérios geotécnicos externos (tombamento, deslizamento, excentricidade, tensão na base e capacidade de carga de Hansen/Meyerhof) e detalhe armaduras longitudinais de flexão, cortante V_Rd1 >= V_d e abertura de fissuras ELS-W.
• Aba 5 — Memorial de Cálculo: Edite dados do estudo e exporte o memorial analítico completo em HTML5/PDF (A4 com diagramas), Markdown (.md) ou desenho vetorial SVG (.svg).

## Menus Superiores e Ações do Sistema
• Menu Arquivo: Novo Projeto (reinicia dados), Abrir Projeto (carrega .geoestabil), Salvar e Salvar Como (grava projeto em JSON estruturado), Exportar Memorial HTML/PDF, Exportar Markdown e Exportar SVG Vetorial.
• Menu Ferramentas: Casos Pré-configurados (Presets 1 a 4), Catálogo de Solos SPT (abre banco SQLite), Alternar Modo Ativo (Talude vs Muro) e Recalcular Estabilidade (força reexecução imediata).
• Menu Janela: Alternar Tema Claro/Escuro (comuta paletas sem oscilação geométrica), Idioma (Português BR / English US) e Redefinir Zoom/Pan do Canvas 2D.
• Menu Ajuda: Manual de Instruções (tecla F1 ou atalho Ctrl+M) e janela Sobre o GeoEstabil (atalho Ctrl+I, com 9 abas de licenciamento, EULA, CLC, privacidade e referências bibliográficas ABNT).

## Barra de Ferramentas (Toolbar) e Controles Rápidos
• Botões de Acesso Rápido: Novo Projeto (📄), Abrir (📂) e Salvar (💾).
• Seletor de Modo Operacional: Alterne entre Estabilidade de Taludes (⛰) e Muro de Contenção (🧱).
• Telemetria Dinâmica ao Vivo: Monitora o Fator de Segurança crítico de Bishop (para taludes) ou empuxo Ea e fatores de tombamento/deslizamento (para muros), com sinalização por cores.
• Alternador Rápido de Tema: Botão solar/lunar para alternar entre Modo Escuro técnico e Modo Claro ergonômico.

## Janelas Não-Modais e Atalhos Globais
• F1 ou Ctrl+M: Abre ou traz para frente o Manual de Instruções em janela flutuante independente.
• Ctrl+I: Abre a janela Sobre com documentação completa, EULA, CLC, Privacidade, Referências Bibliográficas ABNT (aba Terceiros / Notices) e Licenças OSS.
• Na barra de status inferior, os botões (pills) com indicador ▲ alternam o foco Z-Order entre janelas abertas lado a lado."#,
            },
            ManualSection {
                id: "cap2",
                title: "2. Estabilidade de Taludes (ABNT NBR 11682)",
                subtitle: "Modelagem de perfil, lençol freático, varredura paralela Rayon, Fellenius, Bishop e Janbu",
                content: r#"==== ESTABILIDADE DE TALUDES E ENCOSTAS ====

## Passo a Passo de Modelagem na Aba 2
1. Perfil Topográfico e Camadas de Solo: Defina os vértices (X, Y) do terreno e as cotas das camadas estratigráficas. Para cada horizonte informe peso específico natural (γ), peso específico saturado (γ_sat), coesão efetiva (c') e ângulo de atrito efetivo (φ').
2. Lençol Freático e Sobrecarga: Ajuste a polilinha piezométrica do lençol freático. O motor calcula automaticamente a subpressão hidrostática u = γ_w · h_w na base de cada fatia submersa, reduzindo a tensão normal efetiva N'. Aplique sobrecarga distribuída q (kPa) na crista ou berma quando aplicável.
3. Grau de Segurança Requerido (NBR 11682 Tabela 1): Escolha o nível de segurança ao deslizamento em função da perda potencial de vidas humanas e danos materiais/ambientais (Alto FS >= 1,50; Médio FS >= 1,30; Baixo FS >= 1,20).
4. Varredura Paralela Rayon e Mapa de Calor: O algoritmo distribui milhares de círculos candidatos (Xc, Yc, R) pelos núcleos do processador. No Canvas 2D:
   - Use a roda do mouse para Zoom dinâmico centrado no cursor;
   - Arraste com o botão esquerdo para deslocar (Pan);
   - Observe os centros coloridos no Heatmap (vermelho = crítico, verde = seguro) e o arco crítico com suas fatias verticais;
   - Clique em 'Enquadrar' para ajustar o modelo à área visível ou 'Redefinir' para voltar à visualização padrão.

## Critério Numérico dos Métodos Implementados
• Fellenius (Ordinário): Explícito e conservador; útil como estimativa inicial de equilíbrio limite estático global para superfícies de ruptura circulares em maciços terrosos.
• Bishop Simplificado (Método Padrão da Norma): Iterativo sobre o Fator de Segurança até convergência, com guarda numérica m_α >= 0,20 para evitar divisão instável em bases muito inclinadas junto ao pé do talude.
• Janbu Simplificado: Método baseado no equilíbrio de forças horizontais corrigido pelo fator empírico de forma f0 conforme a razão profundidade/comprimento (d/L)."#,
            },
            ManualSection {
                id: "cap3",
                title: "3. Muros de Arrimo — Gravidade & Flexão (NBR 6122 / 6118)",
                subtitle: "Rankine, Coulomb, tombamento, deslizamento, capacidade de carga e dimensionamento CA",
                content: r#"==== PROJETO E VERIFICAÇÃO DE MUROS DE ARRIMO ====

## Tipologias Disponíveis na Aba 3
• Muro em Balanço (Cantilever) de Concreto Armado: O peso do prisma de terra sobre o calcanhar colabora ativamente para estabilização ao tombamento e deslizamento.
• Muro de Gravidade — Concreto Ciclópico: Estabilidade dominada pelo peso próprio do maciço trapezoidal (γ_conc ~ 22 a 24 kN/m³).
• Muro de Gabião Escalonado: Gaiolas em malha metálica preenchidas por rachão (γ_gabião ~ 17 a 19 kN/m³), permeável e com alívio hidrostático.

## Empuxos Ativo e Passivo
• Teoria de Rankine: Adequada para paramento interno vertical liso ou solo sobre o calcanhar (δ = β), fornecendo coeficientes Ka e Kp.
• Teoria de Coulomb: Incorpora o atrito solo-muro δ (em geral 1/2 φ' a 2/3 φ') e a inclinação ψ do paramento interno, gerando componente vertical descendente favorável.
• Sobrecargas e Água: Calcula empuxo uniforme de sobrecarga Ea_q = q · Ka · H e empuxo de água Ea_w quando o nível freático estiver acima da base.

## Verificações Geotécnicas Externas (ABNT NBR 6122)
• Segurança ao Tombamento: FS_tomb = M_estabilizante / M_tombamento >= 1,50.
• Segurança ao Deslizamento: FS_desl = (N·tan δ_b + c_a·B + 0,5·Ep + E_dente) / Ea_h >= 1,50 (com chave de cisalhamento / dente de ancoragem opcional).
• Excentricidade na Base e Distribuição de Tensões: Caso e <= B/6, o diagrama sob a sapata permanece inteiramente comprimido (trapezoidal). Se B/6 < e < B/3, ativa-se a redistribuição triangular em zona comprimida x_R = 3·(B/2 - e) com σ_máx = 2·N / (3·x_R).
• Capacidade de Carga da Fundação: Avaliação pelos fatores Nc, Nq, Nγ de Hansen/Meyerhof com fator de segurança global contra ruptura FS_cap >= 2,00.

## Dimensionamento Estrutural em Concreto Armado (ABNT NBR 6118)
• Parâmetros de Cálculo: Majoração γ_f = 1,40, minoração do concreto γ_c = 1,40, minoração do aço γ_s = 1,15 e cobrimento nominal segundo Classe de Agressividade Ambiental (CAA I a IV).
• Flexão Simples em ELU: Verificação de ductilidade (ξ = x/d <= 0,45), armadura mínima (0,15% · bw · h), força cortante resistente sem estribos (V_Rd1 >= V_d) e controle de fissuração em serviço (ELS-W: w_k <= 0,30 mm) para fuste, ponta e calcanhar."#,
            },
            ManualSection {
                id: "cap4",
                title: "4. Catálogo SPT, Banco SQLite & Automação CLI Completa",
                subtitle: "Uso do banco de solos NBR 6122/6484 e referência exaustiva de comandos do geoestabil-cli",
                content: r#"==== CATÁLOGO GEOTÉCNICO E AUTOMAÇÃO CLI COMPLETA ====

## Catálogo de Solos em SQLite (geoestabil.db)
O GeoEstabil armazena em banco SQLite local (%APPDATA%\GeoEstabil\geoestabil.db) uma biblioteca de solos brasileiros típicos com correlações referenciais entre o ensaio N_SPT (ABNT NBR 6484), peso específico γ, coesão efetiva c', ângulo de atrito φ' e tensão admissível σ_adm conforme ABNT NBR 6122. Abra o diálogo do catálogo pelo menu Ferramentas ou pelo botão pill 🗄 na barra de status para consultar, cadastrar, editar e transferir parâmetros para o projeto ativo.

## O Executável Autônomo de Linha de Comando (geoestabil-cli.exe)
O GeoEstabil acompanha um binário de console autônomo (geoestabil-cli.exe) para processamento em lote, integração em scripts PowerShell/Bash e execução de análises sem abrir a interface gráfica.

## Sintaxe Geral e Parâmetros Globais
```powershell
geoestabil-cli <comando> [argumentos] [opções]
```
• Opções globais: `--help` ou `-h` (exibe tela de ajuda), `--version` ou `-v` (exibe versão e normas) e `--lang <pt|en>` ou `-l <pt|en>` (define o idioma das mensagens e dos relatórios gerados).

## Referência Completa de Todos os Comandos do CLI
1. `geoestabil-cli presets`: Lista em tabela detalhada os 4 casos de benchmark pré-configurados (índice 1 a 4, título localizado, descrição geotécnica completa e modo ativo: Talude ou Muro).
2. `geoestabil-cli run-preset <1..4> [--output | -o <arquivo>] [--lang | -l <pt|en>]`:
   - Executa imediatamente o preset especificado (1 a 4).
   - Se for caso de Talude: roda a varredura multithreaded Rayon, exibe quantidade de círculos testados, tempo de CPU em ms, coordenadas e raio do círculo crítico (Xc, Yc, R) e Fatores de Segurança por Bishop, Fellenius e Janbu com validação frente à NBR 11682.
   - Se for caso de Muro: calcula empuxo Ea, fatores de tombamento e deslizamento, excentricidade, tensão no solo, capacidade de carga Hansen e armadura requerida do fuste (cm²/m e bitola comercial).
   - Flag `--output` / `-o`: grava o relatório completo no formato determinado pela extensão: `.html` (relatório técnico ABNT), `.pdf` (documento PDF nativo), `.md` (memorial Markdown com KaTeX) ou `.svg` (desenho técnico vetorial escalável).
3. `geoestabil-cli slope --input | -i <arquivo.geoestabil> [--output | -o <arquivo>] [--lang | -l <pt|en>]`:
   - Carrega um arquivo de projeto de talude (.geoestabil) salvo pela interface gráfica.
   - Executa a varredura paralela Rayon e imprime no terminal os Fatores de Segurança Bishop, Fellenius e Janbu, tempo de processamento e dados do círculo crítico.
   - Quando `--output` for informado, exporta o Memorial de Cálculo no formato especificado (.html, .pdf, .md ou .svg).
4. `geoestabil-cli wall --input | -i <arquivo.geoestabil> [--output | -o <arquivo>] [--lang | -l <pt|en>]`:
   - Carrega um projeto de muro de contenção (.geoestabil).
   - Executa a verificação geotécnica externa e o dimensionamento estrutural de concreto armado conforme NBR 6122 e NBR 6118, exibindo FS_tomb, FS_desl e a armadura do fuste.
   - Opcionalmente grava o memorial analítico via `--output`.
5. `geoestabil-cli soils [--lang | -l <pt|en>]`:
   - Imprime no terminal a tabela completa de solos típicos da NBR 6122 (Nome, faixa de N_SPT, peso específico natural γ em kN/m³, ângulo de atrito φ' em graus, coesão c' em kPa e tensão admissível σ_adm em kPa).
6. `geoestabil-cli version`:
   - Exibe a versão oficial do pacote (v2026.11.11), referências normativas e canal oficial de suporte.
7. `geoestabil-cli help`:
   - Exibe o guia completo de sintaxe e exemplos de linha de comando.

## Variáveis de Ambiente Suportadas
• `GEOESTABIL_RENDERER`: Seleciona o backend gráfico da aplicação desktop (`glow` para OpenGL leve com consumo de VRAM de ~25-40 MB, ou `wgpu` para WebGPU/DX12).
• `WGPU_POWER_PREF`: Configura o consumo de energia no wgpu (`low` para GPU integrada/economia de energia ou `high` para GPU dedicada).

## Exemplos Práticos de Automação
```powershell
# Listar todos os casos de estudo disponíveis
geoestabil-cli presets

# Executar caso #1 e gerar memorial em PDF em português
geoestabil-cli run-preset 1 --output memorial_caso1.pdf --lang pt

# Executar caso #2 e gerar memorial em HTML em inglês
geoestabil-cli run-preset 2 --output report_case2.html --lang en

# Analisar talude personalizado e exportar relatório
geoestabil-cli slope --input ./obras/talude01.geoestabil --output ./relatorios/talude01.html

# Analisar muro em balanço e exportar desenho vetorial SVG
geoestabil-cli wall -i ./obras/muro01.geoestabil -o ./desenhos/muro01.svg

# Consultar catálogo de solos no terminal
geoestabil-cli soils --lang pt
```"#,
            },
            ManualSection {
                id: "cap5",
                title: "5. Memoriais de Cálculo & Uso Acadêmico",
                subtitle: "Exportação Markdown/HTML/PDF e limites de uso (conferência externa obrigatória)",
                content: r#"==== MEMORIAIS DE ESTUDO E LIMITES DE USO ====

## Exportação do Memorial (Aba 5 / Menu Arquivo)
O módulo MemorialGenerator organiza as hipóteses geométricas, parâmetros geotécnicos adotados por camada, resultados dos métodos de Fellenius, Bishop e Janbu, coordenadas e raio do círculo crítico, verificações externas de muros e sugestões de armadura baseadas nas formulações de referência ABNT NBR 11682:2009, NBR 6122:2022 e NBR 6118:2023.

O relatório de estudo pode ser salvo em:
• HTML5 (.html): Relatório responsivo com diagramas integrados e formatação para impressão em folha A4;
• PDF (.pdf): Documento PDF nativo compilado pelo motor do aplicativo pronto para documentação e arquivamento;
• Markdown (.md): Texto estruturado em GitHub Flavored Markdown com equações em LaTeX KaTeX e tabelas analíticas;
• SVG Vetorial (.svg): Desenho em vetor geométrico puro para importação em softwares de CAD e ilustração técnica.

## Aviso de Uso Acadêmico e Isenção ('AS IS')
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
1. Código-Fonte Público (GitHub): Licenciado sob GNU AGPLv3 (AGPL-3.0-only), permitindo livre uso, estudo, modificação e compartilhamento sob a mesma licença, com proteção nominal de nome e marca 'GeoEstabil' (Seção 7(e) AGPLv3).
2. Edição Comercial (Microsoft Store): Os pacotes oficiais são regidos pelo EULA Comercial e CLC (disponíveis integralmente na janela Sobre, atalho Ctrl+I), sem obrigação de abertura de código da GNU AGPLv3 ao adquirente comercial.

## Privacidade Local (100% Offline)
• Execução 100% local e offline: O GeoEstabil NÃO coleta telemetria, métricas de uso, projetos ou dados pessoais em servidores externos.
• Arquivos .geoestabil, banco geoestabil.db e memoriais permanecem exclusivamente no computador do usuário.

## Canal Único de Suporte (Sem Urgência / AS IS)
• Todo eventual suporte ocorre exclusivamente por e-mail (linceu_lighthouse@outlook.com), conforme a disponibilidade do autor e sem obrigação de urgência, prazo ou SLA (sem telefone, acesso remoto ou consultoria).
• Software fornecido no estado em que se encontra ('AS IS'), com isenção integral por uso indevido."#,
            },
        ],
        Language::EnUs => vec![
            ManualSection {
                id: "cap1",
                title: "1. Overview & Windows 11 Workspace",
                subtitle: "Tabbed workflow, top menus, toolbar, shortcuts, and Modeless Z-Order windows",
                content: r#"==== GEOESTABIL PLATFORM OVERVIEW ====

GeoEstabil unifies lightweight vector rendering via Glow (OpenGL 3.3/WGL) with technical engineering modules for Rayon-parallelized Slope Stability analysis (ABNT NBR 11682:2009), Retaining Wall & Foundation Checks (ABNT NBR 6122:2022), Reinforced Concrete Design (ABNT NBR 6118:2023), local SQLite SPT soil cataloging, and automated calculation memorial export in a native Windows 11 Rust + egui/eframe desktop application.

## Five Main Workspace Tabs
• Tab 1 — Stratigraphy & Soils: Manage project metadata, soil layer horizons (c', φ', γ, γ_sat), foundation allowable pressure (σ_adm), crest surcharge loads, and the local SQLite SPT database.
• Tab 2 — Slope Stability: Configure topographic surface points (X, Y), phreatic water line, crest surcharge loads, and execute multi-threaded Rayon circle grid searches with a live Factor of Safety (FS) Heatmap, vertical slices, and failure arc.
• Tab 3 — Retaining Wall: Design Reinforced Concrete Cantilever walls (stem + continuous footing with toe and heel), Cyclopean Concrete Gravity walls, and Stepped Gabion walls under Rankine or Coulomb active/passive lateral earth pressures.
• Tab 4 — Foundations & Structural Design (NBR 6118): Verify the 5 external geotechnical stability criteria (overturning, sliding, eccentricity, base pressure, and Hansen/Meyerhof bearing capacity) and detail flexural steel, shear resistance V_Rd1 >= V_d, and crack control SLS-W.
• Tab 5 — Calculation Memorial: Edit study metadata and export traceable analytical calculation reports in HTML5/PDF (A4 with embedded diagrams), Markdown (.md), or scalable vector drawings (.svg).

## Top Menus and System Actions
• File Menu: Project file management and report exports: New Project (resets data), Open Project (loads .geoestabil), Save and Save As (writes structured JSON project), Export Memorial HTML/PDF, Export Markdown, and Export Vector SVG.
• Tools Menu: Fast access to engineering benchmarks and databases: Benchmark Presets (presets 1 to 4 for rapid benchmarking), SPT Soil Catalog (opens SQLite database), Toggle Active Mode (Slope vs Wall), and Recalculate Stability (forces immediate recomputation).
• Window Menu: Appearance and viewport controls: Toggle Light / Dark Theme (switches high-contrast palettes without layout shifts), Interface Language (switches between Portuguese BR and English US in real time), and Reset Canvas Zoom/Pan.
• Help Menu: Opens the User Manual (shortcut key F1 or Ctrl+M) and the 9-tab About GeoEstabil dialog (shortcut Ctrl+I, providing full documentation, EULA, CLC, privacy policy, and ABNT bibliographic references).

## Fast Access Toolbar
• Quick Action Buttons: New Project (📄), Open (📂), and Save (💾).
• Operational Mode Selector: Toggle between Slope Stability (⛰) and Retaining Wall (🧱).
• Live Telemetry Display: Monitors real-time critical Bishop Factor of Safety (for slopes) or active lateral thrust Ea and overturning/sliding safety factors (for retaining walls), color-coded for safety.
• Quick Theme Switcher: Sun/moon toggle button to switch dynamically between technical Dark Mode and clean Light Mode.

## Modeless Z-Order Windows & Shortcuts
• F1 or Ctrl+M: Open or focus the User Manual in an independent floating modeless window.
• Ctrl+I: Open the 9-tab About window (EULA, CLC, Privacy Policy, Copyright, ABNT Bibliographic References under Third-Party Notices, and OSS Licenses).
• Bottom status-bar window pills with ▲ indicators cleanly cycle Z-Order depth between open windows with a single click."#,
            },
            ManualSection {
                id: "cap2",
                title: "2. Slope Stability Analysis (ABNT NBR 11682)",
                subtitle: "Profile geometry, water table, Rayon multi-threaded sweep, Fellenius, Bishop & Janbu",
                content: r#"==== SLOPE STABILITY SOLVER GUIDE ====

## Modeling Steps in Tab 2
1. Topography & Soil Stratigraphy: Enter surface polyline points (X, Y) and layer elevations. For each soil stratum define unit weight (γ), saturated weight (γ_sat), effective cohesion (c'), and friction angle (φ').
2. Phreatic Surface & Surcharge: Define the water table polyline. Hydrostatic pore pressure u = γ_w · h_w reduces effective normal stress at each submerged slice base. Apply optional uniform surface load q (kPa) along the crest or berm.
3. Required Factor of Safety (NBR 11682 Table 1): Select required safety level by failure consequence severity (High FS >= 1.50; Medium FS >= 1.30; Low FS >= 1.20).
4. Parallel Rayon Sweep & 2D Canvas: Candidate (Xc, Yc, R) circles are evaluated across CPU cores. On the 2D Canvas:
   - Scroll mouse wheel for dynamic cursor-centered zoom;
   - Click and drag with left mouse button to pan;
   - Observe color-coded candidate centers on the FS Heatmap (red = critical, green = safe) and the critical failure arc with its vertical slices;
   - Click 'Fit View' to frame the full model or 'Reset View' to restore initial scale.

## Limit Equilibrium Formulations
• Ordinary Method (Fellenius): Closed-form conservative baseline formulation assuming zero inter-slice forces, serving as an initial benchmark for circular slip surfaces in geotechnical earth masses.
• Simplified Bishop (Primary Standard): Iterative fixed-point solution with m_α >= 0.20 guard preventing numerical blow-up near steep toe slices.
• Simplified Janbu: Horizontal force equilibrium corrected by empirical shape factor f0 based on the depth-to-length ratio (d/L)."#,
            },
            ManualSection {
                id: "cap3",
                title: "3. Retaining Walls — Gravity & RC Design (NBR 6122 / 6118)",
                subtitle: "Rankine, Coulomb, overturning, sliding, bearing capacity & RC detailing",
                content: r#"==== RETAINING WALL ANALYSIS & DESIGN ====

## Supported Wall Typologies in Tab 3
• Reinforced Concrete Cantilever Wall: Stem + Toe + Heel continuous footing actively leveraging backfill soil weight over the heel for overturning and sliding stability.
• Gravity Cyclopean Concrete: Stability derived entirely from trapezoidal plain concrete self-weight (γ_conc ~ 22 to 24 kN/m³).
• Stepped Gabion Wall: Free-draining wire-basket stone mass (γ_gab ~ 17 to 19 kN/m³) dissipating hydrostatic pore pressures behind the backface.

## Earth Pressure Theories
• Rankine: Assumes a frictionless vertical backface (δ = 0) with a horizontal or inclined backfill slope (β), calculating active Ka and passive Kp coefficients.
• Coulomb: Accounts for wall-soil interface friction δ (typically 1/2 φ' to 2/3 φ') and stem backface batter angle ψ, generating a stabilizing downward vertical component.
• Surcharge & Water Thrust: Computes uniform surcharge thrust Ea_q = q · Ka · H and hydrostatic water thrust Ea_w when the water table is elevated above the footing.

## External Checks (ABNT NBR 6122:2022)
• Overturning Stability: FS_overturning = M_res / M_ovt >= 1.50.
• Base Sliding Stability: FS_sliding = (N·tan δ_b + c_a·B + 0.5·Ep + E_key) / Ea_h >= 1.50 (with optional foundation shear key).
• Base Kern Eccentricity: Calculates e = |B/2 - x_R|. When e <= B/6, the base remains fully in compression under a trapezoidal diagram. When B/6 < e < B/3, triangular stress redistribution with partial base uplift is computed with σ_max = 2·N / (3·x_R).
• Foundation Ultimate Bearing Capacity: Evaluates Hansen/Meyerhof capacity factors Nc, Nq, Nγ, enforcing a global safety factor FS_bearing >= 2.00.

## RC Structural Checks (ABNT NBR 6118:2023)
• Calculation Factors: Partial safety factors γ_f = 1.40, γ_c = 1.40, γ_s = 1.15, and nominal concrete cover per Environmental Aggressiveness Class (CAA I to IV).
• ULS Pure Flexure: Enforces ductility limits (ξ = x/d <= 0.45 in Domain 3), minimum flexural steel ratio (0.15% · bw · h), shear without stirrups (V_Rd1 >= V_d), and crack control (SLS-W: w_k <= 0.30 mm) for stem, toe, and heel sections."#,
            },
            ManualSection {
                id: "cap4",
                title: "4. SPT Soil Catalog, SQLite & Complete CLI Automation",
                subtitle: "NBR 6122/6484 soil library and exhaustive reference of all geoestabil-cli commands",
                content: r#"==== SOIL LIBRARY & COMPLETE CLI AUTOMATION ====

## Local SQLite Soil Catalog (geoestabil.db)
GeoEstabil stores typical Brazilian soil references correlated to the Standard Penetration Test N_SPT index (ABNT NBR 6484) in a local SQLite database (%APPDATA%\GeoEstabil\geoestabil.db), providing unit weight γ, effective cohesion c', friction angle φ', and allowable bearing pressure σ_adm according to ABNT NBR 6122. Open the Soil Catalog dialog via the Tools menu or bottom status pill 🗄 to inspect, insert, edit, and apply soil properties directly to active project layers.

## The Standalone Command-Line Binary (geoestabil-cli.exe)
GeoEstabil provides a dedicated command-line executable (geoestabil-cli.exe) for batch processing, automated parametric sweeps, and script integration in PowerShell/Bash pipelines without loading the graphical user interface.

## General Syntax and Global Options
```powershell
geoestabil-cli <command> [arguments] [options]
```
• Global options: `--help` or `-h` (displays help reference), `--version` or `-v` (displays version and standards), and `--lang <pt|en>` or `-l <pt|en>` (selects output message and report language).

## Exhaustive Reference of All CLI Commands
1. `geoestabil-cli presets`: Prints a formatted benchmark table showing all 4 built-in reference cases (preset index 1 to 4, localized title, engineering scenario description, and active solver mode: Slope or Wall).
2. `geoestabil-cli run-preset <1..4> [--output | -o <file>] [--lang | -l <pt|en>]`:
   - Immediately executes the pre-configured benchmark case specified by index (1 to 4).
   - For Slope cases: runs the Rayon multi-threaded grid search, printing evaluated trial circles, elapsed CPU time in ms, critical circle coordinates (Xc, Yc, R), and Bishop, Fellenius, and Janbu Factors of Safety validated against NBR 11682.
   - For Retaining Wall cases: computes active lateral thrust Ea, overturning and sliding factors, base eccentricity, soil stresses, Hansen bearing capacity, and required stem rebar (cm²/m and rebar sizing).
   - `--output` / `-o` flag: writes the full calculation memorial in the format inferred by the file extension: `.html` (ABNT technical report), `.pdf` (native PDF document), `.md` (Markdown memorial with KaTeX), or `.svg` (scalable engineering vector drawing).
3. `geoestabil-cli slope --input | -i <project.geoestabil> [--output | -o <file>] [--lang | -l <pt|en>]`:
   - Loads a slope stability project file (.geoestabil) saved from the desktop GUI.
   - Executes Rayon multi-threaded slope stability analysis across all CPU cores and prints computed Factors of Safety (Bishop, Fellenius, Janbu), processing time, and critical circle geometry.
   - When `--output` is supplied, exports the complete calculation memorial in the specified format (.html, .pdf, .md, or .svg).
4. `geoestabil-cli wall --input | -i <project.geoestabil> [--output | -o <file>] [--lang | -l <pt|en>]`:
   - Loads a retaining wall project file (.geoestabil).
   - Executes external geotechnical stability checks and structural reinforced concrete design per NBR 6122 and NBR 6118, displaying FS_overturning, FS_sliding, and stem rebar detailing.
   - Optionally saves the analytical report to disk via `--output`.
5. `geoestabil-cli soils [--lang | -l <pt|en>]`:
   - Prints a formatted terminal table containing all standard ABNT NBR 6122 soils and their Standard Penetration Test N_SPT correlation ranges (soil name, SPT range, natural unit weight γ in kN/m³, friction angle φ' in degrees, cohesion c' in kPa, and allowable bearing pressure σ_adm in kPa).
6. `geoestabil-cli version`:
   - Displays the official package version (v2026.11.11), referenced engineering standards, and author support contact.
7. `geoestabil-cli help`:
   - Displays the command-line usage reference and practical syntax examples.

## Supported Environment Variables
• `GEOESTABIL_RENDERER`: Sets the graphical backend for the desktop GUI (`glow` for lightweight OpenGL with ~25-40 MB VRAM footprint, or `wgpu` for WebGPU/DX12).
• `WGPU_POWER_PREF`: Configures power preference when running under wgpu (`low` for power-saving on integrated GPUs or `high` for discrete performance GPUs).

## Practical Command-Line Examples
```powershell
# List all built-in benchmark engineering cases
geoestabil-cli presets

# Execute benchmark case #1 and generate a PDF memorial in Portuguese
geoestabil-cli run-preset 1 --output memorial_case1.pdf --lang pt

# Execute benchmark case #2 and generate an HTML memorial in English
geoestabil-cli run-preset 2 --output report_case2.html --lang en

# Analyze a custom slope project and export an analytical report
geoestabil-cli slope --input ./projects/slope01.geoestabil --output ./reports/slope01.html

# Analyze a cantilever retaining wall and export a vector SVG drawing
geoestabil-cli wall -i ./projects/wall01.geoestabil -o ./drawings/wall01.svg

# Query the NBR 6122 soil catalog in the terminal
geoestabil-cli soils --lang en
```"#,
            },
            ManualSection {
                id: "cap5",
                title: "5. Calculation Memorials & Academic Scope",
                subtitle: "Markdown/HTML/PDF export and verification limits (external check required)",
                content: r#"==== CALCULATION MEMORIALS & DISCLAIMER ====

## Calculation Memorial Export (Tab 5 / File Menu)
The MemorialGenerator module compiles geometric hypotheses, stratum soil parameters, Fellenius, Bishop, and Janbu results, critical circle geometry, retaining wall external stability checks, and reinforcement schedules based on ABNT NBR 11682:2009, NBR 6122:2022, and NBR 6118:2023.

Study reports can be exported in:
• HTML5 (.html): Responsive technical report with embedded vector diagrams and styling formatted for A4 printing;
• PDF (.pdf): Native PDF document compiled directly by GeoEstabil's internal engine ready for study records and archiving;
• Markdown (.md): Clean plain-text file in GitHub Flavored Markdown with KaTeX mathematical formulas and analytical tables;
• Vector SVG (.svg): Scalable vector drawing for technical illustration and import into CAD environments.

## Academic Scope and Disclaimer ('AS IS')
GeoEstabil is intended strictly for academic study, teaching, and computational experimentation, and may contain approximations or numerical tolerances. It MUST NOT be relied upon as definitive software in production, civil works, or professional projects without:
• Full manual verification of all equations, hypotheses, and analytical results;
• Mandatory cross-checking against established commercial market tools and site-specific field investigations;
• Technical supervision and filing of formal professional responsibility records (such as ART/RRT) by a licensed professional engineer registered with CREA/CAU, wholly exempting the author from improper or production misuse."#,
            },
            ManualSection {
                id: "cap6",
                title: "6. Dual Licensing, Local Privacy & Email Support",
                subtitle: "GitHub GNU AGPLv3 vs. Microsoft Store EULA/CLC & Availability-Based Support",
                content: r#"==== DUAL LICENSING, PRIVACY & SUPPORT ====

## Dual-Licensing Architecture
1. Public Open-Source GitHub Repository: Licensed under GNU AGPLv3 (SPDX: AGPL-3.0-only), permitting free use, study, modification, and reciprocal sharing while protecting the 'GeoEstabil' name and trademark under Section 7(e) of AGPLv3.
2. Official Commercial Edition on the Microsoft Store: Pre-compiled official binaries distributed through the store are governed by our Commercial EULA and CLC (readable in full in the About dialog, shortcut Ctrl+I), exempting commercial purchasers from copyleft source-disclosure obligations.

## 100% Offline Privacy (Zero Telemetry)
• 100% local and offline execution: GeoEstabil collects zero telemetry, zero usage analytics, and zero personal data on external servers.
• Project files (.geoestabil), SQLite database (geoestabil.db), and calculation reports remain exclusively on your local computer.

## Official Technical Support Channel (No Urgency / AS IS)
• Support is provided exclusively via email (linceu_lighthouse@outlook.com), strictly according to the author's availability and with no obligation of urgency, deadlines, or service level agreements (no phone, remote access, or consultancy).
• Software provided strictly on an 'AS IS' basis, with complete liability exemption for improper or production use."#,
            },
        ],
    }
}
