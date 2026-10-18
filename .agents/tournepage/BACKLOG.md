# Backlog operacional do Tournepage

Preparado em 18/10/2026. Todos os itens estão **planejados**, sem implementação ou teste do produto concluído. T00–T29 são identificadores locais; registrar os números reais quando criar issues. Fonte: especificação da raiz, 74 RF, 29 RN, 39 CA originais e seis CA locais.

## Procedimento por tarefa

Refinar Dado/Quando/Então e vincular RF/RN/CA; escrever os testes antes do código; adquirir posse e worktree; implementar; executar os testes pertinentes ao final do bloco; revisar de forma independente; integrar por PR e atualizar evidências. Dividir recortes grandes em PRs menores por critério, mantendo as dependências. Tarefas nos mesmos arquivos são sequenciais.

## Fundação e Fase 0 — Prova dos riscos

| ID | Trabalho e teste de saída | Depende |
| --- | --- | --- |
| T00 | Fundação Big Bang: resolver Wiki, registrar escolhas, contrato aplicável, stack/design e esteira do sistema. Conferir geração/validação real; arquivos presentes não comprovam Fundação concluída | Bootstrap em EXECUCAO |
| T01 | Workspace mínimo edição 2024 e toolchain fixada para a prova; testes sem janela e execução nativa mínima. Build dev do host sem instalador, publicação ou dados de produção (CA-LOCAL-01/04) | F2 autorizada |
| T02 | Modelo/origem e CFI: round-trip UTF-16/trecho, emoji, combinantes, nós vizinhos, escapes, HTML reparado, ordenação e limites de intervalos; propriedades de preservação da origem | T01 |
| T03 | cosmic-text, linhas/páginas, ligaduras, bidi, hifenização inicial e cluster/glifo → origem. Seleção conserva texto após reflow; medir custos de layout | T02 |
| T04 | Widget Iced com foco, nomes, árvore acessível e texto lido em ordem. Ensaios reais de teclado/leitor de tela por plataforma; registrar limitações e bloquear avanço se inviabilizarem RNF-14 | T03 |

**Portão 0:** layout, seleção/CFI e acessibilidade demonstrados, com dados, versões e plataforma. Nenhuma tela definitiva antes dessa prova. A sequência F0–F5 do framework é diferente das fases 0–5 do produto; a implementação da prova segue o fluxo de tarefas após a stack autorizada.

## Fase 1 — Fundação executável e camada local

| ID | Trabalho e teste de saída | Depende |
| --- | --- | --- |
| T05 | Launcher local; env/dados isolados; testes afetados transitivos; rustfmt/clippy, limites COD e duplicação; teste de arquitetura detectando violação; CI real. CA-LOCAL-01 a 05, inclusive recusa de caminho inválido/banco de produção | T00–T04 |
| T06 | Portas + SQLite, migrações do zero e da versão anterior, FTS5, WAL, escrita serializada e backup online. Falha parcial, restauração/corrupção; persistência em até 1 s e sete backups íntegros | T05 |

**Portão 1:** ciclo local sem instalador, domínio testável sem UI, configuração gerada e CI do Tournepage verificadas. Implementar os scripts propostos antes de ativar seus comandos; não aceitar jobs ignorados do template como sucesso.

## Fase 2 — Importação e biblioteca

| ID | Trabalho e teste de saída | Depende |
| --- | --- | --- |
| T07 | ZIP/container/OPF/spine EPUB 2/3, limites, DRM e fontes ofuscadas. Traversal, entradas duplicadas/hostis, ZIP inválido e LCP sem escrita indevida (CA-10/18; RN-01/06/07) | T02, T06 |
| T08 | XML válido + fallback HTML5, todos os blocos/inlines/semânticas do anexo A e descarte da apresentação. Fixture/snapshot por elemento com origem preservada (CA-13 a 16) | T07 |
| T09 | Imagens JPEG/PNG/GIF/WebP, SVG saneado, alternativas de mídia, notas, nav/NCX, landmarks, page-list e layout fixo. Nenhum fetch externo ou script | T08 |
| T10 | Cache versionado por hash, invalidação, conversão sob demanda/cancelamento. Abrir com e sem cache produz conteúdo equivalente sem travar UI | T06, T08, T09 |
| T11 | Pastas, varredura recursiva, eventos agrupados, perda/retorno e renomeação. CA-01 a 04; livro novo em até 5 s, sem duplicar; originais intactos | T06, T07 |
| T12 | Metadados, autor/colaborador/file-as, séries, gêneros, edições e duplicados BLAKE3. CA-05 a 08 e RN-02 a 13; mesmo hash preserva identidade, outro hash não sobrescreve edição antiga | T07, T11 |
| T13 | Grade/lista, continuar lendo, detalhes, capas/substituta e protegidos/erros; vazio/carregando/falha. CA-11/12 com medição de mil registros | T10, T12, T19a |
| T14 | Grupos/ordenações/filtros, busca em metadados e estado automático/manual. Gêneros normalizados; limites 1%/98% e escolha manual preservada | T12, T13 |
| T15 | Correções locais e revelar arquivo no gerenciador (importantes). CA-09 e hash do EPUB inalterado; não interpolar caminhos em shell | T12, T14 |
| T16 | Coleções do usuário (importante). Remover prateleira não remove EPUB, progresso ou marcações | T14 |

**Portão 2:** CA-01 a 18 pertinentes e cobertura do anexo com evidência. O corpus cresce ao longo das fases; 500+ livros/fuzzing só são declarados depois de executados, com origem/licenças. Importantes podem ser adiados explicitamente; essenciais não.

## Fase 3 — Leitura e personalização

| ID | Trabalho e teste de saída | Depende |
| --- | --- | --- |
| T17 | Retomar, paginação/rolagem, teclado/mouse/trackpad, sumário, progresso, foco/tela cheia. CA-19/23/26, persistência no prazo e cancelamento de layout obsoleto | T03, T10, T13 |
| T18 | Notas em pop-up, links internos/histórico, confirmação de URL completa, imagens ocultas/zoom, tabelas e copiar texto/referência. CA-20 a 22; recusar esquemas perigosos sem shell | T09, T17 |
| T19a | Extrair tokens da identidade visual fornecida para as funcionalidades do sistema, conforme IDENTIDADE-VISUAL.md; Literata na marca, Inter na UI, Lucide local e sete modos. Para o livro: quatro sugestões compatíveis com o plugin (Georgia, Literata, Atkinson Hyperlegible Next, Lexend) e catálogo completo das fontes instaladas no SO; sem Source Serif 4/Merriweather empacotadas | T05; fazer antes de T13 |
| T19b | Painel de aparência, espaçamentos/alinhamento/hifenização/exceções; CA-24/25, defaults/limites e preview sem perda de posição | T17, T19a |
| T20 | Colunas −/+, 45 caracteres mínimos, limite dinâmico, resize/restauração da preferência e uma coluna em rolagem. CA-38 com geometria real | T17, T19b |
| T26 | Fluent pt-BR/en, tema da UI, pastas, escala/reduzir movimento. Atalhos personalizados e atualização opcional em recortes próprios pela prioridade; rede zero por padrão | T11, T19a; integração final após T17 |

**Portão 3:** CA-19 a 26/38/39, defaults do plugin comprovados, navegação sem perder posição, teclado completo e reflow/fluidez medidos. T19 divide-se em duas issues para evitar ciclo T13 → T19 → T17 → T13.

## Fase 4 — Marcações e busca

| ID | Trabalho e teste de saída | Depende |
| --- | --- | --- |
| T21 | Cinco cores/sublinhado, notas/marcadores, editar/sobrepor, excluir/desfazer e lixeira/restaurar/expirar. CA-27/28/31/32; relógio controlado, limite por capítulo e falhas transacionais | T06, T17, T19b |
| T22 | CFI + citação, migração de edição, ambiguidade, órfãs e edição anterior preservada. CA-29/30 com inserção/remoção de texto e citação repetida | T02, T12, T21 |
| T23 | Painel por livro, filtros/tipo/cor/busca/navegação; visão global (importante) e etiquetas (desejável) separados. Órfãs visíveis, nomes acessíveis das cores | T21, T22 |
| T24 | Markdown e JSON W3C com RN-27, round-trip CFI/citação e copiar referência. CA-33/34; validação local, idempotência por UUID e nenhum fetch pelo @context | T21, T22 |
| T25 | Busca, contexto/capítulo, acentos/caixa, navegar/realçar com origem preservada. CA-37, Unicode e resultados paginados; busca global é desejável | T08, T17 |

**Portão 4:** CA-27 a 34/37, marcações sobrevivem a reflow, reabertura, renomeação e nova edição. Exportação preserva dados de órfãs.

## Fase 5 — Qualidade e distribuição

| ID | Trabalho e teste de saída | Depende |
| --- | --- | --- |
| T27 | Corpus 500+, fuzzing com orçamento, visuais, tamanho/memória/velocidade em medição estável e acessibilidade manual nas três plataformas (CA-17/35/36; RNF-01 a 18) | Fases 1–4 |
| T28 | DMG/MSI/AppImage/Flatpak/deb/rpm conforme plano, seis sistemas, assinatura/notarização, SOs mínimos, SBOM/checksums/procedência e instalações lado a lado. Contratos Big Bang e CA-LOCAL-06; produção com autorização aplicável | T27; certificados/ambientes quando necessários |
| T29 | Essenciais ligados a implementação/teste/commit; importantes/desejáveis entregues ou adiados, guias reais de instalação/backup, ADRs/licenças/limitações e Wiki publicada/verificada | T27, T28 |

**Portão 5:** essenciais e testes passam, auditoria sem impeditivos, manuais realizados, artefato homologado e íntegro. Linux sozinho não comprova macOS/Windows. Promoção não recompila a candidata.

## Observações para o refinamento

- T01 cria só o necessário à prova; T05 completa o ciclo local/CI. Não criar oito crates vazias com abstrações futuras.
- Fundação/F3 reutiliza `Tournepage Identidade Visual.dc.html`, preservado como fonte, para preparar logo, ícone global e Social preview; T13/T17/T21 seguem as aplicações visuais, e T28 exporta os ícones por plataforma. O inventário e os cuidados de extração estão em `IDENTIDADE-VISUAL.md`; esta organização não produz os assets nem valida o protótipo.
- Persistência vem antes de biblioteca/marcações; evoluir o esquema com migrações incrementais testadas.
- Os 39 CA originais não cobrem individualmente os 74 RF. Acrescentar cenários antes de programar os RF sem CA específico, sobretudo configurações, filtros, cancelamento e falhas.
- A matriz abaixo registra o destino de cada requisito, não cobertura realizada. Substituir “a escrever” por caminhos/casos reais e commits na execução. Nenhuma linha é recibo de aprovação.

## Matriz dos 74 requisitos

| RF | Prioridade | Tarefa | Cenário/teste |
| --- | --- | --- | --- |
| RF-BIB-01 | Essencial | T11 | A escrever antes do código: Adicionar e remover pastas da biblioteca |
| RF-BIB-02 | Essencial | T11 | A escrever antes do código: Varrer as pastas recursivamente e encontrar todos os arquivos `.epub` |
| RF-BIB-03 | Essencial | T11 | A escrever antes do código: Monitorar as pastas: livros novos, alterados, removidos, movidos ou renomeados |
| RF-BIB-04 | Essencial | T12 | A escrever antes do código: Extrair metadados: título, subtítulo, autores e colaboradores com papel, série e volume, gêneros/assuntos, idioma, editora, data, identificador (ISBN/UUID), descrição e capa |
| RF-BIB-05 | Essencial | T13 | A escrever antes do código: Exibir a biblioteca em grade de capas e em lista |
| RF-BIB-06 | Essencial | T14 | A escrever antes do código: Agrupar por autor, série, gênero/assunto, idioma, editora, ano e estado de leitura |
| RF-BIB-07 | Essencial | T14 | A escrever antes do código: Ordenar por título, autor, data de adição, última leitura, progresso e ano |
| RF-BIB-08 | Essencial | T14 | A escrever antes do código: Filtrar combinando critérios e buscar por texto em título, autor, série e descrição |
| RF-BIB-09 | Essencial | T14 | A escrever antes do código: Estados de leitura: Não lido, Lendo, Lido, Abandonado, com mudança automática e manual |
| RF-BIB-10 | Essencial | T13 | A escrever antes do código: Seção "Continuar lendo" com os livros abertos recentemente |
| RF-BIB-11 | Essencial | T13 | A escrever antes do código: Página de detalhes do livro: capa, metadados, sinopse, progresso, número de marcações, local do arquivo |
| RF-BIB-12 | Importante | T15 | A escrever antes do código: Corrigir metadados localmente: título, autores, série, volume, gêneros, idioma |
| RF-BIB-13 | Importante | T16 | A escrever antes do código: Coleções (prateleiras) criadas pelo usuário |
| RF-BIB-14 | Essencial | T07/T13 | A escrever antes do código: Mostrar livros protegidos (DRM) e corrompidos com o motivo |
| RF-BIB-15 | Importante | T12 | A escrever antes do código: Detectar duplicados e edições diferentes do mesmo livro |
| RF-BIB-16 | Importante | T15 | A escrever antes do código: Mostrar o arquivo no gerenciador de arquivos do sistema |
| RF-BIB-17 | Essencial | T13 | A escrever antes do código: Gerar capa substituta (título e autor) quando o livro não tem capa |
| RF-IMP-01 | Essencial | T07/T09 | A escrever antes do código: Ler EPUB 2 e EPUB 3: `container.xml`, OPF, `nav.xhtml` e `toc.ncx` |
| RF-IMP-02 | Essencial | T08 | A escrever antes do código: Converter cada documento da spine para o modelo de conteúdo, cobrindo todos os elementos do anexo da seção 9 |
| RF-IMP-03 | Essencial | T08 | A escrever antes do código: Descartar CSS, fontes, estilos em linha, cores e scripts da editora |
| RF-IMP-04 | Essencial | T08 | A escrever antes do código: Preservar a semântica de `epub:type` e `role="doc-*"` |
| RF-IMP-05 | Essencial | T08 | A escrever antes do código: Ler XHTML malformado com um parser HTML5 tolerante |
| RF-IMP-06 | Essencial | T02/T08 | A escrever antes do código: Guardar o localizador de origem de cada trecho para gerar CFIs |
| RF-IMP-07 | Essencial | T09 | A escrever antes do código: Ler imagens JPEG, PNG, GIF, WebP e SVG |
| RF-IMP-08 | Essencial | T09 | A escrever antes do código: Identificar e indexar as notas do livro (rodapé, fim de capítulo, fim de livro) |
| RF-IMP-09 | Essencial | T09 | A escrever antes do código: Ler o sumário hierárquico, os landmarks e a lista de páginas impressas |
| RF-IMP-10 | Importante | T10 | A escrever antes do código: Guardar em cache o modelo convertido para abrir mais rápido na próxima vez |
| RF-LEI-01 | Essencial | T17 | A escrever antes do código: Abrir o livro na última posição lida |
| RF-LEI-02 | Essencial | T17/T20 | A escrever antes do código: Modo paginado em colunas (1 por padrão, escolhidas com + e −; RN-29) e modo rolagem contínua |
| RF-LEI-03 | Essencial | T17 | A escrever antes do código: Navegar por teclado, clique nas margens, roda do mouse, gestos de trackpad e barra de progresso |
| RF-LEI-04 | Essencial | T17 | A escrever antes do código: Sumário lateral com o capítulo atual destacado |
| RF-LEI-05 | Essencial | T18 | A escrever antes do código: Abrir notas do livro em pop-up, sem perder a posição |
| RF-LEI-06 | Essencial | T18 | A escrever antes do código: Seguir links internos com histórico de voltar e avançar |
| RF-LEI-07 | Essencial | T18 | A escrever antes do código: Abrir links externos no navegador padrão, após confirmação |
| RF-LEI-08 | Essencial | T17 | A escrever antes do código: Mostrar progresso: porcentagem, capítulo, posição, página impressa (se houver) e tempo restante estimado |
| RF-LEI-09 | Importante | T18 | A escrever antes do código: Abrir imagem em tela cheia com zoom |
| RF-LEI-10 | Essencial | T18 | A escrever antes do código: Rolar horizontalmente tabelas mais largas que a coluna |
| RF-LEI-11 | Essencial | T17 | A escrever antes do código: Modo foco em tela cheia, com a interface escondida |
| RF-LEI-12 | Importante | T17 | A escrever antes do código: Rolagem automática com velocidade ajustável |
| RF-LEI-13 | Importante | T17 | A escrever antes do código: Ir para porcentagem, posição ou página impressa |
| RF-LEI-14 | Essencial | T18 | A escrever antes do código: Selecionar e copiar texto, com opção de incluir a referência |
| RF-PER-01 | Essencial | T19a/T19b | A escrever antes do código: Temas: Automático (segue o sistema), Claro, Papel, Sépia, Escuro, Grafite e Noite, mais temas personalizados (seção 12.3) |
| RF-PER-02 | Essencial | T19a/T19b | A escrever antes do código: Fonte de leitura: quatro sugestões compatíveis com o plugin (Georgia, Literata, Atkinson Hyperlegible Next, Lexend) e qualquer família instalada listada pelo sistema operacional |
| RF-PER-03 | Essencial | T19b | A escrever antes do código: Tamanho da fonte, altura da linha, espaço entre letras, palavras e parágrafos |
| RF-PER-04 | Essencial | T19b | A escrever antes do código: Alinhamento à esquerda, centralizado, à direita ou justificado, com hifenização ligada ou desligada |
| RF-PER-05 | Essencial | T19b | A escrever antes do código: Largura da coluna de texto e margens |
| RF-PER-06 | Importante | T19b | A escrever antes do código: Recuo da primeira linha do parágrafo, ou espaço entre parágrafos |
| RF-PER-07 | Essencial | T18/T19b | A escrever antes do código: Mostrar ou esconder imagens |
| RF-PER-08 | Importante | T19b | A escrever antes do código: Preferências globais, com exceções salvas por livro |
| RF-PER-09 | Desejável | T19b | A escrever antes do código: Importar as preferências exportadas do plugin |
| RF-PER-10 | Essencial | T20 | A escrever antes do código: Número de colunas no modo paginado, com botões + e − (RN-29) |
| RF-MAR-01 | Essencial | T21 | A escrever antes do código: Destacar a seleção em cinco cores: amarelo, verde, azul, rosa e roxo |
| RF-MAR-02 | Importante | T21 | A escrever antes do código: Sublinhar como estilo alternativo ao destaque |
| RF-MAR-03 | Essencial | T21 | A escrever antes do código: Escrever nota ligada a um destaque |
| RF-MAR-04 | Essencial | T21 | A escrever antes do código: Criar marcador na posição atual, com nome e nota opcionais |
| RF-MAR-05 | Essencial | T21 | A escrever antes do código: Editar cor e nota; excluir com desfazer |
| RF-MAR-06 | Essencial | T23 | A escrever antes do código: Painel de marcações do livro: em ordem de leitura, com filtro por tipo e cor, busca e navegação ao clicar |
| RF-MAR-07 | Importante | T23 | A escrever antes do código: Visão geral com as marcações de todos os livros |
| RF-MAR-08 | Desejável | T23 | A escrever antes do código: Etiquetas nas marcações |
| RF-MAR-09 | Essencial | T24 | A escrever antes do código: Exportar marcações em Markdown e em JSON (W3C Web Annotation) |
| RF-MAR-10 | Importante | T24 | A escrever antes do código: Importar marcações em JSON (W3C Web Annotation) |
| RF-MAR-11 | Importante | T24 | A escrever antes do código: Copiar citação com referência: autor, título e capítulo |
| RF-MAR-12 | Essencial | T22 | A escrever antes do código: Manter as marcações após mudança de preferências, reabertura e troca de local ou nome do arquivo |
| RF-MAR-13 | Importante | T21 | A escrever antes do código: Lixeira de marcações excluídas, com restauração |
| RF-BUS-01 | Essencial | T25 | A escrever antes do código: Buscar no texto do livro aberto, com lista de resultados e trecho de contexto |
| RF-BUS-02 | Essencial | T25 | A escrever antes do código: Ignorar acentos e maiúsculas na busca, com opção de diferenciar |
| RF-BUS-03 | Essencial | T25 | A escrever antes do código: Navegar entre resultados e destacá-los no texto |
| RF-BUS-04 | Desejável | T25 | A escrever antes do código: Buscar no texto de todos os livros da biblioteca |
| RF-CFG-01 | Essencial | T11/T26 | A escrever antes do código: Gerenciar as pastas monitoradas |
| RF-CFG-02 | Essencial | T26 | A escrever antes do código: Idioma da interface: português (Brasil) e inglês |
| RF-CFG-03 | Essencial | T26 | A escrever antes do código: Tema da interface: seguir o sistema, claro ou escuro |
| RF-CFG-04 | Importante | T26 | A escrever antes do código: Ver e personalizar atalhos de teclado |
| RF-CFG-05 | Essencial | T06 | A escrever antes do código: Fazer e restaurar cópia de segurança do banco (marcações, progresso, preferências) |
| RF-CFG-06 | Desejável | T26 | A escrever antes do código: Verificar atualizações, desligado por padrão |
