# NOAH 2.0 — Plataforma de reconstrução e engenharia 3D

> **Nome oficial:** NOAH. **Versão do produto:** 2.0. Projeto local-first e modular, desenvolvido prioritariamente em Rust.
>
> **Versão do projeto:** 2.0. A versão do produto é distinta das versões de schema e de crates.
>
> **Estado:** E1 validada; E2 esparsa validada com COLMAP real, validação densa pendente de CUDA. **Escopo autorizado:** E3 / MVP 02 (pedido do usuário em 2026-11-09).

## Visão

Construir uma plataforma que permita reconstruir objetos físicos a partir de capturas, inspecionar e manipular representações 3D, estabelecer escala e medidas, organizar componentes em montagens, e aprimorar o modelo com novos dados. A fotogrametria é o **primeiro adaptador de aquisição**, não o modelo central do sistema. Entradas futuras incluem LiDAR, RGB-D, medições manuais e geometrias importadas.

### Casos de uso-alvo

1. Fotografar um objeto, obter nuvem esparsa/densa e visualizar a reconstrução.
2. Definir um referencial físico, inspecionar distâncias e preservar a origem da medida.
3. Escanear separadamente componentes de um motor e montá-los em uma hierarquia (com ajustes e restrições explícitas).
4. Adicionar capturas ao objeto existente para comparar ou aperfeiçoar a geometria, com versões reproduzíveis.

**O que NÃO afirmamos:** fotografias isoladas não revelam partes ocultas, uma malha bonita não equivale a um sólido CAD, e escala conhecida não garante exatidão metrológica. A validação metrológica industrial é uma etapa futura.

## Princípios imutáveis

- **Domínio independente da captura:** `Object`/`Assembly` não dependem de COLMAP, LiDAR ou fornecedor.
- **Dados brutos imutáveis:** arquivos de entrada não são alterados por etapas de processamento.
- **Representações coexistem:** um objeto pode referenciar múltiplas nuvens, malhas, sólidos e revisões.
- **Unidades explícitas e rastreáveis:** unidades, sistemas de coordenadas, calibração, fonte e incerteza (quando conhecida) acompanham medidas.
- **Pipeline reexecutável:** parâmetros, versões, entradas, logs e artefatos são rastreados.
- **Local-first:** por padrão não envia dados para serviços externos; dependências externas são executadas localmente.
- **Incremental e verificável:** entregar um MVP operante com testes antes de ampliar escopo.
- **Rust como núcleo:** bindings e processos externos são permitidos onde fizer sentido; não reimplementar algoritmos maduros sem justificativa.

## Arquitetura conceitual

```text
apps/cli  apps/desktop (futuro)
     |       |
     v       v
 core (projetos, objetos, IDs, revisões)
     |-- geometry (pontos, malhas, transformações e referenciais)
     |-- measurement (unidades, referências e proveniência; MVP 03)
     |-- storage (serialização, projetos e artefatos)
     |-- pipeline (jobs, cache, diagnósticos; MVP 01)
     |-- reconstruction (interfaces de processadores; MVP 01)
                 |-- photogrammetry / COLMAP adapter (MVP 01)
                 |-- lidar / RGB-D adapters (MVP 06)
 renderer / editor (MVP 02–03)
```

**Fluxo:** `CaptureSource` → `Processor` → `Artifact` (`GeometryAsset` ou calibração/medida/etc.) → `Object`/`Revision`. Nem toda entrada gera pontos; não criar interface artificial que force isso.

## MVPs e entregas

| Entrega | MVP | Resultado verificável | Estado |
|---|---|---|---|
| E1 | 00 — Foundation | CLI importa PLY de referência, transforma, persiste e reabre preservando dados | **Validada; base de E2** |
| E2 | 01 — Reconstruction | Pipeline fotogramétrico executado localmente; nuvens, câmeras, logs e diagnósticos exportáveis | Esparsa validada; densa pendente de CUDA |
| E3 | 02 — Workspace | App desktop navega e manipula nuvem/malha; salva estado | Planejada |
| E4 | 03 — Measurement | Escala, medições, primitivas e anotações com procedência | Planejada |
| E5 | 04 — Assembly | Hierarquia, alinhamento, ocultação e vista explodida | Planejada |
| E6 | 05 — Refinement | Novas fotos, registro, versões e comparação | Planejada |
| E7 | 06 — Multisensor | LiDAR/RGB-D, registro e fusão com proveniência | Planejada |
| E8 | 07 — Engineering | CAD paramétrico, restrições avançadas e validação | Planejada |

Detalhes, dependências e critérios de passagem: [`docs/roadmap/ROADMAP.md`](docs/roadmap/ROADMAP.md). Plano executável do ciclo atual: [`docs/roadmap/E3-MVP02.md`](docs/roadmap/E3-MVP02.md).

## Estrutura do repositório

```text
AGENTS.md                    # Instruções curtas e obrigatórias para agentes
.agents/skills/              # Skills acionadas conforme a tarefa
README.md                    # Visão humana e contrato do projeto
docs/context/INDEX.md        # Mapa de contexto e ordem de leitura
docs/context/PRODUCT.md      # Objetivo, usuários e limites
docs/context/DOMAIN.md       # Glossário e invariantes
docs/architecture/           # Fronteiras, dados e decisões técnicas
docs/decisions/              # ADRs (decisões registradas)
docs/roadmap/                # Roadmap e escopo por entrega
docs/quality/                # Qualidade, testes e Definition of Done
apps/cli/                    # CLI E1/E2; desktop futuro em E3
crates/                      # core, geometry, storage, pipeline e photogrammetry
tests/datasets/              # Conjuntos sintéticos e licença registrada
```

## Stack e abordagem inicial

- Rust estável e Cargo workspace; builds reproduzíveis com `Cargo.lock` versionado para os executáveis.
- Álgebra linear: avaliar `nalgebra`. CLI: avaliar `clap`. Erros: `thiserror` e/ou `anyhow` conforme fronteiras. Serialização: `serde`.
- E2: COLMAP via processo externo e adaptador isolado. **Não** acoplar formatos COLMAP ao domínio.
- E3: avaliar `egui` + `wgpu` após medir requisitos do visualizador; não escolher UI definitiva na E1.
- SQLite pode entrar quando houver necessidade real de indexação; não obrigar banco no MVP 00.
- Não fixar versões ou garantias de compatibilidade sem validação na implementação.

## Início rápido para um agente

1. Ler `AGENTS.md` → `docs/context/INDEX.md` → `docs/roadmap/E2-MVP01.md`.
2. Confirmar ambiente Rust e motor COLMAP 3.11.x local, sem instalação global automática.
3. Preservar os testes e o schema de E1; adaptar formatos externos apenas em photogrammetry.
4. Executar fmt, clippy, testes e demo real; distinguir sparse CPU de dense CUDA.
5. Consultar [relatório E2](docs/quality/E2-REPORT.md) para evidências e pendências.
6. Não iniciar E3 sem novo pedido do usuário.

## Resultado esperado de E1

Um comando demonstrável como `noah import ...`, `noah transform ...` e `noah inspect ...` (sintaxe final definida na implementação) que consiga preservar uma pequena nuvem PLY, suas coordenadas, identidade de asset e unidade, após gravação e leitura do projeto. E1 **não** exige reconstrução fotogramétrica nem interface gráfica.

## Segurança e licenças

Dados de captura podem conter informação sensível. Nada de uploads silenciosos, telemetria ou execução de comandos derivados diretamente de arquivos não confiáveis. Tratar PLY/OBJ e metadados como entradas não confiáveis, com limites de tamanho e parsing seguro. Verificar licenças de crates, motores e datasets antes da distribuição. Documentar avisos de terceiros.

## Como colaborar

Regras: [`AGENTS.md`](AGENTS.md). Critérios de qualidade: [`docs/quality/QUALITY.md`](docs/quality/QUALITY.md). Decisões: [`docs/decisions/`](docs/decisions/). O roadmap é uma orientação de produto: não interpretar itens futuros como autorização para implementar agora.

## Executar E1

Requer Rust estável (validado com 1.99.1) e linker C. Para a demo, Python 3.10+.

```sh
cargo build --locked
cargo run --locked -p noah-cli -- --help
python3 scripts/demo.py
```

A demo cria diretório temporário novo, importa o dataset RGB, transforma,
reabre, valida hashes e exporta coordenadas conhecidas, preservando a origem.
Para uso manual:

```sh
cargo run --locked -p noah-cli -- create ./meu-projeto "Exemplo"
cargo run --locked -p noah-cli -- import ./meu-projeto tests/datasets/reference-rgb.ply "Peça" --frame z-up
cargo run --locked -p noah-cli -- inspect ./meu-projeto
cargo run --locked -p noah-cli -- validate ./meu-projeto
```

Use o `object_id` impresso por import em `transform` e `export` (ver `--help`).
`transform` substitui a transformação da instância; ângulos em radianos.
`--unit m` ou `--unit mm` declara a unidade, sem converter valores; omitido,
permanece Unknown. `--frame z-up` é uma declaração exigida do usuário.
Exportação PLY aplica a transformação; unidade e referencial precisam acompanhar
separadamente o arquivo exportado. Arquivos existentes nunca são sobrescritos
pela exportação. Um único escritor por projeto é suportado nesta E1.

### PLY suportado e limites

ASCII 1.0, um elemento vertex, XYZ float/double nessa ordem e RGB uchar opcional,
completo e nessa ordem. Comentários são aceitos. Binário, faces, listas, normais e
propriedades adicionais são rejeitados. Limites: 64 MiB/PLY, 1 milhão de pontos
por projeto, 1000 assets/objetos, 4 MiB de metadados, header 64 KiB e linha 4096 bytes.
Não converter eixos nem unidades silenciosamente. Entradas devem ser arquivos
regulares; symlinks internos são rejeitados. Não há proteção contra outro processo
hostil alterando diretórios durante uma operação (TOCTOU); use diretório local confiável.

### Verificação

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
```

Contratos: [ADR 0002](docs/decisions/0002-projeto-e1.md).
Evidências e limitações: [relatório E1](docs/quality/E1-REPORT.md).
Dependências: [licenças](docs/quality/THIRD-PARTY.md).
Histórico: [início do desenvolvimento](docs/roadmap/INICIO-DESENVOLVIMENTO.md).

## Executar E2 — reconstrução por fotos

Requer COLMAP **3.11.x**, instalado separadamente segundo
[as instruções oficiais](https://colmap.github.io/install.html).
NOAH exige o caminho de um executável local confiável; não baixa motores/fotos.
O adaptador usa SIFT/matching em CPU, duas threads, max_image_size=1600,
SIMPLE_RADIAL e matching exaustivo. Outras versões de COLMAP são rejeitadas.

```sh
cargo build --locked
./target/debug/noah reconstruct ./fotos ./novo-run --colmap /caminho/colmap --timeout 600
./target/debug/noah run-inspect ./novo-run
```

`fotos` contém de 3 a 500 JPEG/PNG diretamente no diretório; subdiretórios não
são percorridos. Cada foto tem limite de 32 MiB e o total de 512 MiB.
O run deve ser novo. E2 copia fotos sem alteração e registra SHA-256, argumentos,
parâmetros, versão/caminho/hash do executável invocado e horários UTC em Unix ms.
Se o executável for launcher, o hash é do launcher; o ambiente completo do motor
precisa ser versionado à parte. Snapshots usam nomes normalizados para manter o
parser e o processo independentes dos nomes originais; o manifesto guarda o mapa.

Saídas principais:

```text
novo-run/
  run.json                        # status e proveniência, schema 1
  images/                         # cópias das entradas
  logs/                           # stdout/stderr por etapa
  sparse/                         # modelos nativos preservados
  model-0/                        # export TXT do submodelo (número pode variar)
  exports/sparse.ply              # XYZ f64/RGB ASCII
  exports/reconstruction.json     # câmeras, poses world→camera, IDs, frame/unidade
```

Se houver vários submodelos, NOAH exporta o que registrou mais imagens; todos os
modelos nativos/TXT permanecem disponíveis. Câmeras têm intrínsecos e poses com
quaternion wxyz e translação world→camera. Unidade Unknown e referencial destro
SfM arbitrário: não assumir Z-up nem metros. Export PLY precisa acompanhar o JSON.
Esses runs são independentes do schema E1; não há vínculo automático com objetos.

Cancelamento em outro terminal:

```sh
./target/debug/noah run-cancel ./novo-run
```

Timeout é por etapa (1 a 86400 segundos na CLI), cancelamento por marcador CANCEL.
No Unix, o grupo do processo é encerrado em timeout/cancelamento. Estado terminal,
logs e resultados parciais permanecem para diagnóstico. Um status Running após
interrupção do NOAH não prova que o motor ainda está ativo; não há retomada/cache.
Run-inspect valida hashes das entradas e artefatos registrados; logs e arquivos
intermediários não são autenticados por ele. Usar um escritor e diretório confiável.
Limite de log de 8 MiB por stream é fiscalizado por polling e pode ser ultrapassado
entre inspeções; não existe quota global para database/intermediários do COLMAP.

### Densificação

Acrescentar `--dense` ao comando reconstruct. Etapas: image_undistorter,
patch_match_stereo com consistência geométrica e stereo_fusion. Requer **COLMAP com
CUDA e GPU compatível**; não há fallback CPU implementado. exports/dense.ply mantém
formato binário XYZ/normal/RGB do COLMAP; não passa pelo parser ASCII E1. Envoltório,
contagem/tamanho e valores finitos são validados (máximo de 1 milhão de pontos).

O modo denso teve orquestração/parser testados com motor controlado. A execução
real neste ambiente falhou em PatchMatch por ausência de CUDA, preservando a
reconstrução esparsa e logs. Não está declarado validado de ponta a ponta.

### Demo reproduzível E2

O gerador exige Python, NumPy e Pillow. Usa cenas/texturas sintéticas CC0.
Com essas dependências disponíveis e COLMAP instalado:

```sh
python3 scripts/e2/demo.py --colmap /caminho/colmap --output /caminho/nova-demo
```

Neste workspace, o motor preparado está em
`/workspace/work/colmap-tools/colmap`; a demo final foi salva em
`/workspace/work/e2-demo-final`. Ver [relatório E2](docs/quality/E2-REPORT.md),
[contrato](docs/decisions/0003-pipeline-colmap.md) e
[dataset](tests/datasets/e2/README.md).

## E3 — desktop em desenvolvimento

Consulte [LEIA-ME-PACOTE.md](LEIA-ME-PACOTE.md) e [relatório E3](docs/quality/E3-REPORT.md).
Executar `cargo run --locked -p noah-desktop -- examples/e2-sparse/sparse.ply`.
