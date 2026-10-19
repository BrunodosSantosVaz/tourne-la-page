# Estado atual da distribuição — 19/10/2026

A autorização do dono agora cobre integrar e publicar o aplicativo aprovado, com pacotes Windows/macOS/Linux. Este parágrafo substitui as restrições de preparação das entradas históricas abaixo; não amplia autonomy.toml. Modo Flash ativo, sem inventar aprovação de ambiente. A aprovação final de production continua humana.

Fundação F0/F5 revisadas e integradas por PRs. Wiki canônica publicada e conferida; consulte .bigbang-docs.json e Project34. Issues4/10 agregam a primeira distribuição,5/6 são bloqueios até correção comprovada. Mapeamento retrospectivo em DELIVERY.json; não é evidência de TDD retroativo. Não declarar pacotes ou release prontos antes dos runs e hashes reais.

Workspace ativo de entrega: /home/brunovaz/Projetos/Tourne-la-page-release, branchfeature/4-distribuicao-inicial. Checkouts locais e dados antigos preservados. Fontes originais de documentação desta pasta de entrega foram preservadas nos metadados Git após publicação na Wiki; identidade visual também em assets/brand/reference.

## Registro histórico preservado

# Roteiro operacional para o modelo executor

Preparado em 18/10/2026. Repositório: `BrunodosSantosVaz/tourne-la-page`.

## Ajustes locais de barra, Sobre e marcações — 19/10/2026

- Pedido atual do dono: Sobre compacto, X fora da rolagem; ordem Índice → Play → Velocidade; tema fixo antes da engrenagem. Fonte/alinhamento/largura/entrelinha/marca-texto sempre compactos, imagem alternável por ícone, sliders verticais e alinhamento em lista vertical. Impressão retirada do aplicativo; a referência histórica da especificação não representa mais o escopo desta versão.
- Bandeja aparece só quando há controles recolhidos, com velocidade cedendo espaço primeiro. Expansão usa novas linhas e o mínimo passou a 284 unidades lógicas (sete botões e margens). Tooltips de gatilhos continuam visíveis quando o popup está fechado.
- Marcações são post-its na cor do destaque, texto com contraste calculado e X no canto. Um editor compartilhado acima da lista mostra trecho, comentário e cor ao clicar no cartão; o clique reposiciona a lista para o editor. A rolagem reserva espaço e o cabeçalho lateral permanece acessível. Seleção e gravação automática do leitor foram preservadas.
- Regressões cobrem geometria em larguras intermediárias, ordem dos fixos, bandeja vazia, imagens por clique, sliders verticais, lista de alinhamento, modal sem rolagem em altura normal, X acessível na altura mínima, exclusão de post-it sem ativação acidental, mudança de cor e fechamento da paleta. Diagnóstico CPU com 100 mil parágrafos e capturas sintéticas conferido; não é medição de FPS do compositor.

- Verificação concluída: 57 testes Rust do workspace, dois testes Python de integração desktop, Clippy sem avisos e rustfmt. As duas revisões gráficas manuais também passaram; capturas conferidas incluem Sobre 1280/390, leitor 284/390/626/1280 e lista com 40 post-its. Diagnóstico CPU final: p95 0,164 ms no preparo do viewport, 0,318 ms na reconstrução da UI e 22,751 ms na rasterização/leitura de pixels; são medidas sintéticas distintas, sem promessa de FPS. A nova versão foi recompilada e aberta em Wayland, com biblioteca e dados locais existentes.

## Refinamento da barra — 19/10/2026

Pedido posterior do dono substitui a ordem acima: Índice/Início fixos à esquerda; Play, velocidade, controles de leitura, tema/configurações/comentários/bandeja formam um grupo alinhado à direita. Tamanho sem label, percentuais de fonte e velocidade centralizados entre −/+. Imagem desligada usa o SVG oficial `image-off` do mesmo Lucide 1.59.0 já usado no projeto; ligada mantém `image` e realce. Nenhuma dependência nova. Regressões cobrem nova geometria, centralização e alternância visual do ícone.

## Comentários completos e rolagem compartilhada — 19/10/2026

- Comentários e trechos dos post-its agora têm largura limitada, quebra por palavra/glifo e altura natural, sem o corte de 220 caracteres. O comentário usa TextEditor nativo com conteúdo retido, altura crescente e gravação automática; Enter salva, Shift+Enter quebra a linha. Carregar, trocar ou excluir a marcação sincroniza o editor e o rascunho.
- `reader/adaptive_scroll.rs` mantém o Scrollable nativo para roda, touchpad, foco, clipping e operações de teclado, e desenha/controla a alça pela mesma fórmula do EPUB. A inversão do arraste também é compartilhada. A integração substitui todas as áreas Scrollable do aplicativo: biblioteca, temas, fontes, cores, comentários, índice e modais. O espaço da barra só é reservado quando há overflow; sem conteúdo excedente não há alça nem espaço vazio.
- Regressões verificam crescimento do editor/post-it com texto e URL longos, autosave multilinha, Enter/Shift+Enter, alça diminuindo ao rolar, posicionamento por ID, arraste até o final e ausência de sobreposição/barra em conteúdo curto. A roda sobre o editor crescente rola a lateral. Não há dependências novas. Suíte: 62 testes Rust e dois Python; Clippy sem avisos, rustfmt, compilação e ambas as revisões gráficas manuais passaram. Capturas conferidas com fontes reais do PC, temas, biblioteca com 11 EPUBs e post-its/editor com comentários longos.

## Pedido e ponto de retomada

### Histórico de leitura e progresso na biblioteca — 19/10/2026

- **Lendo** reúne todos os livros iniciados e ainda abaixo de 100%, com ordem de acesso decrescente e busca preservada. A contagem usa a mesma regra. Nessa seção, o seletor de ordenação dá lugar ao texto “Mais recentes”; o destaque de Início continua sendo o último aberto.
- O rodapé compartilhado de capas/lista mostra barra e percentual acima das estrelas. `reading_history.rs` guarda o maior percentual já alcançado e o capítulo atual em preferências por caminho (`reading-progress:`). Releitura não desfaz conclusão; registros ausentes mostram 0%.
- A rolagem só atualiza memória; uma assinatura temporizada grava as mudanças em segundo plano aproximadamente a cada segundo. Voltar à biblioteca, trocar de livro e fechar fazem uma gravação final usando a fila/versionamento existentes. O fechamento nativo aguarda o progresso e permanece aberto com erro se a gravação falhar. Nenhum schema, dependência ou EPUB foi alterado.
- Recuperação dos dados antigos: lê `chapter:` e `last-opened`, mantendo todos os registros conhecidos. Estima o percentual pelo tamanho dos capítulos anteriores ao salvo; o histórico anterior não tinha porcentagem exata nem data de acesso. Não inventa livros iniciados sem registro. O carregamento parcial não marca 100%; o viewport publica novamente sua posição ao concluir o carregamento, sem mudar a posição visual.
- Regressões novas cobrem vários livros em andamento, ordem recente, conclusão e releitura, busca, percentuais inválidos, streaming incompleto, persistência fora de ordem, flush/fechamento, falha de gravação e importação dos capítulos antigos. A geometria do rodapé verifica barra acima das estrelas e ausência de transbordamento; a revisão gráfica usa os 11 EPUBs de teste.
- Diagnóstico CPU após a mudança, documento sintético com 100 mil parágrafos: p95 de 0,127 ms no viewport, 0,239 ms na reconstrução da interface e 15,247 ms na rasterização/leitura de pixels. Não representa FPS do compositor.
- Verificação final: 74 testes Rust do workspace passaram; Clippy sem avisos, rustfmt, build local e `git diff --check`. Ambas as revisões manuais passaram. Capturas de grade 1280/390 e Lendo conferidas; a revisão gráfica também valida a barra dentro do cartão. Versão local recompilada para Linux/Wayland; macOS/Windows não testados nesta etapa.

### Notas, favoritos e detalhes dos livros — 19/10/2026

- Cinco estrelas clicáveis sob o autor, alinhadas à esquerda; repetir a nota atual limpa a avaliação. À direita ficam coração (favorito) e lápis (detalhes). Grade e lista usam o mesmo rodapé, com ações separadas de abrir o EPUB.
- Biblioteca ganhou **Nota** (grupos de cinco até uma estrela, depois sem nota), ordenação por nota e **Favoritos** (filtro que também respeita a busca). A contagem do cabeçalho corresponde ao resultado filtrado, reutilizando a seleção já calculada.
- O lápis abre modal com capa e todos os campos de metadados disponíveis no modelo, nota e comentário geral editável/excluível com autosave. O X fica fixo fora da rolagem e o editor cresce. Os metadados do arquivo são somente leitura; nenhum EPUB é modificado.
- Preferências SQLite por caminho (`book-review:`) guardam nota, favorito e comentário usando a fila assíncrona existente. A versão por chave impede uma gravação antiga de sobrescrever uma nova. Não houve migração nem dependência nova.
- Por pedido posterior do dono, a seção **Marcações** foi removida do menu da biblioteca. Destaques/comentários de trechos dentro do leitor permanecem intactos.
- Verificação: 68 testes Rust passaram; Clippy sem avisos, rustfmt e build local. A revisão visual CPU com os 11 EPUBs de teste passou e as capturas foram conferidas em grade, favoritos e modal largo/estreito. Cinco novas regressões do aplicativo cobrem persistência, exclusão sem perder nota/favorito, gravações fora de ordem, filtros/grupos, cliques independentes e geometria/fechamento do modal; uma nova regressão no storage cobre isolamento de preferências por prefixo. Esta revisão não mede FPS do compositor nem testa macOS/Windows.

O dono autorizou a implementação autônoma local. Uma primeira prova funcional já existe neste checkout: veja `GUIA-TESTE-LOCAL.md` e a seção “Evidência da prova local” em `BACKLOG.md`. Este roteiro contém notas da etapa de preparação e trechos históricos abaixo; quando divergirem do código/estado atual, prevalece a evidência datada e reproduzível da prova local. A especificação completa continua sendo o escopo-alvo.

Leia nesta ordem: `AGENTS.md` (incluindo a seção Projeto), este roteiro, a especificação na raiz com os adendos 16/17/18, `Tourne la page Identidade Visual.dc.html` e seu inventário `.agents/tournepage/IDENTIDADE-VISUAL.md`, `BACKLOG.md`, `.agents/skills/ponytail/SKILL.md` e os padrões aplicáveis de `.bigbang/padroes/`.

Esta entrega local é uma fatia de MVP, não o leitor completo. Ela não fornece instaladores nem versão publicada. O estado inicial do código remoto era `cbf9710eba0239c5e88f5a77fa9cbe4c0294758f`.

## Decisões preservadas

| Assunto | Decisão do plano fornecido |
| --- | --- |
| Produto | Leitor EPUB 2/3 reflowable sem DRM, biblioteca por pastas, leitura, personalização, marcações e busca |
| Plataformas | macOS/Windows/Linux, x64 e ARM64 conforme RNF-01 |
| Implementação | Rust estável edição 2024, Iced + cosmic-text/wgpu; sem webview/Chromium |
| Persistência | SQLite embarcado, WAL, migrações, FTS5, backups e cache descartável |
| Privacidade | Sem conta, nuvem ou telemetria; atualização opcional desligada por padrão |
| Aparência | Identidade visual v1.0 fornecida no HTML aplica-se às funcionalidades do sistema: marca em Literata, Inter na UI, Lucide e paletas da seção 12; a tipografia do livro é escolha do leitor |
| Ambientes | Local rápido e isolado; homologação; produção por promoção do artefato aprovado |
| Código | Padrões Big Bang/L’Essentiel, Ponytail, inglês no código e português na documentação |
| Licença | MIT; fontes OFL; Georgia apenas se instalada no sistema |

As bibliotecas da seção 10 são escolhas do plano, não versões já verificadas. Pesquise fontes oficiais no momento de implementar, fixe versões compatíveis com a MSRV escolhida e registre em STACK/ADR/Cargo.lock. Não troque a stack nem acrescente dependências de execução silenciosamente.

## Evidências e pendências de bootstrap

- Python 3.14.7, Git, GitHub CLI, `cc` e `pkg-config` estão disponíveis. Rust estável foi instalado pelo `rustup-init` oficial após checagem de SHA-256; versões verificadas: `rustc 1.99.1`, `cargo 1.99.1`, `rustfmt 1.10.0-stable`, Clippy `0.1.99`. `pkg-config` encontra xkbcommon, Wayland, X11, Xcursor, Xi e Xrandr. A prova está em `app/` e `crates/`; validação e estado atuais ficam em `BACKLOG.md`.
- O repositório remoto é público, com `main` e Wiki habilitada. Em 18/10/2026, o dono criou a Wiki; `Home.md` existe e `bb docs prepare --repository BrunodosSantosVaz/tourne-la-page` passou, fixando o HEAD `cfa369a43b5d4eaf9b37e14b77b0f32bb90d3c87`.
- Discussions está desabilitado. A criação do primeiro post deve seguir a skill da Fundação, quando a comunidade for habilitada no escopo autorizado; não existe post criado nesta preparação.
- O bootstrap F0 foi executado numa worktree própria a partir de `develop` e está no PR draft #3. `bb verify` e `git diff --check` passaram. `bigbang.toml` está configurado para `autonomous`; o contrato Tourne la page foi criado, mas `bb autonomy check` ainda aguarda rulesets com `gate` em `main`/`develop` e o ambiente `production` restrito à `main`.
- Na base anterior ao F0, `.bigbang-framework.toml`, `.bigbang-docs.json`, `.bigbang-producao.json`, `autonomy.toml` e os workflows `bb-framework-*` são heranças do Big Bang. O F0 remove os contratos/configuração herdados e instala apenas o manifesto documental público e o `autonomy.toml` do Tourne la page aprovado pelo dono. Não reutilize outros projetos como evidência.
- Os workflows herdados condicionam seus jobs ao repositório do Big Bang. Jobs ignorados não são CI verde do produto.
- Rulesets, ambientes, secrets e Project ainda precisam ser inventariados em F4. Não alegar que estão configurados. Não solicitar tokens na conversa.

O conteúdo funcional detalhado ainda precisa de proposta, revisão e publicação versionadas na Wiki pelo processo 18; a Home padrão não comprova cobertura documental. Preserve a especificação fornecida até a publicação confirmada. Não publique conteúdo sem passar pelo PR e pela revisão documental.

## Biblioteca e identidade — 19/10/2026

- Nome visível: **Tourne la page**; repositório efetivamente renomeado para `https://github.com/BrunodosSantosVaz/tourne-la-page`; origin atualizado. Pastas reais: `/home/brunovaz/Projetos/Tourne-la-page`, `Tourne-la-page-local` e `Tourne-la-page-f0`. `git worktree repair` e `move` preservaram branches e alterações. Não houve commit, push, merge ou publicação de release nesta mudança.
- A biblioteca agora usa Grid fluido medido pelo espaço disponível, sem limite de colunas, com capas proporcionais, cartões alinhados, foco e hover. O cabeçalho fica fora da área lateral e da rolagem; o índice da biblioteca começa abaixo dele. Em janela estreita, busca na segunda linha e sem wordmark; os ícones possuem tooltip. O botão T saiu somente da biblioteca. A rolagem nativa da biblioteca chega à borda direita, com margem dentro do conteúdo.
- `reader/library_screen.rs` e seus filhos separam a apresentação da biblioteca do leitor. Modal Saiba mais, ao lado da engrenagem: versão do Cargo, edição local, desenvolvedor, licença, GitHub e issues; links fixos abrem no navegador. Escape e botão fechar dispensam o modal.
- Símbolo SVG extraído da identidade fornecida; PNGs 16–1024, ICO e ICNS exportados. Janela, launcher e ícones Linux usam `io.github.BrunodosSantosVaz.TourneLaPage`. `scripts/install_desktop.py` registra o atalho do usuário ao abrir o aplicativo; nenhuma dependência de execução nova. A fonte visual original permanece arquivada com o hash original; cópia v1.1 usa o nome novo. ADR-0002 registra a decisão do dono.
- Dados SQLite em `.local/tournepage` e identificadores internos foram preservados; a pasta foi movida com a worktree. Treze arquivos de viewport, barra e controles da leitura conferidos por SHA-256: idênticos aos aprovados antes desta tarefa. Mudança global da leitura limita-se à marca/título/ícone da janela.
- A mudança de caminho deixou um teste já compilado com CARGO_MANIFEST_DIR antigo. Recompilação dirigida dos três pacotes locais corrigiu o cache, preservando dependências externas; a suíte do workspace passou em seguida. Não foi enfraquecido nenhum teste.
- Confirmação nativa após a renomeação: processo `tourne-la-page` conectado a `wayland-0`; KWin retornou caption `Tourne la page`, resourceClass e desktopFile `io.github.BrunodosSantosVaz.TourneLaPage`. Captura real da janela mostra o símbolo próprio na decoração, oito capas por linha sem a lateral e o modal Saiba mais. O registro desktop é válido; a biblioteca e preferências anteriores foram carregadas no novo diretório.
- Verificação: 50 testes Rust e um teste de instalação desktop passaram; lint Clippy, rustfmt, validação desktop e revisão visual CPU da biblioteca com os 11 EPUBs locais. Telas inspecionadas entre 246 e 1920 pixels, com modal e menu aberto/fechado. Protótipos e esteira/Fundação ainda têm as pendências históricas abaixo; os formatos ICO/ICNS são assets, não instaladores homologados.

## Ponto de execução em 18/10/2026

### Retomada após queda de energia: correção local de fluidez e interação

- Correção posterior do fechamento ao abrir EPUB: reproduzido `index out of bounds: len is 0` em `popover.rs::diff`, na troca da biblioteca pelo leitor. O componente compartilhava a tag stateless com componentes de outra estrutura e indexava filhos sem reconciliá-los. Recebeu tag própria e reconciliação dos dois filhos pela API do Iced. `reader/transitions.rs` reproduziu o mesmo panic antes da correção e passou depois: abre o EPUB de demonstração, apresenta metadados e cada capítulo, retorna à biblioteca e reabre, preservando a árvore real de widgets nas larguras 1280/626/246/1920. Os testes anteriores partiam de uma tela de leitura já montada e não cobriam essa transição.

- Refinamento posterior da barra aprovado pelo dono: completa → ícones clean → bandeja. Todos os flexíveis tornam-se clean antes de qualquer recolhimento; popups abrem o controle diretamente, e a bandeja restaura o formato completo. Impressão participa da linha principal quando cabe. Velocidade fica depois do Play e percorre 80 ↔ 90 ↔ 100 ↔ 200 ↔ 300%. O mínimo da janela é calculado pelos seis ícones fixos de 32, cinco intervalos de 6 e margens de 12 (246 unidades lógicas); listas e laterais respeitam essa largura. Código em `reader/toolbar.rs` e `reader/bar/arrange.rs`; regressões cobrem fases, ordem, alinhamento, limite mínimo, passos, clique nos ícones e Escape nos nove popups. A rolagem automática pausa enquanto um popup clean está aberto.
- Verificação do refinamento: 44 testes automáticos aprovados, mais o ensaio manual CPU; Clippy sem warnings, rustfmt e build local aprovados. Capturas revisadas entre 246 e 1920 pixels. O resize forçado via KWin desrespeitou inicialmente o hint nativo; foi adicionado um pedido corretivo de largura mínima no evento de resize, com teste do efeito real emitido. Repetido na janela Wayland: pedido de 160 resultou em 246, mantendo a altura. Janela restaurada a 1280 para o teste do dono. Campos percentuais têm largura estável para os botões e popups não mudarem de lugar ao cruzar 100%.

- Trabalho permanece na worktree `/home/brunovaz/Projetos/Tourne-la-page-local`, branch `feature/local-reader-mvp`. A queda não perdeu o código; o compilador descartou entradas incrementais incompletas. Não houve publicação, merge ou alteração dos controles GitHub nesta retomada.
- O viewport antigo com editores invisíveis por parágrafo foi substituído pelo widget nativo retido em `app/src/reader/surface/`: mede/desenha somente a região próxima à tela, mantém um cache limitado e indexa alturas com somas prefixadas. Eventos de rolagem permanecem no widget; notificações à aplicação são limitadas a 10 Hz. A rolagem automática depende do tempo transcorrido, não de reconstruir a aplicação a cada quadro.
- Seleção por arraste mantém offsets UTF-8 de origem e cria marcação somente ao soltar o mouse. Comentário é opcional, com foco automático; preferências e marcações são gravadas em segundo plano, serializadas e versionadas por chave. A chegada de capítulos não reposiciona continuamente a rolagem.
- `bar.rs` mede os controles reais, recolhe um sufixo da direita para a esquerda e expande em linhas. Índice/início e play/configurações/comentários/expandir permanecem fixos. `popover.rs` ancora listas ao próprio campo e limita a janela; `action.rs` fornece foco e ativação por teclado aos botões nativos. Cores são quadradinhos sem código. Redimensionar/recolher fecha listas que poderiam ficar invisíveis.
- Perfil local agora otimiza o aplicativo em nível 1 e as dependências gráficas em nível 2, preservando compilação incremental. Nenhuma dependência de execução foi adicionada nesta correção.
- Verificação de retomada: suíte do workspace, lint Clippy com `-D warnings`, rustfmt e teste manual `cpu_reader_review`. Este último inclui 100 mil parágrafos/720 eventos no viewport e 180 eventos com reconstrução da tela completa, além de rasterização por CPU e capturas de 390 a 1920 pixels. Tempos medidos são diagnósticos locais, sem garantir FPS no compositor ou em qualquer EPUB.
- A execução com `python3 local.py run` foi aberta sem remover `WAYLAND_DISPLAY`. Foi verificado o socket do processo ligado a `/run/user/1000/wayland-0`, e a janela foi capturada com Spectacle. Não persistir overrides X11; o teste temporário anterior em XWayland não é configuração do projeto.
- Roteiro atualizado em `GUIA-TESTE-LOCAL.md`. Código do aplicativo ainda é uma prova local; Fundação, arquitetura/documentação completa e integração continuam com as pendências abaixo. Não interpretar testes locais como homologação do dono ou release.

- PR #1 mantém a preparação da especificação, identidade visual, L’Essentiel/Ponytail, adendos 16–18 e camada de teste local; ainda está draft e aponta para `main`.
- `develop` foi criado do `main` original (`cbf9710eba0239c5e88f5a77fa9cbe4c0294758f`). Issue F0 é #2; PR draft F0 é #3, branch `fundacao/0-ligar-github`, commit atual começa por `c33d13e`.
- O dono autorizou seguir com F0 e o merge para entregar a versão local; PR #3 continua draft enquanto aguarda revisão independente em contexto limpo. O F0 não instala código do leitor nem valida o produto.
- O dono confirmou que as fontes da identidade visual são só para as funcionalidades do sistema. Para os livros, são quatro sugestões compatíveis com o plugin/documentação (Georgia, Literata, Atkinson Hyperlegible Next, Lexend), mais qualquer fonte instalada no computador. O protótipo e o ícone ainda precisam ser apresentados em F3 para validação.
- O dono autorizou criar o contrato de autonomia com risco baixo/médio, teto US$ 50, WIP 3, validade de três meses e escopo local. O contrato está no PR #3; o `bb autonomy check` ainda aguarda controles GitHub.
- O dono autorizou avançar com settings externos do GitHub, mas pediu o passo a passo para ele executar. Guiar um item por vez e conferir antes do próximo; não criar secrets em nome do dono.
- Próxima ação manual do dono: habilitar Discussions em `Settings → Features → Set up discussions`. Se o assistente oferecer publicar uma mensagem inicial, não publique ainda; a IA fará o post factual idempotente quando a função estiver ativa. Depois, confirmar que a aba Discussions aparece; só então orientar o próximo ajuste. A configuração de rulesets com `gate` deve esperar até o workflow F5 existir.

## Ordem de execução

1. Inspecionar branch, mudanças do usuário, PR desta preparação e estado remoto. Preservar alterações alheias. O executor pode continuar a partir desta branch ou da versão integrada pelo dono; nunca fazer reset destrutivo.
2. Wiki e toolchain local já conferidas; Rust estável e ferramentas foram instalados. Quando F2 fixar MSRV, registrar o toolchain no repositório e conferir bibliotecas nativas no runner Linux e nas demais plataformas. O host atual tem os pacotes de desenvolvimento gráficos detectáveis por `pkg-config`.
3. Concluir a Fundação com `.agents/skills/bb-iniciar-projeto/SKILL.md`, reutilizando as decisões já fornecidas. “F0–F5” do Big Bang antecede e difere das fases do aplicativo.
4. Preflight da Wiki e `bb init` foram executados. O modo autonomous continua dependente do contrato e dos controles do framework.
5. Em F2, adaptar a configuração proposta, publicar documentos produto/stack/design/arquitetura/ADRs na Wiki pelo procedimento do framework e preparar sua rastreabilidade. Não ativar comandos vazios ou scripts inexistentes como se a CI estivesse pronta.
6. Em F3, partir da identidade visual fornecida, inventariada em `IDENTIDADE-VISUAL.md`, e manter o vínculo dos tokens com L’Essentiel. Extrair os assets nativos e preparar ícone e protótipo; registrar o que realmente foi validado. O HTML de referência não é um protótipo funcional validado e depende de um `support.js` ausente. Revisão independente e portões seguem o contrato aplicável; não inventar aprovação do dono.
7. Em F4/F5, inventariar e instalar a esteira do **Tourne la page**, Project único, controles e contrato aplicável. Usar `bb generate --dry-run`, `bb generate` e `bb verify`; não editar `.bigbang/` ou arquivos gerados. Não publicar nem alterar permissões porque um exemplo manda.
8. Abrir/refinar os épicos da sequência do BACKLOG, escrever testes de aceite antes da implementação, adquirir posse e usar worktree. Começar pelos riscos da Fase 0, sem telas definitivas ou expansão de escopo.
9. Executar as fases em ordem com os portões definidos. Atualizar a matriz de requisitos com evidência real, commit e teste. Relatar bloqueios com o que foi tentado e continuar o trabalho independente possível.

Comando usado para iniciar F0:

```sh
python .bigbang/bin/bb.py init --name Tourne la page --slug tournepage --repository BrunodosSantosVaz/tourne-la-page --visibility publico --license MIT --mode autonomous --workflow grouping
```

Ele não cria o sistema pronto. A proposta `bigbang.preparado.toml` serve para F2 e **não foi instalada como configuração ativa**. Todos os scripts nela citados ainda são entregas da Fase 1; os comandos de empacotamento são entregas da Fase 5. Validar o esquema e os contratos antes de copiar os valores para a configuração gerada pela Fundação.

## Contrato do ciclo local

Implementar um comando simples por ação, com as mesmas regras de negócio de homologação/produção:

| Ação | Comportamento a implementar |
| --- | --- |
| `local check` | `cargo check` na crate afetada; host atual, sem empacotamento |
| `local test` | Testes afetados e consumidores; fixtures temporárias; sem compilar a UI se só domínio mudou |
| `local build` | Binário `tournepage` do pacote `app`, perfil dev/incremental, sem assinatura |
| `local run` | Abrir binário com `TOURNEPAGE_ENV=local` e dados em `.local/tournepage/` |
| `local clean` | Se oferecido, remover apenas cache/build local explicitamente selecionado; nunca dados do usuário ou EPUBs |

`local.py` implementa `check`, `test`, `build`, `run` e `clean` para o ciclo rápido desta prova. `test` aceita um pacote ou roda o workspace; a seleção transitiva por arquivos alterados ainda falta. Veja `GUIA-TESTE-LOCAL.md`.

Escolher testes pelo grafo transitivo de dependências de `cargo metadata`, não apenas pelo arquivo alterado. Dúvida na seleção, mudanças estruturais, manifestações desconhecidas ou alterações em manifest/lock/migrações/tokens/scripts disparam a suíte apropriada completa. Não omitir dependentes reversos e testes de integração/aceite. Testes com `#[ignore]` de preparação precisam falhar o portão da candidata enquanto pendentes; `cargo test` sozinho pode ignorá-los.

Escrever o teste antes; executar ao final do bloco. Testes específicos de segurança, dados ou fronteiras não desaparecem por estar no ambiente local. A revisão e CI são da integração, não de cada tentativa de abrir o aplicativo.

## Contratos das ferramentas propostas

As ferramentas abaixo constam da proposta de configuração; somente o launcher `local.py` existe nesta prova. Implemente e valide os demais antes de ativar seus comandos:

- `scripts/lint.py`: rustfmt, clippy sem warnings, limites COD-09 e duplicação; não esconder saídas nem mascarar falhas.
- `scripts/check_architecture.py`: grafo do workspace e imports concretos de infraestrutura; permitir I/O apenas nos adaptadores; testar uma violação real.
- `scripts/test_affected.py`: ler `BB_BASE_TESTES`/`BB_ARQUIVOS_ALTERADOS`, incluir consumidores transitivos e aplicar fallback conservador.
- `scripts/test_acceptance.py`: registrar e executar os testes reais de `tests/aceite/`; falhar se faltarem casos obrigatórios ou houver pendência exigível. Eles podem ser alvos de integração de um pacote existente; não criar uma crate só para repetir a contagem de um diagrama perdido.
- `scripts/coverage.py`: medir pelo menos 80% de domínio/aplicação, sem excluir código difícil para inflar o índice; ferramenta de cobertura é dependência de desenvolvimento a fixar.
- `scripts/package.py`: executar no runner correto por plataforma, gerar formatos da seção 10 e cumprir o contrato compilado do Big Bang (`BB_SAIDA`, `BB_VERSAO`, `BB_RC`, `BB_SISTEMA`). O framework exige exatamente um arquivo por sistema; quando houver vários instaladores Linux, agrupá-los em um arquivo de distribuição com manifesto e checksums individuais ou propor uma evolução formal da esteira. Não editar o framework localmente.

Validar na Fase 1 a atualização de versão pelo Big Bang sobre `Cargo.toml`, incluindo `[workspace.package].version` e `version.workspace`. Versionamento com regex que altera dependências por engano não é aceitável. Validar também a coleta de funções Rust do `test_pattern` e o reconhecimento da marca pendente em testes de aceite reais antes de ativar a esteira.

## Arquitetura e riscos prioritários

- O grafo da seção 9 foi reconstruído e corrigido no adendo 17.1: domínio/aplicação não dependem de `storage`; `app` compõe os adaptadores. Oito crates de execução; criar cada uma quando houver responsabilidade concreta.
- Fonte → modelo → cluster/glifo → seleção → UTF-16/CFI precisa funcionar com emoji, diacríticos, ligaduras, RTL e HTML reparado. Normalização de busca e layout não podem destruir a origem.
- Demonstrar texto acessível e seleção com Iced + AccessKit antes das telas definitivas. Se não funcionar, registrar a evidência e a decisão necessária; não adotar webview ou trocar Iced silenciosamente.
- Testar limites de ZIP/XML/SVG, tentativas de saída de diretório, referências externas e preservação dos EPUBs. Limites devem funcionar por recurso, por livro e por operação, com cancelamento; uma decompression ratio não substitui limite absoluto.
- Persistência em até 1 s resolve o conflito entre RN-20 e RNF-11. Testar transação, queda, backup online SQLite/WAL e restauração antes de tratar dados reais como seguros.
- Validar licenças dos dados de hifenização e o carregamento de fontes nativas. Literata, Atkinson Hyperlegible Next e Lexend do plugin são WOFF2; usar TTF/OTF OFL quando o carregador exigir. A prova usa `fontdb::Database::load_system_fonts` (0.24.0), verificada no build local; confirmar desempenho e suporte multiplataforma nas fases posteriores.

## Referência do L’Essentiel

Commit conferido: `03d3468f00da71318da49373805c1fe66a618721`.

Os dez arquivos `.bigbang/padroes/*.md` são idênticos à origem. A Ponytail foi importada sem alteração com LICENSE. Um adaptador em `.claude/skills/ponytail/SKILL.md` aponta para a fonte canônica. Na próxima sessão, a skill estará disponível pela descoberta do projeto; seu conteúdo já está linkado por AGENTS.

Valores do plugin L’Essentiel foram conferidos em `src/platform/reader-panel.ts` e na Wiki `RN-0002-Controles`/`Leitura-local-e-acabamento`: três famílias incluídas (Literata, Atkinson Hyperlegible Next, Lexend) e Georgia do sistema, que em conjunto formam as quatro sugestões; o Tourne la page também lista qualquer família instalada no SO. A API `cosmic_text::FontSystem` pode fornecer o catálogo do SO ([documentação](https://docs.rs/cosmic-text/latest/cosmic_text/struct.FontSystem.html)); confirme versão na F2 e inicialize uma vez. A identidade visual do Tourne la page fornece marca em Literata, Inter para a interface, Papel/Grafite, Lucide e paletas próprias. A prévia Papel/Literata não define a tipografia do livro; referências remotas de fontes/ícones no showcase não devem ir para o runtime nativo.

## Limites desta entrega e da autonomia

Não alterar `autonomy.toml` para ampliar riscos, custos, flags, validade ou publicação. O dono autorizou o contrato atual, local-only, com valores registrados no PR #3. Conferir `bb autonomy check` após instalar os controles solicitados ao dono; publicação em produção permanece fora da delegação atual.

Esta preparação não mescla PR, publica versão, cria segredos, muda permissões ou afirma homologação. A promoção real deve respeitar o “aval” do dono/contrato aplicável e usar o mesmo artefato homologado. Atualizar o handoff e os resultados ao fim de cada entrega para o próximo modelo retomar sem presumir sucesso.
