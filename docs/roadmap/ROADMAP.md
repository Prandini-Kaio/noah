# Roadmap orientado a entregáveis

**Regra:** cada entrega só começa após aceite da anterior (ou autorização explícita de trabalho paralelo com interface estável). Não há compromisso de prazo.

| Entrega | Dependência | Demo objetiva | Fora de escopo |
|---|---|---|---|
| E1/MVP00 Fundação | nenhuma | importar PLY conhecido, transformar, persistir, reabrir, verificar | SfM/UI |
| E2/MVP01 Fotogrametria | E1 | fotos→câmeras/points sparse+dense, status, logs, export | implementação própria de todos algoritmos |
| E3/MVP02 Workspace | E2 | ver pontos/malha, câmera orbit/pan/zoom, seleção e transformações | CAD |
| E4/MVP03 Medições | E3 | definir escala, medir, anotar, criar primitiva | precisão industrial garantida |
| E5/MVP04 Montagens | E4 | objetos hierárquicos, alinhamento manual/guiado, explosão | inferir peças ocultas |
| E6/MVP05 Refinamento | E5 | adicionar fotos e produzir nova revisão comparável | sempre incremental sem reprocessar |
| E7/MVP06 Multissensor | E6 | importar LiDAR/RGB-D, registro e comparação/fusão | compatibilidade universal imediata |
| E8/MVP07 Engenharia | E7 | sólidos CAD e restrições, verificações | certificação sem ensaios |

## Gates obrigatórios
- Demonstração repetível com comandos e dados.
- Testes automatizados e relatório de falhas.
- Docs sincronizadas ao comportamento real.
- Limitações/medidas de desempenho comunicadas sem extrapolação.
- Nada das próximas etapas implementado de modo especulativo.

## Marco de produto
E3 é o primeiro app visual demonstrável; E2 já tem valor como ferramenta CLI de reconstrução. E1 garante os contratos que tornam E2 viável.


## Estado atual — 2026-11-09
E2 autorizada pelo usuário após a entrega de E1. Sparse CPU executado com COLMAP
real; dense implementado/testado com motor controlado, validação real bloqueada
pela ausência de CUDA neste ambiente. E2 não tem aceite completo declarado.
E3 permanece não iniciada. Evidências em ../quality/E2-REPORT.md.

## Atualização — 2026-11-10
Pedido do usuário autoriza E3 integrada e recorte E5 de montagem/explosão manual.
Operações principais estão na interface; E4 não foi iniciada. Não se declara
E5 completa: CAD/restrições, alinhamento guiado e instâncias repetidas continuam
fora desta prévia. Relatório: ../quality/INTERFACE-MONTAGENS-REPORT.md.

## Continuação — E4 inicial
Distâncias entre vértices, referência de escala explícita e anotações persistidas
implementadas na interface. E4 completa (outras primitivas/anotações) permanece
pendente; E5 ainda manual. Relatório: ../quality/E4-REPORT.md.

## Arctic — interface e controle de medição
Paletas do usuário, Dark padrão, explosão animada, modos de projeção e réguas
calibradas implementados. Snap frontal/índice e preferências entram na sessão 4.
Não muda o status metrológico de E4 nem implementa fratura física/CAD.
Relatório: ../quality/ARCTIC-REPORT.md.

## Incremento autorizado: estilhaços e layout
Implementados estilhaços temporários multidirecionais de superfície (malha única),
playback reversível, força/tempo, painéis redimensionáveis/ocultáveis, árvore com
busca, módulos e inspetor contextual. Sem fratura volumétrica, cortes ou colisões.
Guia: docs/desktop/ESTILHACOS-LAYOUT.md. Plano: ESTILHACOS-LAYOUT.md.

## Estado consolidado — 2026-11-12
E1 validada; E2 sparse CPU validada, dense/CUDA real pendente. E3 desktop entregue
em prévias Linux. E4 agora inclui distâncias, ângulos ABC, caixas XYZ, calibração,
anotações e relatório 2; criação de primitivas e validação física permanecem fora
deste incremento. E5 tem hierarquia manual/explosão e efeito de estilhaços visual;
alinhamento guiado e instâncias repetidas ainda pendentes. E6/E7/E8 não iniciadas.
PRs Arctic/estilhaços integrados no ramo principal em 2026-11-11.
Continuação E4: plano E4-ANGULOS-DIMENSOES.md, relatório ../quality/E4-INSPECAO-REPORT.md.

## Ajustes visuais e E5 inicial — 2026-11-12
Autorizados estilhaços menores/dispersão maior, retorno do arame e kit de pipeline.
Alinhamento translacional por vértices, prévia e undo guardado implementados; sem
rotação automática/ICP. Imagens sintéticas e malhas de teste incluídas no pacote.
Pipeline esparso real revalidado pela CLI e GUI com 12/12 imagens; dense/CUDA segue
pendente. Plano E5-ALINHAMENTO-KIT.md; relatório ../quality/E5-KIT-REPORT.md.

## Continuação — 2026-11-13: superfície E2
Fotos → superfície Delaunay CPU e derivação de run sem repetir SfM validadas
com COLMAP real e interface. OBJ/PLY, proveniência, sessão 6 e relatório 3.
67 testes. Poisson CPU verificado isoladamente com normais analíticas; cadeia
CUDA a partir de fotos continua sem validação real. Textura UV, qualidade de
superfície, CAD e novos MVPs permanecem pendentes. Ver E2-MALHA-REPORT.md.

## Preparação e limpeza — 2026-11-13
Escopo autorizado: auditar/selecionar/redimensionar fotografias, editar nuvem,
prévia de SOR/raio e revisões imutáveis. Próxima etapa autorizada interpretada como
superfície da revisão sparse limpa, implementada sem repetir SfM. Incremento E2/E3,
base para refinamento, sem declarar toda E6 (novas capturas/comparação) concluída.
Sessão 7, relatório 4. Máscaras fotográficas e normalização fotométrica entram no
backlog explícito; textura, CUDA real, CAD e novos sensores seguem pendentes.
Plano PREPARO-LIMPEZA.md; evidências ../quality/PREPARO-LIMPEZA-REPORT.md.
