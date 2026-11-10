# ADR 0005 — Montagens de cena, explosão e operação pela interface
Status: Accepted
Data: 2026-11-10

Contexto: usuário solicita interface coesa e montagem/explosão, ampliando E3 com
recorte E5. Sessões anteriores referenciam fontes verificadas e ajustes locais.

Decisão: grupos Assembly com ID estável, nome, pai opcional e Placement; cada item
pode apontar a um grupo. Transformação mundial = ancestrais * ajuste * base.
Profundidade máxima 64, sem ciclos, pais ausentes, IDs duplicados ou coordenadas
mundiais fora dos limites. Alterações de hierarquia preservam os valores locais;
atribuir outro pai pode mudar posição mundial (explicitar no inspetor).

Explosão é deslocamento de visualização em coordenadas mundiais, após compor
montagem. Direção radial pelo centro da caixa de cada peça, ou vetor explícito;
centros coincidentes usam eixos determinísticos. Não reescreve Placement, base ou
geometria. Retornar intensidade a zero restaura exatamente a composição montada.
Sessão schema 2 registra grupos, pais, intensidade/distância/vetores; lê schema 1
com padrões sem grupo e explosão zero. Paths e hashes de origem continuam verificados.

App usa explorador egui embutido para arquivos/pastas e destino de gravação, sem
depender de zenity/portais do SO. Reconstrução chama adaptador existente em worker,
mostra etapas/logs locais e permite cancelamento pelo protocolo CANCEL. Não usa
shell para iniciar motor. Nenhum binário ou sensor novo é baixado pelo app.

Exemplo multiparte é geometria sintética de demonstração, sem alegar objeto
reconstruído ou conjunto mecanicamente validado. Renderer mantém limites E3.
