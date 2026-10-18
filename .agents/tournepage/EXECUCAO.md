# Roteiro operacional para o modelo executor

Preparado em 18/10/2026. Repositório: `BrunodosSantosVaz/tournepage`.

## Pedido e ponto de retomada

O dono pediu aplicar a especificação em modo autônomo (“go horse”), acrescentar uma camada local rápida além de homologação e produção, seguir as regras do L’Essentiel e preparar tudo para trocar o modelo executor. Esta etapa é a preparação; a próxima sessão implementa o produto.

Leia nesta ordem: `AGENTS.md` (incluindo a seção Projeto), este roteiro, a especificação na raiz com os adendos 16/17/18, `Tournepage Identidade Visual.dc.html` e seu inventário `.agents/tournepage/IDENTIDADE-VISUAL.md`, `BACKLOG.md`, `.agents/skills/ponytail/SKILL.md` e os padrões aplicáveis de `.bigbang/padroes/`.

Não confunda esta entrega com um leitor EPUB pronto. Não há workspace Rust, testes do produto, instaladores ou versão publicada. O estado inicial do código remoto é `cbf9710eba0239c5e88f5a77fa9cbe4c0294758f`.

## Decisões preservadas

| Assunto | Decisão do plano fornecido |
| --- | --- |
| Produto | Leitor EPUB 2/3 reflowable sem DRM, biblioteca por pastas, leitura, personalização, marcações e busca |
| Plataformas | macOS/Windows/Linux, x64 e ARM64 conforme RNF-01 |
| Implementação | Rust estável edição 2024, Iced + cosmic-text/wgpu; sem webview/Chromium |
| Persistência | SQLite embarcado, WAL, migrações, FTS5, backups e cache descartável |
| Privacidade | Sem conta, nuvem ou telemetria; atualização opcional desligada por padrão |
| Aparência | Identidade visual v1.0 fornecida no HTML: marca em Literata, Inter na UI, Lucide e paletas da seção 12; sete modos e fonte do livro conforme a especificação |
| Ambientes | Local rápido e isolado; homologação; produção por promoção do artefato aprovado |
| Código | Padrões Big Bang/L’Essentiel, Ponytail, inglês no código e português na documentação |
| Licença | MIT; fontes OFL; Georgia apenas se instalada no sistema |

As bibliotecas da seção 10 são escolhas do plano, não versões já verificadas. Pesquise fontes oficiais no momento de implementar, fixe versões compatíveis com a MSRV escolhida e registre em STACK/ADR/Cargo.lock. Não troque a stack nem acrescente dependências de execução silenciosamente.

## Evidências e pendências de bootstrap

- Python 3.14.7, Git, GitHub CLI, `cc` e `pkg-config` estão disponíveis. Rust estável foi instalado pelo `rustup-init` oficial após checagem de SHA-256; versões verificadas: `rustc 1.99.1`, `cargo 1.99.1`, `rustfmt 1.10.0-stable`, Clippy `0.1.99`. `pkg-config` encontra xkbcommon, Wayland, X11, Xcursor, Xi e Xrandr; nenhum projeto Rust existe para compilar ainda.
- O repositório remoto é público, com `main` e Wiki habilitada. Em 18/10/2026, o dono criou a Wiki; `Home.md` existe e `bb docs prepare --repository BrunodosSantosVaz/tournepage` passou, fixando o HEAD `cfa369a43b5d4eaf9b37e14b77b0f32bb90d3c87`.
- Discussions está desabilitado. A criação do primeiro post deve seguir a skill da Fundação, quando a comunidade for habilitada no escopo autorizado; não existe post criado nesta preparação.
- O bootstrap F0 foi executado numa worktree própria a partir de `develop` e está no PR draft #3. `bb verify` e `git diff --check` passaram. `bigbang.toml` está configurado para `autonomous`, mas a execução continua bloqueada pelo framework até haver contrato próprio `autonomy.toml`, ruleset `gate` em `main` e `develop`, e ambiente `production`; o contrato herdado foi removido pelo `bb init` e não foi reaproveitado.
- `.bigbang-framework.toml`, `.bigbang-docs.json`, `.bigbang-producao.json`, `autonomy.toml` e os workflows `bb-framework-*` são heranças do Big Bang. Projetos, versões, commits e permissões neles não são comprovações do Tournepage.
- Os workflows herdados condicionam seus jobs ao repositório do Big Bang. Jobs ignorados não são CI verde do produto.
- Rulesets, ambientes, secrets e Project ainda precisam ser inventariados em F4. Não alegar que estão configurados. Não solicitar tokens na conversa.

O conteúdo funcional detalhado ainda precisa de proposta, revisão e publicação versionadas na Wiki pelo processo 18; a Home padrão não comprova cobertura documental. Preserve a especificação fornecida até a publicação confirmada. Não publique conteúdo sem passar pelo PR e pela revisão documental.

## Ponto de execução em 18/10/2026

- PR #1 mantém a preparação da especificação, identidade visual, L’Essentiel/Ponytail, adendos 16–18 e camada de teste local; ainda está draft e aponta para `main`.
- `develop` foi criado do `main` original (`cbf9710eba0239c5e88f5a77fa9cbe4c0294758f`). Issue F0 é #2; PR draft F0 é #3, branch `fundacao/0-ligar-github`, commit `4a5bb97`.
- F0 ainda requer revisão/aceite antes do merge. A PR não instala código do leitor e não valida produto.
- O dono ainda precisa responder se a identidade v1.0 está aprovada para prototipação. O padrão de leitura Claro/Georgia permanece conforme a especificação.
- F4 exigirá autorização explícita para configurações externas do GitHub (rulesets, ambiente, Discussions, secrets e painéis). Não criar secrets em nome do dono.

## Ordem de execução

1. Inspecionar branch, mudanças do usuário, PR desta preparação e estado remoto. Preservar alterações alheias. O executor pode continuar a partir desta branch ou da versão integrada pelo dono; nunca fazer reset destrutivo.
2. Wiki e toolchain local já conferidas; Rust estável e ferramentas foram instalados. Quando F2 fixar MSRV, registrar o toolchain no repositório e conferir bibliotecas nativas no runner Linux e nas demais plataformas. O host atual tem os pacotes de desenvolvimento gráficos detectáveis por `pkg-config`.
3. Concluir a Fundação com `.agents/skills/bb-iniciar-projeto/SKILL.md`, reutilizando as decisões já fornecidas. “F0–F5” do Big Bang antecede e difere das fases do aplicativo.
4. Preflight da Wiki e `bb init` foram executados. O modo autonomous continua dependente do contrato e dos controles do framework.
5. Em F2, adaptar a configuração proposta, publicar documentos produto/stack/design/arquitetura/ADRs na Wiki pelo procedimento do framework e preparar sua rastreabilidade. Não ativar comandos vazios ou scripts inexistentes como se a CI estivesse pronta.
6. Em F3, partir da identidade visual fornecida, inventariada em `IDENTIDADE-VISUAL.md`, e manter o vínculo dos tokens com L’Essentiel. Extrair os assets nativos e preparar ícone e protótipo; registrar o que realmente foi validado. O HTML de referência não é um protótipo funcional validado e depende de um `support.js` ausente. Revisão independente e portões seguem o contrato aplicável; não inventar aprovação do dono.
7. Em F4/F5, inventariar e instalar a esteira do **Tournepage**, Project único, controles e contrato aplicável. Usar `bb generate --dry-run`, `bb generate` e `bb verify`; não editar `.bigbang/` ou arquivos gerados. Não publicar nem alterar permissões porque um exemplo manda.
8. Abrir/refinar os épicos da sequência do BACKLOG, escrever testes de aceite antes da implementação, adquirir posse e usar worktree. Começar pelos riscos da Fase 0, sem telas definitivas ou expansão de escopo.
9. Executar as fases em ordem com os portões definidos. Atualizar a matriz de requisitos com evidência real, commit e teste. Relatar bloqueios com o que foi tentado e continuar o trabalho independente possível.

Comando usado para iniciar F0:

```sh
python .bigbang/bin/bb.py init --name Tournepage --slug tournepage --repository BrunodosSantosVaz/tournepage --visibility publico --license MIT --mode autonomous --workflow grouping
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

Esses nomes descrevem a interface desejada, ainda não comandos instalados. Implementá-los sem exigir ferramentas como Make/just/Docker por padrão. Python padrão, já usado pelo framework, é uma opção para um único launcher multiplataforma; reutilizar alternativas existentes antes de criar wrappers.

Escolher testes pelo grafo transitivo de dependências de `cargo metadata`, não apenas pelo arquivo alterado. Dúvida na seleção, mudanças estruturais, manifestações desconhecidas ou alterações em manifest/lock/migrações/tokens/scripts disparam a suíte apropriada completa. Não omitir dependentes reversos e testes de integração/aceite. Testes com `#[ignore]` de preparação precisam falhar o portão da candidata enquanto pendentes; `cargo test` sozinho pode ignorá-los.

Escrever o teste antes; executar ao final do bloco. Testes específicos de segurança, dados ou fronteiras não desaparecem por estar no ambiente local. A revisão e CI são da integração, não de cada tentativa de abrir o aplicativo.

## Contratos das ferramentas propostas

Os scripts abaixo constam da proposta de configuração, mas ainda não existem. O executor implementa e testa apenas ao chegar à tarefa correspondente:

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
- Validar licenças dos dados de hifenização e o carregamento de fontes nativas. As fontes do plugin são WOFF2; usar TTF/OTF OFL quando o carregador exigir.

## Referência do L’Essentiel

Commit conferido: `03d3468f00da71318da49373805c1fe66a618721`.

Os dez arquivos `.bigbang/padroes/*.md` são idênticos à origem. A Ponytail foi importada sem alteração com LICENSE. Um adaptador em `.claude/skills/ponytail/SKILL.md` aponta para a fonte canônica. Na próxima sessão, a skill estará disponível pela descoberta do projeto; seu conteúdo já está linkado por AGENTS.

Valores de aparência foram conferidos em `src/platform/reader-style.ts`, `reader-panel.ts`, `reader-toolbar.ts` e `reader-fonts.ts`. O plugin não possui o JSON compartilhado descrito no plano. Criar tokens locais rastreáveis não exige nem autoriza alterar o plugin. A identidade visual adicionada pelo dono detalha as diferenças do Tournepage: marca em Literata, Papel/Grafite, colunas, Inter, Lucide e fontes da seção 10. A prévia Papel/Literata não muda Claro/Georgia como padrões do leitor; referências remotas de fontes/ícones no showcase não devem ir para o runtime nativo.

## Limites desta entrega e da autonomia

Não alterar `autonomy.toml` para ampliar riscos, custos, flags, validade ou publicação. O arquivo herdado ainda não é um contrato Tournepage validado. Conferir `bb autonomy check` depois da Fundação e dos controles; se faltar autorização para uma etapa, explicar o requisito exato do arquivo/skill aplicável e pedir apenas a decisão indispensável, sem repetir escolhas já dadas.

Esta preparação não mescla PR, publica versão, cria segredos, muda permissões ou afirma homologação. A promoção real deve respeitar o “aval” do dono/contrato aplicável e usar o mesmo artefato homologado. Atualizar o handoff e os resultados ao fim de cada entrega para o próximo modelo retomar sem presumir sucesso.
