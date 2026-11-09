# E1 / MVP 00 — relatório de implementação

Data: 2026-11-09. Estado: implementado, validado localmente, em revisão para aceite.
Não foi iniciado E2. Nenhum commit foi criado; trabalho local no workspace.

## Objetivo e arquivos
Fluxo local create → import PLY → transform → inspect/reopen → validate → export.

- crates/geometry/src/lib.rs: rotação 3D, composição, validação numérica, nuvem e cores.
- crates/core/src/lib.rs: entidades, identidade e atualização de transformação.
- crates/storage/: DTOs schema 1, binário NOAHPC01, PLY restrito, hash e persistência.
- apps/cli/: seis comandos, mensagens/códigos de saída e teste E2E.
- Cargo.toml, manifests dos quatro pacotes e Cargo.lock: build reproduzível.
- tests/datasets/: referência sintética RGB e procedência CC0-1.0.
- scripts/demo.py: demo automatizada sem bibliotecas Python externas.
- docs/decisions/0002-projeto-e1.md, DATA-MODEL.md, README e histórico: contratos.
- THIRD-PARTY.md e third-party-licenses/: inventário e textos fornecidos pelos pacotes.

## Decisões técnicas
Geometria f64, referencial destro Z-up declarado pelo importador, Euler XYZ em
radianos, escala uniforme positiva. Transformação de instância não altera asset.
Unidades são declarações, sem conversão ou promessa de calibração/exatidão.
Metadados JSON separados dos pontos binários e PLY bruto, ambos identificados
por ID e hash SHA-256. DTOs isolam serde do domínio. Sem UI, COLMAP ou banco.
Detalhes e limites no ADR 0002; convenção inicial do ADR 0001 parcialmente superada.

## Testes executados e resultados
Linux x86_64, rustc/cargo 1.99.1; Rust isolado em /workspace/work/toolchain.
Para este ambiente, antes de executar os comandos, configurar:

```sh
export CARGO_HOME=/workspace/work/toolchain/cargo
export RUSTUP_HOME=/workspace/work/toolchain/rustup
export PATH=/workspace/work/toolchain/cargo/bin:$PATH
```

Comandos na raiz do projeto:

| Comando | Resultado observado |
|---|---|
| cargo fmt --check | passou |
| cargo clippy --workspace --all-targets -- -D warnings | passou, sem avisos |
| cargo test --workspace --locked | 13 testes passaram; 0 falhas |
| cargo build --locked | passou |
| python3 scripts/demo.py | passou; XYZ/RGB e hashes conferidos |

Distribuição: 4 testes unitários (core/geometry), 8 testes de integração storage,
1 teste ponta a ponta executando o binário CLI em processos independentes.
Casos cobertos: referências e duplicatas, unidades Unknown/m/mm, f64 não trivial,
rotações/composição, overflow/NaN/Inf, cores incompletas, PLY incompatível/truncado,
limites de arquivos/header/linha, schema desconhecido, JSON corrompido, hash,
comprimento binário inválido, symlinks e rollback após falha no save.

## Demo reproduzível
Requer Rust estável e Python 3.10+. Executar `cargo build --locked` seguido de
`python3 scripts/demo.py`. A demo cria diretório temporário novo e o preserva
para inspeção. N=3, RGB conhecido; escala 2, rotação Z π/2 e translação (3,4,5)
produzem (3,6,9), (1,4,5), (3,4,5), tolerância absoluta 1e-12 para esse dataset.
IDs, unidade Unknown, referencial e transformação são preservados após reabertura.

SHA-256 do dataset: `a91ca21ef167ed4e407ce0214c2377fd9c2710dcda6ab37bace21e3372c78893`.

Saída real da execução:

```text
$ noah create /tmp/noah-e1-demo-1otihm_o/project Referência E1
project_id=a375b77d3ed7490499b75226da5274f9
$ noah import /tmp/noah-e1-demo-1otihm_o/project /workspace/work/NOAH-2.0/tests/datasets/reference-rgb.ply Peça RGB --frame z-up
object_id=6caf71f550d14636aefa1e897fce66a5
$ noah transform /tmp/noah-e1-demo-1otihm_o/project 6caf71f550d14636aefa1e897fce66a5 3 4 5 0 0 1.5707963267948966 2
transformação salva
$ noah inspect /tmp/noah-e1-demo-1otihm_o/project
NOAH 2.0 schema=1 project_id=a375b77d3ed7490499b75226da5274f9 name="Referência E1" objects=1 assets=1
object_id=6caf71f550d14636aefa1e897fce66a5 name="Peça RGB" asset_id=76594eac98d14d7f8057ea73ae494357 points=3 unit=Unknown frame=RightHandedZUp translation=[3.0, 4.0, 5.0] rotation=[[6.123233995736766e-17, -1.0, 0.0], [1.0, 6.123233995736766e-17, 0.0], [-0.0, 0.0, 1.0]] scale=2
$ noah validate /tmp/noah-e1-demo-1otihm_o/project
projeto válido: schema, referências, geometria e hashes conferidos
$ noah export /tmp/noah-e1-demo-1otihm_o/project 6caf71f550d14636aefa1e897fce66a5 /tmp/noah-e1-demo-1otihm_o/transformed.ply
PLY exportado; unidade e referencial devem acompanhar o arquivo separadamente
Demo passou: XYZ/RGB conferidos, origem preservada. Dados em /tmp/noah-e1-demo-1otihm_o
```

## Aceite e limites
| Critério de E1 | Evidência/estado |
|---|---|
| Workspace core/geometry/storage/cli | conforme, todos operacionais |
| N, posições e cores preservados | conforme, import e round-trip |
| Transformação conhecida e origem intacta | conforme, testes e demo |
| IDs, nome, unidade, referencial e transformação persistidos | conforme, igualdade do Project reaberto |
| Unknown permanece Unknown | conforme, CLI e testes |
| Erros sem corromper manifesto anterior | conforme, invalid import/overflow/limite metadata |
| CLI útil e erro não zero | conforme, E2E |
| Gates de fmt/clippy/test | conforme |
| Aceite do usuário | pendente de revisão; não iniciar E2 automaticamente |

Limites: PLY ASCII XYZ/RGB restrito; um escritor por projeto; até 1 milhão de
pontos total, 1000 assets/objetos e 4 MiB de metadados. Não há migração de schema,
leitura incremental para nuvens grandes ou garantia contra queda de energia.
Falhas/interrupções podem deixar assets órfãos, sem remoção automática. Hashes
verificam integridade acidental, não autenticidade contra adulteração coordenada.
Symlinks nos caminhos controlados são rejeitados; alterações concorrentes hostis
no filesystem não são protegidas. Usar diretório local confiável.
Metadados substituídos via temporário sincronizado e rename; sem fsync de diretório.
Arquivos exportados PLY não carregam unidade/frame; preservar essas informações
separadamente. Testado apenas em Linux; outros sistemas exigem validação própria.
Código original do produto publicado sob MIT; dependências do alvo
validado estão inventariadas, sem afirmar distribuição multiplataforma pronta.

## Próximo ciclo proposto (não executado)
Revisar/aceitar E1 com o fluxo demonstrado. Depois definir plano E2: execução local
do COLMAP por adaptador, rastreabilidade de jobs, parâmetros, logs e diagnósticos,
sem acoplar o formato de captura ao domínio. Não implementar UI nessa etapa.
