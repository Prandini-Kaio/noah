# E4 / MVP 03 — primeiro incremento de medição
Data: 2026-11-10. Continuação do desenvolvimento solicitada pelo usuário após
prévia de interface/montagens. Escopo: distâncias entre vértices, referência
explícita de escala e anotações persistidas, integradas ao desktop.

Plano antes de editar: criar módulo measurement em workspace e testes; estender
sessão para schema 3 lendo 1/2; controles e desenho de anotações na interface;
relatório, ADR e prévia Linux. Geometria/pontos brutos e projetos E1/E2 não mudam.

Aceite: medir pares conhecidos; calibrar mm/m sem inferência; explosão não muda
valor; transformação que altera referência invalida calibração; persistir e
reabrir IDs/âncoras/nomes/fonte/método/incerteza desconhecida. Testar entradas
inválidas e referências corrompidas. Exercitar seleção e gravação pela janela.

Não objetivos: metrologia industrial, precisão inferida, superfícies curvas,
CAD, segmentação automática, primitivas ajustadas ou reconstrução densa CUDA.
E4 completa ainda incluirá outras primitivas/anotações e refinamentos de UX.
