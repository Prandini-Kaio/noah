# Cena sintética E2 — CC0-1.0

Gerador: scripts/e2/generate_scene.py. Seed 20261109. Doze imagens PNG 800×600,
projeção pinhole focal 700 pixels. Superfícies texturizadas e caixa central
renderizadas por interseção raio-plano com teste de profundidade.
Nenhuma fotografia ou textura externa foi usada. Gerador/dataset de teste
publicados pelo projeto sob CC0-1.0. Dependências do gerador: NumPy e Pillow.
truth.json registra as poses ideais; não se compara erro métrico diretamente
com SfM, pois uma reconstrução monocular tem liberdade de similaridade.

Gerar: `python3 scripts/e2/generate_scene.py /caminho/novo-dataset`.
Os PNGs são gerados sob demanda. SHA-256 das imagens usadas na validação
está em docs/quality/e2-dataset-hashes.json; não é garantia de PNG idêntico
entre versões de Pillow/NumPy. Não há download de fotos pela aplicação.
As imagens reais do usuário precisam de sobreposição, textura e exposição
adequadas; o sucesso sintético não garante reconstrução de qualquer objeto.
