# Arctic — interface, animação e auxílio à medição
Pedido do usuário: paletas Arctic Light/Dark (Dark padrão), animação de explosão,
medição mais controlada, opções visuais e réguas baseadas na escala.

Plano: módulos de tema/animação/auxiliares no desktop; projeção ortográfica e
opções visuais no workspace; réguas e seleção de vértice com controle de oclusão;
sessão 4 lendo 1/2/3; testes numéricos, animação e compatibilidade; validação da
janela e nova prévia Linux. Não modificar geometria original nem escalas físicas.

Aceite: duas paletas conforme especificação; explosão com easing reversível e
movimento reduzido; temas/opções persistidos; régua uniforme em ortográfica e
referência explícita de plano-alvo em perspectiva; escala desatualizada não rotula
régua como física; snap/índice e coordenadas verificáveis. Testes/lint e demo real.

Não objetivos: fraturar/segmentar malha, física de fragmentos, precisão industrial,
renderer GPU com z-buffer ou novos motores de captura.
