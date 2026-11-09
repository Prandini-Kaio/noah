# E3 / MVP 02 — estado incluído no ZIP

Data: 2026-11-09. E3 autorizada pelo usuário; pacote solicitado durante desenvolvimento.

Implementado: noah-workspace (cena, câmera perspectiva, seleção e sessões),
noah-desktop (egui/eframe/OpenGL), TriangleMesh e parser OBJ triangular restrito.
Abertura de projeto E1, run E2 esparso, PLY ASCII e OBJ; órbita/pan/zoom,
visibilidade, seleção, ajustes de instância, enquadramento e revisão de sessão.
Salvar não sobrescreve arquivos existentes nem fontes. Fontes têm caminhos
absolutos e hashes; mover arquivos exige reabri-los. Plano e decisões: E3-MVP02.md
e ADR 0004.

Verificado realmente: cargo fmt --check, cargo clippy --workspace --all-targets
-- -D warnings, cargo test --workspace --locked e build noah-desktop passaram.
25 testes: 19 anteriores, 1 OBJ e 5 workspace (câmera/projeção, seleção,
transformação sem alterar fonte, sessão round-trip, schema e fontes corrompidas).
A janela real foi executada em Xorg dummy :99, com renderização Mesa software.
Visualizados run E2 de 7592 pontos e malha de referência; seleção pela lista
realizada por entrada XTEST e inspecionada em captura. Não houve teste visual
completo de cada controle nem de salvar/reabrir pela interface. As confirmações
de fechamento/substituição e scroll do inspetor foram compiladas e verificadas
por lint, mas ainda aguardam teste interativo após a última alteração.

O pacote é snapshot de desenvolvimento de E3; não declarar aceite completo.
Limites: projeção CPU, ordenação por profundidade sem z-buffer, amostra de até
50 mil pontos/20 mil faces; sem CAD, edição topológica ou medições. Parser OBJ
aceita apenas v/f triangulares com índices positivos. Uso inicial validado em
Linux X11; não foi validado em Windows/macOS. Dense E2 continua pendente de CUDA.

Demo sem COLMAP e sem dados externos:

```sh
cargo run --locked -p noah-desktop -- tests/datasets/reference-mesh.obj tests/datasets/reference-rgb.ply
```

Exemplo real E2 esparso incluído em examples/e2-sparse/sparse.ply:

```sh
cargo run --locked -p noah-desktop -- examples/e2-sparse/sparse.ply
```

A demo gráfica requer display X11 e OpenGL. Não são incluídos compilados,
Rust, COLMAP ou o ambiente conda no ZIP; instalar ferramentas separadamente.
