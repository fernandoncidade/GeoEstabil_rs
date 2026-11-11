# Notas de Lançamento / Release Notes — `v2026.11.11.0` (GeoEstabil)

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

<details open>
<summary><strong id="ptbr">🇧🇷 Português (BR) — Marco Inicial `v2026.11.11.0`</strong></summary><br>

**Data de Lançamento:** 11 de novembro de 2026  
**Versão Semântica (Cargo):** `2026.11.11` | **Versão Empacotada:** `2026.11.11.0`  
**Autor:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)

## Novidades Principais

1. **Estabilidade de Taludes Paralela (`ABNT NBR 11682:2009`):**
   * Implementação completa dos métodos de **Fellenius (Ordinário)**, **Bishop Simplificado** (iterações com guarda numérica $m_\alpha \ge 0{,}20$) e **Janbu Simplificado** (fator corretivo $f_0$).
   * Varredura multithread via **Rayon** sobre grade paramétrica de centros e raios, com renderização de **Mapa de Calor (Heatmap)** e destaque do círculo crítico no Canvas 2D responsivo.
2. **Módulo Completo de Muros de Arrimo (`ABNT NBR 6122:2022` & `NBR 6118:2023`):**
   * Geometrias de Gravidade (Concreto Ciclópico), Gabião Escalonado e Flexão em Concreto Armado (Fuste, Ponta e Calcanhar).
   * Empuxos ativo/passivo (Rankine e Coulomb), verificação de tombamento, deslizamento, excentricidade e capacidade de carga (Meyerhof/Hansen), além de armadura de flexão, cortante $V_{Rd1} \ge V_d$ e fissuração ELS-W ($w_k \le 0{,}30\text{ mm}$).
3. **Documentação Integrada, Referências ABNT e Janelas Modeless Z-Order:**
   * **Manual de Instruções (`F1`)** interativo bilíngue com busca instantânea e exportação Markdown.
   * **Janela Sobre (`Ctrl+I`)** com 9 abas (Visão Geral, Autor/Suporte, Histórico, EULA, CLC, Copyright/Marcas, Privacidade, Terceiros e Licenças).
   * **Referências Bibliográficas das Normas ABNT (`Ctrl+I` → Terceiros / Notices)** cobrindo NBR 11682:2009, NBR 6122:2022, NBR 6118:2023 e normas correlatas com respeito integral aos direitos autorais reservados da ABNT, e motor tipográfico (`doc_renderer.rs`) com frações empilhadas.
4. **Duplo Licenciamento, Privacidade Local e Suporte por E-mail:**
   * Código-fonte público sob **GNU AGPLv3** + edições comerciais da Microsoft Store sob **EULA/CLC** (`src/assets/EULA/`, `src/assets/CLC/`), com execução 100% offline, uso voltado a estudo acadêmico (AS-IS) e suporte exclusivo via e-mail (`linceu_lighthouse@outlook.com`) conforme disponibilidade e sem urgência.

</details>

<details>
<summary><strong id="english-us">🇺🇸 English (US) — Initial Release `v2026.11.11.0`</strong></summary><br>

**Release Date:** November 11, 2026  
**Semantic Version (Cargo):** `2026.11.11` | **Packaged Version:** `2026.11.11.0`  
**Author:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)

## Core Highlights

1. **Parallel Slope Stability Solver (`ABNT NBR 11682:2009`):**
   * Full implementation of **Ordinary (Fellenius)**, **Simplified Bishop** (iterative solver with $m_\alpha \ge 0.20$ numerical guard), and **Simplified Janbu** ($f_0$ shape factor).
   * Multi-threaded **Rayon** sweep across parameterized grids of centers and radii, rendering live **Safety Factor Heatmaps** and critical failure arc overlays on the responsive 2D Canvas.
2. **Comprehensive Retaining Wall Module (`ABNT NBR 6122:2022` & `NBR 6118:2023`):**
   * Supports Gravity (Cyclopean Concrete), Stepped Gabion, and Reinforced Concrete Cantilever walls (Stem, Toe, and Heel).
   * Rankine and Coulomb active/passive earth pressures, overturning, base sliding, eccentricity, and bearing capacity (Meyerhof/Hansen), plus RC flexural steel, unreinforced shear $V_{Rd1} \ge V_d$, and ELS-W crack-width checking ($w_k \le 0.30\text{ mm}$).
3. **Integrated Documentation, ABNT References & Modeless Z-Order Windows:**
   * **User Manual (`F1`)** bilingual interactive guide with real-time search and Markdown export.
   * **9-Tab About Dialog (`Ctrl+I`)** (Overview, Author/Support, History, EULA, CLC, Copyright/Trademarks, Privacy, Notices, and Full Licenses).
   * **ABNT Bibliographic References (`Ctrl+I` → Third-Party Notices)** covering NBR 11682:2009, NBR 6122:2022, NBR 6118:2023, and allied standards in full respect of ABNT reserved copyrights, paired with stacked-fraction math typography (`doc_renderer.rs`).
4. **Dual Licensing, Local Privacy & Email-Only Support:**
   * Public source code under **GNU AGPLv3** paired with Microsoft Store commercial editions under **EULA/CLC** (`src/assets/EULA/`, `src/assets/CLC/`), 100% offline execution, academic study notice (AS-IS), and support solely via email (`linceu_lighthouse@outlook.com`) by availability with no urgency.

</details>
