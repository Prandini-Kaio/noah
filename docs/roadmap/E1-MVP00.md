# E1 / MVP 00 — Plano operacional aprovado

**Objetivo:** construir núcleo mínimo em Rust para manipulação persistente de nuvens de pontos com unidades explícitas, coordenadas e identidade.

## Entregas obrigatórias
1. Workspace Cargo com crates mínimas `core`, `geometry`, `storage` e app `cli`; evitar crates vazias adicionais.
2. `Project` com identidade, nome, versão de schema e coleção de objetos/assets.
3. `PointCloud` com XYZ e cor opcional; importação PLY com subconjunto suportado explicitamente definido (por exemplo ASCII XYZ e RGB opcional). Rejeitar variações não suportadas com erro útil; não fingir suporte PLY universal.
4. Transformação de instância (translação, rotação e escala controlada); coordenadas originais mantidas.
5. `UnitState` explícito: desconhecido ou unidade informada; não aplicar escala automaticamente.
6. Persistência e reabertura do projeto; validação de schema e tratamento de arquivo corrompido.
7. CLI mínima para criar projeto, importar PLY, aplicar transform, inspecionar e exportar/validar (o conjunto exato de comandos pode ser refinado sem mudar a capacidade demonstrável).
8. Dataset sintético pequeno, gerado no repo, com geometria e valores conhecidos.
9. Testes unitários, de integração e execução end-to-end local documentados.

## Sequência sugerida
- T0: registrar ADR de eixos, transformações, unidades e persistência.
- T1: workspace + tipos de domínio + testes de identidade.
- T2: operações e invariantes de geometria + testes numéricos.
- T3: parser PLY seguro + casos válidos, inválidos e extremos limitados.
- T4: armazenamento e round-trip + teste com schema.
- T5: CLI + demonstração ponta-a-ponta.
- T6: revisão de docs, formatação, lint, testes e relatório de entrega.

## Aceite verificável
- Um pequeno PLY com N pontos e valores conhecidos importa preservando N e posições/cores (se suportadas).
- Transformação conhecida resulta nas coordenadas esperadas, dentro de tolerância numérica justificada.
- Salvar/reabrir preserva IDs, nomes, unidade, referencial e transformações.
- Unidade ausente permanece Unknown após round-trip.
- Arquivo malformado falha claramente sem corromper projeto existente.
- Operações não reescrevem a captura original.
- CLI imprime resultado útil e códigos de saída não zero em erros.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` passam ou falhas são reportadas sem ocultação.

## Não implementar
COLMAP, SfM, MVS, UI/desktop, medição dimensional avançada, LiDAR, CAD, multiplayer, rede, plugins genéricos, otimização GPU.

## Relatório final
Escopo realizado; comandos de demo; testes e resultados; ADRs; limitações; proposta de E2 (somente plano, não implementação).
