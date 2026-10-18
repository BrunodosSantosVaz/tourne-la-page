# Referência operacional da identidade visual

Fonte fornecida pelo dono em 18/10/2026: [Tournepage Identidade Visual.dc.html](<../../Tournepage Identidade Visual.dc.html>), identificada no próprio documento como v1.0 · Out 2026. O HTML original foi preservado, sem alteração ou execução. SHA-256 da fonte lida: `2443121469e4f138ba74066d4ba45fd46b9b8a20db97d558b2d184d47ec587bf`.

Esta referência completa o design da especificação para a Fundação/F3 e as tarefas T19a/T19b. São orientações de preparação; não há tokens de produção, arquivos de ícone exportados ou protótipo validado nesta entrega.

## Marca e símbolo

- Logotipo escrito **tournepage**, sempre em minúsculas; assinatura “Menos ruído, mais leitura.”. O nome editorial do produto continua Tournepage.
- Símbolo: página com canto inferior direito dobrado; corpo na cor do texto, dobra na cor de destaque/link. A fonte inclui os polígonos SVG originais, com `viewBox="-4 -4 88 108"`; reutilizar a geometria ao produzir os assets.
- Composição horizontal é a principal; há versões vertical, símbolo isolado, escura, sobre destaque, sépia e monocromática.
- Área livre igual à largura da dobra; símbolo a partir de 16 px, logotipo completo a partir de 80 px de largura. Não distorcer, girar, trocar cores arbitrariamente ou usar “TournePage”.
- Ícone do app: página em Tinta sobre Papel; mesma geometria nas três plataformas. O contêiner varia: cantos contínuos no macOS, raio de 4% na proposta Windows, SVG/PNG no Linux. As saídas `.icns`, `.ico` e SVG/PNG ainda precisam ser produzidas e conferidas nos tamanhos reais.

## Paleta e temas

Marca: Papel `#FCFBF7`, Tinta `#2B2722`, Azul-tinta `#1A4FB4`, Grafite `#141516` e Pedra `#6D665D`.

Valores das seis paletas concretas em `THEMES` no script de dados do HTML:

| Tema | bg | text | text-muted | accent/link | bg-elevated | border |
| --- | --- | --- | --- | --- | --- | --- |
| Claro | #FFFFFF | #222222 | #6B6B6B | #1A4FB4 | #FFFFFF | #E6E6E6 |
| Papel | #FCFBF7 | #2B2722 | #6D665D | #1A4FB4 | #FFFFFF | #E7E3DA |
| Sépia | #FAF7F0 | #342D22 | #70624F | #7A4B12 | #FFFDF8 | #E6DECF |
| Escuro | #292A2D | #D4D4D4 | #AAAAAA | #8AB4F8 | #34353A | #3E3F44 |
| Grafite | #141516 | #DDDDDD | #AAAAAA | #8AB4F8 | #1F2022 | #2A2B2E |
| Noite | #000000 | #E6E6E6 | #AAAAAA | #8AB4F8 | #141414 | #262626 |

Automático continua sendo o sétimo modo da especificação, escolhendo a paleta conforme o sistema; não é uma sétima paleta fixa. Temas personalizados continuam no escopo. A ausência deles no seletor da prévia não os remove do produto.

Marcações usam OKLCH, com matizes Amarelo 95°, Verde 150°, Azul 245°, Rosa 350° e Roxo 305°:

| Grupo de temas | L | C | Aplicação |
| --- | --- | --- | --- |
| Claro/Papel/Sépia | 0,91 | 0,09 | Fundo do destaque |
| Escuro/Grafite | 0,45 | 0,08 | Fundo do destaque |
| Noite | 0,72 | 0,14 | Sublinhado grosso, fundo transparente |

O HTML contém a conversão `ok(L, C, h)` para sRGB. Na implementação, versionar a conversão/valores resultantes e validar contraste/gamut, inclusive estado selecionado; não presumir conformidade AA pela escolha da cor.

Tokens semânticos adicionais estão explicitados para Papel/Grafite: `danger` = `#B3261E`/`#F2B8B5`, `focus-ring` = `#1A4FB4`/`#8AB4F8`, e `selection` = OKLCH `(0.88, 0.05, 255)`/`(0.38, 0.07, 255)`. Completar e validar os demais temas ao gerar o contrato de tokens; não declarar o contrato completo só com esses pares.

## Tipografia, ícones e geometria

| Elemento | Referência fornecida |
| --- | --- |
| Marca/logotipo/títulos/capas geradas | Literata; logotipo com peso 500 e tracking −0,02 em |
| Interface | Inter; xs 12/500, sm 13/400, md 15/400, lg 18/500, xl 24/500 (tamanho/peso) |
| Código/dados/CFI | JetBrains Mono |
| Texto do livro | Escolha do leitor: Georgia do sistema, Literata, Source Serif 4, Merriweather, Atkinson Hyperlegible Next e Lexend |
| Ícones | Base Lucide, traço linear sem preenchimento, cantos arredondados, grade 24 px; exibição de 16 px em listas e 20 px nas barras; `text-muted`, ativo em `text` |
| Espaços | 4, 8, 12, 16, 24, 32, 48 e 64 px |
| Raios | sm 4 px em botões/campos, md 8 px em cartões/menus, lg 12 px em pop-ups/diálogos |
| Foco | Anel de 2 px, offset 2 px |
| Movimento | ease-out; fast 120 ms, base 200 ms, slow 300 ms; respeitar reduzir movimento |
| Medida da coluna | Mínimo 45 ch, padrão 66 ch, máximo 90 ch, conforme preferências/RN-29 |

Os nomes de cada ícone estão em `iconGroups` no HTML, agrupados em Biblioteca, Estados do livro, Leitor e Marcações e dados. Reutilizar esse mapeamento ao selecionar os SVGs; não inventar outra família. O showcase referencia `lucide-static@0.460.0` por CDN: é a referência do arquivo recebido, não uma dependência de execução instalada ou uma versão pesquisada nesta etapa. Preservar avisos/licença dos assets escolhidos; uma nova crate/biblioteca requer o portão de stack.

## Relação com o plano e próximos recortes

- Marca em Literata não altera a fonte padrão do livro. O HTML usa `wordmarkFont=Literata` e `readerTheme=Papel` na prévia; a especificação mantém **Claro + Georgia** no leitor, com Literata quando Georgia não estiver disponível. As alternativas Source Serif 4/Inter no tweak do logotipo são explorações da prévia, não novas variantes padronizadas.
- T19a extrai o contrato de tokens para o aplicativo nativo a partir desta fonte, preservando o vínculo com L’Essentiel. O JSON compartilhado com o plugin continua sendo uma entrega futura; o HTML não comprova que já exista.
- F3 reaproveita a identidade recebida para produzir o ícone global, variantes do logotipo e Social preview. T28 usa os formatos de ícone necessários ao empacotamento. Essa extração/exportação ainda não foi realizada.
- As aplicações mostram capa substituta (RF-BIB-17), página de leitura, menu de seleção com cinco cores e ferramentas, progresso e aviso com desfazer. Orientam T13/T17/T21; não substituem prova de interação/acessibilidade, telas de biblioteca/configurações ou portões da Fase 0.
- O HTML depende de `./support.js`, ausente no checkout lido, e de componentes `x-dc`/`sc-for`, `DCLogic` e expressões de template. Também referencia Google Fonts e ícones do unpkg. É fonte de design, não uma página autônoma nem código para embutir no app Rust.
- Fontes/ícones devem estar empacotados localmente no aplicativo; não transportar as chamadas às CDNs para a execução do produto. A implementação continua nativa e sem webview, conforme RNF-02/12.
- Conservar o HTML original no projeto como fonte fornecida pelo dono enquanto a Wiki não estiver publicada/verificada; não mover nem remover a referência durante a organização.
