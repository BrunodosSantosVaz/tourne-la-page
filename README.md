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

- [Por que Tourne la page](#-por-que-tourne-la-page)
- [Funcionalidades](#-funcionalidades)
- [Downloads](#-downloads)
- [Instalação](#-instalação)
- [Temas](#-temas)
- [Arquitetura](#-arquitetura)
- [Desenvolvimento](#-desenvolvimento)
- [Dados e privacidade](#-dados-e-privacidade)
- [Roadmap](#-roadmap)
- [Contribuindo](#-contribuindo)
- [Segurança](#-segurança)
- [Licença e créditos](#-licença-e-créditos)

</details>

## ✦ Por que Tourne la page

<table>
<tr>
<td width="33%" valign="top">

### 🦀 Nativo de verdade
Interface em Rust/Iced com renderização via `wgpu`, sem webview e sem Electron. Abre rápido e roda leve.

</td>
<td width="33%" valign="top">

### 🔒 Local por padrão
Biblioteca, progresso e anotações em SQLite no seu diretório de usuário. Nenhuma conta, nenhum servidor.

</td>
<td width="33%" valign="top">

### 📖 Seus EPUBs intactos
O aplicativo lê os arquivos e nunca os altera. Remover o app não apaga a sua biblioteca pessoal.

</td>
</tr>
</table>

## ✨ Funcionalidades

| | Biblioteca | | Leitura |
|:-:|---|:-:|---|
| 🖼️ | Capas, com capa substituta quando o livro não tem | 📜 | Leitura contínua com rolagem fluida e viewport virtualizado |
| 🔎 | Busca na biblioteca | 🧭 | Índice (sumário) do livro |
| ❤️ | Favoritos | ▶️ | Modo *play* com velocidade percentual |
| ⭐ | Avaliação por estrelas e detalhes do livro | 🔤 | Qualquer fonte instalada no computador, além de quatro sugestões |
| 💬 | Comentários pessoais por livro | 🎨 | Temas editáveis, com importação e exportação em JSON |
| 📊 | Progresso por livro e seção *Lendo* | 🖍️ | Destaques em cinco cores com comentários em post-its |

> [!NOTE]
> A série 0.x parte da versão aprovada em testes locais. Os limites conhecidos e os requisitos ainda pendentes estão documentados na [Wiki](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki).

## 📦 Downloads

Pacotes portáteis compilados nativamente para cada plataforma, publicados em [Releases](https://github.com/BrunodosSantosVaz/tourne-la-page/releases). O aplicativo distribuído funciona sem Rust, Python ou checkout do repositório.

| Sistema | Arquitetura | Arquivo | Requisito mínimo |
|---|---|---|---|
| <img src="https://img.shields.io/badge/-Windows-2B2722?style=flat-square&logo=windows&logoColor=FCFBF7" alt="Windows"> | x64 · arm64 | `tourne-la-page-<versão>-windows-<arch>.zip` | — |
| <img src="https://img.shields.io/badge/-macOS-2B2722?style=flat-square&logo=apple&logoColor=FCFBF7" alt="macOS"> | x64 · arm64 (Apple Silicon) | `tourne-la-page-<versão>-macos-<arch>.zip` | macOS 12 |
| <img src="https://img.shields.io/badge/-Linux-2B2722?style=flat-square&logo=linux&logoColor=FCFBF7" alt="Linux"> | x64 · arm64 | `tourne-la-page-<versão>-linux-<arch>.tar.gz` | glibc 2.35 (Ubuntu 22.04+) |

Cada pacote inclui um `manifest.json` com o SHA-256 de todos os arquivos, verificado na esteira após extrair o pacote.

## 🚀 Instalação

<details open>
<summary><b>Windows</b></summary>

1. Baixe e extraia o `.zip`.
2. Abra `tourne-la-page.exe`.

</details>

<details>
<summary><b>macOS</b></summary>

1. Baixe e extraia o `.zip`.
2. Arraste **Tourne la page.app** para **Aplicativos**.

</details>

<details>
<summary><b>Linux</b></summary>

```bash
tar -xzf tourne-la-page-<versão>-linux-x64.tar.gz
cd Tourne-la-page
./AppRun

# Opcional: registra o atalho e os ícones no menu do sistema
python3 install.py
```

</details>

> [!IMPORTANT]
> Os pacotes ainda não têm certificado comercial. O sistema pode pedir autorização para abrir o aplicativo na primeira execução. Detalhes em [Como instalar](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki/Uso).

## 🎨 Temas

Seis paletas prontas, um modo **Automático** que segue o sistema e temas personalizados editáveis em JSON.

| Tema | Fundo | Texto | Destaque |
|---|:-:|:-:|:-:|
| **Claro** | ![#FFFFFF](https://placehold.co/14x14/FFFFFF/FFFFFF.png) `#FFFFFF` | ![#222222](https://placehold.co/14x14/222222/222222.png) `#222222` | ![#1A4FB4](https://placehold.co/14x14/1A4FB4/1A4FB4.png) `#1A4FB4` |
| **Papel** | ![#FCFBF7](https://placehold.co/14x14/FCFBF7/FCFBF7.png) `#FCFBF7` | ![#2B2722](https://placehold.co/14x14/2B2722/2B2722.png) `#2B2722` | ![#1A4FB4](https://placehold.co/14x14/1A4FB4/1A4FB4.png) `#1A4FB4` |
| **Sépia** | ![#FAF7F0](https://placehold.co/14x14/FAF7F0/FAF7F0.png) `#FAF7F0` | ![#342D22](https://placehold.co/14x14/342D22/342D22.png) `#342D22` | ![#7A4B12](https://placehold.co/14x14/7A4B12/7A4B12.png) `#7A4B12` |
| **Escuro** | ![#292A2D](https://placehold.co/14x14/292A2D/292A2D.png) `#292A2D` | ![#D4D4D4](https://placehold.co/14x14/D4D4D4/D4D4D4.png) `#D4D4D4` | ![#8AB4F8](https://placehold.co/14x14/8AB4F8/8AB4F8.png) `#8AB4F8` |
| **Grafite** | ![#141516](https://placehold.co/14x14/141516/141516.png) `#141516` | ![#DDDDDD](https://placehold.co/14x14/DDDDDD/DDDDDD.png) `#DDDDDD` | ![#8AB4F8](https://placehold.co/14x14/8AB4F8/8AB4F8.png) `#8AB4F8` |
| **Noite** | ![#000000](https://placehold.co/14x14/000000/000000.png) `#000000` | ![#E6E6E6](https://placehold.co/14x14/E6E6E6/E6E6E6.png) `#E6E6E6` | ![#8AB4F8](https://placehold.co/14x14/8AB4F8/8AB4F8.png) `#8AB4F8` |

**Fontes sugeridas para o texto do livro:** Georgia, Literata, Atkinson Hyperlegible Next e Lexend, além de qualquer família instalada no sistema.

## 🏛️ Arquitetura

Workspace Cargo com quatro crates e dependências fixadas em versões exatas.

```mermaid
flowchart LR
    subgraph app["app · binário tourne-la-page"]
        UI["Iced 0.14<br/>wgpu · tiny-skia"]
        R["reader<br/>biblioteca · leitor · anotações"]
    end
    EPUB["epub-core<br/>zip · roxmltree · scraper"]
    CORE["app-core"]
    DB[("storage<br/>SQLite · rusqlite")]
    FILES[/"Arquivos .epub<br/>somente leitura"/]

    UI --> R
    R --> EPUB
    R --> CORE
    R --> DB
    EPUB --> FILES
```

| Crate | Responsabilidade |
|---|---|
| [`app`](app) | Interface nativa, leitor, biblioteca, temas, fontes e identidade visual |
| [`epub-core`](crates/epub-core) | Abertura segura e leitura do conteúdo EPUB |
| [`storage`](crates/storage) | Persistência local em SQLite (WAL), pastas da biblioteca e preferências |
| [`app-core`](crates/app-core) | Tipos e regras compartilhados |

<details>
<summary><b>Stack completa</b></summary>

| Camada | Tecnologia |
|---|---|
| Linguagem | Rust 1.99.1 · edition 2024 |
| UI | `iced` 0.14 (wgpu, tiny-skia, svg, X11, Wayland) |
| EPUB | `zip` 9 · `roxmltree` 0.21 · `scraper` 0.27 |
| Dados | `rusqlite` 0.40 (SQLite embutido) |
| Imagens | `image` 0.25 (PNG, JPEG, WebP, GIF) |
| Fontes do sistema | `fontdb` 0.24 |
| Diálogos nativos | `rfd` 0.17 |
| Ícones | Lucide |
| Fontes da interface | Inter · Literata |

</details>

## 🛠️ Desenvolvimento

**Pré-requisitos:** [Rust via rustup](https://rustup.rs/) (a versão é fixada por `rust-toolchain.toml`) e Python 3.

```bash
git clone https://github.com/BrunodosSantosVaz/tourne-la-page.git
cd tourne-la-page
python3 local.py run
```

| Comando | O que faz |
|---|---|
| `python3 local.py run` | Compila e abre o app com dados locais isolados em `.local/tournepage` |
| `python3 local.py build` | Compila no perfil rápido de desenvolvimento |
| `python3 local.py check` | Verifica o workspace sem empacotar |
| `python3 local.py test [pacote]` | Roda os testes do workspace ou de um pacote |
| `python3 local.py clean` | Remove apenas o cache de build do Cargo |

<details>
<summary><b>Dependências de sistema no Linux</b></summary>

```bash
sudo apt-get install build-essential pkg-config libfontconfig1-dev libxkbcommon-dev \
  libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev \
  libgl1-mesa-dev libvulkan-dev
```

</details>

<details>
<summary><b>Gerar um pacote de distribuição</b></summary>

```bash
python3 scripts/package.py linux-x64   # linux-arm64 · windows-x64 · windows-arm64 · macos-x64 · macos-arm64
```

Cada alvo é compilado no seu sistema nativo. No Linux, o build roda em um contêiner Ubuntu 22.04 fixado por digest para garantir compatibilidade com glibc 2.35.

</details>

## 🔐 Dados e privacidade

- Biblioteca, preferências, progresso e anotações ficam em um banco SQLite no diretório do usuário.
- Os arquivos EPUB são abertos somente para leitura.
- Local, homologação e produção mantêm dados separados.

| Variável | Uso |
|---|---|
| `TOURNEPAGE_ENV` | Ambiente de execução (`local`, `homologacao`, produção) |
| `TOURNEPAGE_DATA_DIR` | Diretório alternativo para os dados |

Backup, variáveis e operação: [Wiki → Operação](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki/Operacao).

## 🗺️ Roadmap

Acompanhe no [quadro público](https://github.com/users/BrunodosSantosVaz/projects/34).

- [x] Importação segura de EPUB e abertura progressiva
- [x] Rolagem fluida com viewport virtualizado
- [x] Fontes locais, temas editáveis e paletas
- [x] Destaques e comentários em post-its
- [x] Estrelas, favoritos, detalhes e progresso por livro
- [x] Release multiplataforma
- [ ] Renderização semântica completa de EPUB
- [ ] Posição exata com EPUB CFI e identidade estável
- [ ] Busca integral, coleções e biblioteca avançada
- [ ] Acessibilidade e seleção por teclado
- [ ] Exportação de anotações e backup/restauração
- [ ] Instaladores e assinatura comercial
- [ ] Desempenho em bibliotecas grandes e fuzzing
- [ ] Internacionalização e acabamento multiplataforma

## 🤝 Contribuindo

Contribuições são bem-vindas. O projeto é construído com o framework [Big Bang](https://github.com/BrunodosSantosVaz/big-bang): uma pessoa decide, IAs executam, e toda mudança passa por uma esteira com testes, revisão e homologação.

1. Procure ou abra uma [issue](https://github.com/BrunodosSantosVaz/tourne-la-page/issues) pelo formulário.
2. Crie uma branch a partir de `develop` (bugs a partir de `main`).
3. Use [Conventional Commits](https://www.conventionalcommits.org/pt-br/) em inglês. Issues, PRs e documentação em português do Brasil.
4. Abra o PR com *O que muda*, *Testes* e *Issue* (`Refs #n`). A CI precisa ficar verde.

Leia o [guia de contribuição](CONTRIBUTING.md), o [`AGENTS.md`](AGENTS.md) e o [código de conduta](CODE_OF_CONDUCT.md).

## 🛡️ Segurança

Não abra issue pública com detalhes de uma vulnerabilidade. Use o [relato privado](https://github.com/BrunodosSantosVaz/tourne-la-page/security/advisories/new) do GitHub. Veja a [política de segurança](SECURITY.md).

## 📄 Licença e créditos

Distribuído sob a licença [MIT](LICENSE). Fontes (Inter, Literata, Atkinson Hyperlegible, Lexend · OFL), ícones (Lucide · ISC) e dependências Rust têm seus avisos próprios incluídos nos pacotes.

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
