# Referência da Linha de Comando / Command-Line Reference (`geoestabil-cli`)

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

<details open>
<summary><strong id="ptbr">🇧🇷 Português (BR) — Automação Headless e Relatórios</strong></summary><br>

O binário **`geoestabil-cli.exe`** permite executar análises geotécnicas, dimensionamentos de muros de arrimo e geração automatizada de memoriais sem abrir a interface gráfica (`egui`).

## 1. Sintaxe Geral

```powershell
geoestabil-cli <comando> [opções]
```

## 2. Subcomandos Disponíveis

| Subcomando | Opções / Argumentos | Descrição |
| :--- | :--- | :--- |
| **`presets`** | — | Lista todos os casos pré-configurados de engenharia (`get_preset_cases`) com descrição e modo ativo. |
| **`run-preset`** | `<1..4>` `[--output \| -o <arquivo.html>]` | Executa o caso de referência indicado (talude via Rayon ou muro NBR 6122/6118) e exporta Memorial em HTML se `--output` for informado. |
| **`slope`** | `--input \| -i <proj.geoestabil>` `[--output \| -o <arquivo.html>]` | Carrega projeto JSON `.geoestabil`, executa varredura paralela Rayon (Fellenius, Bishop Simplificado e Janbu Simplificado) e exibe o círculo crítico. |
| **`wall`** | `--input \| -i <proj.geoestabil>` `[--output \| -o <arquivo.html>]` | Executa verificação externa do muro (`FS_tomb`, `FS_desl`) e dimensionamento da armadura do fuste conforme NBR 6118. |
| **`soils`** | — | Imprime em tabela a base padrão de solos típicos brasileiros correlacionados ao $N_{SPT}$ (ABNT NBR 6122). |
| **`version`** | (`--version` / `-v`) | Exibe a versão semântica do pacote (`2026.11.11`), normas centrais e dados de autoria. |
| **`help`** | (`--help` / `-h`) | Imprime o guia rápido de uso no terminal. |

## 3. Exemplos Práticos

```powershell
# Listar casos de referência disponíveis
.\geoestabil-cli.exe presets

# Rodar caso pré-configurado #1 e exportar memorial em HTML
.\geoestabil-cli.exe run-preset 1 --output memorial_caso1.html

# Analisar um arquivo de projeto salvo pela interface gráfica
.\geoestabil-cli.exe slope --input .\obras\talude_corte.geoestabil --output .\relatorios\talude.html

# Analisar muro de arrimo e exportar memorial
.\geoestabil-cli.exe wall -i .\obras\muro_flexao.geoestabil -o .\relatorios\muro.html

# Consultar tabela de solos típicos SPT (NBR 6122)
.\geoestabil-cli.exe soils
```

</details>

<details>
<summary><strong id="english-us">🇺🇸 English (US) — Headless Automation & Reports</strong></summary><br>

The **`geoestabil-cli.exe`** binary executes geotechnical analyses, retaining wall structural sizing, and automated calculation memorial generation without opening the graphical UI (`egui`).

## 1. General Syntax

```powershell
geoestabil-cli <command> [options]
```

## 2. Available Subcommands

| Subcommand | Options / Arguments | Description |
| :--- | :--- | :--- |
| **`presets`** | — | Lists all pre-configured benchmark engineering cases (`get_preset_cases`) with descriptions and active solver mode. |
| **`run-preset`** | `<1..4>` `[--output \| -o <file.html>]` | Runs the specified benchmark case (Rayon slope sweep or NBR 6122/6118 wall) and writes an HTML Memorial when `--output` is supplied. |
| **`slope`** | `--input \| -i <proj.geoestabil>` `[--output \| -o <file.html>]` | Loads a `.geoestabil` JSON project, executes parallel Rayon sweeps (Fellenius, Simplified Bishop, Simplified Janbu), and reports the critical circle. |
| **`wall`** | `--input \| -i <proj.geoestabil>` `[--output \| -o <file.html>]` | Runs external retaining wall checks (`FS_overturning`, `FS_sliding`) and stem rebar design under ABNT NBR 6118. |
| **`soils`** | — | Prints a formatted catalog table of reference Brazilian soils correlated to $N_{SPT}$ (ABNT NBR 6122). |
| **`version`** | (`--version` / `-v`) | Displays semantic package version (`2026.11.11`), core standards, and authorship credits. |
| **`help`** | (`--help` / `-h`) | Prints the quick CLI usage reference in the terminal. |

## 3. Practical Examples

```powershell
# List available benchmark reference cases
.\geoestabil-cli.exe presets

# Run benchmark case #1 and export an HTML memorial
.\geoestabil-cli.exe run-preset 1 --output memorial_case1.html

# Analyze a project file saved from the desktop UI
.\geoestabil-cli.exe slope --input .\projects\cut_slope.geoestabil --output .\reports\slope.html

# Analyze a retaining wall project and export its memorial
.\geoestabil-cli.exe wall -i .\projects\cantilever_wall.geoestabil -o .\reports\wall.html

# Query the SPT soil correlation table (NBR 6122)
.\geoestabil-cli.exe soils
```

</details>
