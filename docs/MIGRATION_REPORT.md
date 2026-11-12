# Relatório de Migração Arquitetural / Architecture Migration Report — Ecossistema Rust (`v2026.11.12.0`)

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

<details open>
<summary><strong id="ptbr">🇧🇷 Português (BR) — Migração e Consolidação da Plataforma</strong></summary><br>

## 1. Contextualização

O **GeoEstabil** consolida os aprendizados estruturais de suítes de engenharia em Rust + `egui`/`wgpu` e a migração de scripts e planilhas fragmentadas de equilíbrio limite para um binário nativo unificado de alta velocidade no Windows 11/10.

## 2. Ganhos Mensuráveis da Arquitetura Nativa Rust + Rayon

| Aspecto | Planilhas / Scripts Avulsos | GeoEstabil (`v2026.11.12.0`) |
| :--- | :--- | :--- |
| **Motor de Fatias** | Loops interpretados lentos e grades rígidas | **Rayon work-stealing** em paralelo na CPU avaliando varreduras multiponto em milissegundos |
| **Interface 2D** | Sem visualização dinâmica integrada ou licenças de alto custo | **Canvas 2D `egui`/`wgpu`** com Zoom/Pan, Heatmap térmico do FS e diagramas de empuxo em tempo real |
| **Rastreabilidade Normativa** | Referências dispersas sem vínculo direto | **Referências Bibliográficas Formais (ABNT NBR 6023)** embarcadas em *Sobre → Terceiros (Notices)* (`Ctrl+I`), respeitando integralmente o copyright da ABNT |
| **Licenciamento e Termos** | Código ou scripts sem modelo claro de distribuição | **Duplo Licenciamento simples** (GNU AGPLv3 no GitHub + EULA/CLC Microsoft Store), suporte apenas via e-mail por disponibilidade e aviso AS-IS |
| **Portabilidade de Assets** | Caminhos absolutos quebram em pacotes MSIX | Resolvedor portátil `resolver_caminho_asset` com fallback embutido via `include_str!` |

## 3. Evolução para Empacotamento Windows 11 / MSIX

* Separação clara entre núcleo puro (`geoestabil_rs` library crate), interface desktop (`geoestabil`) e executor CI/CD (`geoestabil-cli`).
* Integração de manifesto de recursos Windows (`winres` em `src/build.rs`), ícone multirresolução autêntico (`geoestabil.ico`), identificação de tarefa no Windows (`AppUserModelID = "fernandoncidade.GeoEstabil"`) e script automatizado de distribuição limpa (`scripts/gerar_distribuicao_limpa.ps1`).

</details>

<details>
<summary><strong id="english-us">🇺🇸 English (US) — Architecture Migration & Consolidation</strong></summary><br>

## 1. Contextual Background

**GeoEstabil** consolidates architectural patterns from Rust + `egui`/`wgpu` engineering suites and migrates fragmented spreadsheets and standalone limit-equilibrium scripts into a unified, high-speed native Windows 11/10 binary.

## 2. Measurable Gains from the Native Rust + Rayon Architecture

| Aspect | Spreadsheets / Legacy Scripts | GeoEstabil (`v2026.11.12.0`) |
| :--- | :--- | :--- |
| **Slice Solver Engine** | Slow interpreted loops and rigid search meshes | **Rayon work-stealing** parallel CPU sweeps evaluating multi-point grids in milliseconds |
| **2D Graphical UI** | No integrated live canvas or high recurring license fees | **`egui`/`wgpu` 2D Canvas** featuring Zoom/Pan, thermal FS Heatmap, and live earth-pressure diagrams |
| **Standards Traceability** | Scattered references lacking direct traceability | Embedded **Formal Bibliographic Citations (ABNT NBR 6023)** under *About → Third-Party Notices* (`Ctrl+I`), strictly respecting ABNT copyright |
| **Licensing & Terms** | Ad-hoc scripts lacking a clear distribution license | **Simple Dual Licensing** (GNU AGPLv3 on GitHub + Commercial Microsoft Store EULA/CLC), email-only support by availability, and AS-IS disclaimer |
| **Asset Portability** | Absolute filesystem paths break inside MSIX packages | Portable `resolver_caminho_asset` resolver backed by compile-time `include_str!` embedded fallbacks |

## 3. Readiness for Windows 11 / MSIX Packaging

* Clean separation among the pure computation library crate (`geoestabil_rs`), desktop GUI (`geoestabil`), and CI/CD batch runner (`geoestabil-cli`).
* Windows resource manifest integration (`winres` in `src/build.rs`), authentic multi-resolution icon (`geoestabil.ico`), explicit Windows shell task identity (`AppUserModelID = "fernandoncidade.GeoEstabil"`), and automated clean packaging script (`scripts/gerar_distribuicao_limpa.ps1`).

</details>
