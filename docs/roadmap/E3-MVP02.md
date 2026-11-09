# E3 / MVP 02 — Workspace visual

Autorização: «Vamos para E3», 2026-11-09. E2 sparse é base validada; a validação
real dense continua pendente de CUDA. Esse pedido autoriza E3 com contratos
estáveis, sem fingir que a pendência de E2 desapareceu.

## Plano antes das alterações
Criar crates/workspace (cena, câmera, seleção, sessão) e apps/desktop (egui/eframe).
Adicionar TriangleMesh em geometry e parser OBJ triangular restrito em storage.
Atualizar workspace Cargo, ADR 0004, README, fronteiras, testes e relatório E3.

## Capacidades
Abrir projeto E1, run E2 esparso, PLY ASCII E1 e OBJ triangular. Navegar com
órbita/pan/zoom e enquadrar cena/seleção. Selecionar objeto por clique/lista,
ocultar e alterar translação/rotação/escala de instância. Salvar sessão em arquivo
novo, reabrir preservando câmera, seleção, visibilidade, transformações e origem.

## Aceite
Testes de projeção/seleção/transformação, parser malha, round-trip de sessão,
hash/arquivo alterado, origem intacta e falha sem descartar cena existente.
Executar fmt/clippy/testes e demo real da janela com nuvem E2/mesh sintético.

## Limites/riscos
Renderer inicial projeta em f64 na CPU e envia primitivas egui via OpenGL; ordena
por profundidade, sem z-buffer 3D. Não é visualizador para milhões de elementos
sem redução: limitar display a 50 mil pontos e 20 mil triângulos por cena, sem
alterar assets. Faces que se cruzam podem ter oclusão aproximada; sem CAD,
edição topológica, medidas, montagem, GPU compute ou LOD espacial nesta E3.
Não importar binário dense PLY automaticamente. Validação gráfica depende de
um display/OpenGL disponível; registrar bloqueios concretos, sem alegar UI testada.
