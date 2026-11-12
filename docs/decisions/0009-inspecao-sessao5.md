# ADR 0009 — Inspeção E4 e sessão 5 / relatório 2
Status: aceito na continuação autorizada de E4, 2026-11-12.

Mantêm-se as distâncias anteriores sem alterar seu DTO. AngularMeasurement guarda
ABC (B central); DimensionMeasurement guarda a identidade da peça/grupo. Ambas
são definições vivas, recalculadas sobre a geometria original no estado montado.
Sessão 5 acrescenta listas default vazias, lendo versões 1–4. Apps anteriores
recusam versão 5. IDs são únicos entre tipos; métodos/âncoras/limites são validados.

Caixas são AABB mundial XYZ sobre todos os vértices/descendentes, mesmo ocultos;
não são caixas mínimas orientadas. Ângulos usam atan2 do produto vetorial/escalar
normalizado, sem depender de calibração. Dimensões usam o contrato de escala E4.
O cache de apresentação invalida geometria/transformações/hierarquia; não participa
da persistência nem altera resultados do domínio.

Relatório 2 mantém distâncias e acrescenta ângulos, dimensões, transformações das
instâncias e hierarquia para reprodução. Escala inválida continua impedindo export
completo. Nenhuma nova dependência, sensor, precisão industrial ou schema E1/E2.
