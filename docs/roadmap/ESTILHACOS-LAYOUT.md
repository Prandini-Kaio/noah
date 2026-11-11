# Estilhaços temporários e organização do workspace
Pedido autorizado em 2026-11-10. A especificação visual anexada é referência de
produto; sugestões de novas ferramentas não implicam implementar CAD, seções ou
metrologia de ângulo/área/volume nesta entrega.

Plano: criar blast.rs (fragmentos visuais determinísticos, orçamento limitado,
rotação/impulso, playback reversível); shell.rs (navegação, painéis contextuais,
árvore pesquisável, toolbar e status); adaptar main.rs para renderização temporária.
Manter geometria, exportação, medidas e schema 4 intactos. Documentar contratos.
Aceite: malha única se estilhaça; pausa/scrub/restaurar; componentes continuam
separáveis; nuvem usa dispersão de pontos, sem inventar superfície; painéis
redimensionáveis/ocultáveis, busca, navegação e ações reais; fmt/clippy/testes e
validação nativa Linux. Riscos: não há fratura volumétrica nem física de colisão;
renderização continua amostrada/ordenada por profundidade, custo limitado.
