# Ajustes visuais, alinhamento E5 e kit de pipeline
Data: 2026-11-12. Pedido explícito: estilhaços menores, maior dispersão, retorno
do wireframe, próximo passo e dados preparados para testar o pipeline.
Plano: ../roadmap/E5-ALINHAMENTO-KIT.md. Decisão: ADR 0010.

## Alterações
- Desktop/blast: subdivisão recursiva/adaptativa, 64 triângulos por face em malhas
  pequenas no padrão; força 1,4, impulso radial maior, faixa até 6 e câmera opcional.
  Malhas grandes reduzem subdivisão antes de amostrar; orçamento 20 mil preservado.
- Desktop/main/shell: arame legado sem preenchimento, overlay separado; seleção de
  kit fotográfico/nuvem pronta e abertura das duas peças pela interface.
- Workspace/alignment: plano translacional entre âncoras no estado montado,
  conversão pelo pai, aplicação/undo com contexto protegido e geometria imutável.
- Desktop/alignment: escolha de peças/índices ou A/B já escolhidos em Medir,
  prévia efêmera, aplicar/desfazer e avisos de escala. Ajuste aplicado persiste em
  Placement; preview/undo não persistem. Sessão 5/relatório 2 mantidos.
- Exemplos/pacote: 12 PNGs E2 conferidos contra hashes históricos, verdade sintética,
  nuvem explicitamente pré-processada e malhas fixa/móvel independentes; guias CC0.
  Sem nova dependência, motor embutido ou download em runtime.

## Gates executados
Linux x86_64, Rust 1.99.1, Xorg dummy :99, Mesa OpenGL software, COLMAP 3.11.1 CPU
isolado já existente em /workspace/work/colmap-tools; sem instalação global nova.

| Comando/fluxo | Resultado |
|---|---|
| `cargo fmt --all --check` | passou |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passou |
| `cargo test --workspace --locked` | 60 testes, 0 falhas |
| `cargo build --release --workspace --locked` | passou |
| `scripts/desktop/package-linux.sh <pasta-nova>` | pacote com fotos, hashes, nuvem, peças e guias |
| CLI real `noah reconstruct ... --colmap ... --timeout 180` | Succeeded, 12/12 imagens, 7590 pontos |
| GUI real, botão Usar fotos do kit | Succeeded, 12/12 imagens, 7611 pontos |
| Originais/snapshots GUI | 12 hashes coincidem; fotos de entrada não alteradas |
| GUI: abrir nova nuvem, medir, salvar, relatório | 17,6830528255 unidades de cena entre vértices 1/2; sem mm inventados |
| GUI: exportar nuvem | PLY com 7611 pontos + metadados montados |
| GUI: movel/1 → fixa/2 | ΔX=-4, ΔY=0, ΔZ=0; prévia e aplicação conferidas |
| GUI: salvar aplicado e desfazer | translação salva [-4,0,0]; revisão desfeita [0,0,0] |
| Wireframe e overlay | arame sem faces e sólido com arestas inspecionados separadamente |
| Estilhaços finos | duas caixas: 1536 fragmentos; câmera e dispersão conferidas |
| Pacote final, reabertura da sessão do pipeline | 7611 pontos e distância 17,6831 u. cena restaurados |
| Botão do kit no pacote final | resolveu examples/pipeline/fotos junto ao executável, nome novo 002 |
| Compatibilidade binária | maior símbolo GLIBC exigido: 2.39 |
| Vídeo nativo | 100 frames, 1440×900, 20 fps, 5s |

Seis testes novos: alinhamento conhecido/undo/fontes; pais com rotações/escalas
mais base e ajuste local; âncoras/contextos inválidos sem mutação; persistência e
explosão ignorada; detalhe/impulso/limites de estilhaços; semântica arame/overlay.
Os testes anteriores de cobertura de superfície/rigidez/orçamento/reversão continuam
passando. Tolerância numérica de 1e-10 no caso sintético de alinhamento hierárquico.
Nenhuma afirmação de exatidão física foi adicionada.

## Reproduzir
Pacote: Reconstrução → Usar fotos do kit → COLMAP 3.11.x local → destino e nome
novos → Iniciar → Abrir nuvem → Medidas → Salvar/Exportação. Sem densificação.
Em seguida, Workspace → Abrir peças de alinhamento do kit → Montagem → Alinhar
peças → movel/1 até fixa/2 → prévia/aplicar/desfazer. Testar Arame e Explodir.
Guia detalhado: ../desktop/E5-KIT-PIPELINE.md e READMEs nos exemplos do pacote.

Dados e resultados dos testes: /workspace/work/e5-pipeline-verified e
/workspace/work/e5-gui/reconstrucao-kit-001. São evidências locais, não caminhos
para o usuário. Os dados reproduzíveis ficam em examples/pipeline/fotos; hashes
coincidem com docs/quality/e2-dataset-hashes.json. Contagem SfM pode variar entre
execuções/threads; não é teste de erro físico ou promessa de saída determinística.
Capturas em docs/desktop/e5-*.png e docs/quality/e5-evidence/. Vídeo na distribuição,
fora do histórico de fonte. Não contém fotos externas ou dados do usuário.

## Aceite e limites
Conforme: correções visuais, alinhamento guiado translacional inicial e kit pronto;
pipeline sparse real executado pela CLI e GUI até inspeção/salvar/exportar;
integridade verificada. Prévias não alteram fontes, contexto alterado impede aplicar
planos/undo antigos e destinos existentes continuam protegidos.

Não há rotação automática/ICP, alinhamento de grupos inteiros ou restrições CAD.
Desfazer é de um único alinhamento e temporário; outros ajustes posteriores podem
inabilitá-lo. A referência linear entre peças pode ficar desatualizada. Estilhaços
são de superfície, não volumes/fratura física/colisões. Acima do orçamento a malha
pode ser amostrada. Câmera manual interrompe acompanhamento; alta força sem ele
pode lançar peças para fora da tela. Arame inclui arestas triangulares e não é
remoção exata de linhas ocultas; o renderer mantém profundidade aproximada.

Teste completo aqui significa **pipeline esparso mais fluxos manuais posteriores**.
Malhas sintéticas do kit não são apresentadas como resultado das fotos. Dense/CUDA,
malha reconstruída, segmentação automática, precisão física, novas primitivas E4,
E6/E7/E8 permanecem pendentes. COLMAP continua separado do pacote NOAH.
