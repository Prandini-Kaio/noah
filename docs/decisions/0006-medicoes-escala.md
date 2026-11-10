# ADR 0006 — Distâncias ancoradas e escala explícita
Status: Accepted
Data: 2026-11-10

Âncora referencia ID estável da peça e índice de vértice original. Distâncias
são euclidianas no estado montado (hierarquia e ajustes), em f64, sem aplicar
explosão. A visualização desenha linhas sobre posições explodidas quando ativas,
mas identifica o valor como montado. Seleção faz snap de vértice próximo ao cursor;
não mede um ponto arbitrário interpolado sobre a face nem resolve oclusão exata.

Uma referência global de escala registra duas âncoras, distância física declarada,
unidade m/mm, comprimento da cena no momento da declaração e procedência. Fator
é físico/cena; não altera pontos, Placement ou unidades dos assets. Se a distância
da referência mudar além de tolerância relativa 1e-9, escala fica desatualizada;
não se usa fator antigo silenciosamente. Movimento rígido que preserva o
comprimento mantém validade. Precisão da seleção não é incerteza metrológica:
incerteza fica None (desconhecida) e método é explicitado.

Medições guardam IDs, nomes, âncoras, fonte/método e incerteza None. Valor é
recalculado na abertura, nunca confiado a um número salvo sem validar geometria.
Sem escala, informar unidades da cena; não inferir mm/metros dos assets mistos.
Sessão schema 3, compatível com 1/2 sem calibração/medidas. Até 1000 medidas,
nomes/procedência limitados; âncoras/IDs/schema/tolerâncias validados ao carregar.
