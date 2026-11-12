# Referências Técnicas e Aviso Legal / Technical References & Legal Disclaimer — GeoEstabil (`v2026.11.12.0`)

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

<details open>
<summary><strong id="ptbr">🇧🇷 Português (BR) — Referências Técnicas, Privacidade e Isenção</strong></summary><br>

Este documento resume as referências bibliográficas das normas técnicas implementadas para fins didáticos no **GeoEstabil** (`geoestabil.exe` e `geoestabil-cli.exe`), além do seu modelo simples de privacidade local, licenciamento e isenção legal.

## 1. Licenciamento, Privacidade e Canal de Suporte

| Aspecto | Tratamento Simples no GeoEstabil |
|---|---|
| **Código Aberto (GitHub)** | **GNU AGPLv3** ([`LICENSE`](../LICENSE)), permitindo livre uso, estudo, modificação e compartilhamento sob os mesmos termos (com proteção nominal de marca pela Seção 7e). |
| **Edição Comercial (Microsoft Store)** | Regida por **EULA** (`src/assets/EULA/`) e **CLC** (`src/assets/CLC/`), preservando eventuais direitos legais irrenunciáveis aplicáveis sem compromissos de SLA. |
| **Execução e Privacidade** | Arquitetura **100% Local-First e Offline** (`docs/security-model.md`), sem coleta de dados, sem nuvem e sem telemetria; armazenamento local em `%APPDATA%\GeoEstabil\geoestabil.db`. |
| **Canal Oficial de Suporte** | Atendimento prestado **exclusivamente por e-mail (`linceu_lighthouse@outlook.com`)**, **conforme a disponibilidade do autor e sem obrigação de urgência**. |

## 2. Referências Técnicas Consultadas (Estudo Acadêmico)

| Norma Técnica Referenciada | Tema Abordado para Estudo | Rotinas Implementadas para Simulação Didática |
|---|---|---|
| **ABNT NBR 11682:2009** | Estabilidade de Encostas e Taludes | Métodos de Fellenius, Bishop Simplificado e Janbu Simplificado; valores referenciais de Fator de Segurança ($1{,}50$ Alto / $1{,}30$ Médio / $1{,}20$ Baixo); poropressão freática; sobrecargas. |
| **ABNT NBR 6122:2022** | Fundações & Muros de Arrimo | Empuxos Rankine e Coulomb; referências de $FS_{\text{tomb}} \ge 1{,}50$, $FS_{\text{desl}} \ge 1{,}50$, excentricidade $e \le B/6$ e capacidade de carga Meyerhof/Hansen. |
| **ABNT NBR 6118:2023** | Estruturas de Concreto Armado | Coeficientes $\gamma_f=1{,}40$, $\gamma_c=1{,}40$, $\gamma_s=1{,}15$; cobrimento por CAA; flexão simples ($x/d \le 0{,}45$); $A_{s,\min}=0{,}15\%$; cortante $V_{Rd1}$; estimativa de fissuração ($w_k \le 0{,}30\text{ mm}$). |
| **ABNT NBR 6484:2020** | Sondagem SPT e Catálogo de Solos | Correlações empíricas ilustrativas e editáveis ($N_{SPT}$ vs. $c'$, $\phi'$, $\gamma$) em banco local SQLite (`geoestabil.db`). |

## 3. Isenção de Responsabilidade e Caráter Não Certificador

> [!NOTE]
> **Aviso Legal:** O **GeoEstabil** é uma ferramenta computacional auxiliar desenvolvida exclusivamente para estudo acadêmico, ensino e experimentação didática, podendo conter erros de cálculo ou limitações numéricas. O software **não possui certificação nem afiliação com a ABNT, CREA ou órgãos reguladores**, não substitui projetos executivos, ensaios geotécnicos ou conferência manual em ferramentas consolidadas de mercado e é fornecido **"no estado em que se encontra" ("AS IS")**. Qualquer utilização definitiva em produção ocorre por conta e risco exclusivos do usuário, com total isenção legal do autor.

</details>

<details>
<summary><strong id="enus">🇺🇸 English (US) — Technical References, Privacy & Legal Disclaimer</strong></summary><br>

This document summarizes the technical standard references used for educational calculations in **GeoEstabil** (`geoestabil.exe` and `geoestabil-cli.exe`), together with its simple local privacy, licensing, and legal exemption model.

## 1. Licensing, Privacy & Support Channel

| Aspect | Simple Handling in GeoEstabil |
|---|---|
| **Open Source (GitHub)** | **GNU AGPLv3** ([`LICENSE`](../LICENSE)), allowing free use, study, modification, and sharing under the same license (with Section 7(e) brand protection). |
| **Commercial Edition (Microsoft Store)** | Governed by **EULA** (`src/assets/EULA/`) and **CLC** (`src/assets/CLC/`), preserving applicable non-waivable statutory rights without SLA commitments. |
| **Execution & Privacy** | **100% Local-First & Offline** architecture (`docs/security-model.md`) with zero data collection, cloud calls, or telemetry; local storage in `%APPDATA%\GeoEstabil\geoestabil.db`. |
| **Official Support Channel** | Assistance handled **exclusively via email (`linceu_lighthouse@outlook.com`)**, **according to author availability and with no obligation of urgency**. |

## 2. Technical Standards Referenced (Academic Study)

| Referenced Standard | Educational Topic Covered | Simulation Routines Implemented |
|---|---|---|
| **ABNT NBR 11682:2009** | Slope Stability Analysis | Ordinary (Fellenius), Simplified Bishop, and Simplified Janbu; reference Safety Factors ($1.50$ High / $1.30$ Medium / $1.20$ Low); pore pressures; surcharges. |
| **ABNT NBR 6122:2022** | Foundations & Retaining Walls | Rankine and Coulomb earth pressures; reference checks $FS_{\text{overturning}} \ge 1.50$, $FS_{\text{sliding}} \ge 1.50$, eccentricity $e \le B/6$, and Meyerhof/Hansen capacity. |
| **ABNT NBR 6118:2023** | Reinforced Concrete Structures | Factors $\gamma_f=1.40$, $\gamma_c=1.40$, $\gamma_s=1.15$; CAA environmental cover; flexure ($x/d \le 0.45$); $A_{s,\min}=0.15\%$; shear $V_{Rd1}$; crack estimate ($w_k \le 0.30\text{ mm}$). |
| **ABNT NBR 6484:2020** | SPT Sounding & Soil Catalog | Editable illustrative empirical correlations ($N_{SPT}$ vs. $c'$, $\phi'$, $\gamma$) stored in local SQLite (`geoestabil.db`). |

## 3. Legal Disclaimer & Non-Certification Notice

> [!NOTE]
> **Legal Notice:** **GeoEstabil** is an auxiliary computational tool developed exclusively for academic study, teaching, and didactic experimentation, and may contain calculation errors or numerical limitations. The software holds **no certification or affiliation with ABNT, CREA, or regulatory authorities**, does not replace formal engineering design, field testing, or manual cross-checking against established market tools, and is supplied strictly **"AS IS"**. Any definitive production use occurs at the user's sole risk, with full legal exemption of the author.

</details>
