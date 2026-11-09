# ADR 0001 — Fundação numérica inicial
Status: Superseded parcialmente por ADR 0002 (rotação 3D, persistência e IDs)
Data: 2026-11-09

Contexto: início de E1, sem código de captura, UI ou persistência existente.

Decisão: usar f64, XYZ destro com Z para cima e vetores coluna.
Transformações de instância aplicam escala uniforme positiva, rotação e translação
nessa ordem; a primeira API oferece rotação apenas sobre Z. Valores não finitos e
resultados com overflow são rejeitados. Pontos originais permanecem imutáveis.
Unidade Unknown é o padrão; declarar unidade não converte coordenadas.
IDs têm tipos distintos e valor u128 fornecido pela aplicação; coleções rejeitam
IDs duplicados e objetos sem asset. Nenhum gerador de IDs foi definido ainda.

Alternativas consideradas: f32 (menor memória, menor precisão), escala por eixo
(mais estados inválidos) e mutação direta dos pontos (perde origem).

Consequências / trade-offs: base pequena sem dependências externas. A rotação 3D
completa e composição ainda precisam de uma API e testes antes de concluir E1.
O armazenamento, schema e geração de IDs serão definidos em ADR próprio antes
de implementar persistência; esta decisão não congela um formato de arquivo.

Como validar: testes de identidade, transformação conhecida com tolerância absoluta
1e-12 para valores pequenos, preservação dos pontos, cores, entradas não finitas,
overflow, unicidade de IDs e integridade das referências.
