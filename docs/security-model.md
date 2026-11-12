# Modelo de Segurança e Privacidade / Security & Privacy Model — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details open>
<summary><b>Clique para expandir / recolher o modelo de segurança em português</b></summary><br>

# GeoEstabil — Modelo de Segurança, Privacidade Soberana e Análise STRIDE

**Versão:** `2026.11.12.0` (Cargo: `2026.11.12`) | **Data:** `12 de novembro de 2026`  
**Nível de Segurança:** 100% Local-First | Zero Telemetria | Cálculo Determinístico (Sem IA Externa)  
**Ambiente Operacional:** Windows 11 / Windows 10 (x64) | **AppUserModelID:** `fernandoncidade.GeoEstabil`  
**Controlador Local da Aplicação:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  
**Controladora da Loja e Plataforma Windows:** Microsoft Corporation (<https://www.microsoft.com/pt-br/privacy/privacystatement>)

---

### 1. Filosofia de Privacidade Soberana e Garantias Centrais

Em projetos de contenções, fundações e estabilidade de encostas, perfis estratigráficos, laudos SPT, dados de clientes e memórias de ART constituem propriedade técnica sigilosa. Por isso, o **GeoEstabil** (`geoestabil.exe` e `geoestabil-cli.exe`) foi projetado sob arquitetura **100% Local-First**:

1. **Zero Pilha de Rede no Aplicativo (Offline por Definição):** O executável não utiliza `reqwest`, `hyper`, sockets TCP/UDP, WebSockets, DNS ou SDKs de analytics e telemetria, funcionando integralmente em estações isoladas da Internet (*air-gapped*).
2. **Determinismo Numérico (Sem LLMs ou Nuvem):** Todos os fatores de segurança (Fellenius, Bishop Simplificado, Janbu Simplificado), empuxos (Rankine, Coulomb), capacidade de carga (Hansen/Meyerhof) e dimensionamento em concreto armado (NBR 6118:2023) executam localmente em aritmética `f64` determinística e paralelismo em CPU via Rayon.
3. **Execução em Privilégio de Usuário Padrão:** Opera sem elevação administrativa (`UAC`), sem drivers de kernel e sem serviços residentes em segundo plano.

---

### 2. Ciclo de Vida dos Dados, SQLite Local e Exportações

* **Banco de Dados Local SQLite (`%APPDATA%\GeoEstabil\geoestabil.db`):** Gerenciado por `rusqlite` com **consultas parametrizadas (*prepared statements*, `params![...]`)**, armazenando localmente apenas o catálogo editável de solos (`soils_catalog`) e o histórico local de projetos (`project_history`).
* **Exportações Explícitas do Usuário:** Gravação em disco ocorre exclusivamente quando acionada pelo usuário via diálogos nativos do Windows (`rfd`) ou via CLI (`geoestabil-cli --output`), produzindo arquivos `.geoestabil` (JSON tipado via `serde_json` com validação contra `NaN`, infinitos e parâmetros físicos inválidos), memoriais `.html` (HTML5/PDF), `.md` (Markdown) e perfis vetoriais `.svg`.
* **Resolução Segura de Ativos (`resolver_caminho_asset`):** Restringe a leitura de documentação jurídica e ícones às pastas relativas autorizadas junto ao executável (`distribuicao\src\assets`, pacote MSIX ou manifesto Cargo), com fallback determinístico compilado em `include_str!`.

---

### 3. Separação de Controladores e Execução Offline

| Esfera de Tratamento | Entidade Controladora | Dados Tratados e Controle do Usuário |
|---|---|---|
| **Execução Desktop (`geoestabil.exe` / `cli`)** | **Fernando Nillsson Cidade** | **Coleta zero de dados pela rede.** Geometrias, solos, anotações e arquivos `.geoestabil`, `.html`, `.md` e `.svg` permanecem 100% na estação local do usuário. |
| **Permissões de Sistema & Isolamento** | **Windows** | Não solicita permissão de **Diagnóstico de Aplicativos (*App Diagnostics*)** sobre outros processos, nem localização, câmera, microfone, contatos ou rede. Executa apenas para renderização local, SQLite em `%APPDATA%` e seletores de arquivo do usuário. |
| **Suporte Voluntário por E-mail** | **Fernando Nillsson Cidade** | Apenas mensagens enviadas ativamente pelo usuário para `linceu_lighthouse@outlook.com`, atendidas conforme a disponibilidade do autor e sem obrigação de urgência, jamais usadas para marketing ou treinamento de IA. |
| **Compra na Loja, Conta & Diagnóstico Windows** | **Microsoft Corporation** (Controladora Independente) | Faturamento, impostos, reembolsos, Autenticação Microsoft, licença da loja e **Dados de Diagnóstico do Windows** seguem exclusivamente a [Declaração de Privacidade da Microsoft](https://www.microsoft.com/pt-br/privacy/privacystatement). O desenvolvedor vê apenas agregados anonimizados da loja. |

* **Política de Privacidade Local:** Disponível em `docs/security-model.md`, `src/assets/PRIVACY_POLICY/` e na janela **Sobre → Privacidade** dentro do aplicativo.
* **Uso Acadêmico:** Ferramenta computacional auxiliar offline sem comunicação entre usuários, publicidade ou rastreamento.

---

### 4. Matriz STRIDE e Segurança em Memória Rust

* **Spoofing e Tampering:** Arquivos `.geoestabil` desserializados via `serde_json` com checagem de parâmetros numéricos finitos positivos e invariantes geométricos.
* **Information Disclosure e Elevation:** Zero conexões externas; execução sem privilégios administrativos; ausência de scripts dinâmicos ou consultas SQL sem parametrização.
* **Memory Safety (Rust 2021):** Proteção em tempo de compilação contra *buffer overflows*, *use-after-free* e *data races* no pool Rayon.
* **Relato de Vulnerabilidades:** Consulte [`SECURITY.md`](../SECURITY.md) (contato exclusivo por e-mail em `linceu_lighthouse@outlook.com`, sem SLA nem obrigação de urgência).

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary><b>Click to expand / collapse the security model in English</b></summary><br>

# GeoEstabil — Security Model, Sovereign Privacy & STRIDE Threat Analysis

**Version:** `2026.11.12.0` (Cargo: `2026.11.12`) | **Date:** `November 12, 2026`  
**Security Posture:** 100% Local-First | Zero Telemetry | Deterministic Calculation (No External Cloud AI)  
**Operating Environment:** Windows 11 / Windows 10 (x64) | **AppUserModelID:** `fernandoncidade.GeoEstabil`  
**Local Application Controller:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  
**Store & Windows Platform Controller:** Microsoft Corporation (<https://www.microsoft.com/pt-br/privacy/privacystatement>)

---

### 1. Sovereign Privacy Philosophy & Core Architectural Guarantees

In earth-retaining structures, foundations, and slope stability engineering, stratigraphic profiles, SPT boring logs, client metadata, and ART calculation books constitute sensitive proprietary intellectual property. Accordingly, **GeoEstabil** (`geoestabil.exe` and `geoestabil-cli.exe`) is architected as **100% Local-First**:

1. **Zero Network Stack inside the Application (Offline by Design):** The executable excludes `reqwest`, `hyper`, TCP/UDP sockets, WebSockets, DNS calls, or analytics and telemetry SDKs, functioning reliably inside completely isolated air-gapped workstations.
2. **Numerical Determinism (Zero LLMs or Cloud Services):** Every safety factor (Fellenius, Simplified Bishop, Simplified Janbu), earth pressure (Rankine, Coulomb), bearing capacity check (Hansen/Meyerhof), and reinforced concrete structural sizing calculation (NBR 6118:2023) executes locally using deterministic `f64` arithmetic and Rayon CPU parallelism.
3. **Standard User Privilege Execution (Least Privilege):** Executes strictly without Administrator elevation (`UAC`), kernel-mode drivers, or background Windows services.

---

### 2. Data Lifecycle, Local SQLite Persistence & Exports

* **Local SQLite Database (`%APPDATA%\GeoEstabil\geoestabil.db`):** Managed through `rusqlite` using **parameterized queries (*prepared statements*, `params![...]`)**, storing locally only the editable soils catalog (`soils_catalog`) and local project history (`project_history`).
* **Explicit User-Initiated Exports:** Disk writes occur exclusively when triggered explicitly by the user via native Windows File Explorer pickers (`rfd`) or via CLI (`geoestabil-cli --output`), creating `.geoestabil` project files (typed JSON via `serde_json` validated against `NaN`, infinities, and invalid physical inputs), `.html` memorials (HTML5/PDF), `.md` (Markdown), and `.svg` vector profiles.
* **Bounded Asset Resolution (`resolver_caminho_asset`):** Confines legal document and icon lookups to authorized relative directories adjacent to the binary (`distribuicao\src\assets`, MSIX package root, or Cargo manifest), backed by compile-time embedded fallbacks via `include_str!`.

---

### 3. Controller Separation & Offline Execution

| Processing Sphere | Data Controller | Data Handled & User Control |
|---|---|---|
| **Desktop Execution (`geoestabil.exe` / `cli`)** | **Fernando Nillsson Cidade** | **Zero network data collection.** Geometries, soil parameters, study notes, and `.geoestabil`, `.html`, `.md`, and `.svg` exports stay 100% on the user's local computer. |
| **System Permissions & Isolation** | **Windows** | Requests zero **App Diagnostics** permissions across other processes, and zero location, camera, microphone, contacts, or network access. Runs solely for local rendering, `%APPDATA%` SQLite storage, and user-triggered file pickers. |
| **Voluntary E-Mail Support** | **Fernando Nillsson Cidade** | Solely e-mail messages actively sent by the user to `linceu_lighthouse@outlook.com`, handled according to maintainer availability with no obligation of urgency, and never used for marketing or AI training. |
| **Store Checkout, Account & Windows Diagnostics** | **Microsoft Corporation** (Independent Controller) | Billing, taxes, refunds, Microsoft Account sign-in, store entitlement, and **Windows Diagnostic Data** fall solely under the [Microsoft Privacy Statement](https://www.microsoft.com/pt-br/privacy/privacystatement). The developer sees only anonymized Store aggregates. |

* **Local Privacy Policy:** Maintained in `docs/security-model.md`, `src/assets/PRIVACY_POLICY/`, and in-app under **About → Privacy**.
* **Academic Use:** Auxiliary offline computational tool with no user chat, advertising, or tracking.

---

### 4. STRIDE Threat Matrix & Rust Memory Safety Guarantees

* **Spoofing & Tampering:** `.geoestabil` project files deserialized via `serde_json` enforcing geometric invariants and finite positive numerical parameters.
* **Information Disclosure & Elevation of Privilege:** Zero external sockets; unprivileged user execution; zero dynamic scripting or non-parameterized SQL statements.
* **Memory Safety (Rust 2021):** Compile-time prevention of buffer overflows, use-after-free faults, and data races across the Rayon worker pool.
* **Vulnerability Reporting:** See [`SECURITY.md`](../SECURITY.md) (contact exclusively via email at `linceu_lighthouse@outlook.com`, no SLA or urgency obligation).

</details>
