# Política de Segurança / Security Policy — GeoEstabil

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details open>
<summary><b>Clique para expandir / recolher a política de segurança em português</b></summary><br>

# GeoEstabil — Política de Segurança e Relato de Vulnerabilidades

**Versão:** `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  
**Responsável:** Fernando Nillsson Cidade  
**Canal Oficial de Contato (Exclusivamente via E-mail):** `linceu_lighthouse@outlook.com`  

---

### 1. Versões Recebendo Atenção Comunitária

Eventuais análises e atualizações voluntárias concentram-se exclusivamente na linha atual do repositório:

| Versão / Version | Status de Análise / Focus |
| :--- | :---: |
| `v2026.11.x` | :white_check_mark: |
| `< v2026.11` | :x: |

---

### 2. Filosofia de Segurança e Privacidade Local

O **GeoEstabil** é desenvolvido sob o princípio de execução local e offline (**100% Local-First**):
- O aplicativo não abre sockets de rede, não contém telemetria e não envia dados ou projetos para servidores externos;
- Não há integração com serviços remotos de Inteligência Artificial ou nuvem, mantendo arquivos `.geoestabil` e banco SQLite exclusivamente no dispositivo do usuário;
- O programa executa em modo usuário padrão no Windows, sem exigir privilégios administrativos ou permissões de diagnóstico remoto.

---

### 3. Como Reportar uma Vulnerabilidade

Caso identifique uma falha de segurança, comportamento inesperado em dependências Rust ou erro numérico relevante:
1. **Envie uma mensagem exclusivamente para o e-mail:** `linceu_lighthouse@outlook.com`
2. **Inclua no assunto:** `[Segurança - GeoEstabil] Relato de Vulnerabilidade`
3. **No corpo do e-mail, informe:**
   - Descrição objetiva do problema e passos para reprodução;
   - Versão do executável (`geoestabil.exe` ou `geoestabil-cli.exe`);
   - Versão do sistema operacional utilizado.
4. **Regime de Atendimento:** Todo contato é tratado **exclusivamente por e-mail**, em regime voluntário e de melhor esforço **conforme a disponibilidade do autor, sem obrigação de urgência, prazo mínimo ou SLA**, e com o software fornecido "no estado em que se encontra" ("AS IS").

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary><b>Click to expand / collapse the security policy in English</b></summary><br>

# GeoEstabil — Security Policy & Vulnerability Reporting

**Version:** `2026.11.11.0`  
**Date:** `November 11, 2026`  
**Maintainer:** Fernando Nillsson Cidade  
**Official Contact Channel (Email Only):** `linceu_lighthouse@outlook.com`  

---

### 1. Versions Receiving Voluntary Attention

Voluntary reviews and updates focus solely on the current release line of the repository:

| Release / Version | Focus Status / Maintained |
| :--- | :---: |
| `v2026.11.x` | :white_check_mark: |
| `< v2026.11` | :x: |

---

### 2. Security Philosophy & Local-First Privacy

**GeoEstabil** is built around local, offline execution (**100% Local-First**):
- The application opens no network sockets, embeds no telemetry, and sends no project files or analytics to external servers;
- No remote cloud AI or LLM APIs are used, keeping `.geoestabil` files and SQLite data strictly on the user's computer;
- The software runs in standard Windows user mode without requiring Administrator privileges or remote diagnostic permissions.

---

### 3. How to Report a Vulnerability

If you discover a security flaw, dependency issue, or notable numerical defect:
1. **Send an email exclusively to:** `linceu_lighthouse@outlook.com`
2. **Include in the subject line:** `[Security - GeoEstabil] Vulnerability Report`
3. **In the message body, provide:**
   - Clear description of the issue and reproduction steps;
   - Exact executable version (`geoestabil.exe` or `geoestabil-cli.exe`);
   - Operating system version used.
4. **Support & Response Policy:** All inquiries are handled **exclusively via email** on a voluntary, best-effort basis **according to the author's availability and with no obligation of urgency, response timeline, or SLA**, under an "AS IS" software supply model.

</details>
