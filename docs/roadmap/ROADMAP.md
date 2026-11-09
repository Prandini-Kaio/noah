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
