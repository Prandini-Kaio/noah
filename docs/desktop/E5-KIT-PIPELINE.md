# Testar o pipeline e o alinhamento sem terminal

## 1. Reconstrução esparsa real
Extraia toda a pasta do pacote, mantenha examples junto de NOAH e abra o app.
Em Reconstrução, clique Usar fotos do kit incluído. As 12 imagens já vêm prontas.
Escolha um COLMAP **3.11.x** instalado localmente, um diretório gravável de
resultados e um nome novo. Mantenha densificação desmarcada e inicie.
O motor é separado: instalação oficial em https://colmap.github.io/install.html.
No seletor escolha o executável confiável, frequentemente /usr/bin/colmap ou o
arquivo da instalação local. Pacotes da distribuição podem ter outra versão.

Acompanhe as etapas, logs e cancelamento. Ao concluir, Abrir nuvem no workspace
carrega a nova reconstrução. O fluxo validado é CPU/esparso, não uma malha densa.
Não há compromisso de tempo, número idêntico de pontos ou reconstrução de qualquer
conjunto de fotos. Resultados em outro hardware/motor podem variar.

## 2. Inspecionar, medir, salvar e exportar
Selecione a nuvem na árvore. Orbite, use vistas ortográficas e réguas. Em Medidas,
escolha dois vértices por snap ou índices e registre uma distância. Sem referência
física válida o resultado permanece em unidades da cena. **Não use automaticamente
as dimensões das caixas do kit para calibrar esta nuvem SfM**: são dados distintos.

Salvar cria uma sessão nova, com fonte e medidas; Workspace → Abrir workspace a
restaura. Não mova a pasta do run depois de salvar (caminhos absolutos).
Em Exportação, exporte a peça/nuvem selecionada e o relatório de medições. PLY e
metadados usam o estado montado; relatório inclui fontes, transformações e métodos.
Não são sobrescritos destinos existentes.

Sem COLMAP, Abrir nuvem pronta carrega sparse-preprocessado.ply: é um resultado
histórico do exemplo E2, **não uma reconstrução executada pelo botão**. O JSON ao
lado tem 7592 pontos/12 câmeras. A nova execução desta entrega produziu 7590 pontos
pela CLI e 7611 pela interface, sempre 12/12 imagens. A variação não é erro métrico.

## 3. Alinhamento de peças
Salve seu workspace antes de trocar. Workspace → Abrir peças de alinhamento do
kit abre fixa.obj e movel.obj. São malhas sintéticas independentes da fotogrametria,
não peças separadas automaticamente da nuvem reconstruída.

Em Montagem → Alinhar peças:
- Peça móvel: movel.obj, vértice 1.
- Destino fixo: fixa.obj, vértice 2.
- Calcular: ΔX=-4, ΔY=0, ΔZ=0 unidades da cena.
- Pré-visualizar aproxima as caixas sem alterar a montagem real.
- Aplicar alinha os vértices e preserva rotação/escala e a fonte.
- Desfazer volta ao estado anterior, enquanto não houver mudança posterior de
  geometria, hierarquia ou transformações. Seleção/câmera/nomes não impedem desfazer.

É possível usar dois pontos de peças distintas já escolhidos em Medir: A move até
B. A operação move uma peça, respeitando rotação/escala dos pais. Não alinha grupos
inteiros, não gira automaticamente, não faz ICP e não cria restrições CAD.
Fechar a janela cancela a prévia; somente o ajuste aplicado é salvo. Histórico de
desfazer é temporário. Referências lineares entre peças podem ficar desatualizadas.

## 4. Wireframe e estilhaços
No inspetor/Visualização, **Arame** mostra somente as arestas triangulares, como
antes. **Sólido** mostra as faces. **Sobrepor arestas ao sólido** é independente;
não substitui o modo Arame. Pontos/nuvens não se tornam malhas por escolher Arame.
A preferência wireframe salva na sessão volta a significar arame sem preenchimento.
A sobreposição é transitória. Profundidade visual permanece aproximada/amostrada.

Em Explosão e estilhaços, escolha Médios/Pequenos/Muito pequenos e clique Explodir.
Detalhe padrão em caixa simples: 12 faces → 768 fragmentos, antes 48. Malhas maiores
ajustam subdivisão ao orçamento; só acima de 20 mil faces-base precisa amostrar.
Nuvens apenas dispersam pontos, sem inventar superfícies. Força 0,2–6, padrão 1,4,
com impulso radial ampliado e pequena queda Z; duração nominal 1,6s.

A câmera pode acompanhar a explosão; ajuste a opção antes de iniciar. Navegar
interrompe o acompanhamento; F/Enquadrar considera os fragmentos no instante atual.
Sem acompanhar, alta força pode jogar fragmentos para fora da tela. Pausa, Tempo,
Remontar e Restaurar permanecem disponíveis. Não há fratura volumétrica, interior
ou colisões. As fontes, medidas e exportação são preservadas.

## Dados e limites
examples/pipeline contém fotos, hashes, câmeras ideais e resultado pré-processado;
examples/alinhamento contém duas peças; inspection-box.obj é a caixa E4 de referência.
Tudo é sintético e CC0-1.0, sem fotos externas. Scripts de geração estão no código.
Não há download de motor nem de dados ao clicar nos exemplos.

O teste completo desta entrega cobre o **pipeline esparso** e os fluxos manuais
posteriores. Dense/CUDA, malha reconstruída, separação automática de componentes,
CAD, precisão física e etapas E6/E7/E8 não estão validados/implementados aqui.
Sessão 5 e relatório 2 mantidos; ângulos/dimensões da E4 continuam disponíveis.
