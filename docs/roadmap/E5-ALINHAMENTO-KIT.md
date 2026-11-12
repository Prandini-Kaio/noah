# Ajustes visuais, alinhamento guiado inicial e kit de pipeline
Autorização direta do usuário em 2026-11-12: estilhaços menores, dispersão maior,
restaurar wireframe, próximo passo e dados preparados para pipeline completo.

Plano: blast.rs com detalhe adaptativo limitado e maior impulso; main/shell com
arame sem preenchimento e sobreposição separada; workspace/alignment.rs com
translação por par de vértices, conversão pelo pai, prévia, aplicação e desfazer
com proteção contra contexto alterado; desktop/alignment.rs com formulário GUI.
Adicionar exemplos CC0: 12 imagens da cena sintética E2, sparse já identificado,
duas malhas para alinhamento. Incluir no pacote e guia sequencial sem terminal.
Testes: cobertura/limites/reversibilidade de estilhaços; semântica wireframe;
alinhamento mundial sob hierarquia/rotação/escala, âncoras inválidas, undo e sessão;
reexecutar pipeline sparse COLMAP real com hashes e inspeção nativa Linux.
Limites: alinhamento translacional de uma peça, sem rotação automática/ICP;
prévia efêmera, sem alterar fontes; não declarar dense/CUDA, CAD ou malha
reconstruída validada. Não executar E6/E7/E8. Base contém PR E4 #5 ainda aberto.
