# Instruções para agentes — NOAH 2.0

## Missão e limites
Desenvolva uma plataforma Rust local-first para reconstrução e manipulação 3D; no ciclo atual implemente **ajustes de estilhaços/wireframe, alinhamento guiado inicial E5 e kit de teste do pipeline**, autorizados diretamente pelo usuário em 2026-11-12; E2 dense continua pendente. A fotogrametria será o primeiro motor de captura em E2; LiDAR e CAD são expansões futuras, não dependências atuais.

## Leitura obrigatória (nesta ordem)
1. `README.md` (visão e roadmap geral)
2. `docs/context/INDEX.md` (roteamento de contextos)
3. `docs/context/DOMAIN.md` (invariantes)
4. `docs/architecture/BOUNDARIES.md` (contratos)
5. `docs/roadmap/E5-ALINHAMENTO-KIT.md` (escopo autorizado atual; E1/E2 são a base)
6. `docs/quality/QUALITY.md` (critério de aceite)

## Regras operacionais
- **Sem expansão de escopo**: não criar editor, renderizador, integração COLMAP, LiDAR, banco ou plugins vazios em E1.
- **Planeje antes de modificar**: liste arquivos a criar/mudar, testes de aceitação e riscos em um plano curto.
- **Interfaces pequenas** e dependências em uma direção: domínio não importa adapters nem UI.
- **Sem inventar precisão**: escala física, medição, erro e incerteza têm significados distintos.
- **Dados brutos preservados**; alterações criam derivados/versionamentos; nunca sobrescrever silenciosamente.
- **Coordenadas/unidades explícitas**; não supor milímetros ou metros por padrão ao ler dados externos sem metadados.
- **Entrada é não confiável**: conferir limites, parser, caminhos, arquivos grandes e erros de execução.
- **Testar a cada mudança**; nunca declarar `cargo test`/`clippy` como executado se não foi.
- **Não instalar ferramentas globalmente** ou usar serviços remotos sem autorização.
- **Atualizar docs de contrato** e adicionar ADR para decisão estrutural irreversível.
- **Não executar próximo MVP** sem aprovação; ao concluir produzir relatório com evidências e pendências.

## Skills locais — carregamento por necessidade
- `.agents/skills/project-context/SKILL.md`: início de tarefa, escopo e registro de decisões.
- `.agents/skills/rust-core/SKILL.md`: alterações em crates, CLI, serialização e testes Rust.
- `.agents/skills/geometry-metrology/SKILL.md`: geometria, coordenadas, unidades, calibração e medição.
- `.agents/skills/pipeline-adapters/SKILL.md`: **E2 atual**, processos e novos sensores.
- `.agents/skills/delivery-gates/SKILL.md`: revisão e aceite de entregas.

## Formato obrigatório de reporte
1. Objetivo e MVP em execução.
2. Arquivos alterados e decisões técnicas.
3. Testes executados com comandos e resultados reais.
4. Demo reproduzível (comandos e dados).
5. Limitações, riscos e próximos passos **sem executar**.

Se houver ambiguidade não bloqueante, faça escolha mínima reversível, documente e prossiga. Pergunte apenas quando a decisão for incompatível com os objetivos ou irreversível.
