# Início do desenvolvimento — E1 / MVP 00

## Análise
O starter continha documentos e um workspace vazio. O produto é uma plataforma
3D local-first em Rust. O primeiro ciclo trata de nuvens de pontos, sem UI ou
reconstrução fotogramétrica. O escopo total de E1 permanece em E1-MVP00.md.

## Plano do primeiro incremento
Criar crates geometry/core, testes e ADR numérico; atualizar Cargo.toml e README.
Aceite: identidade, translação/rotação/escala conhecida, preservação de origem,
rejeição de NaN/Inf/overflow e referências consistentes. Riscos: a API ainda é
inicial e não oferece rotação arbitrária nem persistência.

## Implementado
Tipos de nuvem e cores validados; unidades explícitas; transformação de instância;
projetos, objetos e assets com IDs distintos e validação de referências.
Sem dependências de terceiros em runtime. Dados dos testes são sintéticos,
criados diretamente neste projeto, sem capturas externas.

## Próximos incrementos à época do primeiro incremento
1. Rotação 3D e composição, incluindo testes numéricos.
2. ADR de persistência/schema/IDs e crate storage com round-trip.
3. Parser PLY ASCII XYZ/RGB com limites de tamanho e rejeição explícita.
4. CLI, dataset versionado e demo import/transform/inspect.
5. Gates de entrega E1, revisão de licenças e aceite antes de E2.

Registro histórico do primeiro incremento. O estado atual está em docs/quality/E1-REPORT.md.

## Evidências de validação — 2026-11-09
Ambiente Linux, Rust/Cargo 1.99.1, toolchain isolada no workspace (sem instalação
global). Código entregue em ZIP; nenhum commit foi criado.

- `cargo fmt --check`: passou.
- `cargo clippy --workspace --all-targets -- -D warnings`: passou.
- `cargo test --workspace`: passou, 3 testes unitários, 0 falhas.

Demo reproduzível com Rust estável instalado:

```sh
cargo test --workspace
cargo test -p noah-geometry identity_and_known_transform_preserve_source
```

O segundo comando verifica o ponto (1,0,2), escala 2, rotação Z de 90 graus e
translação (3,4,5), com resultado (3,6,9), preservando origem e RGB.
Ainda não existem testes de integração entre CLI e armazenamento: essas camadas
não foram implementadas neste incremento.

## Plano de continuação
Alterar geometry (rotação 3D/composição), core (transformação de objeto), workspace;
criar storage (DTOs, PLY, assets, validação/atomicidade), CLI, dataset e testes E2E.
Aceite: fluxo create/import/transform/inspect/export/validate, reabertura exata,
Unknown preservado, entradas inválidas rejeitadas sem alterar projeto ou origem.
Riscos: formato inicial sem migração; um escritor por projeto; arquivos órfãos
podem restar após interrupção. ADR 0002 define limites e contratos.
