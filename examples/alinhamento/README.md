# Duas peças para alinhamento — CC0-1.0
Malhas sintéticas independentes da fotogrametria; sem unidade física declarada.
fixa.obj: caixa XYZ 2×3×4 com origem (0,0,0).
movel.obj: mesma caixa com origem (6,0,0), geometria original diferente/preservada.

Workspace → Abrir peças de alinhamento do kit. Em Montagem → Alinhar peças:
Peça móvel=movel.obj, vértice=1; Destino fixo=fixa.obj, vértice=2.
Calcular deslocamento deve mostrar ΔX=-4, ΔY=0, ΔZ=0 unidades da cena.
Marque a prévia: as caixas ficam adjacentes (X 0–2 e 2–4). Aplique e desfaça.
Apenas a transformação da instância móvel muda; seus vértices-fonte não mudam.
Salvar guarda o ajuste aplicado; a prévia e o desfazer são temporários.
Não há rotação automática, ICP ou restrições CAD. Uma referência de escala
entre peças pode ficar desatualizada após alinhamento e deve ser conferida.
