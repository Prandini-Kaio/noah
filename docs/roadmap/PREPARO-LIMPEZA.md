# Preparação, limpeza e superfície derivada — 2026-11-13
Autorizado pelo usuário: preparação de imagens, outliers/edição da nuvem e próxima
etapa. Interpretação: próxima etapa é gerar superfície da revisão limpa; não CAD,
LiDAR nem iniciar todo E6. Original sempre preservado.

Plano antes de modificar:
- photogrammetry/preparation: analisar JPEG/PNG com limites, desfoque/exposição,
  resolução e duplicatas; seleção e cópias preparadas opcionais redimensionadas;
  métricas/decisões/hashes registrados, entrada preparada reconhecida pelo run.
- geometry/cleaning: seleção por caixa, filtro estatístico k-NN e raio com índice
  espacial; índices originais e cores preservados, parâmetros validados.
- workspace/cleanup: revisões imutáveis com fonte/hash, índices e receita; prévia
  e desfazer na UI, sem reinterpretar medidas antigas. Sessão 7/relatório 4.
- desktop: preparo em worker, edição por retângulo projetado/caixa local, prévia
  de outliers, comparação de contagens, aplicar/desfazer e salvar/abrir revisão.
- photogrammetry/surface: revisão sparse do run → modelo filtrado com observações
  removidas marcadas -1, tracks mantidos, novo run e malha CPU rastreável.
- CLI, testes, docs, kit de outliers e pacote Linux.

Aceite: casos sintéticos conhecidos e limites, originais intactos/hashes,
preparo/revisão round-trip/adulteração, âncoras antigas intactas; COLMAP real para
malha limpa, UI nativa inspecionada, fmt/clippy/test/build.
Limites: métricas fotográficas heurísticas não excluem automaticamente; filtros
podem remover detalhe, precisam de prévia. Sem pintura/escultura, fratura física,
texture UV, máscaras fotográficas arbitrárias ou garantia metrológica.
