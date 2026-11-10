# ADR 0007 — Arctic, animação de apresentação e réguas
Status: Accepted
Data: 2026-11-10

Aplicar cores Arctic fornecidas pelo usuário; Dark padrão, Light alternativo.
Tema e opções visuais pertencem à sessão, não ao domínio geométrico. Sessão 4
lê 1–3 com padrões. Câmera ganha projeção Perspective/Orthographic com default
compatível. Réguas em U/V da câmera são relativas à origem do mundo projetada no
plano do alvo: fator uniforme em ortográfica; em perspectiva, apenas naquele
plano (nunca afirmar escala universal para profundidades diferentes). Sem escala,
rotular unidades da cena; referência desatualizada não usa mm/m.

Explosão interpola somente apresentação com easing entre estados, mantendo
Scene.explosion no destino para persistência/exportação. Renderizar cópia leve
com geometria Arc compartilhada. Nenhuma malha é quebrada. Câmera de enquadramento
também interpola. Interromper navegação congela a câmera visual e encerra a
transição; modo de movimento reduzido salta ao destino. Réguas/medidas são
calculadas com a câmera efetivamente exibida, valores sempre no estado montado.

Seleção precisa: snap ajustável, filtro da peça/grupo selecionado, pré-visualização,
proteção contra triângulos que ocultam vértices e entrada explícita por índice.
Pontos em nuvens não definem superfície oclusora. Teste de oclusão usa geometria
triangular completa, dois lados e f64, com tolerância numérica; não é medição de
superfície interpolada. Casas decimais são formatação, não garantia física.
