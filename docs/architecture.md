# Arquitetura de Software / Software Architecture — GeoEstabil (`v2026.11.12.0`)

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

<details open>
<summary><strong id="ptbr">🇧🇷 Português (BR) — Arquitetura Modular em Rust</strong></summary><br>

## 1. Visão Geral do Sistema

O **GeoEstabil** (`geoestabil_rs`) adota uma arquitetura estritamente modularizada em **Rust (Edição 2021)**, desacoplando o núcleo determinístico de engenharia geotécnica e estrutural da camada gráfica acelerada por GPU (`egui 0.37` + `eframe` / `Glow` & `wgpu`) e do executor automatizado em linha de comando (`geoestabil-cli`).

```text
GeoEstabil_rs/
├── src/
│   ├── core/            # Estruturas de domínio (GeoProject, SoilLayer, RetainingWall, Presets)
│   ├── engine/
│   │   ├── slope/       # Fatias verticais: Fellenius, Bishop Simplificado, Janbu + Varredura Rayon
│   │   ├── wall/        # Empuxos (Rankine/Coulomb), Estabilidade Externa, Capacidade de Carga, NBR 6118
│   │   └── soils/       # Catálogo geotécnico e correlações NSPT (ABNT NBR 6484)
│   ├── data/            # Persistência SQLite embarcada (rusqlite bundled) e projetos .geoestabil
│   ├── reports/         # Gerador de Memorial de Cálculo em HTML5/PDF, Markdown e SVG (uso de estudo)
│   ├── locale/          # Localização dinâmica PT-BR/EN-US e Manual interativo de instruções
│   ├── utils/           # Resolvedor portátil de assets (paths.rs), SVG (export_dxf_svg.rs) e ícones (icons.rs)
│   ├── ui/              # Interface egui/wgpu (Canvas 2D responsivo, 5 abas e janelas Modeless Z-Order)
│   └── bin/main_cli.rs  # Binário CLI headless (geoestabil-cli)
├── docs/                # Documentação técnica, licenciamento, privacidade e guias operacionais
└── src/assets/          # EULA, CLC, Privacidade, Copyright, Notices, Releases, Licenças e Ícones MSIX
```

## 2. Motor de Estabilidade de Taludes (`src/engine/slope/`)

* **Discretização por Fatias Verticais:** Interseção exata de círculos candidatos $(x_c, y_c, R)$ contra poligonal de terreno, estratigrafia de múltiplos solos ($c'$, $\phi'$, $\gamma$, $\gamma_{\text{sat}}$), lençol freático piezométrico ($u = \gamma_w \cdot h_w$) e sobrecarga superficial distribuída $q$.
* **Equilíbrio Limite Implementado:**
  * **Fellenius (Método Ordinário):** Solução explícita sem iteração.
  * **Bishop Simplificado:** Iteração de ponto fixo com critério de convergência rigoroso e guarda numérica de $m_\alpha \ge 0{,}20$.
  * **Janbu Simplificado:** Solução baseada em equilíbrio horizontal com fator corretivo de forma $f_0$.
* **Paralelismo Real com Rayon:** A função de varredura distribui a grade tridimensional $(N_x \times N_y \times N_R)$ pelos núcleos da CPU via *work-stealing*, retornando os resultados para construção do **Mapa de Calor (Heatmap)** e destaque do círculo crítico ($FS_{\min}$).

## 3. Módulo de Muros de Arrimo e Concreto Armado (`src/engine/wall/`)

* **Geometrias Suportadas:** Gravidade (Concreto Ciclópico), Gabião Escalonado e Flexão em Concreto Armado com Sapata Corrida (fuste, ponta e calcanhar).
* **Empuxos Ativo e Passivo (`earth_pressure.rs`):** Teoria de Rankine ($K_a$, $K_p$ com talude inclinado $\beta$) e Coulomb (incorporando ângulo de atrito solo-muro $\delta$ e inclinação da face $\psi$).
* **Estabilidade Externa (`stability.rs`) & Capacidade de Carga (`bearing_capacity.rs`):** Tombamento ($FS \ge 1{,}50$), deslizamento na base ($FS \ge 1{,}50$), excentricidade no núcleo central ($e \le B/6$ ou diagrama triangular redistribuído $\sigma_{\max} = \frac{2N}{3 x_R}$) e tensão admissível via formulação de Hansen/Meyerhof (ABNT NBR 6122).
* **Concreto Armado (`concrete_design.rs`):** Dimensionamento de fuste, ponta e calcanhar conforme ABNT NBR 6118 ($\gamma_f = 1{,}40$, $\gamma_c = 1{,}40$, $\gamma_s = 1{,}15$, ductilidade $x/d \le 0{,}45$, $A_{s,\min} = 0{,}15\% \cdot b_w h$, esforço cortante $V_{Rd1} \ge V_d$ e abertura característica de fissura $w_k \le 0{,}30\text{ mm}$ em ELS-W).

## 4. Camada Gráfica e Janelas Não-Modais (`src/ui/`)

* **Canvas 2D Responsivo & Pipeline Otimizado:** Ajuste automático à área útil em `CentralPanel`, zoom ancorado no cursor, pan por arraste e renderização vetorial ao vivo via `Glow`/OpenGL com baixo consumo de VRAM e estabilidade dimensional em hover.
* **Janelas Modeless Z-Order (`dialogs/modeless.rs`):** Uso de *viewports* independentes no Windows 11/10, permitindo manter o **Manual de Instruções (`F1`)**, a janela **Sobre (`Ctrl+I`, incluindo referências bibliográficas ABNT na aba *Terceiros (Notices)*)** e o **Catálogo SPT** abertos lado a lado com a área de cálculo.

</details>

<details>
<summary><strong id="english-us">🇺🇸 English (US) — Modular Rust Architecture</strong></summary><br>

## 1. System Overview

**GeoEstabil** (`geoestabil_rs`) adopts a strictly modularized architecture in **Rust (Edition 2021)**, decoupling the deterministic geotechnical and structural engineering engine from the GPU-accelerated presentation layer (`egui 0.37` + `eframe` / `Glow` & `wgpu`) and the automated command-line runner (`geoestabil-cli`).

```text
GeoEstabil_rs/
├── src/
│   ├── core/            # Domain structures (GeoProject, SoilLayer, RetainingWall, Presets)
│   ├── engine/
│   │   ├── slope/       # Vertical slices: Fellenius, Simplified Bishop, Janbu + Rayon Grid Search
│   │   ├── wall/        # Earth pressures (Rankine/Coulomb), External Stability, Bearing Capacity, NBR 6118
│   │   └── soils/       # Geotechnical catalog & NSPT correlations (ABNT NBR 6484)
│   ├── data/            # Embedded SQLite persistence (rusqlite bundled) and .geoestabil projects
│   ├── reports/         # Calculation Memorial generator in HTML5/PDF, Markdown, and SVG (study use)
│   ├── locale/          # Dynamic PT-BR/EN-US localization and interactive Operations Manual
│   ├── utils/           # Portable asset resolver (paths.rs), SVG export (export_dxf_svg.rs), and icons
│   ├── ui/              # egui/wgpu GUI (Responsive 2D Canvas, 5 tabs, and Modeless Z-Order dialogs)
│   └── bin/main_cli.rs  # Headless CLI binary (geoestabil-cli)
├── docs/                # Technical documentation, licensing, privacy, and operational guides
└── src/assets/          # EULA, CLC, Privacy, Copyright, Notices, Releases, Licenses, and MSIX Icons
```

## 2. Slope Stability Engine (`src/engine/slope/`)

* **Vertical-Slice Discretization:** Exact intersection of candidate slip circles $(x_c, y_c, R)$ against the terrain polyline, multi-soil stratigraphy ($c'$, $\phi'$, $\gamma$, $\gamma_{\text{sat}}$), piezometric phreatic water table ($u = \gamma_w \cdot h_w$), and distributed surface surcharge $q$.
* **Implemented Limit Equilibrium Methods:**
  * **Fellenius (Ordinary Method):** Explicit non-iterative formulation.
  * **Simplified Bishop:** Fixed-point iteration with strict convergence criterion and $m_\alpha \ge 0.20$ numerical guard.
  * **Simplified Janbu:** Horizontal force-equilibrium solution with empirical shape correction factor $f_0$.
* **True Parallelism with Rayon:** The sweep routine distributes the 3D mesh $(N_x \times N_y \times N_R)$ across CPU cores via work-stealing, returning evaluated grid points for **Safety Factor Heatmap** rendering and critical circle identification ($FS_{\min}$).

## 3. Retaining Wall & Reinforced Concrete Engine (`src/engine/wall/`)

* **Supported Geometries:** Gravity Wall (Cyclopean Concrete), Stepped Gabion Wall, and Reinforced Concrete Cantilever Wall with continuous footing (stem, toe, and heel).
* **Active and Passive Earth Pressures (`earth_pressure.rs`):** Rankine theory ($K_a$, $K_p$ with inclined backfill $\beta$) and Coulomb theory (incorporating wall-soil friction angle $\delta$ and wall backface inclination $\psi$).
* **External Stability (`stability.rs`) & Bearing Capacity (`bearing_capacity.rs`):** Overturning ($FS \ge 1.50$), base sliding ($FS \ge 1.50$), middle-third kern eccentricity ($e \le B/6$ or redistributed triangular diagram $\sigma_{\max} = \frac{2N}{3 x_R}$), and allowable soil bearing stress via Hansen/Meyerhof equations (ABNT NBR 6122).
* **Reinforced Concrete (`concrete_design.rs`):** Structural detailing of stem, toe, and heel per ABNT NBR 6118 ($\gamma_f = 1.40$, $\gamma_c = 1.40$, $\gamma_s = 1.15$, ductility limit $x/d \le 0.45$, $A_{s,\min} = 0.15\% \cdot b_w h$, unreinforced shear check $V_{Rd1} \ge V_d$, and serviceability crack-width check $w_k \le 0.30\text{ mm}$ under ELS-W).

## 4. Presentation & Modeless Viewport Layer (`src/ui/`)

* **Responsive 2D Canvas & Optimized Pipeline:** Automatic fitting inside `CentralPanel` with pointer-anchored zoom, drag pan, and live vector rendering via low-VRAM `Glow`/OpenGL with zero hover jitter.
* **Modeless Z-Order Windows (`dialogs/modeless.rs`):** Independent desktop viewports on Windows 11/10 allowing the **User Manual (`F1`)**, the **About Dialog (`Ctrl+I`, including ABNT bibliographic references under *Third-Party Notices*)**, and the **SPT Catalog** to remain open alongside the active calculation workspace.

</details>
