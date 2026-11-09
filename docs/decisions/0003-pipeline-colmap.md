# ADR 0003 — Runs locais e adaptador COLMAP
Status: Accepted
Data: 2026-11-09

Contexto: E2 autorizada pelo usuário; preservar domínio independente do motor e
não confundir saída de SfM com referencial físico calibrado.

Decisão: pipeline executa listas de argumentos usando Command (sem shell) e
registra run.json schema 1, entradas SHA-256, configuração, etapas e logs. Um run
sempre usa diretório novo; entradas são copiadas para images/ e nomes normalizados.
Diretório original não é passado ao motor. Photos originais permanecem intocadas.
Status persistidos distinguem falha, cancelamento e sucesso; falhas parciais
preservam logs/arquivos. Sem reuso automático/cache nesta versão.

Limites iniciais: 500 imagens JPEG/PNG, 32 MiB cada, 512 MiB no total; mínimo 3.
Limites de saída: 8 MiB por stream de log por etapa (checado por polling), TXT
256 MiB por arquivo, 1 milhão de pontos. Uma etapa tem timeout configurável;
um arquivo CANCEL no run solicita cancelamento. Polling 25ms, grupo de processo
no Unix encerrado ao abortar. Windows não é alvo validado. Não há sandbox do
motor: só executar binário local confiável escolhido explicitamente pelo usuário.

photogrammetry constrói as etapas do COLMAP 3.11 e traduz TXT para core/geometry.
core declara Camera, CameraView e Reconstruction sem importar formatos de motor.
Extrínsecos: quaternion wxyz e translação descrevem world→camera; unidades Unknown,
referencial destro arbitrário da reconstrução, sem inferir vertical/escala.
Resultado esparso em sparse.ply + reconstruction.json com câmeras, poses e IDs.
Modelo principal é o submodelo com mais imagens registradas; outros submodelos
são preservados. Dense é opt-in, requer CUDA; fused.ply é exportado no formato
original do COLMAP, separado do parser PLY ASCII de E1.

Artefatos em run.json têm hashes e tamanho, verificados por run-inspect. Imagens,
logs e arquivos intermediários são locais. E1 não sofre migração: vínculo de run
com objetos/assets fica para uma evolução de schema explícita.

Alternativas: chamar automatic_reconstructor (menor controle de etapas/diagnóstico),
ligar libcolmap diretamente (acoplamento/build mais pesado), converter referencial
arbitrário silenciosamente para Z-up (semanticamente incorreto).

Consequências: outputs independentes do projeto E1, sem integração artificial que
alegue preservar origem PLY. Textos de modelos exigem parser com limites e
validação. Um run só é concluído após validar resultados, nunca pelo exit code
isoladamente. Queda do processo NOAH pode deixar status Running; inspect informa
esse estado sem afirmar atividade ou retomar automaticamente.

Validação: motor controlado para falhas e ciclo completo, dataset sintético
renderizado para execução real CPU sparse, tentativa real de modo denso e
relatório específico da capacidade do ambiente.
