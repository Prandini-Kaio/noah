# E6 — fotos adicionais e comparação

## Testar sem COLMAP
1. Abra **Comparar…** na barra de ferramentas.
2. Clique **Usar comparação pronta do kit** e **Alinhar câmeras e comparar**.
3. Escolha Sobrepor, Anterior, Atual alinhada ou Diferenças na atual e clique
   **Visualizar comparação**. Azul identifica a anterior; laranja, a atual.
4. Use **Voltar ao workspace original** para restaurar a câmera e a cena.
5. Salve o relatório JSON. **Reabrir comparação salva…** verifica os hashes das
   fontes e recalcula os resultados; mantenha os runs nas mesmas pastas.

## Reconstruir com novas fotos
1. Em Reconstruir → Configurar, use as fotografias de `examples/refinamento/base`
   (10 imagens), selecione COLMAP 3.11.x e um destino novo; execute.
2. Em Resultados, abra a geometria. Em Configurar, clique **Adicionar fotos ao
   run selecionado** e **Usar fotos adicionais do kit** (duas imagens).
3. Escolha o resultado desejado e execute em outra pasta. As fotos anteriores e
   as novas únicas são copiadas e o SfM é reprocessado completamente.
4. Em Resultados, clique **Comparar revisões / adicionar captura…** e compare.

As 12 fotografias são as mesmas imagens sintéticas CC0 do kit anterior, agora
separadas em 10+2. Não representam uma nova captura física. `pronto/anterior` e
`pronto/atual` contêm snapshots e artefatos registrados de execuções reais de
COLMAP CPU; os bytes e manifests foram preservados. Intermediários não registrados
(banco de dados/binários) não são distribuídos. Os caminhos históricos nos logs e
na proveniência pertencem à máquina de geração; selecione os runs locais pelo kit.
Não execute os caminhos históricos. Não é necessário COLMAP para compará-los.

## Limites
As comparações usam a nuvem esparsa, não a malha ou a nuvem editada. Exigem quatro
fotos únicas comuns e câmeras não colineares. Ajustam rotação, escala positiva e
translação; rejeitam residual de câmera acima de 5% do raio de referência.
Estatísticas bidirecionais usam até 20 mil consultas e 200 mil pontos-alvo por
sentido, com amostragem determinística. São distâncias entre pontos, em unidades
do run anterior, sem precisão física validada. Mais pontos não garantem qualidade.
A revisão não herda limpeza, calibração ou transformações do workspace; o run pai
é preservado. Não há registro incremental, ICP geral ou textura nesta entrega.
