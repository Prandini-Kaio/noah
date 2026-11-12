# Continuação E4 — ângulos e dimensões
Autorização: pedido de continuar o desenvolvimento das entregas em 2026-11-10.
Próximo incremento reversível escolhido: completar ferramentas de inspeção E4
antes de avançar E6/E7/E8. Não declara E4/E5 completas.

Arquivos: workspace/inspection.rs (ângulos e caixas XYZ), lib.rs, session.rs e
measurement.rs (validação e relatório), desktop/metrology.rs e integração no
viewport/inspetor; testes de inspeção, docs e ADR de sessão 5/relatório 2.
Aceite: ângulo ABC no vértice B, 0–180 graus; rejeitar braços nulos; caixa alinhada
aos eixos mundiais da peça/grupo no estado montado, todos os vértices/filhos mesmo
ocultos; nenhum dado amostrado; calibração explícita para dimensões físicas;
medidas/anotações persistidas, sessão 5 lê 1–4; relatório reprodutível com estado,
método, transformações e incerteza desconhecida. Verificação numérica e nativa.
Riscos/limites: caixa XYZ não é caixa orientada mínima; escala desatualizada não
produz mm/m; ângulos independem de escala uniforme; geometria não é alterada.
Sem área/volume, CAD, nova primitiva, alinhamento guiado ou nova captura nesta entrega.
