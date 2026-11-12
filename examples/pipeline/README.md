# Kit sintético de pipeline — CC0-1.0

As 12 imagens PNG em fotos/ são a cena sintética E2 gerada por
scripts/e2/generate_scene.py, seed 20261109, 800×600/focal 700 pixels, sem fotos ou
texturas externas. Conferidas contra docs/quality/e2-dataset-hashes.json.
hashes.json registra os arquivos incluídos; truth.json guarda câmeras ideais,
sem constituir validação física/metrológica de uma reconstrução.

Na interface: Reconstrução → Usar fotos do kit incluído → escolher executável
COLMAP **3.11.x** local → pasta gravável de resultados → Iniciar reconstrução.
Deixe a densificação desmarcada para o teste CPU. Cada run exige nome novo e
preserva originais, snapshots, parâmetros, hashes, etapas, logs e resultados.
Ao concluir, use Abrir nuvem no workspace. Inspecione pontos, registre medidas
em unidades da cena, salve/reabra a sessão e exporte relatório/peça selecionada.

COLMAP não vem no pacote. Instale 3.11.x segundo as instruções oficiais:
https://colmap.github.io/install.html. Em Linux costuma estar em /usr/bin/colmap
ou no diretório da instalação local. A versão da sua distribuição pode diferir;
a integração atual verifica 3.11.x. O seletor deve apontar ao executável confiável.
Não é necessário Rust para operar o pacote NOAH. Sem COLMAP, o botão Abrir nuvem
pronta permite inspecionar um resultado já processado; não executa reconstrução.

sparse-preprocessado.ply e reconstruction-preprocessada.json são cópias do exemplo
E2 histórico: 7592 pontos, 12 câmeras registradas. Esse resultado é explicitamente
pré-processado. Uma nova execução pode produzir outra contagem: SfM não é
prometido bit a bit determinístico. Nenhuma unidade física é inferida.

O pipeline validado é fotos → reconstrução **esparsa** → pontos → inspeção →
salvar/exportar. Não se promete malha densa ou separação automática em peças.
Para wireframe, alinhamento e estilhaços, há malhas sintéticas independentes em
../alinhamento e ../inspection-box.obj: não são apresentadas como malhas obtidas
pela reconstrução destas fotos. Dense exige COLMAP com CUDA/GPU e segue sem
validação real nesta entrega. Guia completo no LEIA-ME do aplicativo.
