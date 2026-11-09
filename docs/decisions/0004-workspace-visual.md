# ADR 0004 — Workspace visual e sessão E3
Status: Accepted
Data: 2026-11-09

Contexto: iniciar E3 autorizado mantendo formatos/contratos E1/E2 e dados brutos.

Decisão: noah-workspace contém cena/câmera e persistência de sessão; apps/desktop
usa egui/eframe 0.33 com backend glow/X11. Geometry mantém malha triangular sem
I/O; storage faz parsing OBJ restrito. Nenhuma biblioteca de domínio importa UI.

O renderer calcula projeção perspectiva em f64 na CPU e gera primitivas egui.
Demonstra navegação/seleção em nuvens pequenas sem novo pipeline gráfico próprio.
Ordem de profundidade por ponto/centróide de face; sem z-buffer 3D. Até 50k pontos
+20k faces exibidos por cena; amostragem determinística, dados originais intactos.
Primitivas fora do near-plane são descartadas, não cortadas geometricamente.
Geometria/transformações visíveis têm alcance numérico limitado a |coord|<=1e12.
Câmera usa Z como up de navegação, sem afirmar vertical física de SfM.

Sessão JSON schema 1 referencia fontes canônicas por caminho e SHA-256; não
serializa geometria. Cada fonte tem tipo e fingerprint (arquivo ou manifesto).
Ao reabrir, loaders E1/E2 validam também seus assets/hashes. Transformações,
visibilidade, câmera, seleção e opções visuais são dados da sessão; não alteram
project.json, run.json nem pontos/malhas de origem. Salvar usa create_new para
não sobrescrever sessões: cada gravação é uma revisão com nome novo. Caminhos
são locais absolutos nesta versão; mover origens exige abrir novamente a fonte.
Caminhos em sessão são tratados como dados e nunca executados.

OBJ: somente v x y z e f a b c com índices positivos, sem UV/normais/material,
polígonos, índices negativos, slash syntax ou comandos externos. Comentários e
linhas vazias permitidos; outros registros rejeitados. Até 1M vértices, 100k faces,
64 MiB/arquivo, 4096 bytes/linha. Malha sintética de referência acompanha projeto.

Alternativas: renderer wgpu dedicado (mais trabalho de infraestrutura antes de
validar UX), WebView/serviço local (muda natureza desktop e dependências), salvar
transformações direto em E1 (mistura cena com origem e não atende runs E2/OBJ).

Consequências: versão visual utilizável e testável, com limites explícitos de
oclusão/performance. Sem promessa de CAD, precisão metrológica ou compatibilidade
universal de malhas. Renderer GPU com depth buffer e LOD pode substituir esse
backend após medições e decisão futura. Sem cache de geometria na sessão.

Validação: testes numéricos, sessão round-trip e origem intacta, imagens/captura
da execução nativa quando display disponível. Sem teste visual, não declarar
interação desktop como verificada.
