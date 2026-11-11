# NOAH — Arctic

Arctic Dark é o padrão. Em **Preferências → Tema e opções visuais**, alterne para Arctic Light e ajuste réguas,
eixos, anotações, grade, arame, projeção e movimento reduzido. Salvar revisão do
workspace preserva as preferências; um workspace novo começa no padrão escuro.

| Papel | Arctic Light | Arctic Dark |
|---|---|---|
| Background | #ECF1F3 | #121C26 |
| Surface | #D1DEE4 | #1B2A36 |
| Surface Elevated | #F7FAFB | #273A49 |
| Primary | #587E95 | #8EBBD2 |
| Text | #324D60 | #E3ECF2 |
| Text Secondary | #536B79 | #A3B8C6 |
| Border | #BACBD4 | #354B5C |
| Viewport | #DCE5E9 | #202D38 |

Cores funcionais Dark: Success #8AC1A9, Warning #E4B779, Error #D48F8F,
Selection #324E61. No Light, cores funcionais são ajustadas para contraste.
Cores RGB das nuvens e cores de peças são dados/auxílios visuais, não substituídas
integralmente pela paleta da interface. Botões principais têm hierarquia de cor,
contornos arredondados e realce de foco/hover animado; foco por teclado permanece.

## Explosão
**Explodida / Montada** anima separação e recomposição de componentes por cerca
de 0,9s nominais, com easing e enquadramento de câmera. Depende da capacidade de
renderização do equipamento. Não corta a malha nem simula fratura física.
O destino é salvo/exportado como estado de apresentação; as posições montadas
não são reescritas. Navegar durante a transição a interrompe no estado visual
atual. **Reduzir movimentos** aplica o destino sem animação.

## Réguas
Réguas horizontal e vertical usam os eixos U/V do plano da câmera, relativos à
origem do mundo. **Ortográfica** tem escala uniforme na tela em qualquer
profundidade. **Perspectiva** informa escala apenas no plano do alvo da câmera.
Uma aresta oblíqua pode ter projeção menor que seu comprimento 3D; use a anotação
para distância real entre vértices no estado montado.

Sem referência de escala, mostram unidades da cena. Com referência válida,
mostram mm/m. Uma referência alterada não mantém unidade física silenciosamente:
a régua volta a unidades da cena e exibe aviso. Barra inferior, marcas graduadas,
grade ortográfica e guias do cursor acompanham zoom/pan/projeção. Vistas frontal,
superior e lateral estão em Visual. Os eixos XYZ do canto são orientação, não
calibração de vertical física.

## Seleção de medição
Clique **Medir → Escolher pontos A e B**. A janela recolhe durante a seleção e
retorna ao escolher o segundo ponto. O cursor pré-visualiza o vértice; raio de
snap configurável (3–24 px), filtro para peça/montagem selecionada e teste frontal
em malhas estão em Visual. O teste usa até oito candidatos e malhas completas,
com limite de 200 mil faces visíveis; acima disso, isole peças ou desative a
proteção. Nuvens não definem superfície oclusora. O renderer ainda tem ordenação
aproximada; a proteção de seleção não é um z-buffer novo do renderer.

Para escolhas reproduzíveis, selecione uma peça na árvore e abra **Seleção exata /
coordenadas**, informe índices A/B (começam em 1) e aplique. Coordenadas montadas,
componentes ΔX/ΔY/ΔZ e casas decimais de exibição ajudam a conferir os pontos.
Até nove casas decimais são formatação de f64, não precisão metrológica adicional.
A incerteza continua desconhecida. Geometria, origem e calibração devem ser
adequadas ao uso; não há garantia de tolerância industrial.

Sessão 4 lê 1/2/3. Apps anteriores não abrem as novas sessões; mantenha revisões
antigas se precisar voltar de versão. Fotos/PLY/OBJ originais permanecem intactos.
