<p align="center"><img src="docs/design/icone.svg" alt="Ícone do Tourne la page" width="96"></p>

# Tourne la page

Leitor EPUB para Windows, macOS e Linux, com interface nativa em Rust/Iced e dados locais em SQLite.

Biblioteca com capas, busca, favoritos, estrelas e progresso; leitura contínua com índice, fontes do computador, temas editáveis e destaques com comentários. A versão estável **v0.1.0** já está disponível para Windows, macOS e Linux. Limites conhecidos e requisitos ainda pendentes estão documentados na Wiki.

[Downloads e candidatas](https://github.com/BrunodosSantosVaz/tourne-la-page/releases) ·
[Como instalar](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki/Uso) ·
[Wiki](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki) ·
[Discussions](https://github.com/BrunodosSantosVaz/tourne-la-page/discussions) ·
[Quadro público](https://github.com/users/BrunodosSantosVaz/projects/34) ·
[Issues](https://github.com/BrunodosSantosVaz/tourne-la-page/issues)

Para desenvolver localmente: `python3 local.py run`. O aplicativo distribuído funciona sem Rust, Python ou checkout. Não modifica os EPUBs. Local, homologação e produção mantêm dados separados; instruções de backup, variáveis opcionais (`TOURNEPAGE_ENV`, `TOURNEPAGE_DATA_DIR`) e operação ficam na [Wiki](https://github.com/BrunodosSantosVaz/tourne-la-page/wiki/Operacao).

Desenvolvido por Bruno dos Santos Vaz. Licença [MIT](LICENSE); fontes, ícones e dependências têm seus avisos próprios incluídos nos pacotes.
