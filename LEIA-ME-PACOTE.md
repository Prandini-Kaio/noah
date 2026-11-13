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

Atualização 2026-11-13: superfície CPU implementada e validada com COLMAP real;
67 testes passaram. Interface inclui derivação de run e malha pronta do kit.
Pacote Linux compilado disponível nas Releases (prévia v0.8); fontes e binário
são downloads distintos. Sessão 6 lê 1–5, relatório 3. Consulte README.md e
docs/quality/E2-MALHA-REPORT.md para o estado atual; registros acima são históricos.

Atualização v0.9: preparação/seleção de imagens, limpeza de nuvem e superfície da
revisão limpa. Sessão 7 lê 1–6; relatório 4. Guia em docs/desktop/PREPARO-LIMPEZA.md;
relatório atual em docs/quality/PREPARO-LIMPEZA-REPORT.md.

Atualização v0.10: fluxo guiado, logs/cópia, pastas independentes e desmontagem
por camadas/armadura/painéis. Sessão 8 lê 1–7; relatório 4. Guia atual em
docs/desktop/UX-GUIA.md e evidências em docs/quality/UX-FLUXO-REPORT.md.
