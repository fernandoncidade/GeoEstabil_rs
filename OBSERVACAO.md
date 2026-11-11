# Observações Técnicas / Technical Notes — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details open>
<summary><b>Clique para expandir / recolher as observações técnicas em português</b></summary><br>

# GeoEstabil — Notas de Engenharia de Software, Compilação e Empacotamento

Versão documentada: `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  
**Autor:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  

---

### 1. Isolamento dos Diretórios de Compilação (`.cargo/config.toml`)
Para preservar a raiz do repositório limpa e evitar dispersão de artefatos intermediários do compilador `rustc`:
- A configuração do Cargo (`.cargo/config.toml`) redireciona o diretório de destino (`target-dir`) para a pasta `builds/geoestabil/`.
- Os binários executáveis otimizados em modo release (`lto = true`, `codegen-units = 1`, `strip = true`) são gerados em:
  - `builds/geoestabil/release/geoestabil.exe` (Aplicativo desktop principal com GUI)
  - `builds/geoestabil/release/geoestabil-cli.exe` (Utilitário autônomo de linha de comando)

---

### 2. Recursos Win32, Metadados PE e Ícones (`src/build.rs`)
- O arquivo `src/build.rs` utiliza a biblioteca `winres` para compilar o ícone oficial multi-resolução (`src/assets/icons/geoestabil.ico`) diretamente na tabela de recursos PE (*Portable Executable*) do Windows;
- O `build.rs` também estampa metadados formais no cabeçalho binário (Copyright de Fernando Nillsson Cidade, Descrição do Produto e Nome do Produto);
- Em tempo de execução, a função `carregar_icone_janela()` (`src/utils/icons.rs`) tenta carregar o ícone oficial a partir de `src/assets/icons/` (ou `assets/icons/`) e conta com fallback garantido em memória através de `include_bytes!`.

---

### 3. Registro do AppUserModelID no Windows 11
- Para garantir que o GeoEstabil seja reconhecido de forma única pelo Windows Shell e pela Barra de Tarefas do Windows 11, o ponto de entrada `src/main.rs` executa a chamada Win32:
  ```rust
  SetCurrentProcessExplicitAppUserModelID("fernandoncidade.GeoEstabil\0")
  ```
- Essa configuração assegura o agrupamento correto de janelas, a exibição do ícone de alta densidade e a fixação estável de atalhos.

---

### 4. Distribuição Limpa e Integração de Assets (`scripts/gerar_distribuicao_limpa.ps1`)
- O script `scripts/gerar_distribuicao_limpa.ps1` empacota os binários Release (`geoestabil.exe`, `geoestabil-cli.exe`), documentação (`README.md`, `LICENSE`, `MANUAL.md`, `OBSERVACAO.md`, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`), a pasta `docs\*.md` e todo o diretório `src/assets/` (`ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY`, `RELEASE` e `icons`) para a distribuição local (`builds/geoestabil/distribuicao`) e distribuição externa.

---

### 5. Banco Local SQLite3 (`src/data/database.rs`)
- Utiliza `rusqlite` com a *feature* `bundled` (motor C do SQLite embutido estaticamente, sem necessidade de DLLs externas).
- Armazena catálogo de solos NBR 6122/SPT, histórico de projetos e log de auditoria em `%APPDATA%\GeoEstabil\geoestabil.db`.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary><b>Click to expand / collapse the technical notes in English</b></summary><br>

# GeoEstabil — Software Engineering, Build, and Packaging Notes

Documented version: `2026.11.11.0`  
**Date:** `November 11, 2026`  
**Author:** Fernando Nillsson Cidade (`linceu_lighthouse@outlook.com`)  

---

### 1. Isolated Build Target Directory (`.cargo/config.toml`)
To keep the repository root clean and prevent intermediate compiler clutter:
- Cargo configuration (`.cargo/config.toml`) redirects the `target-dir` to `builds/geoestabil/`.
- Optimized production release binaries (`lto = true`, `codegen-units = 1`, `strip = true`) are produced at:
  - `builds/geoestabil/release/geoestabil.exe` (Main desktop GUI application)
  - `builds/geoestabil/release/geoestabil-cli.exe` (Headless command-line utility)

---

### 2. Win32 Resources, PE Metadata, and Icons (`src/build.rs`)
- The `src/build.rs` build script uses `winres` to compile the official multi-resolution icon (`src/assets/icons/geoestabil.ico`) directly into the Windows Portable Executable (PE) resource table;
- `build.rs` also embeds formal metadata into the binary header (Copyright by Fernando Nillsson Cidade, Product Description, and Product Name);
- At runtime, `carregar_icone_janela()` (`src/utils/icons.rs`) loads the official icon from `src/assets/icons/` (or `assets/icons/`) with a resilient in-memory fallback via `include_bytes!`.

---

### 3. Windows Shell AppUserModelID Registration
- To ensure unique identification by the Windows 11 Taskbar and Explorer Shell, the entry point in `src/main.rs` invokes the native Win32 API:
  ```rust
  SetCurrentProcessExplicitAppUserModelID("fernandoncidade.GeoEstabil\0")
  ```
- This guarantees proper window grouping, crisp high-DPI icon rendering, and reliable shortcut pinning behavior.

---

### 4. Clean Distribution & Asset Bundling (`scripts/gerar_distribuicao_limpa.ps1`)
- The script `scripts/gerar_distribuicao_limpa.ps1` packages the Release binaries (`geoestabil.exe`, `geoestabil-cli.exe`), root documentation (`README.md`, `LICENSE`, `MANUAL.md`, `OBSERVACAO.md`, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`), the `docs\*.md` guides, and the complete `src/assets/` directory (`ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY`, `RELEASE`, and `icons`) into both local (`builds/geoestabil/distribuicao`) and external distribution targets.

---

### 5. Local SQLite3 Database (`src/data/database.rs`)
- Uses `rusqlite` with the `bundled` feature (statically compiled SQLite C engine requiring zero external runtime DLLs).
- Persists the NBR 6122/SPT soil catalog, project history, and audit log locally in `%APPDATA%\GeoEstabil\geoestabil.db`.

</details>

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\src`

```
├── 📁 assets
│   ├── 📁 ABOUT
│   │   ├── 📄 ABOUT_en_US.txt
│   │   ├── 📄 ABOUT_pt_BR.txt
│   │   ├── 📄 History_APP_en_US.txt
│   │   └── 📄 History_APP_pt_BR.txt
│   ├── 📁 CLC
│   │   ├── 📄 CLC_en_US - GeoEstabil.txt
│   │   └── 📄 CLC_pt_BR - GeoEstabil.txt
│   ├── 📁 COPYRIGHT
│   │   ├── 📄 AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt
│   │   └── 📄 COPYRIGHT AND TRADEMARK NOTICE_en_US.txt
│   ├── 📁 EULA
│   │   ├── 📄 EULA_en_US - GeoEstabil.txt
│   │   └── 📄 EULA_pt_BR - GeoEstabil.txt
│   ├── 📁 LICENSES
│   │   ├── 📄 Apache License Version 2.0.txt
│   │   ├── 📄 BSD License Clause-2.txt
│   │   ├── 📄 BSD License Clause-3.txt
│   │   ├── 📄 Boost Software License 1.0.txt
│   │   ├── 📄 GNU AFFERO GENERAL PUBLIC LICENSE Version3.txt
│   │   ├── 📄 GNU GENERAL PUBLIC LICENSE Version 3.txt
│   │   ├── 📄 GNU LESSER GENERAL PUBLIC LICENSE Version 2.1.txt
│   │   ├── 📄 GNU LESSER GENERAL PUBLIC LICENSE Version 3.txt
│   │   ├── 📄 ISC LICENSE.txt
│   │   ├── 📄 MIT License.txt
│   │   ├── 📄 PYTHON SOFTWARE FOUNDATION LICENSE.txt
│   │   ├── 📄 SIL Open Font License 1.1.txt
│   │   ├── 📄 SQLite Is Public Domain.txt
│   │   ├── 📄 Ubuntu Font Licence 1.0.txt
│   │   ├── 📄 Unicode License v3.txt
│   │   └── 📄 Zlib License.txt
│   ├── 📁 NOTICES
│   │   ├── 📄 NOTICE_en_US.txt
│   │   └── 📄 NOTICE_pt_BR.txt
│   ├── 📁 PRIVACY_POLICY
│   │   ├── 📄 Privacy_Policy_en_US.txt
│   │   └── 📄 Privacy_Policy_pt_BR.txt
│   ├── 📁 RELEASE
│   │   ├── 📄 RELEASE NOTES_en_US.txt
│   │   └── 📄 RELEASE NOTES_pt_BR.txt
│   └── 📁 icons
│       ├── 📄 geoestabil.ico
│       ├── 🖼️ geoestabil.png
│       ├── 🖼️ geoestabil_1080-1080.png
│       ├── 🖼️ geoestabil_1440-2160.png
│       ├── 🖼️ geoestabil_150-150.png
│       ├── 🖼️ geoestabil_1919-1079.png
│       ├── 🖼️ geoestabil_1920-1080.png
│       ├── 🖼️ geoestabil_2160-2160.png
│       ├── 🖼️ geoestabil_300-300.png
│       ├── 🖼️ geoestabil_3840-2160.png
│       ├── 🖼️ geoestabil_584-800.png
│       └── 🖼️ geoestabil_71-71.png
├── 📁 core
│   ├── 🦀 mod.rs
│   ├── 🦀 presets.rs
│   ├── 🦀 project.rs
│   └── 🦀 types.rs
├── 📁 data
│   ├── 🦀 database.rs
│   └── 🦀 mod.rs
├── 📁 engine
│   ├── 📁 slope
│   │   ├── 🦀 bishop.rs
│   │   ├── 🦀 fellenius.rs
│   │   ├── 🦀 geometry.rs
│   │   ├── 🦀 grid_search.rs
│   │   ├── 🦀 janbu.rs
│   │   ├── 🦀 mod.rs
│   │   ├── 🦀 results.rs
│   │   └── 🦀 slices.rs
│   ├── 📁 soils
│   │   ├── 🦀 mod.rs
│   │   └── 🦀 nbr6122_soils.rs
│   ├── 📁 wall
│   │   ├── 🦀 bearing_capacity.rs
│   │   ├── 🦀 concrete_design.rs
│   │   ├── 🦀 earth_pressure.rs
│   │   ├── 🦀 geometry.rs
│   │   ├── 🦀 mod.rs
│   │   └── 🦀 stability.rs
│   └── 🦀 mod.rs
├── 📁 locale
│   ├── 🦀 manual.rs
│   ├── 🦀 mod.rs
│   └── 🦀 translations.rs
├── 📁 reports
│   ├── 🦀 memorial_generator.rs
│   ├── 🦀 mod.rs
│   └── 🦀 pdf_writer.rs
├── 📁 ui
│   ├── 📁 dialogs
│   │   ├── 🦀 about_dialog.rs
│   │   ├── 🦀 doc_renderer.rs
│   │   ├── 🦀 manual_dialog.rs
│   │   ├── 🦀 mod.rs
│   │   ├── 🦀 modeless.rs
│   │   ├── 🦀 settings_dialog.rs
│   │   └── 🦀 soil_catalog_dialog.rs
│   ├── 📁 tabs
│   │   ├── 🦀 mod.rs
│   │   ├── 🦀 tab1_geotechnical.rs
│   │   ├── 🦀 tab2_slope_stability.rs
│   │   ├── 🦀 tab3_retaining_wall.rs
│   │   ├── 🦀 tab4_structural_design.rs
│   │   └── 🦀 tab5_memorial.rs
│   ├── 🦀 app.rs
│   ├── 🦀 canvas_2d.rs
│   ├── 🦀 export_actions.rs
│   ├── 🦀 menu_bar.rs
│   ├── 🦀 mod.rs
│   ├── 🦀 theme.rs
│   └── 🦀 toolbar.rs
├── 📁 utils
│   ├── 🦀 export_dxf_svg.rs
│   ├── 🦀 icons.rs
│   ├── 🦀 mod.rs
│   └── 🦀 paths.rs
├── 🦀 build.rs
├── 🦀 lib.rs
└── 🦀 main.rs
```

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\tests`

```
├── 🦀 dialog_render_test.rs
├── 🦀 export_zorder_and_i18n_test.rs
├── 🦀 geotechnical_tests.rs
└── 🦀 theme_and_i18n_test.rs
```

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\version`

```
├── 📁 .cargo
│   └── ⚙️ config.toml
├── 📁 src
│   ├── 🦀 app.rs
│   ├── 🦀 engine.rs
│   └── 🦀 main.rs
├── 📄 Cargo.lock
├── ⚙️ Cargo.toml
└── 📝 README.md
```

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\docs`

```
├── 📝 MIGRATION_REPORT.md
├── 📝 architecture.md
├── 📝 cli-reference.md
├── 📝 compliance.md
├── 📝 licensing.md
├── 📝 release-v2026.11.11.0.md
├── 📝 security-model.md
├── 📝 suggested-issues.md
└── 📝 user-guide.md
```

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\repo\LISTAGEM`

```
├── 📁 en_US
│   ├── 🖼️ Captura de tela 2026-11-11 002418.png
│   ├── 🖼️ Captura de tela 2026-11-11 002447.png
│   ├── 🖼️ Captura de tela 2026-11-11 002513.png
│   ├── 🖼️ Captura de tela 2026-11-11 002525.png
│   └── 🖼️ Captura de tela 2026-11-11 002540.png
└── 📁 pt_BR
    ├── 🖼️ Captura de tela 2026-11-11 002316.png
    ├── 🖼️ Captura de tela 2026-11-11 002325.png
    ├── 🖼️ Captura de tela 2026-11-11 002335.png
    ├── 🖼️ Captura de tela 2026-11-11 002344.png
    └── 🖼️ Captura de tela 2026-11-11 002354.png
```

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\scripts`

```
└── 📄 gerar_distribuicao_limpa.ps1
```

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\.github`

```
├── 📁 ISSUE_TEMPLATE
│   ├── 📝 bug_report.md
│   ├── ⚙️ config.yml
│   ├── 📝 feature_request.md
│   └── 📝 question.md
├── 📁 workflows
│   └── ⚙️ update-readme-version.yml
└── 📝 PULL_REQUEST_TEMPLATE.md
```

**Root Path:** `c:\Users\ferna\APLICATIVOS\RS\GeoEstabil_rs\.cargo`

```
└── ⚙️ config.toml
```
