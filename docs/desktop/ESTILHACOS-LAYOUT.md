# Workspace Arctic: organização e estilhaços

## Navegação
A barra superior contém Workspace (novo/abrir/projeto/exemplo), Importar
(fotos/geometria), Salvar, execução do pipeline configurado e Preferências.
Os módulos trocam o contexto do inspetor, preservando a cena. Reconstrução abre
o acompanhamento existente; Captura escolhe uma pasta de fotos. Escolher fotos
não inicia o pipeline nem comprova sua validade: o motor ainda valida as entradas.
Rodar pipeline só é habilitado com diretórios/motor configurados e sem outro job.

Estrutura à esquerda: fluxo recolhível com estado real, busca sem distinguir
maiúsculas, árvore com visibilidade e seleção, e criação de montagens recolhível.
A busca por peça mantém os ancestrais; a busca por montagem mostra seu subgrupo.
Arraste as divisórias para redimensionar os painéis. Ocultar, Inspetor, Estrutura
e Maximizar/Restaurar painéis estão na toolbar ou nas Preferências.

O inspetor tem Propriedades e Análise, rolagem independente e seções recolhíveis.
Transformações seguem a seleção. Análise contém distâncias, escala e relatório;
tamanho de ponto e arame ficam em Visualização. Câmera e Exibição ficam na barra
do viewport. Selecionar não orbita ao arrastar; Pan/Órbita mudam a ação principal.
Botão direito/meio ou Shift continuam deslocando a câmera; roda faz zoom.

## Explosão temporária
1. Abra `tests/datasets/reference-mesh.obj` em Importar → Geometria, ou uma malha sua.
2. Em Montagem/Inspeção, abra Explosão e estilhaços e escolha **Estilhaçar**.
3. Opcionalmente marque **Somente seleção** para uma peça ou montagem visível.
4. Ajuste Força e clique **Explodir**. Os fragmentos saem em direções diferentes,
   giram e recebem uma pequena queda no eixo Z mundial.
5. Pause/reproduza, arraste Tempo ou use **Remontar** para reproduzir ao contrário.
   **Restaurar** remove a prévia imediatamente. Remontagem concluída também restaura.
6. Enquadrar/F ajusta a câmera aos fragmentos no instante atual. Zoom e navegação
   continuam disponíveis; fragmentos podem sair da tela quando a força é alta.

**Componentes** continua oferecendo Separar peças/Montada e afastamento em unidades
da cena. É a vista técnica anterior, independente da animação de estilhaçamento.

Movimento reduzido nas opções visuais aplica o estado final sem reprodução; a
linha do tempo permanece manual. A prévia bloqueia seleção/edição das peças e
oculta suas anotações: estilhaços não são novos objetos metrológicos. Abrir Medir
restaura o objeto. Salvar e exportar continuam usando o estado original montado
(exportação) e as preferências/estrutura reais (sessão); nenhum fragmento é salvo.

## Contrato e limites
Efeito somente de apresentação, sem nova representação de domínio. Cada face
triangular amostrada é subdividida em quatro triângulos coplanares, com cobertura
exata da face no tempo zero. Não existem faces de corte, interior volumétrico,
colisões, fratura física ou edição topológica. São estilhaços de superfície.
Nuvens dispersam pontos e preservam suas cores; nenhuma superfície é inferida.

Preparação determinística em coordenadas mundiais XYZ destras, incluindo hierarquia
e posição visual atual. Rotação rígida por fragmento, impulso radial com variação
reprodutível e duração nominal 1,6s. A linha do tempo é normalizada 0–1; não é uma
simulação científica. Sem aleatoriedade externa nem novas dependências.

Orçamento máximo de 20 mil fragmentos por prévia. Malhas grandes são amostradas e
podem apresentar lacunas; não são segmentadas integralmente. Renderer mantém
ordenação aproximada por profundidade, sem z-buffer. Não se altera calibração,
unidades, incerteza, fontes, identidades ou schema 4. Layout/painéis e modo de
exibição são transitórios; tema/réguas/câmera continuam seguindo a sessão.

A especificação visual anexada orientou hierarquia e agrupamento. Seção de corte,
ângulo/área/volume, bibliotecas de ícones e novas fontes não foram implementados:
continuam em uso as fontes distribuídas/licenciadas pelo egui. Não há controles
que prometam esses recursos futuros.

Atualização E4: ângulos e dimensões disponíveis na prévia v0.6; sessões novas
são versão 5. Consulte [Inspeção E4](E4-INSPECAO.md) para o comportamento atual.
