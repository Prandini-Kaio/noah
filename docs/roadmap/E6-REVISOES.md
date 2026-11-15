# E6 inicial — novas fotos e comparação, 2026-11-15
Autorização: usuário pediu resumo das pendências e execução da próxima etapa.
Próxima etapa selecionada: refinamento com nova captura e revisão comparável.

Plano antes de modificar:
- pipeline: snapshot de lista explícita, limites/hashes/não sobrescrita existentes.
- photogrammetry/refinement: pai validado + fotos adicionais, deduplicação por SHA,
  novo run independente, provenance de entradas e pai, reprocessamento SfM completo.
- geometry/registration: similaridade positiva por correspondências não colineares,
  rotação própria, residual; vizinho mais próximo para comparação geométrica.
- photogrammetry/comparison: câmeras comuns por SHA da foto, registro de novo→pai,
  estatísticas bidirecionais/amostragem/limiar declarado e relatório versionado.
  Residual alto ou câmeras insuficientes não autoriza comparação geométrica.
- desktop: configurar revisão com novas fotos no fluxo, worker e cancelamento;
  comparar runs pelo UI, contagens, antes/depois/sobreposição, exportar relatório.
- CLI, kit base 10 + 2 novas (mesmas 12 imagens sintéticas conhecidas), docs/ADR,
  testes geométricos e de adulteração, pipeline COLMAP real e interface nativa.

Aceite: original intacto; novas imagens únicas realmente incorporadas; pai e
inputs rastreados; novo run reabrível; alinhamento conhecido validado e degenerado
rejeitado; comparação sem inferir precisão física; fmt/clippy/test/build/pacote.
Limites: não incremental (reprocessa fotos), não herda limpeza/escala/transformações
do workspace; não garante ganho de qualidade. Sem textura/GPU real/CAD/ICP geral.
