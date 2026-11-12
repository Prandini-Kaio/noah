# ADR 0010 — Alinhamento translacional guiado e kit local
Status: aceito no pedido explícito do usuário, 2026-11-12.

Primeiro alinhamento E5 usa um par de âncoras de peças distintas no estado montado.
Converte delta mundial pela inversa da rotação/escala dos pais e altera somente a
translação local da instância. Plano é validado antes da prévia; aplicação e undo
verificam contexto de geometria/transformações/hierarquia. Referências Arc mantidas
no contexto evitam reutilização acidental de identidade de memória. Não são
persistidos plano/undo; o ajuste aplicado usa Placement existente na sessão 5.

Arame legado volta a remover faces, enquanto sobreposição é opção distinta e
transitória. Estilhaços usam subdivisão adaptativa (até 64 triângulos/face por
padrão em malhas pequenas), maior impulso e câmera opcional. Limite de 20k preservado.

Kit de 12 imagens reutiliza os hashes E2 publicados e inclui resultado histórico
explicitamente pré-processado. Malhas de alinhamento são dados sintéticos separados,
sem fingir ser resultado do SfM. Pacote encontra exemplos ao lado do executável;
em desenvolvimento pode resolver a pasta de exemplos do repositório. Sem download
ou execução de motor implícita, sem nova dependência ou schema de domínio.
