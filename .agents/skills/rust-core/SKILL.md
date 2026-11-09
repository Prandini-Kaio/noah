---
name: rust-core
description: Implementa componentes Rust do núcleo geométrico, CLI, parser e persistência com testes, isolamento de dependências e APIs pequenas.
---
# Skill: Rust core

- Priorize safe Rust, erros tipados em bibliotecas, mensagens amigáveis na CLI.
- `core` não depende de `storage` ou `cli`; `geometry` não faz I/O.
- Escreva testes unitários e de integração junto com cada função relevante.
- Não comprometa o formato de persistência sem schema versionado e teste round-trip.
- Rejeite arquivos PLY incompatíveis explicitamente; limite consumo de memória e contagem de vértices.
- Rode fmt, clippy e testes; registre resultados sem inventar evidência.
