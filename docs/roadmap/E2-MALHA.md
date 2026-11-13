# Continuação E2: fotos/nuvem → superfície
Autorização do usuário em 2026-11-13 após esclarecer ausência de geração de malha.
Plano: parser PLY COLMAP em storage, normalização OBJ, Delaunay sparse CPU e Poisson
após dense no adapter; derivar malha de run existente em diretório novo sem repetir
SfM; CLI e GUI com modos e abertura de superfície/nuvens. Textura UV fica pendente.
Arquivos: storage/colmap_ply.rs, photogrammetry/lib.rs + surface.rs, workspace loader,
desktop/jobs/main/shell, CLI, testes, guias e ADR. Sem nova dependência.
Aceite: pipeline real CPU das fotos até OBJ com faces, derivação rastreável e fontes
intactas, sessão/reabertura/exportação e janela inspecionadas; parser rejeita índices,
formatos/contagens inválidos; falha/CUDA mantém sparse disponível; fmt/clippy/test.
Limites: superfície rápida de sparse pode ter buracos/artefatos; não é sólido CAD,
volume ou precisão garantida. Dense exige CUDA/GPU; separar teste controlado de real.
