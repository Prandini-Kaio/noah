# ADR 0011 — Superfície de run e identidade geométrica
Status: implementado em 2026-11-13.

Delaunay com câmeras/pontos esparsos oferece superfície CPU sem CUDA. Poisson
recebe nuvem densa com normais; profundidade 6, trim 0, dois threads. Trim padrão
eliminou uma superfície pequena no ensaio real, por isso fica explicitamente 0.
Nenhum modo gera textura UV. PLY bruto é preservado, OBJ normaliza geometria.
PLY binário little-endian e ASCII têm limites, índices e atributos conferidos.

RunSurface é fonte distinta de Run: id run:UUID:surface, contra run:UUID da
nuvem. Sessão 6 explicita essa escolha e lê versões 1–5 sempre como suas fontes
originais; adicionar uma malha não muda âncoras/medidas de sessões antigas.
Relatório passa a versão 3 pelo novo tipo de fonte. Abertura de run novo prefere
OBJ registrado junto com proveniência, enquanto sessões antigas usam sparse.

Continuar run cria novo job, valida hashes do pai e fotos, copia os modelos TXT,
restaura BIN pelo motor configurado explicitamente e executa apenas meshing.
Não executa automaticamente motor apontado por manifesto externo. Pai, brutos e
resultados parciais permanecem preservados. Não é edição destrutiva, CAD ou
validação metrológica; não implica conclusão da densificação GPU.
