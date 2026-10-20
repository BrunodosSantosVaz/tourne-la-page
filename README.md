<!-- markdownlint-disable MD033 MD041 -->
<div align="center">

<a href="https://github.com/BrunodosSantosVaz/tourne-la-page">
  <img src="assets/brand/icons/app-256.png" alt="Tourne la page" width="112" height="112">
</a>

# Tourne la page

**Menos ruído, mais leitura.**

Leitor EPUB nativo para **Windows, macOS e Linux**, escrito em Rust com Iced.<br>
Sua biblioteca fica no seu computador, e os seus livros nunca são modificados.

<br>

[![Release](https://img.shields.io/github/v/release/BrunodosSantosVaz/tourne-la-page?include_prereleases&style=flat-square&label=release&labelColor=2B2722&color=1A4FB4)](https://github.com/BrunodosSantosVaz/tourne-la-page/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/BrunodosSantosVaz/tourne-la-page/bb-ci.yml?branch=main&style=flat-square&label=CI&labelColor=2B2722)](https://github.com/BrunodosSantosVaz/tourne-la-page/actions/workflows/bb-ci.yml)
[![CodeQL](https://img.shields.io/github/actions/workflow/status/BrunodosSantosVaz/tourne-la-page/bb-codeql.yml?style=flat-square&label=CodeQL&labelColor=2B2722)](https://github.com/BrunodosSantosVaz/tourne-la-page/actions/workflows/bb-codeql.yml)
[![License: MIT](https://img.shields.io/badge/licen%C3%A7a-MIT-1A4FB4?style=flat-square&labelColor=2B2722)](LICENSE)
<br>
[![Rust](https://img.shields.io/badge/Rust-1.99-6D665D?style=flat-square&logo=rust&logoColor=FCFBF7&labelColor=2B2722)](rust-toolchain.toml)
[![Iced](https://img.shields.io/badge/Iced-0.14-6D665D?style=flat-square&labelColor=2B2722)](https://iced.rs)
[![SQLite](https://img.shields.io/badge/SQLite-local-6D665D?style=flat-square&logo=sqlite&logoColor=FCFBF7&labelColor=2B2722)](crates/storage)
[![Plataformas](https://img.shields.io/badge/Windows%20%C2%B7%20macOS%20%C2%B7%20Linux-x64%20%7C%20arm64-6D665D?style=flat-square&labelColor=2B2722)](#-downloads)

<br>

<a href="https://github.com/BrunodosSantosVaz/tourne-la-page/releases"><img src="https://img.shields.io/badge/Baixar-Releases-1A4FB4?style=for-the-badge&logo=github&logoColor=FCFBF7" alt="Baixar"></a>
<a href="https://github.com/BrunodosSantosVaz/tourne-la-page/wiki/Uso"><img src="https://img.shields.io/badge/Como%20instalar-Guia-2B2722?style=for-the-badge&logo=bookstack&logoColor=FCFBF7" alt="Como instalar"></a>
<a href="https://github.com/BrunodosSantosVaz/tourne-la-page/wiki"><img src="https://img.shields.io/badge/Wiki-Documenta%C3%A7%C3%A3o-2B2722?style=for-the-badge&logo=readthedocs&logoColor=FCFBF7" alt="Wiki"></a>
<a href="https://github.com/BrunodosSantosVaz/tourne-la-page/discussions"><img src="https://img.shields.io/badge/Discussions-Conversar-2B2722?style=for-the-badge&logo=github&logoColor=FCFBF7" alt="Discussions"></a>

<br><br>

<img src="assets/brand/social-preview.png" alt="Tourne la page — Um livro. Seu tempo. Sua página." width="100%">

</div>

<!-- Capturas de tela: adicione imagens reais da Biblioteca e do Leitor em assets/screenshots/ e descomente.
<p align="center">
  <img src="assets/screenshots/biblioteca.png" alt="Biblioteca" width="49%">
  <img src="assets/screenshots/leitor.png" alt="Leitor" width="49%">
</p>
-->

<br>

<details>
<summary><b>Sumário</b></summary>

- [Recursos](#-recursos)
- [Downloads e instalação](#-downloads-e-instalação)
- [Documentação](#-documentação)
- [Comunidade e segurança](#-comunidade-e-segurança)
- [Licença](#-licença)

</details>

## ✨ Recursos

<table>
<tr>
<td width="33%" valign="top">

### 📚 Biblioteca
Capas, busca, favoritos, avaliações e progresso de leitura.

</td>
<td width="33%" valign="top">

### 📖 Leitura
Rolagem contínua, índice, fontes do sistema, temas e marcações comentáveis.

</td>
<td width="33%" valign="top">

### 🔒 Privacidade
Aplicativo desktop em Rust/Iced. Livros e dados pessoais ficam no dispositivo.

</td>
</tr>
</table>

## 📦 Downloads e instalação

A versão estável [**v0.1.0**](https://github.com/BrunodosSantosVaz/tourne-la-page/releases/tag/v0.1.0) traz pacotes portáteis. Escolha seu sistema e arquitetura em [Releases](https://github.com/BrunodosSantosVaz/tourne-la-page/releases).

| Sistema | Arquitetura | Arquivo | Requisito mínimo |
|---|---|---|---|
| Windows | x64 · arm64 | `.zip` | — |
| macOS | x64 (Intel) · arm64 (Apple Silicon) | `.zip` | macOS 12 |
| Linux | x64 · arm64 | `.tar.gz` | glibc 2.35 (Ubuntu 22.04+) |

Confira os arquivos e seus checksums SHA-256 na página da release.

## 🧭 Documentação

| Guia | Conteúdo |
|---|---|
| [📖 Wiki](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki) | Recursos, temas, instalação, arquitetura e limites conhecidos |
| [🛠️ Desenvolver](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki/Uso) | Execução local, ferramentas e contribuição |
| [🗺️ Quadro público](https://github.com/users/BrunodosSantosVaz/projects/34) | Próximos passos do projeto |

## 🤝 Comunidade e segurança

Sugestões e dúvidas: [Discussions](https://github.com/BrunodosSantosVaz/tourne-la-page/discussions). Relate vulnerabilidades em particular pela [política de segurança](SECURITY.md).

## 📄 Licença

Licença [MIT](LICENSE). Avisos de fontes, ícones e dependências acompanham os pacotes.

<div align="center">

<br>

<img src="assets/brand/mark.svg" alt="" width="40">

<sub>Desenvolvido por <a href="https://github.com/BrunodosSantosVaz">Bruno dos Santos Vaz</a></sub>

<br><br>

<a href="https://star-history.com/#BrunodosSantosVaz/tourne-la-page&Date">
  <img src="https://api.star-history.com/svg?repos=BrunodosSantosVaz/tourne-la-page&type=Date" alt="Star History" width="560">
</a>

<br>

<sub>Se o Tourne la page te ajuda a ler mais, deixe uma ⭐ no repositório.</sub>

</div>
