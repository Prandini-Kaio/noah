# ADR 0014 — revisões de captura e comparação rastreável
Data: 2026-11-15. Estado: implementado na prévia E6 inicial.

Uma nova captura gera outro run independente por reprocessamento SfM completo.
Entradas anteriores e adicionais são deduplicadas por SHA-256 e copiadas sob os
limites existentes; parent-run.json e refinement.json registram origem e hashes.
Não se transportam automaticamente limpeza, calibração ou transformações.

Comparação usa câmeras comuns identificadas pelos hashes das imagens, não nomes.
Similaridade de escala positiva alinha novo→anterior, exige quatro correspondências
não colineares e rejeita residual normalizado >5%. Distâncias de vizinho próximo
são bidirecionais, amostradas deterministicamente e sem interpretação metrológica.
Relatório próprio schema 1 contém hashes, parâmetros e limites; reabertura verifica
fontes e recalcula. A prévia usa uma cena descartável e restaura a câmera original.
Sessão 8 e relatório geométrico 4 não mudam. Sem dependências novas.
