# UX, pipeline guiada e desmontagem — 2026-11-13
Pedido autoriza oito melhorias: logs selecionáveis/copiáveis; pastas independentes;
fluxo/feedback/menus; edição de nuvens acessível; propriedades identificadas;
configuração geral separada; abertura focada de resultados; explosão por camadas
e desmontagem tipo armadura, conservando estilhaços. Desenvolvimento completo
refere-se a este recorte; não inventar peças internas de um scan nem concluir CAD.

Plano antes de editar:
- desktop/preferences + browser: histórico por finalidade persistido localmente,
  COLMAP lembrado, seletores de preparo/limpeza independentes; erros de gravação.
- jobs/logs: resumo por etapa, saída limitada claramente identificada, seleção
  textual, copiar tudo/etapa, atualizar e filtros de stdout/stderr.
- reconstruction: uma entrada de pipeline com abas Entrada, Processamento e
  Resultados; validação/preflight; preferências gerais em janela separada.
- shell/main: estado da cena e seleção sempre visíveis, inspetor com abas estáveis
  Objeto/Análise/Ferramentas, ações contextuais identificadas, árvore legível;
  abrir resultado fecha janelas, seleciona/foca geometria e evita duplicatas.
- edição: ação destacada para nuvens/resultados, modo de edição no viewport e
  feedback antes/depois, descarte/preview/undo identificados.
- workspace/disassembly + desktop/effects: separar componentes, camadas e armadura
  da fragmentação; controle de profundidade e preservação de submontagens;
  demonstração de carcaça/núcleo aninhados. Não altera/exporta assets explodidos.
- testes de caminhos/persistência, logs, navegação de resultado e hierarquia;
  sessão versionada se contratos mudarem; docs/ADR, testes nativos e pacote Linux.

Aceite: fluxo real do kit, logs selecionados/copiados, caminhos independentes após
reinício, resultados focados/edição, montagem aninhada desmonta e remonta mantendo
fontes/medidas; fmt/clippy/test/build, screenshots e relatório reais.
