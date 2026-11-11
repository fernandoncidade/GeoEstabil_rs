# Guia de Contribuição / Contributing Guide — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details open>
<summary><b>Clique para expandir / recolher o guia de contribuição em português</b></summary><br>

# GeoEstabil — Diretrizes para Contribuição e Comunidade

**Versão:** `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  
**Autor e Mantenedor:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  
**Licenciamento:** GNU AGPLv3 (Código Aberto) / Licença Comercial (Microsoft Store)  

---

### 1. Aviso Inicial ao Colaborador e Canal de Contato

> [!NOTE]
> **Aviso de Projeto Pessoal e Ritmo Voluntário:**  
> Este repositório é um projeto pessoal, técnico e educacional mantido pelo autor em seu ritmo próprio de pesquisa e estudo. Contribuições, relatos de problemas e Pull Requests são bem-vindos, mas a análise, resposta ou incorporação de código ocorrem de forma voluntária, **sem obrigação de urgência, SLA ou compromisso de merge**.  
> Eventual suporte ou contato relacionado ao projeto é feito **exclusivamente por e-mail (`linceu_lighthouse@outlook.com`)**, conforme a disponibilidade do mantenedor. Caso suas melhorias não sejam incorporadas de imediato, sinta-se plenamente livre e incentivado a manter, modificar e compartilhar suas alterações em seu próprio *fork* público sob a licença **GNU AGPLv3**.

---

### 2. Licenciamento, Compartilhamento e Modificação

O **GeoEstabil** segue um modelo simples de **Duplo Licenciamento**:
1. O código-fonte público no GitHub é licenciado sob a **GNU Affero General Public License v3 (`AGPL-3.0-only`, [`LICENSE`](LICENSE))**, com preservação de créditos autorais e proteção nominal de nome e marca sob a Seção 7(e) (`"GeoEstabil"`);
2. As compilações comerciais distribuídas na Microsoft Store são regidas pelo **EULA Comercial** (`src/assets/EULA/`) e pelas **Condições de Licença Comercial (CLC)** (`src/assets/CLC/`).

Ao submeter qualquer contribuição voluntária ao repositório público (Issue ou Pull Request):
- Você declara que a contribuição é de sua autoria original (ou livre para submissão aberta), autoriza sua inclusão sob a licença **GNU AGPLv3** e concorda que ela também possa compor as distribuições oficiais do projeto;
- Respeite integralmente os direitos autorais das normas ABNT: **não copie textos integrais, tabelas protegidas ou reproduções literais de normas técnicas**.

---

### 3. Boas Práticas Técnicas de Contribuição

Para manter o projeto organizado e simples de compilar:
1. **Referência às Normas Técnicas (ABNT NBR 11682 / NBR 6122 / NBR 6118):**
   - Ao ajustar rotinas de cálculo em `src/engine/`, indique de forma objetiva a equação clássica ou referência bibliográfica adotada e adicione testes unitários em `tests/`;
   - Mantenha sempre o caráter acadêmico e experimental das mensagens, sem prometer certificação profissional automática.
2. **Qualidade em Rust (Edição 2021):**
   - Verifique se o código compila e passa nos testes locais antes de abrir um Pull Request:
     ```powershell
     cargo fmt --check
     cargo check --all-targets
     cargo test
     ```
3. **Privacidade Local (Local-First):**
   - Não introduza conexões de rede externas, telemetria, rastreamento ou chamadas a APIs remotas.

---

### 4. Como Enviar um Pull Request

1. Faça um *fork* do repositório oficial no GitHub;
2. Crie uma branch a partir de `main`:
   ```bash
   git checkout -b feature/melhoria-calculo
   ```
3. Implemente as alterações, escreva testes e execute `cargo fmt`;
4. Envie seus commits com descrição clara do que foi alterado;
5. Abra o Pull Request explicando o objetivo acadêmico/técnico da mudança e ciente de que a revisão ocorre conforme a disponibilidade do autor.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary><b>Click to expand / collapse the contributing guide in English</b></summary><br>

# GeoEstabil — Contribution Guidelines & Community

**Version:** `2026.11.11.0`  
**Date:** `November 11, 2026`  
**Author & Maintainer:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  
**Licensing:** GNU AGPLv3 (Open Source) / Commercial License (Microsoft Store)  

---

### 1. Contributor Notice & Support Channel

> [!NOTE]
> **Personal Project Notice & Voluntary Pace:**  
> This repository is a personal, technical, and educational project maintained by the author at his own research and study pace. Contributions, issue reports, and Pull Requests are welcome; however, review, responses, and merging happen voluntarily **with no obligation of urgency, SLA, or merge commitment**.  
> Any support or project inquiry is handled **exclusively via email (`linceu_lighthouse@outlook.com`)**, according to the maintainer's availability. If your improvements are not merged immediately, you are completely free and encouraged to maintain, modify, and share your work in your own public fork under the **GNU AGPLv3** license.

---

### 2. Licensing, Sharing & Modification

**GeoEstabil** follows a simple **Dual-Licensing** framework:
1. The public source code on GitHub is licensed under the **GNU Affero General Public License v3 (`AGPL-3.0-only`, [`LICENSE`](LICENSE))**, preserving authorship attribution and Section 7(e) brand and trademark protection (`"GeoEstabil"`);
2. Official commercial binaries distributed via the Microsoft Store are governed by our **Commercial EULA** (`src/assets/EULA/`) and **Commercial License Conditions (CLC)** (`src/assets/CLC/`).

By submitting a voluntary contribution to this public repository (Issue or Pull Request):
- You affirm that your submission is your own original work (or freely submittable), license it under **GNU AGPLv3**, and allow its inclusion in official distributions of the project;
- Strictly respect ABNT copyright rules: **do not paste verbatim standard texts, protected tables, or literal standard reproductions**.

---

### 3. Good Engineering & Contribution Practices

To keep the repository organized and easy to build:
1. **Technical Standards References (ABNT NBR 11682 / NBR 6122 / NBR 6118):**
   - When modifying calculation routines in `src/engine/`, briefly document the classical equation or literature reference used and include unit tests under `tests/`;
   - Keep user-facing text strictly framed for academic study without promising automated engineering certification.
2. **Rust Code Quality (Edition 2021):**
   - Verify compilation and local tests before opening a Pull Request:
     ```powershell
     cargo fmt --check
     cargo check --all-targets
     cargo test
     ```
3. **Local-First Privacy:**
   - Do not introduce external network connections, telemetry, analytics tracking, or remote API calls.

---

### 4. How to Submit a Pull Request

1. Fork the official repository on GitHub;
2. Create a branch from `main`:
   ```bash
   git checkout -b feature/calculation-improvement
   ```
3. Implement your changes, add tests, and run `cargo fmt`;
4. Push commits with a clear summary of what changed;
5. Open the Pull Request describing the academic/technical goal while noting that reviews depend on maintainer availability.

</details>
