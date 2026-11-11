# Modelo de Pull Request / Pull Request Template — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

> [!NOTE]
> **Aviso ao Colaborador / Contributor Notice:**  
> 🇧🇷 *Este repositório é um projeto pessoal e educacional mantido pelo autor em ritmo próprio de estudo. Pull Requests são bem-vindos, mas a análise técnica e incorporação de código ocorrem de forma voluntária e sem compromisso de prazo ou urgência. Eventual suporte é feito exclusivamente via e-mail (`linceu_lighthouse@outlook.com`) conforme disponibilidade. Caso não haja resposta ou merge imediato, sinta-se livre para manter, evoluir e compartilhar suas melhorias em seu próprio fork sob a licença **GNU AGPLv3**.*  
> 🇺🇸 *This repository is a personal and educational project maintained by the author at his own study pace. Pull Requests are welcome, but technical review and merging occur voluntarily without any guaranteed timeline or urgency obligation. Any support is handled exclusively via email (`linceu_lighthouse@outlook.com`) according to availability. If there is no immediate response or merge, feel free to maintain, evolve, and share your improvements in your own fork under the **GNU AGPLv3** license.*

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher o template em português</summary><br>

## Descrição
Descreva brevemente a finalidade deste Pull Request no **GeoEstabil** e qual ajuste técnico ou melhoria de estudo ele entrega.

## Tipo de Alteração
- [ ] Correção de bug (alteração compatível que corrige uma falha)
- [ ] Motor de Estabilidade de Taludes (`Fellenius`, `Bishop Simplificado`, `Janbu Simplificado`, `Rayon`)
- [ ] Módulo de Muros de Arrimo & Fundações (`NBR 6122:2022` / `NBR 6118:2023`)
- [ ] Canvas 2D (`egui`/`wgpu`), Exportação SVG/HTML ou Documentação & Avisos (`src/assets/`)
- [ ] Utilitário CLI (`geoestabil-cli.exe`) / Exportação (`.geoestabil`, HTML5, Markdown, SVG)
- [ ] Otimização em Rust / Documentação (`docs/`, `src/assets/`) / Internacionalização (PT-BR & EN-US)

## Testes e Validação Realizados
- [ ] Verificação estática sem erros (`cargo check --all-targets`)
- [ ] Suíte de testes automatizados 100% aprovada (`cargo test`)
- [ ] Binários compilados (`cargo build --release --bin geoestabil --bin geoestabil-cli`)

## Licenciamento, Privacidade e Referências
- [ ] As alterações são autorais ou livres e seguem a licença **GNU AGPLv3** ([`LICENSE`](../LICENSE)).
- [ ] Os módulos de núcleo (`src/core/`, `src/engine/`, `src/data/`, `src/reports/`) permanecem desacoplados da GUI para suportar `geoestabil.exe` e `geoestabil-cli.exe`.
- [ ] Nenhuma telemetria, conexão de rede externa ou API de IA/LLM em nuvem foi introduzida (100% Local-First / Offline).
- [ ] Respeito integral aos direitos autorais da ABNT (nenhuma reprodução não autorizada de textos protegidos).
- [ ] Novas strings de UI possuem tradução dinâmica PT-BR / EN-US (`src/locale/translations.rs`).

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the template in English</summary><br>

## Description
Briefly describe the purpose of this Pull Request in **GeoEstabil** and what bug fix or academic study improvement it delivers.

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] Slope Stability Solver (`Fellenius`, `Simplified Bishop`, `Simplified Janbu`, `Rayon`)
- [ ] Retaining Wall & Foundations Design (`NBR 6122:2022` / `NBR 6118:2023`)
- [ ] 2D Canvas (`egui`/`wgpu`), SVG/HTML Export, or Documentation & Notices (`src/assets/`)
- [ ] Headless CLI (`geoestabil-cli.exe`) / Export (`.geoestabil`, HTML5, Markdown, SVG)
- [ ] Rust Performance / Documentation (`docs/`, `src/assets/`) / Localization (PT-BR & EN-US)

## Testing & Verification Performed
- [ ] Static check passed (`cargo check --all-targets`)
- [ ] Automated test suite passed (`cargo test`)
- [ ] Production binaries built (`cargo build --release --bin geoestabil --bin geoestabil-cli`)

## Licensing, Privacy & References
- [ ] Contributions are original or freely submittable and comply with **GNU AGPLv3** ([`LICENSE`](../LICENSE)).
- [ ] Core modules (`src/core/`, `src/engine/`, `src/data/`, `src/reports/`) stay decoupled from GUI code so both `geoestabil.exe` and `geoestabil-cli.exe` build cleanly.
- [ ] Zero network requests, telemetry, or external cloud AI/LLM dependencies introduced (100% Local-First / Offline).
- [ ] Full respect for ABNT copyright (zero unauthorized copying of protected ABNT standard texts).
- [ ] Every new UI string supports dynamic PT-BR / EN-US localization (`src/locale/translations.rs`).

</details>
