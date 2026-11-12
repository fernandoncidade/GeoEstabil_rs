# Licenciamento e Esclarecimento Comercial / Licensing & Commercial Model — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details open>
<summary><b>Clique para expandir / recolher o documento de licenciamento em Português</b></summary><br>

# GeoEstabil — Licenciamento, Termos de Código Aberto e Esclarecimento Comercial

**Versão:** `2026.11.12.0` (Cargo: `2026.11.12`)  
**Data:** `12 de novembro de 2026`  
**Licença do Código-Fonte:** GNU Affero General Public License v3.0 (`GNU AGPLv3` / `AGPL-3.0-only`)  
**Autor e Titular dos Direitos Autorais:** Fernando Nillsson Cidade  
**Suporte (exclusivamente via e-mail, por disponibilidade e sem urgência):** `linceu_lighthouse@outlook.com`

---

## 1. Visão Geral do Modelo de Licenciamento

O **GeoEstabil** (`geoestabil.exe` e `geoestabil-cli.exe`) é disponibilizado de forma simples e transparente sob modelo de **Duplo Licenciamento**, com base na titularidade exclusiva dos direitos autorais de **Fernando Nillsson Cidade**:

| Componente | Licença | Usos Permitidos e Condições |
|---|---|---|
| **Código-Fonte (Repositório Público)** | [GNU AGPLv3](../LICENSE) | Livre para estudo, execução, modificação, compilação e compartilhamento (comercial ou não comercial), mantendo créditos autorais, licença GNU AGPLv3 nas derivações e reserva de marca da Seção 7(e). |
| **Binários Oficiais da Microsoft Store** | EULA Comercial / CLC (`src/assets/EULA/`, `src/assets/CLC/`) | Uso do aplicativo distribuído pela Microsoft Store sem obrigação de abertura de código da GNU AGPLv3 aplicável ao código-fonte público. |
| **Documentação em `src/assets/`** | Autoria do Projeto (Uso Livre em Forks) | Os textos de suporte e distribuição podem ser livremente adaptados, editados, substituídos ou removidos por mantenedores de forks, respeitando a marca `"GeoEstabil"`. |
| **Binários Compilados por Terceiros** | GNU AGPLv3 | Livres para compilar e distribuir por conta própria, desde que acompanhados do código-fonte correspondente sob a GNU AGPLv3. |
| **Crates Rust Upstream, Fontes e SQLite** | MIT / Apache-2.0 / BSD / OFL-1.1 / Domínio Público | Atribuições preservadas conforme as licenças permissivas originais; `self_cell` eleito sob `Apache-2.0`; SQLite em Domínio Público. |

---

## 2. Direitos de Código Aberto (GNU AGPLv3) e Marca (Seção 7e)

Todo o código-fonte público do **GeoEstabil** — incluindo os módulos de estabilidade de taludes (Fellenius, Bishop Simplificado, Janbu Simplificado com paralelismo via Rayon), empuxos e muros de arrimo (Rankine, Coulomb, tombamento, deslizamento, excentricidade e Hansen/Meyerhof), cálculos didáticos de concreto armado (referência NBR 6118), interface `egui`/`wgpu`, persistência local SQLite, exportações HTML5/PDF/Markdown/SVG e CLI `geoestabil-cli.exe` — é licenciado sob a **GNU Affero General Public License v3 (`AGPL-3.0-only`)**.

- Qualquer distribuição de versão modificada ou disponibilização em rede (Seção 13) deve manter a **GNU AGPLv3** e disponibilizar o respectivo código-fonte modificado;
- Nos termos da **Seção 7(e)** da GNU AGPLv3, a licença do código não transfere direitos sobre o nome ou marca **"GeoEstabil"**; forks ou builds modificados de terceiros devem usar nome próprio e não sugerir endosso oficial do autor.

---

## 3. Duplo Licenciamento, Suporte por E-mail e Isenção de Garantias

O projeto adota um modelo simples de **Duplo Licenciamento**:
1. **Edição Comunitária (GNU AGPLv3):** Acesso aberto e gratuito ao código-fonte para estudo acadêmico, ensino, modificação e compartilhamento;
2. **Edição Comercial (Microsoft Store):** Binários distribuídos na Microsoft Store sob **EULA** (`src/assets/EULA/`) e **CLC** (`src/assets/CLC/`), dispensando o adquirente da loja das obrigações de reciprocidade da GNU AGPLv3.

> **Declaração Expressa de Fernando Nillsson Cidade:**  
> Eu, **Fernando Nillsson Cidade**, autor original e titular dos direitos autorais do **GeoEstabil**, autorizo qualquer pessoa a usar, estudar, copiar, modificar e redistribuir o código-fonte público deste projeto sob os termos da **GNU AGPLv3**.  
> As contribuições submetidas voluntariamente no repositório seguem o mesmo espírito aberto, sem exigência de contratos complexos (CLA/DCO), permitindo que o projeto permaneça simples para a comunidade e compatível com as edições oficiais.  
> Todo eventual suporte técnico relacionado ao software é prestado **exclusivamente via e-mail (`linceu_lighthouse@outlook.com`)**, **conforme a disponibilidade do autor e sem obrigação de urgência, SLA ou prazo mínimo**.

O software é ferramenta auxiliar de estudo acadêmico fornecida **"no estado em que se encontra" ("AS IS")**, sem garantias expressas ou implícitas. Os resultados não substituem conferência manual, ferramentas consolidadas de mercado, ensaios reais ou acompanhamento por profissional habilitado, ficando o autor integralmente isento por qualquer uso indevido ou aplicação produtiva por conta e risco do usuário.

---

## 4. Documentação em `src/assets/` e Flexibilidade para Forks

Os textos informativos e contratuais em `src/assets/` (`ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY`, `RELEASE`) destinam-se à distribuição oficial do projeto:
- Desenvolvedores que criarem *forks* têm liberdade para editar, traduzir, substituir ou remover esses arquivos conforme suas necessidades locais, preservando os créditos autorais e a distinção nominal da marca.

---

## 5. Dependências de Terceiros e Normas ABNT

O **GeoEstabil** utiliza exclusivamente bibliotecas compatíveis com código aberto permissivo ou Domínio Público (`egui`/`eframe`, `wgpu`, `rayon`, `rusqlite` + SQLite em Domínio Público, `serde`, `chrono`, `rfd`, `self_cell` eleito como `Apache-2.0`, `windows-sys` e fontes `OFL-1.1` / `Ubuntu Font Licence 1.0`).

As normas técnicas eventualmente mencionadas (como ABNT NBR 11682, NBR 6122, NBR 6118 e NBR 6484) são de titularidade exclusiva da **Associação Brasileira de Normas Técnicas (ABNT)** (<https://www.abnt.org.br>). O GeoEstabil não possui vínculo nem certificação da ABNT e não distribui textos normativos protegidos.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary><b>Click to expand / collapse the licensing document in English</b></summary><br>

# GeoEstabil — Licensing, Open-Source Terms & Commercial Clarification

**Version:** `2026.11.12.0` (Cargo: `2026.11.12`)  
**Date:** `November 12, 2026`  
**Source Code License:** GNU Affero General Public License v3.0 (`GNU AGPLv3` / `AGPL-3.0-only`)  
**Author & Copyright Holder:** Fernando Nillsson Cidade  
**Support (email only, by availability, no urgency):** `linceu_lighthouse@outlook.com`

---

## 1. Licensing Model Overview

**GeoEstabil** (`geoestabil.exe` and `geoestabil-cli.exe`) is made available under a simple and transparent **Dual-Licensing** framework grounded in the exclusive copyright ownership of **Fernando Nillsson Cidade**:

| Component | License | Permitted Uses & Conditions |
|---|---|---|
| **Source Code (Public Repository)** | [GNU AGPLv3](../LICENSE) | Free to study, execute, modify, compile, and share (commercially or non-commercially), preserving authorship credits, GNU AGPLv3 on derivatives, and Section 7(e) trademark notice. |
| **Official Microsoft Store Binaries** | Commercial EULA / CLC (`src/assets/EULA/`, `src/assets/CLC/`) | Use of Store-distributed binaries without triggering GNU AGPLv3 source-disclosure obligations applicable to the open-source codebase. |
| **Documentation in `src/assets/`** | Project Authorship (Free Developer Grant) | Reference distribution texts that fork maintainers may freely adapt, edit, replace, or remove while respecting `"GeoEstabil"` brand naming. |
| **Third-Party Compiled Binaries** | GNU AGPLv3 | Free to build and distribute under your own responsibility, accompanied by the corresponding source code under GNU AGPLv3. |
| **Upstream Rust Crates, Fonts & SQLite** | MIT / Apache-2.0 / BSD / OFL-1.1 / Public Domain | Upstream attributions preserved under permissive licenses; `self_cell` elected as `Apache-2.0`; SQLite in Public Domain. |

---

## 2. Open-Source Rights (GNU AGPLv3) & Trademark Notice (Section 7e)

All public source code of **GeoEstabil** — including slope stability modules (Fellenius, Simplified Bishop, Simplified Janbu with Rayon parallelism), earth pressures and retaining walls (Rankine, Coulomb, overturning, sliding, eccentricity, and Hansen/Meyerhof), educational reinforced concrete sizing (NBR 6118 reference), `egui`/`wgpu` UI, local SQLite storage, HTML5/PDF/Markdown/SVG exports, and `geoestabil-cli.exe` — is licensed under the **GNU Affero General Public License v3 (`AGPL-3.0-only`)**.

- Any distribution of modified versions or network-hosted services (Section 13) must preserve **GNU AGPLv3** and provide the corresponding modified source code;
- Under **Section 7(e)** of GNU AGPLv3, this source license grants no rights in the `"GeoEstabil"` name or mark; third-party forks must adopt distinct names and not imply official endorsement.

---

## 3. Dual-Licensing, Email-Only Support & Warranty Disclaimer

The project operates a simple **Dual-Licensing** model:
1. **Community Edition (GNU AGPLv3):** Free open access to the source code for academic study, teaching, modification, and sharing;
2. **Commercial Edition (Microsoft Store):** Binaries distributed via the Microsoft Store under our **EULA** (`src/assets/EULA/`) and **CLC** (`src/assets/CLC/`), exempting Store buyers from GNU AGPLv3 copyleft obligations.

> **Express Statement by Fernando Nillsson Cidade:**  
> I, **Fernando Nillsson Cidade**, original creator and copyright holder of **GeoEstabil**, authorize anyone to use, study, copy, modify, and redistribute the public source code of this project under **GNU AGPLv3**.  
> Voluntary contributions to the repository follow this same open spirit without heavy legal bureaucracy (CLA/DCO), keeping the project simple for contributors and compatible with official distributions.  
> Any support related to this software is provided **exclusively via email (`linceu_lighthouse@outlook.com`)**, **according to the author's availability and with no obligation of urgency, SLA, or guaranteed timeline**.

This software is an auxiliary academic study tool provided **"AS IS"** without express or implied warranties. Outputs do not replace manual verification, established market software, field testing, or oversight by a licensed professional, and the author is wholly exempt from liability for improper use or production deployment at the user's sole risk.

---

## 4. Documentation in `src/assets/` & Fork Flexibility

The reference and packaging documents under `src/assets/` (`ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY`, `RELEASE`) are provided for official distribution:
- Downstream fork maintainers are free to edit, translate, replace, or remove these files to suit their local requirements while retaining original authorship attribution and distinct fork branding.

---

## 5. Third-Party Dependencies & ABNT Intellectual Property

**GeoEstabil** relies exclusively on permissively licensed or Public-Domain crates (`egui`/`eframe`, `wgpu`, `rayon`, `rusqlite` + Public-Domain SQLite, `serde`, `chrono`, `rfd`, `self_cell` elected under `Apache-2.0`, `windows-sys`, and `OFL-1.1` / `Ubuntu Font Licence 1.0` fonts).

Any referenced technical standards (such as ABNT NBR 11682, NBR 6122, NBR 6118, and NBR 6484) belong exclusively to **Associação Brasileira de Normas Técnicas (ABNT)** (<https://www.abnt.org.br>). GeoEstabil is not affiliated with or endorsed by ABNT and does not distribute protected standard texts.

</details>
