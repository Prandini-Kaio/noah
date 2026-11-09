# E2 / MVP 01 — Reconstrução fotogramétrica local

Autorização: pedido do usuário «Vamos montar a E2» após a entrega de E1, em
2026-11-09. Esse pedido atualiza o limite anterior de E1 no starter.

## Plano antes das alterações
- Criar crates pipeline e photogrammetry, contratos neutros de câmeras em core,
  comandos CLI reconstruct/run-inspect/run-cancel e testes com motor controlado.
- Pipeline: snapshot de fotos, hashes, parâmetros, versão e caminho do motor,
  argumentos estruturados, logs separados, timeout/cancelamento, status persistido.
- Adaptador COLMAP: features CPU, exhaustive matcher CPU, mapper, conversão TXT,
  câmeras/poses e nuvem esparsa exportáveis; modo denso opcional executa undistort,
  PatchMatch e fusion, com erro explícito se CUDA não estiver disponível.
- Registrar ADR 0003; atualizar README, fronteiras, inventário e relatório E2.

## Aceite
Reconstruir dataset reprodutível com COLMAP real; conferir número de câmeras/pontos,
proveniência, preservação dos originais e exports. Testar falhas de processo,
timeout, cancelamento, arquivos incompatíveis e saída ausente/corrompida. Gates
fmt, clippy e testes de workspace; E1 continua passando.

## Limites e riscos
COLMAP não faz parte do executável NOAH: instalação local e caminho explícito.
SIFT CPU e matching têm compatibilidade com COLMAP 3.11.x. Densificação PatchMatch
requer build CUDA e GPU adequada; ausência disso impede aceite real do modo denso.
Reconstrução não implica escala física ou Z-up: unidade Unknown, referencial de
reconstrução arbitrário. Não importar automaticamente na persistência E1, cujo
schema exige origem PLY/canônico. Run contém exports e metadados próprios versionados.
Sem GUI, alinhamento, otimização própria, cache/resume ou integração com sensores.
