# ADR 0013 — Fluxo guiado, preferências e desmontagem hierárquica
Status: implementado, 2026-11-13.

A pipeline tem uma entrada com Configurar, Processar e Resultados. Atalhos apenas
abrem esse fluxo; somente o passo Configurar dispara execução. Preferências gerais
(motor e históricos de pastas) são locais ao app e não pertencem a um run/sessão.
Cada finalidade tem chave própria, inclusive preparo/limpeza/exportação. Arquivo
versionado em XDG_CONFIG_HOME/noah/preferences.json ou ~/.config/noah; gravação por
arquivo temporário e rename, erros reportados. Lembrar motor não o executa.

Inspetor mantém identidade/tipo/caminho da seleção e abas fixas Objeto, Análise,
Ações; não troca transformações pelo contexto da pipeline. Abrir resultado fecha
janelas, seleciona/enquadra a representação correta, reativa tipo de geometria e
oculta outros objetos para comparação explícita por visibilidade. Fontes já abertas
são reutilizadas, evitando erro de duplicação. Editar resultado abre a revisão
não destrutiva existente. Logs são texto read-only selecionável, com cópia explícita,
filtragem por etapa/fluxo e limites de trecho identificados; arquivos completos ficam.

Desmontagem e fragmentação são ferramentas de apresentação separadas. Radial
preserva comportamento anterior. Camadas preserva grupos no nível escolhido;
armadura abre fronteiras da hierarquia progressivamente. Transformações rígidas de
translação são aplicadas em mundo XYZ só na visualização. O núcleo contido pode
ficar fixo enquanto grupos exteriores abrem; profundidade revela filhos. Não se
inferem componentes internos de uma captura. Exemplo sintético com 13 peças e seis
montagens explicita carcaça e núcleo.

Painéis de superfície agrupam faces originais por orientação e região com uma
transformação rígida compartilhada; sem subdivisão fractal. Estilhaços anteriores
continuam disponíveis. Ambos temporários, sem interiores/colisões/fratura física;
superfícies amostradas podem ter descontinuidades. Exportação e medidas usam o
estado montado. Sessão passa a 8, lê 1–7 com opções novas padrão radial; relatório
geométrico permanece 4 porque não inclui a apresentação. Preferências têm schema 1.
