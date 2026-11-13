# ADR 0012 — Preparação, nuvem revisada e malha limpa
Status: implementado, 2026-11-13.

Preparação analisa pixels decodificados com limites (JPEG/PNG, 500 arquivos,
32 MiB/foto, 512 MiB/conjunto, 32 MP/256 MiB alocação por decodificação). Variância
laplaciana usa miniatura de até 512 px; exposição é fração de pixels ≤8/≥247.
São heurísticas, sem exclusão automática. Duplicata é SHA-256 exato, não similaridade
visual. Preparo copia selecionadas (mínimo 3) ou gera PNG redimensionado Lanczos3,
sem rotação automática, correção fotométrica ou máscara. PNG não conserva EXIF;
COLMAP estima intrínsecos novamente. Arquivo e coordenadas originais preservados.
Manifesto preparation.json versiona seleção, parâmetros, métricas e hashes e é
verificado antes e durante o snapshot do novo run, que incorpora sua proveniência.

Limpeza em coordenadas locais usa k-NN exato em árvore kd, SOR com desvio padrão
populacional e filtro por raio; k exclui o ponto consultado e aceita duplicatas
como vizinhos. Até 200 mil pontos, k≤64; raio/unidade da fonte desconhecida quando
não declarada. Filtros podem remover detalhes reais. Edição temporária mantém
máscara sobre índices originais: retângulo projetado atravessa profundidade,
caixa XYZ, remover/isolar, prévia e oito operações de desfazer. Não muda fonte,
medidas ou âncoras. Salvamento em diretório novo guarda original.ply (snapshot
normalizado dos pontos locais), cloud.ply, índices originais, fonte/hash e receita.
Não é cópia de todos os atributos proprietários do arquivo de captura; RGB é mantido.

Fonte CloudRevision distinta; sessão 7 lê 1–6 e relatório 4. Não há migração de
âncoras para a revisão, nem transferência automática de escala calibrada da sessão.
Ao abrir revisão após salvar, original é ocultado para comparação por visibilidade.
Revisões encadeadas resolvem até 16 pais com hashes e composição de índices.

Malha limpa só usa revisões que remontam à nuvem sparse do run COLMAP, não PLY
isolado sem câmeras. Novo run recebe modelos copiados, pontos removidos são
retirados de points3D.txt e observações correspondentes viram -1 em images.txt.
IDs retidos e índices de observação/tracks não mudam. BIN é restaurado e Delaunay
executado no novo modelo. Seleção em IDs/índices e receita entram nos artefatos;
fotos, pai e malhas anteriores não mudam. Dense com normais não é filtrado por
este fluxo; CUDA completo e texturização permanecem pendentes.
