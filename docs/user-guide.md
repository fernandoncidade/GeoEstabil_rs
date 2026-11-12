# Guia Rápido do Usuário / Quick User Guide — GeoEstabil (`v2026.11.12.0`)

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

<details open>
<summary><strong id="ptbr">🇧🇷 Português (BR) — Operação Completa na Interface Desktop</strong></summary><br>

## 1. Fluxo de Trabalho pelas 5 Abas Principais

1. **Aba 1 — Estratigrafia & Solos:** Defina camadas de solo ($c'$, $\phi'$, $\gamma$, $\gamma_{\text{sat}}$), propriedades do projeto e utilize o Catálogo de Solos SPT em SQLite (`geoestabil.db`).
2. **Aba 2 — Estabilidade de Taludes (`ABNT NBR 11682:2009`):** Configure os pontos da superfície topográfica, nível freático e sobrecarga $q$; escolha o Grau de Segurança exigido pela Tabela 1 da NBR 11682 (`1,50`, `1,30` ou `1,20`) e a faixa da grade de busca $(x_c, y_c, R)$; execute a varredura paralela **Rayon** e inspecione em tempo real o **Mapa de Calor (Heatmap)** e o **Círculo Crítico** no Canvas 2D interativo (*Zoom*: roda do mouse; *Pan*: arraste).
3. **Aba 3 — Muro de Contenção (`ABNT NBR 6122:2022`):** Selecione a tipologia (**Gravidade — Concreto Ciclópico**, **Gabião Escalonado** ou **Flexão em Concreto Armado**), geometria ($H$, ponta, calcanhar, inclinação), teoria de empuxo (**Rankine** ou **Coulomb**) e verifique tombamento, deslizamento e excentricidade com diagramas em tempo real.
4. **Aba 4 — Fundações & Estrutural (`ABNT NBR 6122 / NBR 6118:2023`):** Analise capacidade de carga (Meyerhof/Hansen), tensões na base, classe de agressividade ambiental (CAA), armadura de flexão do fuste e sapata, esforço cortante ($V_{Rd1} \ge V_d$) e abertura de fissuras ELS-W ($w_k \le 0{,}30\text{ mm}$).
5. **Aba 5 — Memorial de Cálculo:** Gere, visualize e exporte o Memorial de estudo em **HTML5/PDF (A4)**, **Markdown (`.md`)** ou **SVG vetorial (`.svg`)** para fins acadêmicos e conferência manual em ferramentas consolidadas.

## 2. Janelas Não-Modais (Modeless Z-Order) e Atalhos

| Atalho / Menu | Janela Aberta | Recurso Principal |
| :--- | :--- | :--- |
| **`F1` ou `Ctrl+M`** | **Manual de Instruções** | Guia detalhado em 6 capítulos bilíngues com busca rápida integrada. |
| **`Ctrl+I`** | **Sobre o GeoEstabil** | Janela de 9 abas com EULA, CLC, Privacidade, Copyright, Histórico, Avisos/Referências ABNT e Licenças integrais. |
| **Barra Superior / Menu** | **Alternador `PT-BR` / `EN-US` & Tema** | Troca imediata de idioma e alternância entre Modo Escuro e Modo Claro técnico em tempo real. |

Para mais detalhes, utilize o atalho **`F1`** dentro do aplicativo ou consulte [`MANUAL.md`](../MANUAL.md).

</details>

<details>
<summary><strong id="english-us">🇺🇸 English (US) — Complete Desktop Operation Guide</strong></summary><br>

## 1. Workflow Across the 5 Main Workspaces

1. **Tab 1 — Stratigraphy & Soils:** Define stratified soil layers ($c'$, $\phi'$, $\gamma$, $\gamma_{\text{sat}}$), project metadata, and query the local SQLite SPT Soil Catalog (`geoestabil.db`).
2. **Tab 2 — Slope Stability (`ABNT NBR 11682:2009`):** Configure topographic surface vertices, phreatic water line, and surcharge $q$; select the required Safety Level from NBR 11682 Table 1 (`1.50`, `1.30`, or `1.20`) and the search grid bounds $(x_c, y_c, R)$; execute the parallel **Rayon** sweep and inspect the live **Heatmap** and **Critical Slip Circle** on the interactive 2D Canvas (*Zoom*: scroll wheel; *Pan*: drag).
3. **Tab 3 — Retaining Wall (`ABNT NBR 6122:2022`):** Select wall type (**Gravity — Cyclopean Concrete**, **Stepped Gabion**, or **Reinforced Concrete Cantilever**), dimensions ($H$, toe, heel, slope), earth pressure theory (**Rankine** or **Coulomb**), and verify overturning, sliding, and eccentricity with live diagrams.
4. **Tab 4 — Foundations & Structural Design (`ABNT NBR 6122 / NBR 6118:2023`):** Analyze foundation bearing capacity (Meyerhof/Hansen), base stresses, environmental aggressiveness class (CAA), flexural steel for stem and footing, shear resistance ($V_{Rd1} \ge V_d$), and ELS-W crack width ($w_k \le 0.30\text{ mm}$).
5. **Tab 5 — Calculation Memorial:** Generate, preview, and export the study Memorial to **HTML5/PDF (A4)**, **Markdown (`.md`)**, or **vector SVG (`.svg`)** for academic study and manual verification in established market tools.

## 2. Modeless Z-Order Windows and Shortcuts

| Shortcut / Menu | Window Opened | Primary Capability |
| :--- | :--- | :--- |
| **`F1` or `Ctrl+M`** | **Operations Manual** | Detailed bilingual 6-chapter guide featuring integrated instant text search. |
| **`Ctrl+I`** | **About GeoEstabil** | 9-tab dialog housing complete EULA, CLC, Privacy Policy, Copyright, History, Notices/ABNT References, and Open-Source Licenses. |
| **Top Toolbar / Menu** | **`PT-BR` / `EN-US` & Theme Toggle** | Live runtime language switching and toggling between high-contrast Dark and Light technical themes. |

For additional technical details, press **`F1`** inside the application or consult [`MANUAL.md`](../MANUAL.md).

</details>
