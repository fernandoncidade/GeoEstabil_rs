# Roteiro Técnico e Problemas Sugeridos / Technical Roadmap & Suggested Issues — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

<details open>
<summary><strong id="ptbr">🇧🇷 Português (BR) — Evolução Futura Aberta à Comunidade</strong></summary><br>

Este documento lista ideias técnicas de estudo abertas à comunidade (sob a licença GNU AGPLv3, com revisão voluntária conforme disponibilidade do autor e sem urgência):

1. **[Módulo Taludes] Método de Morgenstern-Price / Spencer (Equilíbrio Rigoroso de Forças e Momentos):** Extensão do solver `src/engine/slope/` para incluir forças interlamelares completas ($E, X$) e superfícies não-circulares poligonais arbitrárias.
2. **[Módulo Taludes] Reforço com Solo Grampeado e Geossintéticos (ABNT NBR 11682 / ISO):** Inclusão de elementos passivos tracionados (chumbadores/grampos e mantas geotêxteis) interceptando a base das fatias com resistência adicional de arrancamento.
3. **[Módulo Muros] Muros Escalonados com Contrafortes e Cortinas Atirantadas / Estacas-Prancha:** Suporte a cortinas em balanço ou com um ou dois níveis de tirantes protendidos (método da viga equivalente / Blum).
4. **[Interoperabilidade] Importação DXF / CSV Topográfico Direta no Canvas 2D:** Leitor leve de curvas e polilinhas CAD para importação instantânea de seções geotécnicas geradas em levantamento topográfico (NBR 13133).
5. **[Relatórios] Exportação Direta para PDF Vetorial Nativo:** Complementar à exportação atual em HTML5/PDF, Markdown e SVG, permitindo geração direta de caderno A4 PDF com diagramas vetoriais incorporados.

</details>

<details>
<summary><strong id="english-us">🇺🇸 English (US) — Community Technical Roadmap & Suggested Issues</strong></summary><br>

This document outlines community study and technical improvement ideas (under GNU AGPLv3, subject to voluntary review according to maintainer availability with no urgency):

1. **[Slope Module] Morgenstern-Price / Spencer Method (Rigorous Force & Moment Equilibrium):** Extending the `src/engine/slope/` solver to compute full interslice forces ($E, X$) across arbitrary non-circular polygonal slip surfaces.
2. **[Slope Module] Soil Nailing & Geosynthetic Reinforcement (ABNT NBR 11682 / ISO):** Incorporating passive tensile inclusions (soil nails/anchors and geotextile layers) intersecting slice bases with pullout resistance checks.
3. **[Wall Module] Counterfort Retaining Walls & Anchored Sheet-Pile / Secant Walls:** Support for cantilevered sheet-pile walls or single/multi-tier prestressed tieback walls (equivalent beam / Blum method).
4. **[Interoperability] Direct Topographic DXF / CSV Import on the 2D Canvas:** Lightweight CAD polyline parser enabling instant import of geotechnical cross-sections generated from field topographic surveys (NBR 13133).
5. **[Reports] Direct Native Vector PDF Export:** Complementing existing HTML5/PDF, Markdown, and SVG exports with direct generation of native A4 vector PDF calculation books containing embedded diagrams.

</details>
