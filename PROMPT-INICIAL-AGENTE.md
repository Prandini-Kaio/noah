# Prompt inicial — cole no seu agente de programação

Você está iniciando o repositório **NOAH 2.0**. Sua missão imediata é executar **somente a entrega E1 / MVP 00**, preparando um núcleo Rust pequeno e testável de objetos e nuvens de pontos. O README descreve o produto futuro, mas **não autoriza implementar os MVPs futuros**.

**Antes de programar:** leia `AGENTS.md`, siga o índice em `docs/context/INDEX.md`, leia `docs/roadmap/E1-MVP00.md`, `docs/context/DOMAIN.md`, `docs/architecture/BOUNDARIES.md`, `docs/quality/QUALITY.md`; invoque as skills `project-context`, `rust-core`, `geometry-metrology` e `delivery-gates` apenas conforme necessidade.

1. Audite o ambiente e os arquivos do repo. Não presuma que dependências estão instaladas.
2. Produza um plano curto com arquivos a criar, decisões pendentes, sequência de implementação e testes de aceitação; adote escolhas mínimas e reversíveis.
3. Registre ADRs de convenções de coordenadas, unidades, schema e persistência.
4. Construa workspace Cargo, núcleo `Project`/`Object`/`GeometryAsset`, `PointCloud`, transformações, importação PLY do subconjunto declarado, persistência e CLI de demonstração.
5. Garanta dados brutos imutáveis e unidade desconhecida quando não informada.
6. Crie dataset sintético e testes de importação, transformação, round-trip, falhas e integridade.
7. Execute comandos reais de validação e reporte resultados; atualize README quando o comportamento real diferir do planejado.
8. Encerre E1 com documentação da demo, pendências e sugestão de E2. **Não implemente E2 sem aprovação.**

Prefira clareza e código pequeno a abstrações genéricas. Nunca alegue validação ou precisão métrica que não foi medida.
