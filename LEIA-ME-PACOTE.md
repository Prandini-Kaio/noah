# NOAH 2.0 — snapshot E1/E2/E3

Pacote de código-fonte solicitado em 2026-11-09.
E1 validada; E2 esparsa validada com COLMAP real; E2 densa depende de CUDA.
E3 implementada e compilada, com testes de domínio/sessão e inspeção inicial
da janela; testes interativos completos ainda pendentes.

Requer Rust estável (validado com 1.99.1), linker C e, para a interface,
display X11 e OpenGL. Cargo baixa dependências na primeira compilação.

```sh
cargo test --workspace --locked
cargo run --locked -p noah-desktop -- examples/e2-sparse/sparse.ply
cargo run --locked -p noah-desktop -- tests/datasets/reference-mesh.obj
cargo run --locked -p noah-cli -- --help
```

Na interface: arrastar para órbita, direito/Shift para pan, roda para zoom,
F para enquadrar. Seleção por clique ou lista. Ajustes são da sessão e preservam
fontes. Cada gravação exige arquivo novo. Relatórios em docs/quality/.
O README original de E1/E2 deve ser lido junto ao relatório E3, que atualiza o
estado mais recente. Não há binários, instalação do COLMAP ou toolchain no ZIP.
