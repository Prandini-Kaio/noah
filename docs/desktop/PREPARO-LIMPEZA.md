# Preparar imagens, limpar nuvem e gerar superfície

## 1. Preparar fotografias
Reconstrução → Usar fotos do kit (ou Escolher fotografias) → Preparar imagens… →
Analisar pasta escolhida. Revise dimensões, detalhe e frações de pixels escuros e
claros; desmarque as fotos que não deseja usar. Desmarcar duplicatas exatas é uma
ação explícita, nunca automática. São necessárias pelo menos três selecionadas.

Lado maior 0 mantém bytes originais. Entre 320 e 12000 redimensiona
proporcionalmente se necessário e grava PNG. Não aplica orientação EXIF, máscara,
correção de lente ou de cor; PNG não mantém EXIF e COLMAP estima intrínsecos
novamente. Não representa calibração garantida. Escolha uma pasta de resultados
e nome novos → Salvar preparo e usar no pipeline. preparation.json guarda todas
as decisões/métricas/hashes; o motor usa o subdiretório images do preparo.
Avisos de desfoque/exposição são heurísticos e dependem do conteúdo da foto.

## 2. Reconstruir e abrir a nuvem
Escolha COLMAP 3.11.x confiável e novo nome de run. Nuvem esparsa · CPU basta para
limpeza; Malha rápida da esparsa · CPU também gera uma superfície inicial. Iniciar
pelas fotos → aguardar → Abrir esparsa. Esse botão abre a fonte do run com suas
câmeras, não um PLY desconectado. Para run antigo: Workspace → Projeto/run; caso
abra como malha, Reconstrução → Abrir nuvem esparsa do run selecionado.

## 3. Prévia e edição
Selecione a nuvem na árvore → Limpar nuvem… → Editar nuvem selecionada.

- SOR: k é o número de vizinhos sem contar o próprio ponto; σ controla o limiar
  da média das distâncias. Prévia de outliers estatísticos marca candidatos.
- Raio: distância nas unidades locais da fonte e mínimo de vizinhos. Prévia de
  outliers por raio marca pontos isolados. Nenhum filtro garante identificar
  somente erros; detalhe fino e regiões pouco amostradas podem ser removidos.
- Selecionar retângulo no viewport fecha a janela. Arraste na cena, atravessando
  toda a profundidade; Esc cancela. Reabra Limpar nuvem… para remover ou isolar.
- Caixa XYZ usa coordenadas locais originais, não réguas U/V projetadas ou a
  posição de uma peça transformada. Limpar seleção apenas desmarca pontos.
- Aplicar remoção prevista confirma o filtro. Cancelar prévia mantém a nuvem.
  Comparar mostra removidos em vermelho; Desfazer guarda oito operações.

Até 200 mil pontos, 64 vizinhos e 64 operações por revisão. Cálculo de filtros em
worker; exibição continua amostrada até 50 mil pontos. Contagens/seleção usam todos
os pontos aceitos. Fechar mantém edição temporária; Descartar edição temporária
restaura a fonte. Salvar workspace exige primeiro salvar/descartar essa edição.

Salvar e abrir revisão limpa cria uma pasta nova, preservando fonte e snapshot
local original.ply, cloud.ply e cloud-revision.json. RGB é preservado. Posição da
montagem e calibração da sessão não são embutidas: coordenadas da fonte são
mantidas. Original é ocultado após abrir a revisão; use sua visibilidade para
comparar. Medidas/âncoras antigas continuam na fonte antiga, sem remapeamento.

## 4. Superfície da revisão limpa
Selecione a nova nuvem → Reconstrução → Usar run selecionado para gerar malha.
Confira Origem e Revisão limpa, escolha Malha rápida da esparsa · CPU, COLMAP,
pasta e nome novos → Gerar malha do run → Abrir malha no workspace.

Funciona para revisão cuja origem é a nuvem sparse do run, inclusive cadeia de
até 16 revisões com os pais disponíveis. Remoção é traduzida para os IDs COLMAP:
observações removidas viram -1, índices de observação e tracks retidos permanecem.
Novo run guarda seleção e receita, preservando o pai. Não repete extração/matching/
SfM. O resultado continua aproximado, com possível fundo/buracos, sem textura UV,
CAD ou precisão metrológica. Dense/GPU completo continua sem validação real.

PLY isolado serve para edição/exportação e não contém câmeras para esse fluxo.
Sem COLMAP, use examples/limpeza/nuvem-com-outliers.ply: SOR k=16, σ=2 identifica
5 candidatos; 343 pontos permanecem. Raio=0.2 e mínimo=3 dá a mesma contagem.

Sessões novas são versão 7 (lê 1–6), relatórios JSON versão 4. Não mova os arquivos
referenciados. A sessão não é um pacote autocontido; leve junto fontes e revisões.

## CLI equivalente
```sh
noah photos-audit ./fotos
noah photos-prepare ./fotos ./preparo
# Opcional: --max-size 1600; seleção individual de fotos é feita pela interface.
noah reconstruct ./preparo/images ./run --colmap /caminho/colmap
noah cloud-clean ./run ./nuvem-limpa --sor 16 2
noah mesh-from-clean ./nuvem-limpa ./malha-limpa --colmap /caminho/colmap
```
