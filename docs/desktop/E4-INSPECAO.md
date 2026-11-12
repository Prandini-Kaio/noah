# E4 — Ângulos e dimensões XYZ

## Ângulos
Medidas → Abrir ferramentas de medida → Ângulo. Escolha A, B e C na cena ou
selecione uma peça na árvore e informe índices originais em Seleção exata.
**B é o vértice**; mede-se o ângulo entre BA e BC, de 0 a 180 graus.
Ao escolher na cena a janela se recolhe e retorna após o terceiro ponto.
A prévia mostra as coordenadas montadas. Dê um nome e clique Registrar ângulo.

Os três índices devem ser distintos; os braços devem exceder 1e-12 unidades da
cena. Pontos coincidentes em B são rejeitados. Pontos colineares podem fornecer
0° ou 180°. A conta 3D usa f64 e atan2; não é o ângulo aparente na projeção 2D.
A explosão não altera o resultado; anotações seguem âncoras visuais e rotulam
explicitamente o valor montado. Estilhaços são restaurados ao abrir Medir.

Ângulos independem de escala uniforme. Continuam calculáveis com calibração
linear desatualizada, mas um relatório completo continua recusando referência
linear inválida: remova ou refaça a referência antes de exportar.

## Dimensões
Selecione uma peça ou montagem. A aba Análise já apresenta X/Y/Z; use Registrar
dimensões ou a aba Dimensões XYZ na janela para nomear e registrar uma caixa.
Todos os vértices e descendentes entram no cálculo, **mesmo ocultos**, sem
amostragem. Sem referência válida não se inventam mm ou m.

A caixa é alinhada aos **eixos mundiais numéricos XYZ**, com transformações de
instância/hierarquia, no estado montado. Não normaliza o referencial dos arquivos
nem afirma orientação física validada. Girar o objeto pode mudar X/Y/Z. Não é uma
caixa orientada mínima e não mede espessura local, área ou volume.

As definições são vivas: mover/escalar peças ou mudar filhos recalcula valores,
e uma escala desatualizada impede dimensões físicas. O cache de caixas é invalidado
por geometria, transformações e hierarquia; câmera, visibilidade e explosão não
mudam seus valores. O primeiro cálculo de uma geometria grande pode custar mais.

Anotações de caixas aparecem somente no estado montado, pois englobar peças
explodidas produziria uma indicação enganosa. Com todos os itens-alvo ocultos a
caixa também é ocultada no viewport. Ângulos e distâncias continuam marcados como
valores montados. Exibição/Preferências permite ocultar anotações.

## Persistência e relatório
Salvar grava sessão **5**; abre sessões **1–4**. Apps antigos não abrem schema 5.
Ângulos guardam IDs e âncoras ABC; dimensões guardam a identidade da peça/grupo.
As definições são recalculadas ao abrir; não são congeladas como medidas passadas.
A lista oferece Remover; fontes nunca são reescritas.

Relatório **2** conserva measurements (distâncias), acrescenta angles e dimensions,
fontes/hashes, instâncias com transformação mundial decomposta, hierarquia, método,
estado assembled e unidade explícita. Incerteza é null, metrology_validated=false.
Não sobrescreve arquivos existentes. Não presume que um consumidor de relatório 1
saiba ler versão 2. Relação numérica: escala → rotação → translação, vetores coluna;
os referenciais originais continuam identificados por fonte.

Limites: 1000 distâncias, 1000 ângulos e 100 caixas. Sessão segue limite 4 MiB.
Precisão da reconstrução, fonte e referência não é validada por exibir mais casas.
Área, volume, criação de primitivas e alinhamento guiado ficam pendentes. Esta
continuação não declara E4 completa nem muda a pendência dense/CUDA da E2.

## Demo sem terminal
O pacote Linux inclui examples/inspection-box.obj e instruções CC0. Abra-o em
Importar → Geometria. A=6, B=5, C=8 dá 90°. Calibre a distância 2–1 como 200 mm:
Dimensões XYZ dá X=200, Y=300, Z=400 mm. Registre, salve e reabra a sessão.
