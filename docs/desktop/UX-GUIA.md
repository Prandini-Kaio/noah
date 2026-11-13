# NOAH v0.10 — reconstruir, editar e desmontar

## Fluxo principal
**Reconstruir…** e o módulo **Reconstrução** abrem a mesma janela. Atalhos na
estrutura apenas abrem um passo; não executam o motor diretamente.

1. **Configurar:** escolha fotografias ou continue um run/revisão selecionado.
   Preparar/selecionar imagens abre o preparo não destrutivo. Escolha nuvem sparse,
   superfície CPU ou modos densos CUDA, pasta e nome novos. A configuração avançada
   contém o timeout por etapa. A checagem informa o que falta; o botão **Iniciar
   reconstrução** fica no rodapé, separado dos campos roláveis.
2. **Processar:** acompanhe as etapas numeradas e seu estado. O rodapé dá acesso
   ao diagnóstico e cancelamento. Saídas parciais válidas permanecem disponíveis.
3. **Resultados:** cada cartão identifica tipo e disponibilidade. **Visualizar no
   workspace** fecha a janela, seleciona/enquadra a saída, habilita sua exibição e
   oculta outros objetos para comparação explícita na estrutura. Uma fonte já
   aberta é reutilizada. **Editar nuvem…** faz o mesmo e inicia a edição.

Fotos escolhidas não significam objeto 3D criado. A estrutura informa quantas
nuvens/malhas estão abertas, enquanto a pipeline informa os arquivos produzidos.
Abra um resultado para colocá-lo na cena. Para voltar a um run existente, selecione
sua fonte e use Ações → Resultados desta reconstrução, ou carregue a última
execução lembrada no passo 3. Nenhum desses comandos reexecuta COLMAP.

## Configurações e pastas
**Configurações…** contém o caminho do COLMAP 3.11.x e o histórico de pastas por
finalidade. Imagens, motor, resultado da pipeline, preparo, limpeza, importação,
exportação e sessões têm históricos independentes, inclusive ao cancelar o
seletor após navegar. Reiniciar mantém motor/pastas; não inicia processos.

Preferências locais: `$XDG_CONFIG_HOME/noah/preferences.json`, ou
`~/.config/noah/preferences.json`. Motor e pastas são locais ao app. Tema/réguas e
opções de apresentação acompanham o workspace. Parâmetros do run ficam no passo 1.

## Diagnóstico e cópia
No passo 2: **Abrir diagnóstico / copiar logs**. O texto é somente leitura e pode
ser selecionado. Ctrl+A seleciona tudo; Ctrl+C copia a seleção. Os botões copiam o
texto exibido ou o relatório completo; filtros escolhem etapa e stdout/stderr.
**Atualizar** carrega novo conteúdo sem interromper o processamento. O diagnóstico
mostra run, pasta, motor, estado, duração/código por etapa e separadores claros.

A exibição usa até 128 KiB finais por fluxo/etapa e 2 MiB totais; trechos finais são
identificados. Logs completos permanecem nos arquivos do run. Mensagens em stderr
do COLMAP não são automaticamente tratadas como falha; confira o estado da etapa.

## Objeto, análise e ações
O inspetor mantém nome, tipo, visibilidade e caminho da seleção no topo:

- **Objeto:** dados, montagem pai e transformação local desse objeto/grupo.
- **Análise:** medições, escala e dimensões calculadas no estado montado.
- **Ações:** editar nuvem, ver resultados, gerar superfície, alinhar, apresentar e
  exportar. Parâmetros da pipeline e efeitos não ficam misturados às propriedades.

Na toolbar, o seletor inicial escolhe Selecionar/Pan/Órbita. **Editar nuvem** atua
na seleção; a janela informa o alvo da edição. **Exibição…** reúne tipos de
geometria, grade, arame e maximização. Um resultado focado oculta os demais;
reative suas caixas na árvore para comparar.

## Edição de nuvens
Disponível no cartão do resultado, toolbar e Ações. Retângulo atravessa toda a
profundidade; caixa XYZ usa coordenadas locais. SOR/raio têm prévia, aplicar,
cancelar e desfazer. Salvar cria uma nova revisão, mantendo fontes e índices
originais. Até 200 mil pontos nesta prévia. Fechar a janela mantém a edição
pendente; conclua ou descarte antes de substituir/salvar o workspace ou sair.
[Guia de preparação/limpeza](PREPARO-LIMPEZA.md).

## Construir objetos aninhados
Na estrutura, crie uma montagem. Se outra montagem estiver selecionada, **Criar
montagem na seleção** cria um filho dentro dela; caso contrário, cria na raiz.
Selecione uma peça/grupo e escolha **Montagem pai** na aba Objeto. A hierarquia é
validada contra ciclos. Mudar pai conserva valores locais e pode mudar a posição
mundial. Não há inferência automática de peças ou CAD/restrições.

## Desmontagem e fragmentação
**Desmontagem…** abre uma ferramenta própria, fora do inspetor de propriedades.
Ao iniciar/remontar uma animação, a janela fecha para liberar a visualização;
reabra para ajustar/restaurar. Os dados originais não mudam.

**Desmontar componentes:**
- Radial: separação anterior das peças inteiras.
- Camadas: abre até o nível escolhido, mantendo filhos de submontagens juntos.
- Armadura: abre os níveis em sequência; o núcleo contido pode permanecer fixo
  enquanto a carcaça abre. Profundidade maior revela mais filhos.

Afastamento usa unidades da cena. Abertura permite inspecionar a posição
intermediária; Desmontar/Explodir e Remontar animam a transição. Não se criam
objetos internos que não existam na hierarquia. Para uma malha única, use painéis
visuais ou construa uma montagem com componentes separados.

**Fragmentar superfície:**
- Estilhaços pequenos: subdivisão e dispersão anteriores, preservadas.
- Painéis de superfície: faces originais agrupadas em placas rígidas por orientação
  e região, sem subdivisão fractal. Em uma caixa simples, seis placas.

Não há fratura volumétrica, preenchimento de interior ou colisões. Os painéis
podem agrupar regiões desconectadas. Prévia amostrada até 20 mil faces/fragmentos;
fragmentos não persistem nem são exportados. Medidas/exportação usam o montado.

**Workspace → Explorar armadura por camadas…** materializa um exemplo sintético
CC0 com 13 peças e seis montagens: carcaça, peitoral, capacete, braços e núcleo.
Escolha uma pasta nova, abra Desmontagem e teste profundidade 1 versus 4.

Sessão nova: **8**, leitura **1–7**. Relatório geométrico: **4**. Caminhos de fontes
continuam absolutos: preserve arquivos/revisões. Pacote Linux requer x86_64,
glibc 2.39+, X11/XWayland e OpenGL. COLMAP não é incluído. Dense GPU completo,
textura UV, CAD e precisão metrológica continuam pendentes.
