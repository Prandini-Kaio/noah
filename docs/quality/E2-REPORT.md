# E2 / MVP 01 — relatório de implementação

Data: 2026-11-09. Autorização: «Vamos montar a E2», após entrega de E1.
Estado: reconstrução **esparsa validada com COLMAP real**; densificação implementada,
mas validação real **bloqueada pela ausência de CUDA**. E2 não tem aceite integral
concluído. E3 não foi iniciada. Nenhum commit foi criado; código no workspace local.

## Objetivo e arquivos
Fotos locais → run rastreável → câmeras/poses e nuvem → exports e diagnóstico.

- crates/pipeline/: snapshot, hashes, parâmetros, etapas, processos sem shell,
  timeout/cancelamento, logs/status e validação de artefatos registrados.
- crates/photogrammetry/: comandos COLMAP, seleção de submodelo, parser TXT,
  validação de poses/referências e PLY denso, exportação.
- crates/core/src/reconstruction.rs: contratos neutros Camera/CameraView/Reconstruction.
- apps/cli/: reconstruct, run-inspect, run-cancel; E1 mantida.
- scripts/e2/: cena sintética reprodutível e demo com verificações reais.
- testes de pipeline/adaptador: processos reais controlados, falhas e parsing.
- workspace/Cargo.lock, README, ADR 0003, E2-MVP01 e fronteiras atualizadas.
- inventários de dependências/dataset/motor e evidências em e2-evidence/.

## Decisões técnicas
COLMAP via executável confiável informado pelo usuário, sem shell ou integração
com libcolmap. Binário invocado tem SHA-256 e versão registrados; se for launcher,
o hash é do launcher. Inventário do ambiente de teste do motor em
e2-engine-packages.json. NOAH não baixa nem instala motores em runtime.

Runs novos separados do schema E1, com fotos copiadas integralmente. Não converter
coordenadas para Z-up nem inferir unidade: resultado é SfM destro arbitrário e
Unknown. Quaternion wxyz/translation descrevem world→camera. Modelo com maior
número de imagens registradas é selecionado; todos os submodelos são preservados.
Saídas verificadas antes de status Succeeded. Nenhum processamento altera fotos
originais; falhas mantêm logs e saídas parciais. Não há cache/retomada automática.

## Verificações executadas
Ambiente Linux x86_64, Rust/Cargo 1.99.1, COLMAP 3.11.1 CPU (sem CUDA), duas threads.
Motor instalado isoladamente em /workspace/work/colmap-tools, sem instalação
sistêmica. Pacotes conda-forge foram resolvidos por micromamba, baixados com
verificação SHA-256, extraídos/relocados localmente; inventário retido. Arquivos
do motor não são distribuídos junto ao projeto.

| Comando/fluxo | Resultado real |
|---|---|
| cargo fmt --check | passou |
| cargo clippy --workspace --all-targets -- -D warnings | passou |
| cargo test --workspace --locked | 19 testes passaram, 0 falhas |
| cargo build --workspace --locked | passou |
| scripts/e2/demo.py com COLMAP 3.11.1 real CPU | passou, 12/12 imagens, 7592 pontos |
| reconstruct --dense com COLMAP 3.11.1 real CPU | falhou em PatchMatch por ausência de CUDA; status Failed correto |

Os 19 testes incluem 13 existentes de E1, 3 de pipeline e 3 de adaptador/modelo.
Um dos testes percorre sparse/dense com executável controlado: valida orquestração,
parsing e outputs, sem alegar testar algoritmo SfM/MVS. Outros exercitam exit code
7, timeout, cancelamento, limite de logs, snapshots, alteração de hash, argumentos
sem interpretação de shell, poses inválidas, referências inexistentes, NaN,
saída ausente/corrompida apesar de exit code zero e não reutilização de run.
Testes Unix exigem Python 3 para o executável controlado.

## Demo real e procedência
Dataset gerado no projeto, CC0-1.0, seed 20261109: 12 PNGs 800×600 com superfícies
texturizadas, caixa, projeção pinhole e oclusão. NumPy/Pillow são dependências de
teste; o executável NOAH não depende deles. Gerador e README no repositório,
hashes das fotos em e2-dataset-hashes.json. Câmeras ideais em truth.json ajudam
reprodução, mas não foi feita avaliação metrológica nem alinhamento de similaridade.

Execução final: /workspace/work/e2-demo-final. Run ID: b16b22a5-f94f-42a3-9d1b-2dd8de0a6909.
Cinco etapas concluídas e cinco artefatos registrados. Tempo observado:
39.993s, incluindo snapshot/hashes/etapas e gravação, medição pontual
neste ambiente, sem promessa de desempenho. Originais e snapshots conferidos
por hash pela demo. Fotos registradas: 12/12; pontos: 7592.
A execução anterior gerou 7612 pontos; a execução final gerou 7592. SfM com threads
não é garantidamente idêntico entre execuções. São rastreados entradas/parâmetros
e ambiente; não se promete saída bit a bit determinística.

Reproduzir na raiz, com Rust estável disponível e NumPy/Pillow instalados:

```sh
cargo build --workspace --locked
python3 scripts/e2/demo.py --colmap /workspace/work/colmap-tools/colmap --output /workspace/work/minha-nova-demo-e2
```

O diretório de saída deve ser novo. Em outra máquina, substituir o caminho do
motor por instalação local de COLMAP 3.11.x. Neste ambiente Rust está isolado em
/workspace/work/toolchain; usar CARGO_HOME/RUSTUP_HOME/PATH conforme E1-REPORT.md.
O gerador não usa fotos/texturas remotas nem expõe dados do usuário.

## Tentativa densa e diagnóstico
Run real em /workspace/work/e2-real-dense, status Failed. Feature extraction,
matching, mapper, model_converter e image_undistorter passaram. PatchMatch saiu
com código 1; stereo_fusion não foi iniciado. Mensagem real:

```text
Dense stereo reconstruction requires CUDA, which is not available on your system.
```

Reconstrução esparsa e cinco artefatos permaneceram registrados nesse run.
Evidência: e2-evidence/dense-run.json e dense-cuda-error.txt. Não se gerou nem se
alegou nuvem densa real. Validar o mesmo comando com --dense em COLMAP CUDA/GPU
compatível é pendência técnica de E2, sem iniciar E3.

## Critérios e limites
| Critério | Estado |
|---|---|
| Fotos → sparse, câmeras e poses com motor local real | conforme |
| Entradas, parâmetros, versão, logs, status, exports | conforme |
| Unidade/frame explícitos, origem preservada | conforme |
| Falhas, timeout, cancelamento e outputs inválidos | conforme nos testes |
| Regressão de E1, fmt/clippy/test | conforme |
| Dense com motor controlado | conforme (orquestração/parser) |
| Fotos → dense com CUDA real | bloqueado por capacidade do ambiente |
| Integração automática run → Object E1 | não implementada; decisão explícita de schema |
| Aceite integral E2 | pendente da validação densa e revisão |

Limites operacionais: 3–500 imagens de diretório plano; 32 MiB/imagem, 512 MiB total;
TXT 256 MiB/arquivo, linha TXT 2 MiB; 1 milhão de pontos; até 32 submodelos e 100
etapas. Logs fiscalizados por polling (8 MiB/stream, possível excesso entre polls).
Sem quota global de disco para database/intermediários do COLMAP; motor confiável
pode usar memória/disco conforme dados. Timeout por etapa; um escritor/run.
Snapshots são imutáveis pelo NOAH, mas não têm proteção contra adulteração externa.
Hashes detectam alterações acidentais, não autenticam um manifesto adulterado.

Cancelamento por arquivo CANCEL é observado entre etapas/no polling do motor;
parsing/exportação/hashing locais não têm interrupção imediata. Em timeout ou
cancelamento, grupo de processo Unix é encerrado; comportamento Windows não
validado. Se NOAH for morto abruptamente, run pode permanecer Running e motor
pode continuar; não há daemon, recovery ou inferência automática de atividade.
Metadados usam temporário + sync + rename, sem garantia contra queda de energia.
Diretórios precisam ser confiáveis: symlinks controlados são rejeitados, sem
proteção contra races hostis no filesystem. Não há upload/telemetria do NOAH;
um binário externo escolhido pelo usuário continua tendo suas próprias permissões.

## Próximos passos dentro de E2
1. Executar/validar densificação real em GPU com COLMAP CUDA.
2. Experimentar dataset fotográfico local autorizado, além da cena sintética.
3. Revisar contrato de vínculo run/assets/projetos antes de mudar schema E1.
4. Revisar e aceitar E2; só então planejar E3 com autorização do usuário.
