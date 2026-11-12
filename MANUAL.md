# Manual Técnico de Engenharia / Geotechnical & Structural Engineering Manual — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details open>
<summary><b>Clique para expandir / recolher o manual técnico em português</b></summary><br>

# GeoEstabil — Manual Técnico de Formulações e Engenharia

**Versão:** `2026.11.12.0`  
**Data:** `12 de novembro de 2026`  
**Normas Referenciadas:** ABNT NBR 11682:2009 (*Estabilidade de encostas*), ABNT NBR 6122:2022 (*Projeto e execução de fundações*) e ABNT NBR 6118:2023 (*Projeto de estruturas de concreto*)  
**Normas Correlatas:** ABNT NBR 6484:2020 (*Sondagem SPT*), NBR 6502:2022 (*Rochas e solos*), NBR 6120:2019 (*Ações*), NBR 7480:2022 (*Aço CA-50*) e NBR 13133:2021 (*Levantamento topográfico*)  
**Autor e Canal de Suporte (somente e-mail, sem urgência):** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  

---

### 1. Visão Geral do Sistema

O **GeoEstabil** é uma ferramenta desktop auxiliar voltada a **estudo acadêmico, ensino e experimentação computacional** em engenharia geotécnica e estrutural, desenvolvida nativamente em **Rust (Edição 2021)** com interface imediata em **`egui` / `eframe`**, renderização vetorial leve em **Glow (OpenGL 3.3/WGL)**, paralelismo em CPU via **Rayon** e banco local **SQLite3**. O sistema executa simulações de estabilidade de taludes pelo método das fatias, cálculos de empuxos de terra, verificações de estabilidade externa e dimensionamento didático em concreto armado para muros de arrimo.

---

### 2. Equações e Métodos Analíticos Implementados

#### 2.1. Estabilidade de Taludes — Equilíbrio Limite (ABNT NBR 11682:2009)
- **Critério de Resistência de Mohr-Coulomb:**
  $$\tau_f = c' + (\sigma - u) \cdot \tan\phi' \quad [kPa]$$
  onde $c'$ é a coesão efetiva, $\phi'$ é o ângulo de atrito efetivo e $u = \gamma_w \cdot h_w$ é a poropressão hidrostática na base da fatia.
- **Método de Fellenius (Ordinário / Sueco):**
  $$FS = \frac{\sum \left[ c' \cdot \ell_i + (W_i \cos\alpha_i - u_i \cdot \ell_i)\tan\phi' \right]}{\sum W_i \sin\alpha_i}$$
- **Método de Bishop Simplificado (Iterativo com $m_{\alpha}$):**
  $$FS = \frac{\sum \frac{c' \cdot b_i + (W_i - u_i \cdot b_i)\tan\phi'}{m_{\alpha,i}}}{\sum W_i \sin\alpha_i}, \quad m_{\alpha,i} = \cos\alpha_i \left(1 + \frac{\tan\alpha_i \cdot \tan\phi'}{FS}\right)$$
- **Método de Janbu Simplificado (com fator corretivo de forma $f_0$):**
  $$FS = f_0 \cdot \frac{\sum \frac{c' \cdot b_i + (W_i - u_i \cdot b_i)\tan\phi'}{n_{\alpha,i}}}{\sum W_i \tan\alpha_i}, \quad n_{\alpha,i} = \cos^2\alpha_i \left(1 + \frac{\tan\alpha_i \cdot \tan\phi'}{FS}\right)$$
- **Critérios Mínimos da Tabela 1 da ABNT NBR 11682:2009:**
  - Nível de Segurança **Alto**: $FS \ge 1{,}50$;
  - Nível de Segurança **Médio**: $FS \ge 1{,}30$;
  - Nível de Segurança **Baixo**: $FS \ge 1{,}20$.

#### 2.2. Empuxos de Terra (Rankine e Coulomb)
- **Teoria de Rankine ($\delta = 0$):**
  $$K_a = \tan^2\left(45^\circ - \frac{\phi'}{2}\right), \quad K_p = \tan^2\left(45^\circ + \frac{\phi'}{2}\right)$$
- **Teoria de Coulomb (atrito solo-muro $\delta$, inclinação do tardoz $\theta$ e talude de montante $\beta$):**
  $$K_a = \frac{\cos^2(\phi' - \theta)}{\cos^2\theta \cdot \cos(\theta + \delta) \left[1 + \sqrt{\frac{\sin(\phi' + \delta)\sin(\phi' - \beta)}{\cos(\theta + \delta)\cos(\theta - \beta)}}\right]^2}$$
- **Componentes Atuantes Totais:**
  - Empuxo do solo (seco/submerso), empuxo da sobrecarga uniforme $E_{a,q} = q \cdot K_a \cdot H$ e empuxo hidrostático da água $E_w = \frac{1}{2}\gamma_w h_w^2$;
  - Profundidade de fenda de tração em solos coesivos: $z_0 = \frac{2c'}{\gamma\sqrt{K_a}}$.

#### 2.3. Estabilidade Externa e Fundações (ABNT NBR 6122:2022)
- **Tombamento:** $FS_{\text{tomb}} = \frac{M_{\text{est}}}{M_{\text{tomb}}} \ge 1{,}50$;
- **Deslizamento na Base (com $50\%$ do passivo frontal + dente de ancoragem opcional):** $FS_{\text{desl}} = \frac{N \tan\delta_b + c_a B + 0{,}5 E_p + E_{\text{dente}}}{E_{a,h}} \ge 1{,}50$;
- **Excentricidade e Núcleo Central de Inércia:**
  $$e = \left|\frac{B}{2} - x_R\right| \le \frac{B}{6} \implies \sigma = \frac{N}{B}\left(1 \pm \frac{6e}{B}\right); \quad \text{para } e > \frac{B}{6}: \sigma_{\max} = \frac{2N}{3 x_R}$$
- **Capacidade de Carga Última (Hansen / Meyerhof):**
  $$q_{\text{ult}} = c \cdot N_c \cdot i_c + q_0 \cdot N_q \cdot i_q + 0{,}5 \cdot \gamma \cdot B' \cdot N_\gamma \cdot i_\gamma, \quad B' = B - 2e, \quad FS_{\text{cap}} = \frac{q_{\text{ult}}}{\sigma_{\text{toe}}} \ge 2{,}00$$

#### 2.4. Dimensionamento Estrutural em Concreto Armado (ABNT NBR 6118:2023)
- **Flexão Simples em ELU (Seções do Fuste, Ponta e Calcanhar):**
  $$\mu = \frac{M_d}{b \cdot d^2 \cdot \alpha_c \cdot f_{cd}}, \quad \xi = \frac{1 - \sqrt{1 - 2\mu}}{\lambda} \le 0{,}45 \text{ (Domínio 3 dúctil)}, \quad A_s = \frac{M_d}{f_{yd} \cdot d(1 - 0{,}4\xi)}$$
- **Taxa Mínima e Cisalhamento sem Estribos ($V_{Rd1}$):** $\rho_{\min} = 0{,}15\%$ e verificação $V_{Rd1} \ge V_d$, controle de fissuração ($w_k \le 0{,}30\text{ mm}$) e armadura horizontal de distribuição ($A_{s,\text{dist}} \ge 20\% A_s$).

---

### 3. As 5 Abas de Trabalho de Engenharia

1. **Aba 1 — Estratigrafia & Solos:** Cadastro de camadas Mohr-Coulomb ($\gamma, \gamma_{\text{sat}}, c', \phi', \sigma_{\text{adm}}$), banco de solos NBR 6122/SPT, lençol freático, sobrecarga na crista e enquadramento na NBR 11682;
2. **Aba 2 — Estabilidade de Taludes:** Método primário (Bishop/Fellenius/Janbu), malha Grid Search, varredura Rayon multithreaded, pontos do perfil topográfico e canvas 2D interativo com Heatmap FS e fatias;
3. **Aba 3 — Muro de Contenção:** Tipologia (Cantilever ou Gravidade), teoria (Rankine ou Coulomb), dimensões paramétricas, dente de cisalhamento, solo de reaterro e fundação, com diagramas proporcionais em tempo real;
4. **Aba 4 — Fundações & Estrutural (NBR 6118):** Cards de segurança externa ($FS_{\text{tomb}}, FS_{\text{desl}}, e, \sigma_{\text{toe}}, q_{\text{ult}}$ Hansen), materiais ($f_{ck}, f_{yk}, c_{\text{nom}}, \gamma_f$) e detalhamento completo de armaduras do fuste, ponta e calcanhar;
5. **Aba 5 — Memorial de Cálculo:** Dados de identificação do estudo, pré-visualização em tela e exportação para **HTML5/PDF (A4)**, **Markdown (`.md`)** e **SVG vetorial (`.svg`)**.

---

### 4. Automação por Linha de Comando (`geoestabil-cli.exe`)

O GeoEstabil disponibiliza o executável independente **`geoestabil-cli.exe`** para execução em lote e integração em rotinas automatizadas sem interface gráfica:
- **`presets`**: Lista os 4 casos de benchmark integrados com índice, título, descrição de engenharia e modo ativo (Talude ou Muro);
- **`run-preset <1..4> [--output | -o <arquivo>] [--lang | -l <pt|en>]`**: Executa o caso pré-configurado e exporta memorial em `.html`, `.pdf`, `.md` ou `.svg`;
- **`slope --input | -i <proj.geoestabil> [--output | -o <relatorio>] [--lang | -l <pt|en>]`**: Executa varredura multithreaded Rayon em arquivo de talude e grava relatório;
- **`wall --input | -i <proj.geoestabil> [--output | -o <relatorio>] [--lang | -l <pt|en>]`**: Executa estabilidade externa (tombamento/deslizamento) e armadura do fuste de muro;
- **`soils [--lang | -l <pt|en>]`**: Imprime tabela dos solos típicos da ABNT NBR 6122 correlacionados ao índice de penetração $N_{\text{SPT}}$ (NBR 6484);
- **`version`** (`--version` / `-v`): Exibe versão semântica (`v2026.11.12`), normas centrais e dados de autoria;
- **`help`** (`--help` / `-h`): Exibe o guia completo de sintaxe e opções no terminal;
- **Variáveis de Ambiente**: `GEOESTABIL_RENDERER=glow` (OpenGL de baixo consumo de VRAM) ou `wgpu`, e `WGPU_POWER_PREF=low` ou `high`.

---

### 5. Aviso de Uso Acadêmico, Suporte por E-mail e Isenção

O **GeoEstabil** destina-se exclusivamente a estudo acadêmico, ensino e experimentação computacional, podendo conter falhas ou aproximações numéricas. **NÃO deve ser utilizado como software definitivo em produção, obras ou projetos reais** sem conferência manual completa, validação cruzada em outras ferramentas consolidadas, ensaios de campo e supervisão de profissional habilitado (incluindo eventual ART/RRT junto ao CREA/CAU). Fornecido **"no estado em que se encontra" ("AS IS")**, com suporte exclusivo via e-mail (**`linceu_lighthouse@outlook.com`**) conforme a disponibilidade do autor e sem obrigação de urgência, e com isenção integral por uso indevido ou aplicação produtiva por conta e risco do usuário.

Versão documentada: `2026.11.12.0`

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary><b>Click to expand / collapse the technical manual in English</b></summary><br>

# GeoEstabil — Technical Formulations & Engineering Manual

**Version:** `2026.11.12.0`  
**Date:** `November 12, 2026`  
**Referenced Standards:** ABNT NBR 11682:2009 (*Slope stability*), ABNT NBR 6122:2022 (*Design and execution of foundations*), and ABNT NBR 6118:2023 (*Design of concrete structures*)  
**Correlated Standards:** ABNT NBR 6484:2020 (*SPT sounding*), NBR 6502:2022 (*Rocks and soils*), NBR 6120:2019 (*Actions*), NBR 7480:2022 (*CA-50 steel*), and NBR 13133:2021 (*Topographic surveying*)  
**Author & Support Email (availability only, no urgency):** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  

---

### 1. System Overview

**GeoEstabil** is an auxiliary desktop tool intended for **academic study, teaching, and computational experimentation** in geotechnical and structural engineering, built natively in **Rust (Edition 2021)** with an immediate-mode interface powered by **`egui` / `eframe`**, lightweight vector rendering via **Glow (OpenGL 3.3/WGL)**, CPU parallelism via **Rayon**, and local **SQLite3** storage. The system runs limit-equilibrium slope stability slice simulations, lateral earth pressure calculations, external stability checks, and educational reinforced concrete sizing for retaining walls.

---

### 2. Analytical Methods & Formulations Implemented

#### 2.1. Slope Stability — Limit Equilibrium (ABNT NBR 11682:2009)
- **Mohr-Coulomb Shear Strength Criterion:**
  $$\tau_f = c' + (\sigma - u) \cdot \tan\phi' \quad [kPa]$$
  where $c'$ is effective cohesion, $\phi'$ is effective friction angle, and $u = \gamma_w \cdot h_w$ is hydrostatic pore-water pressure along the slice base.
- **Ordinary / Fellenius Method (Swedish Circle):**
  $$FS = \frac{\sum \left[ c' \cdot \ell_i + (W_i \cos\alpha_i - u_i \cdot \ell_i)\tan\phi' \right]}{\sum W_i \sin\alpha_i}$$
- **Simplified Bishop Method (Iterative with $m_{\alpha}$):**
  $$FS = \frac{\sum \frac{c' \cdot b_i + (W_i - u_i \cdot b_i)\tan\phi'}{m_{\alpha,i}}}{\sum W_i \sin\alpha_i}, \quad m_{\alpha,i} = \cos\alpha_i \left(1 + \frac{\tan\alpha_i \cdot \tan\phi'}{FS}\right)$$
- **Simplified Janbu Method (with correction shape factor $f_0$):**
  $$FS = f_0 \cdot \frac{\sum \frac{c' \cdot b_i + (W_i - u_i \cdot b_i)\tan\phi'}{n_{\alpha,i}}}{\sum W_i \tan\alpha_i}, \quad n_{\alpha,i} = \cos^2\alpha_i \left(1 + \frac{\tan\alpha_i \cdot \tan\phi'}{FS}\right)$$
- **Minimum Criteria from Table 1 of ABNT NBR 11682:2009:**
  - **High** safety level: $FS \ge 1.50$;
  - **Medium** safety level: $FS \ge 1.30$;
  - **Low** safety level: $FS \ge 1.20$.

#### 2.2. Lateral Earth Pressures (Rankine and Coulomb)
- **Rankine Theory ($\delta = 0$):**
  $$K_a = \tan^2\left(45^\circ - \frac{\phi'}{2}\right), \quad K_p = \tan^2\left(45^\circ + \frac{\phi'}{2}\right)$$
- **Coulomb Theory (wall-soil friction $\delta$, backface angle $\theta$, and backfill slope $\beta$):**
  $$K_a = \frac{\cos^2(\phi' - \theta)}{\cos^2\theta \cdot \cos(\theta + \ delta) \left[1 + \sqrt{\frac{\sin(\phi' + \delta)\sin(\phi' - \beta)}{\cos(\theta + \delta)\cos(\theta - \beta)}}\right]^2}$$
- **Total Active Force Components:**
  - Soil earth thrust (dry/submerged), uniform surcharge thrust $E_{a,q} = q \cdot K_a \cdot H$, and hydrostatic water wedge $E_w = \frac{1}{2}\gamma_w h_w^2$;
  - Tension crack depth in cohesive soils: $z_0 = \frac{2c'}{\gamma\sqrt{K_a}}$.

#### 2.3. External Stability & Foundations (ABNT NBR 6122:2022)
- **Overturning:** $FS_{\text{overturning}} = \frac{M_{\text{res}}}{M_{\text{ovt}}} \ge 1.50$;
- **Base Sliding (with $50\%$ passive toe resistance + optional shear key):** $FS_{\text{sliding}} = \frac{N \tan\delta_b + c_a B + 0.5 E_p + E_{\text{key}}}{E_{a,h}} \ge 1.50$;
- **Eccentricity & Middle-Third Kern:**
  $$e = \left|\frac{B}{2} - x_R\right| \le \frac{B}{6} \implies \sigma = \frac{N}{B}\left(1 \pm \frac{6e}{B}\right); \quad \text{for } e > \frac{B}{6}: \sigma_{\max} = \frac{2N}{3 x_R}$$
- **Ultimate Bearing Capacity (Hansen / Meyerhof):**
  $$q_{\text{ult}} = c \cdot N_c \cdot i_c + q_0 \cdot N_q \cdot i_q + 0.5 \cdot \gamma \cdot B' \cdot N_\gamma \cdot i_\gamma, \quad B' = B - 2e, \quad FS_{\text{bearing}} = \frac{q_{\text{ult}}}{\sigma_{\text{toe}}} \ge 2.00$$

#### 2.4. Reinforced Concrete Structural Design (ABNT NBR 6118:2023)
- **ULS Pure Flexure (Stem, Toe, and Heel Sections):**
  $$\mu = \frac{M_d}{b \cdot d^2 \cdot \alpha_c \cdot f_{cd}}, \quad \xi = \frac{1 - \sqrt{1 - 2\mu}}{\lambda} \le 0.45 \text{ (ductile Domain 3)}, \quad A_s = \frac{M_d}{f_{yd} \cdot d(1 - 0.4\xi)}$$
- **Minimum Ratio & Shear Without Stirrups ($V_{Rd1}$):** $\rho_{\min} = 0.15\%$ and verification $V_{Rd1} \ge V_d$, crack-width check ($w_k \le 0.30\text{ mm}$), and horizontal distribution reinforcement ($A_{s,\text{dist}} \ge 20\% A_s$).

---

### 3. The 5 Engineering Workspaces

1. **Tab 1 — Stratigraphy & Soils:** Mohr-Coulomb layer management ($\gamma, \gamma_{\text{sat}}, c', \phi', \sigma_{\text{adm}}$), NBR 6122/SPT soil catalog, water table, crest surcharge, and NBR 11682 classification;
2. **Tab 2 — Slope Stability:** Primary method (Bishop/Fellenius/Janbu), Grid Search bounds, multi-threaded Rayon sweep, topographic profile points, and interactive 2D canvas with FS Heatmap and vertical slices;
3. **Tab 3 — Retaining Wall:** Structural type (Cantilever or Gravity), earth pressure theory (Rankine or Coulomb), parametric geometry, shear key, backfill and foundation soils, with live proportional diagrams;
4. **Tab 4 — Foundations & Structural (NBR 6118):** External stability cards ($FS_{\text{overturning}}, FS_{\text{sliding}}, e, \sigma_{\text{toe}}, q_{\text{ult}}$ Hansen), materials ($f_{ck}, f_{yk}, c_{\text{nom}}, \gamma_f$), and complete rebar detailing for stem, toe, and heel;
5. **Tab 5 — Calculation Memorial:** Study identification fields, on-screen preview, and export to **HTML5/PDF (A4)**, **Markdown (`.md`)**, and **vector SVG (`.svg`)**.

---

### 4. Command-Line Interface Automation (`geoestabil-cli.exe`)

GeoEstabil provides the standalone **`geoestabil-cli.exe`** binary for headless batch processing and automated pipeline execution without the graphical UI:
- **`presets`**: Lists all 4 built-in benchmark cases with index, title, engineering description, and active solver mode (Slope or Wall);
- **`run-preset <1..4> [--output | -o <file>] [--lang | -l <pt|en>]`**: Executes the pre-configured case and exports calculation memorials in `.html`, `.pdf`, `.md`, or `.svg`;
- **`slope --input | -i <proj.geoestabil> [--output | -o <report>] [--lang | -l <pt|en>]`**: Executes multi-threaded Rayon slope sweeps from a project file and writes reports;
- **`wall --input | -i <proj.geoestabil> [--output | -o <report>] [--lang | -l <pt|en>]`**: Runs external stability checks (overturning/sliding) and stem rebar sizing for walls;
- **`soils [--lang | -l <pt|en>]`**: Prints a formatted table of standard ABNT NBR 6122 soils correlated to the $N_{\text{SPT}}$ penetration index (NBR 6484);
- **`version`** (`--version` / `-v`): Displays the semantic package version (`v2026.11.12`), core standards, and author credits;
- **`help`** (`--help` / `-h`): Displays the complete syntax guide and command options in the terminal;
- **Environment Variables**: `GEOESTABIL_RENDERER=glow` (low-VRAM OpenGL renderer) or `wgpu`, and `WGPU_POWER_PREF=low` or `high`.

---

### 5. Academic Scope, Email-Only Support & Liability Exemption

**GeoEstabil** is intended exclusively for academic study, teaching, and computational experimentation, and may contain defects or numerical approximations. **It MUST NOT be used as definitive software in production, civil works, or professional projects** without complete manual verification, cross-checking in established market tools, field testing, and oversight by a licensed professional (including any required ART/RRT filing before CREA/CAU). Provided **"AS IS"** with support solely via email (**`linceu_lighthouse@outlook.com`**) by maintainer availability without urgency, and complete author exemption from improper or production misuse.

Documented version: `2026.11.12.0`

</details>
