# GeoEstabil — Estabilidade de Taludes e Muros de Arrimo (ABNT NBR 11682 / NBR 6122 / NBR 6118)

**Ferramenta desktop desenvolvida em Rust (Edição 2021) para estudo acadêmico, ensino e análise computacional de estabilidade de encostas e taludes (Métodos de Fatias com paralelismo via Rayon), empuxos de terra (Rankine e Coulomb), verificação de fundações e dimensionamento didático em concreto armado de muros de contenção com base nas formulações de referência das normas ABNT NBR 11682:2009, NBR 6122:2022 e NBR 6118:2023. Conta com interface gráfica imediata (`egui`/`eframe`) e utilitário de linha de comando (`geoestabil-cli.exe`).**

[![Latest Release](https://img.shields.io/badge/Release-v2026.11.11.0-blue.svg)](docs/release-v2026.11.11.0.md)
[![Rust 2021](https://img.shields.io/badge/Rust-2021%20Edition-DEA584?logo=rust)](Cargo.toml)
[![egui 0.37](https://img.shields.io/badge/egui-0.37-5C7CFA?logo=rust)](Cargo.toml)
[![Rayon Parallel](https://img.shields.io/badge/Parallel-Rayon_CPU-orange.svg)](Cargo.toml)
[![Normas ABNT](https://img.shields.io/badge/Referências-NBR_11682_%7C_6122_%7C_6118-2ea44f.svg)](docs/compliance.md)
[![Platform Windows](https://img.shields.io/badge/Platform-Windows_11_%2F_10-0078D4?logo=windows)](https://apps.microsoft.com/detail/9P9BDZ9RHRC7)
[![Microsoft Store](https://img.shields.io/badge/Microsoft_Store-Download-0078D4?logo=microsoft)](https://apps.microsoft.com/detail/9P9BDZ9RHRC7)
[![License AGPLv3](https://img.shields.io/badge/License-GNU_AGPLv3-blue.svg)](LICENSE)
[![Dual License](https://img.shields.io/badge/License-Commercial%20Store%20EULA-purple.svg)](docs/licensing.md)
[![100% Offline](https://img.shields.io/badge/Security-100%25_Offline-success.svg)](docs/security-model.md)

<p align="center">
  <a href="docs/user-guide.md">📖 Guia do Usuário</a> •
  <a href="docs/architecture.md">🏗️ Arquitetura</a> •
  <a href="docs/compliance.md">📋 Referências Técnicas</a> •
  <a href="docs/security-model.md">🛡️ Privacidade & Segurança</a> •
  <a href="docs/cli-reference.md">💻 CLI</a> •
  <a href="docs/licensing.md">⚖️ Duplo Licenciamento</a> •
  <a href="https://apps.microsoft.com/detail/9P9BDZ9RHRC7">🪟 Microsoft Store</a><br>
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> •
  <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

> **Observação:** Este repositório refere-se à versão **`2026.11.11.0`** do **GeoEstabil** (implementação nativa em **Rust**), disponibilizada sob modelo simples de **Duplo Licenciamento** (**GNU AGPLv3** para o código-fonte público no GitHub e **EULA Comercial + CLC** para os binários oficiais distribuídos na Microsoft Store, com execução 100% local e offline). Apoie o projeto e instale a versão oficial assinada digitalmente através do link: [Instalar via Microsoft Store](https://apps.microsoft.com/detail/9P9BDZ9RHRC7).

<details>
<summary><b>Clique para expandir / recolher o README em português</b></summary><br>

---

### ⚖️ Manifesto e Autorização Legal do Titular dos Direitos Autorais

> **Declaração Expressa de Fernando Nillsson Cidade:**  
> Eu, **Fernando Nillsson Cidade**, na qualidade de único autor, criador original e titular exclusivo dos direitos autorais do projeto **GeoEstabil**, **autorizo o uso, estudo, cópia, modificação, compilação e distribuição do código-fonte público deste repositório** nos exatos termos da licença **[LICENSE](LICENSE)** (**GNU Affero General Public License v3 — GNU AGPLv3**, identificador SPDX `AGPL-3.0-only`), ressalvada a proteção nominal de nome e marca sob a Seção 7(e) (`"GeoEstabil"`).  
> Qualquer versão modificada ou trabalho derivado do código-fonte público deve manter os créditos da autoria original e disponibilizar o código-fonte correspondente sob a mesma licença **GNU AGPLv3**.  
> As compilações comerciais distribuídas oficialmente através da Microsoft Store são regidas pelos respectivos **EULA Comercial** (`src/assets/EULA/`) e **Condições de Licença Comercial — CLC** (`src/assets/CLC/`), dispensando o adquirente comercial das obrigações de abertura de código da GNU AGPLv3 aplicáveis exclusivamente ao código aberto.  
> **Canal de Suporte e Aviso Legal:** O suporte eventual relacionado a este projeto é prestado **exclusivamente via e-mail (`linceu_lighthouse@outlook.com`)**, **conforme a disponibilidade do autor e sem obrigação de urgência ou SLA**. O software é ferramenta auxiliar de estudo acadêmico fornecida **"no estado em que se encontra" ("AS IS")**, sem garantias e com total isenção do autor quanto a uso indevido ou aplicação definitiva em produção. Consulte [`docs/licensing.md`](docs/licensing.md).

---

### 📌 Ficha Técnica do Projeto

- **Nome Oficial:** GeoEstabil
- **Versão:** `2026.11.11.0` (Data: 11 de novembro de 2026)
- **Autor e Mantenedor:** Fernando Nillsson Cidade
- **E-mail Oficial de Suporte (exclusivamente por disponibilidade, sem urgência):** `linceu_lighthouse@outlook.com`
- **Repositório Oficial:** `https://github.com/fernandoncidade`
- **Download Oficial (Microsoft Store):** [Instalar via Microsoft Store](https://apps.microsoft.com/detail/9P9BDZ9RHRC7) (`https://apps.microsoft.com/detail/9P9BDZ9RHRC7`)
- **Normas Técnicas Brasileiras Referenciadas:** ABNT NBR 11682:2009 (*Estabilidade de encostas*), ABNT NBR 6122:2022 (*Projeto e execução de fundações*) e ABNT NBR 6118:2023 (*Projeto de estruturas de concreto*)
- **Normas Correlatas Referenciadas:** ABNT NBR 6484:2020 (SPT), NBR 6502:2022 (Solos e Rochas), NBR 6120:2019 (Ações), NBR 7480:2022 (Aço CA-50), NBR 13133:2021 (Topografia)
- **Executáveis Gerados:**
  - `geoestabil.exe`: Interface Gráfica Desktop com renderização na GPU (`src/main.rs`)
  - `geoestabil-cli.exe`: Utilitário autônomo de Linha de Comando para scripts e estudos em lote (`src/bin/main_cli.rs`)
- **Identificador Windows (AppUserModelID):** `fernandoncidade.GeoEstabil`

---

### 🌟 Principais Recursos e Módulos Implementados

1. **Motor de Estabilidade de Taludes com Paralelismo em CPU (`Rayon`):**
   - Métodos de Equilíbrio Limite por Fatias: **Bishop Simplificado (Iterativo com $m_\alpha$)**, **Fellenius (Ordinário / Sueco)** e **Janbu Simplificado (com fator corretivo $f_0$)**;
   - Varredura paramétrica paralela sobre malhas configuráveis de centros $(X_c, Y_c)$ e raios $R$, avaliando milhares de superfícies potenciais em milissegundos;
   - Consideração de múltiplas camadas estratigráficas (critério Mohr-Coulomb $c', \phi', \gamma, \gamma_{\text{sat}}$), lençol freático (poropressão $u$) e sobrecarga na crista ($q$);
   - Mapa de Calor (**Heatmap FS**) renderizado em escala contínua sobre a geometria do talude, destacando as fatias e o arco circular crítico de deslizamento.
2. **Módulo Integrado de Muros de Contenção e Fundações:**
   - Suporte a **Muros de Flexão em Concreto Armado (Perfil L/T Cantilever)** e **Muros de Gravidade (Concreto Ciclópico / Gabião)** com dente de ancoragem opcional;
   - Cálculo de empuxos ativos e passivos pelas teorias de **Rankine** ($\delta = 0$) e **Coulomb** (atrito solo-muro $\delta$, inclinação do tardoz $\theta$ e rampa do terreno $\beta$), sobrecarga, cunha hidrostática e profundidade de fenda de tração;
   - Verificações externas referenciadas na ABNT NBR 11682 e NBR 6122: **Tombamento** ($FS \ge 1{,}50$), **Deslizamento** ($FS \ge 1{,}50$), **Excentricidade no núcleo central** ($e \le B/6$ e descolamento parcial $\sigma_{\max} = 2N/(3x_R)$) e **Capacidade de Carga Última pelo método de Hansen/Meyerhof** ($q_{\text{ult}}$ e $FS \ge 2{,}00$).
3. **Dimensionamento Estrutural em Concreto Armado (Referência ABNT NBR 6118:2023):**
   - Cálculo à flexão simples (Domínio 3 dúctil $\xi \le 0{,}45$), taxa mínima ($\rho_{\min} = 0{,}15\%$), cortante sem estribos ($V_{Rd1} \ge V_d$), estimativa de abertura de fissuras ($w_k \le 0{,}30\text{ mm}$) e sugestão de bitolas comerciais ($\varnothing 8{,}0$ a $\varnothing 25{,}0\text{ mm}$) para **Fuste**, **Ponta** e **Calcanhar**, além de armadura horizontal de retração e distribuição.
4. **Ambiente Gráfico em 5 Abas & Documentação Integrada:**
   - **Aba 1 (Estratigrafia & Solos):** Catálogo didático de solos NBR 6122/SPT, edição de camadas, lençol freático, sobrecarga e nível de segurança (Alto/Médio/Baixo);
   - **Aba 2 (Estabilidade de Taludes):** Painel redimensionável, botão Rayon, resultados críticos e **Canvas 2D interativo** com zoom ancorado no cursor, pan e enquadramento automático;
   - **Aba 3 (Muro de Contenção):** Parametrização com atualização dos diagramas proporcionais de empuxo, tensões de base e indicadores de referência;
   - **Aba 4 (Fundações & Estrutural NBR 6118):** Visualização de estabilidade externa e detalhamento de cálculo por seção;
   - **Aba 5 (Memorial de Cálculo):** Emissão de relatórios de estudo em **HTML5/PDF (A4)**, **Markdown (`.md`)** e **SVG vetorial (`.svg`)**;
   - **Janelas Modeless:** **Manual de Instruções** bilíngue pesquisável com equações formatadas e tabelas; janela **Sobre** em 9 abas reunindo termos, privacidade, licenças e referências bibliográficas das normas ABNT em **Terceiros (Notices)**.
5. **Privacidade Local (100% Local-First):**
   - Zero rede, zero telemetria e zero IA na nuvem; persistência local SQLite em `%APPDATA%\GeoEstabil\geoestabil.db` e arquivos `.geoestabil` (`.json`).

---

### 📸 Capturas de Tela da Interface (`pt-BR`)

Principais telas do **GeoEstabil** em Português (Brasil) cobrindo todo o fluxo de simulação geotécnica e cálculo estrutural:

1. **Aba 1 — Estratigrafia & Solos:** Catálogo de solos ABNT NBR 6122 / SPT, camadas de solo Mohr-Coulomb, lençol freático, sobrecarga e nível de segurança.
   ![GeoEstabil (PT-BR) — Aba 1: Estratigrafia e Solos](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-11%20002316.png)

2. **Aba 2 — Estabilidade de Taludes:** Busca em grade paralela via Rayon com Mapa de Calor (Heatmap FS), fatias e círculo crítico (Bishop Simplificado / Fellenius / Janbu).
   ![GeoEstabil (PT-BR) — Aba 2: Estabilidade de Taludes](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-11%20002325.png)

3. **Aba 3 — Muro de Contenção:** Diagrama interativo de empuxos (Rankine/Coulomb), tensões na base ($\sigma_{\text{ponta}}$ e $\sigma_{\text{calcanhar}}$) e verificações de referência NBR 11682 / NBR 6122.
   ![GeoEstabil (PT-BR) — Aba 3: Muro de Contenção](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-11%20002335.png)

4. **Aba 4 — Fundações & Estrutural (NBR 6118):** Verificação externa da fundação e dimensionamento didático de armaduras à flexão, cortante e fissuração (Fuste, Ponta e Calcanhar).
   ![GeoEstabil (PT-BR) — Aba 4: Fundações e Estrutural NBR 6118](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-11%20002344.png)

5. **Aba 5 — Memorial de Cálculo:** Pré-visualização formatada de relatório de estudo e exportação direta para HTML5/PDF (A4), Markdown e perfil vetorial SVG.
   ![GeoEstabil (PT-BR) — Aba 5: Memorial de Cálculo](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-11%20002354.png)

---

### 🚀 Compilação e Execução a partir do Código-Fonte

#### Pré-requisitos
- Toolchain estável de **Rust 1.80+** (Edição 2021) instalada via `rustup`;
- Ambiente Windows 10/11 x64 com compilador C/C++ (MSVC / Build Tools for Visual Studio).

#### Comandos de Compilação
```powershell
# Clonar o repositório oficial
git clone https://github.com/fernandoncidade/GeoEstabil_rs.git
cd GeoEstabil_rs

# Executar a suíte de testes geotécnicos e de layout UI
cargo test

# Compilar ambos os binários em modo release otimizado
cargo build --release --bin geoestabil --bin geoestabil-cli

# Os executáveis finais são gerados em:
# builds\geoestabil\release\geoestabil.exe
# builds\geoestabil\release\geoestabil-cli.exe
```

---

### 💻 Execução via Linha de Comando (`geoestabil-cli.exe`)

```powershell
# Listar os 4 casos práticos pré-configurados
.\builds\geoestabil\release\geoestabil-cli.exe presets

# Executar análise do Caso 1 (Talude Rodoviário em Corte) e exportar Memorial HTML5/PDF A4
.\builds\geoestabil\release\geoestabil-cli.exe run-preset 1 --output memorial_talude.html

# Analisar muro de arrimo a partir de arquivo salvo e gerar relatório
.\builds\geoestabil\release\geoestabil-cli.exe wall --input projeto.geoestabil --output muro.html

# Consultar catálogo de solos de referência da ABNT NBR 6122 / SPT
.\builds\geoestabil\release\geoestabil-cli.exe soils
```

---

### 🏛️ Natureza Acadêmica, Responsabilidade do Usuário e Isenção

O **GeoEstabil** é uma ferramenta computacional auxiliar destinada **exclusivamente a estudo acadêmico, ensino e experimentação didática**, podendo conter erros de cálculo, aproximações ou limitações numéricas. Os seus resultados **NÃO devem ser utilizados de forma definitiva em produção, projetos executivos, obras reais ou tomadas de decisão profissionais** sem conferência manual completa, validação cruzada em outras ferramentas consolidadas de mercado, ensaios de campo/laboratório e supervisão formal de profissional legalmente habilitado (incluindo eventual ART/RRT perante o CREA/CAU). O software é fornecido **"no estado em que se encontra" ("AS IS")**, sem garantias, e **qualquer uso fora do caráter estritamente acadêmico ocorre por conta e risco exclusivos do usuário, com total isenção legal do autor**.

---

### ⚖️ Licenciamento, Suporte e Comunidade

- **Código-Fonte (GitHub):** Disponibilizado sob **GNU Affero General Public License v3 (GNU AGPLv3)**, permitindo livre uso, estudo, modificação e compartilhamento sob a mesma licença;
- **Edição Comercial (Microsoft Store):** Licenciada sob o **Contrato de Licença de Usuário Final (EULA)** (`src/assets/EULA/`) e **Condições de Licença Comercial (CLC)** (`src/assets/CLC/`) — download oficial em [Microsoft Store](https://apps.microsoft.com/detail/9P9BDZ9RHRC7);
- **Canal de Suporte e Limites:** Todo contato ou pedido de suporte é atendido **exclusivamente por e-mail (`linceu_lighthouse@outlook.com`)**, **conforme a disponibilidade do autor e sem obrigação de urgência, prazo de resposta ou SLA** (sem suporte telefônico, acesso remoto ou consultoria de engenharia);
- **Direitos Legais:** Na aquisição pela loja oficial, preservam-se os eventuais direitos legais irrenunciáveis aplicáveis (como arrependimento junto à plataforma nos termos da loja). Projeto pessoal e educacional mantido em ritmo próprio de desenvolvimento — veja [`CONTRIBUTING.md`](CONTRIBUTING.md).

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

> **Note:** This repository corresponds to release **`2026.11.11.0`** of **GeoEstabil** (native **Rust** implementation), made available under a simple **Dual-Licensing** framework (**GNU AGPLv3** for public source code on GitHub and **Commercial EULA + CLC** for official commercial binaries distributed on the Microsoft Store, with 100% local and offline execution). Support the project and get the officially signed release via: [Install via Microsoft Store](https://apps.microsoft.com/detail/9P9BDZ9RHRC7).

<details>
<summary><b>Click to expand / collapse the README in English</b></summary><br>

---

### ⚖️ Copyright Holder Manifesto & Legal Authorization

> **Express Statement by Fernando Nillsson Cidade:**  
> I, **Fernando Nillsson Cidade**, as the sole author, original creator, and exclusive copyright holder of the **GeoEstabil** project, **authorize the use, study, copying, modification, compilation, and distribution of the public source code in this repository** strictly under the terms of the **[LICENSE](LICENSE)** file (**GNU Affero General Public License v3 — GNU AGPLv3**, SPDX identifier `AGPL-3.0-only`), reserving name and trademark protection under Section 7(e) (`"GeoEstabil"`).  
> Any modified version or derivative work of the public source code must retain original authorship attribution and release its corresponding source code under the same **GNU AGPLv3** license.  
> Official commercial builds distributed through the Microsoft Store are governed by the respective **Commercial EULA** (`src/assets/EULA/`) and **Commercial License Conditions — CLC** (`src/assets/CLC/`), exempting commercial buyers from GNU AGPLv3 source-disclosure duties applicable solely to the open-source codebase.  
> **Support Channel & Legal Disclaimer:** Any user support for this project is provided **exclusively via email (`linceu_lighthouse@outlook.com`)**, **according to the author's availability and with no obligation of urgency or SLA**. The software is an auxiliary academic study tool provided **"AS IS"** without warranties, and the author is wholly exempt from liability for improper use or definitive production deployment. See [`docs/licensing.md`](docs/licensing.md).

---

### 📌 Project Fact Sheet

- **Official Name:** GeoEstabil
- **Version:** `2026.11.11.0` (Date: November 11, 2026)
- **Author & Maintainer:** Fernando Nillsson Cidade
- **Official Support Email (strictly by availability, no urgency):** `linceu_lighthouse@outlook.com`
- **Official Repository:** `https://github.com/fernandoncidade`
- **Official Download (Microsoft Store):** [Install via Microsoft Store](https://apps.microsoft.com/detail/9P9BDZ9RHRC7) (`https://apps.microsoft.com/detail/9P9BDZ9RHRC7`)
- **Referenced Brazilian Technical Standards:** ABNT NBR 11682:2009 (*Slope stability*), ABNT NBR 6122:2022 (*Design and execution of foundations*), and ABNT NBR 6118:2023 (*Design of concrete structures*)
- **Correlated Referenced Standards:** ABNT NBR 6484:2020 (SPT), NBR 6502:2022 (Rocks and Soils), NBR 6120:2019 (Actions), NBR 7480:2022 (Rebar Steel), NBR 13133:2021 (Surveying)
- **Generated Binaries:**
  - `geoestabil.exe`: Desktop Graphical User Interface with GPU rendering (`src/main.rs`)
  - `geoestabil-cli.exe`: Standalone Command-Line Interface for scripting and study batch runs (`src/bin/main_cli.rs`)
- **Windows Identifier (AppUserModelID):** `fernandoncidade.GeoEstabil`

---

### 🌟 Key Capabilities & Implemented Modules

1. **Slope Stability Engine with CPU Parallelism (`Rayon`):**
   - Limit Equilibrium Slice Methods: **Simplified Bishop (Iterative with $m_\alpha$)**, **Fellenius (Ordinary / Swedish)**, and **Simplified Janbu (with correction factor $f_0$)**;
   - Parallel parametric sweep across configurable meshes of centers $(X_c, Y_c)$ and radii $R$, evaluating thousands of potential surfaces in milliseconds;
   - Support for multiple soil strata (Mohr-Coulomb criterion $c', \phi', \gamma, \gamma_{\text{sat}}$), phreatic water table (pore pressure $u$), and crest surcharge ($q$);
   - Thermal **FS Heatmap** rendered across the slope geometry in continuous scale, highlighting vertical slices and the critical circular slip arc.
2. **Integrated Retaining Wall & Foundation Module:**
   - Supports **Reinforced Concrete Cantilever Walls (L/T Profile)** and **Gravity Walls (Cyclopean Concrete / Gabion)** with optional base shear key;
   - Active and passive earth pressure calculation via **Rankine** ($\delta = 0$) and **Coulomb** (wall-soil friction $\delta$, backface slope $\theta$, and backfill slope $\beta$), surcharge pressure, hydrostatic wedge, and tension crack depth;
   - External reference checks based on ABNT NBR 11682 and NBR 6122: **Overturning** ($FS \ge 1.50$), **Sliding** ($FS \ge 1.50$), **Middle-Third Eccentricity** ($e \le B/6$ and partial uplift $\sigma_{\max} = 2N/(3x_R)$), and **Ultimate Bearing Capacity via Hansen/Meyerhof** ($q_{\text{ult}}$ and $FS \ge 2.00$).
3. **Reinforced Concrete Structural Sizing (Reference ABNT NBR 6118:2023):**
   - Flexural sizing calculation (ductile Domain 3 $\xi \le 0.45$), minimum steel ratio ($\rho_{\min} = 0.15\%$), unreinforced shear check ($V_{Rd1} \ge V_d$), crack-width estimate ($w_k \le 0.30\text{ mm}$), and commercial rebar suggestions ($\varnothing 8.0$ to $\varnothing 25.0\text{ mm}$) across **Stem**, **Toe**, and **Heel**, plus horizontal shrinkage and distribution bars.
4. **5-Tab Graphical Environment & Integrated Documentation:**
   - **Tab 1 (Stratigraphy & Soils):** Educational NBR 6122/SPT soil catalog, layer editing, phreatic line, surcharge, and safety level (High/Medium/Low);
   - **Tab 2 (Slope Stability):** Resizable panel, Rayon sweep button, critical summary, and **interactive 2D Canvas** with pointer-anchored zoom, pan, and auto-fit;
   - **Tab 3 (Retaining Wall):** Parametric inputs updating proportional thrust diagrams, base pressure distributions, and reference indicators;
   - **Tab 4 (Foundations & Structural NBR 6118):** External stability display and per-section calculation detailing;
   - **Tab 5 (Calculation Memorial):** Study report output in **HTML5/PDF (A4)**, **Markdown (`.md`)**, and **vector SVG (`.svg`)**;
   - **Modeless Windows:** Searchable bilingual **Operations Manual** with formatted equations and tables; 9-tab **About Dialog** hosting legal terms, privacy, licenses, and bibliographic references to ABNT standards under **Third-Party Notices**.
5. **Local Privacy (100% Local-First):**
   - Zero network traffic, zero telemetry, and zero cloud AI; local SQLite persistence in `%APPDATA%\GeoEstabil\geoestabil.db` and `.geoestabil` (`.json`) files.

---

### 📸 Interface Screenshots (`en-US`)

Core screens of **GeoEstabil** in English (US) covering the complete geotechnical simulation and structural calculation workflow:

1. **Tab 1 — Stratigraphy & Soils:** ABNT NBR 6122 / SPT soil library, Mohr-Coulomb soil layers, phreatic water table, surcharge, and safety level.
   ![GeoEstabil (EN-US) — Tab 1: Stratigraphy and Soils](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-11%20002418.png)

2. **Tab 2 — Slope Stability:** Parallel Rayon grid search with FS Heatmap overlay, vertical slices, and critical slip circle (Simplified Bishop / Fellenius / Janbu).
   ![GeoEstabil (EN-US) — Tab 2: Slope Stability](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-11%20002447.png)

3. **Tab 3 — Retaining Wall:** Interactive earth pressure diagram (Rankine/Coulomb), base contact pressures ($\sigma_{\text{toe}}$ and $\sigma_{\text{heel}}$), and NBR 11682 / NBR 6122 reference checks.
   ![GeoEstabil (EN-US) — Tab 3: Retaining Wall](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-11%20002513.png)

4. **Tab 4 — Foundations & Structural (NBR 6118):** External foundation check and educational flexural, shear, and crack-width rebar sizing (Stem, Toe, and Heel).
   ![GeoEstabil (EN-US) — Tab 4: Foundations and Structural NBR 6118](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-11%20002525.png)

5. **Tab 5 — Calculation Memorial:** Formatted study report preview with direct export to HTML5/PDF (A4), Markdown, and vector SVG profiles.
   ![GeoEstabil (EN-US) — Tab 5: Calculation Memorial](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-11%20002540.png)

---

### 🚀 Building and Running from Source

#### Prerequisites
- Stable **Rust 1.80+** toolchain (Edition 2021) installed via `rustup`;
- Windows 10/11 x64 environment with C/C++ build tools (MSVC / Build Tools for Visual Studio).

#### Build Commands
```powershell
# Clone the official repository
git clone https://github.com/fernandoncidade/GeoEstabil_rs.git
cd GeoEstabil_rs

# Run the geotechnical and UI layout test suite
cargo test

# Build both optimized release binaries
cargo build --release --bin geoestabil --bin geoestabil-cli

# Final executables are produced at:
# builds\geoestabil\release\geoestabil.exe
# builds\geoestabil\release\geoestabil-cli.exe
```

---

### 💻 Command-Line Execution (`geoestabil-cli.exe`)

```powershell
# List the 4 pre-configured practical benchmark cases
.\builds\geoestabil\release\geoestabil-cli.exe presets

# Run Case 1 (Highway Cut Slope) and export HTML5/PDF A4 Memorial
.\builds\geoestabil\release\geoestabil-cli.exe run-preset 1 --output slope_memorial.html

# Analyze a retaining wall from a saved project file and export report
.\builds\geoestabil\release\geoestabil-cli.exe wall --input project.geoestabil --output wall.html

# Print the reference ABNT NBR 6122 / SPT soil catalog
.\builds\geoestabil\release\geoestabil-cli.exe soils
```

---

### 🏛️ Academic Scope, User Responsibility & Liability Exemption

**GeoEstabil** is an auxiliary computational tool intended **exclusively for academic study, teaching, and didactic experimentation**, and may contain calculation bugs, approximations, or numerical limitations. Its outputs **MUST NOT be used as definitive results in production, construction designs, civil works, or professional decision-making** without complete manual verification, cross-checking against established market tools, field/laboratory testing, and formal oversight by a legally licensed engineer (including any required ART/RRT filing before CREA/CAU). The software is provided **"AS IS"** without warranties, and **any deployment beyond this strictly academic scope occurs at the user's sole risk, with complete legal exemption of the author**.

---

### ⚖️ Licensing, Support & Community

- **Source Code (GitHub):** Licensed under **GNU Affero General Public License v3 (GNU AGPLv3)**, allowing free use, study, modification, and sharing under the same license;
- **Commercial Edition (Microsoft Store):** Licensed under the **End User License Agreement (EULA)** (`src/assets/EULA/`) and **Commercial License Conditions (CLC)** (`src/assets/CLC/`) — official download on the [Microsoft Store](https://apps.microsoft.com/detail/9P9BDZ9RHRC7);
- **Support Channel & Limits:** All inquiries or support requests are handled **exclusively via email (`linceu_lighthouse@outlook.com`)**, **according to the author's availability and with no obligation of urgency, response timeline, or SLA** (no phone support, remote access, or engineering consulting);
- **Statutory Rights:** Store purchases preserve applicable non-waivable consumer rights (such as platform withdrawal windows under store policies). Personal educational project maintained at the author's own pace — see [`CONTRIBUTING.md`](CONTRIBUTING.md).

</details>
