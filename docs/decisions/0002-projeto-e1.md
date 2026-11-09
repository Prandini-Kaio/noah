# ADR 0002 — Persistência e operações E1
Status: Accepted
Data: 2026-11-09

Contexto: completar o fluxo local E1 preservando entrada, identidade e unidades.

Decisão: diretório de projeto com project.json (schema_version=1), raw/ para PLY
original e derived/ para nuvens binárias internas versionadas. Metadados JSON usam
DTOs próprios em storage; domínio não depende de serde. IDs persistidos como
strings hexadecimais u128; novos IDs usam aleatoriedade do SO via UUID v4.
Cada importação registra origem, hash SHA-256 e referencial declarado. A CLI exige
--frame z-up, pois PLY não define eixos. Unidade ausente permanece Unknown.

Assets são criados exclusivamente, sem sobrescrever arquivos. Metadados são
validados antes de salvar e substituídos por rename de temporário sincronizado.
Falha antes do rename preserva os metadados anteriores; arquivos órfãos podem
restar após interrupção. E1 não oferece escrita concorrente (um escritor por
projeto); não há coleta automática de órfãos. Rejeitar symlinks internos e caminhos
não canônicos. Não seguir referências fora dos diretórios controlados.

PLY aceito: ASCII 1.0, um elemento vertex, XYZ float/double nessa ordem, RGB uchar
opcional nessa ordem. Comentários permitidos; faces, listas, outras propriedades
e binários rejeitados. Limites: 64 MiB por PLY, 1 milhão de pontos, header 64 KiB, linha 4096 bytes,
metadados 4 MiB, 1000 assets/objetos, 1 milhão de pontos total por projeto.
Nuvem interna: magic NOAHPC01, contagem u64 LE, flag RGB u8, XYZ f64 LE e RGB u8.

Transformações passam a usar matriz de rotação ortonormal 3x3 e escala uniforme
positiva. Euler XYZ em radianos aplica Rx, Ry, Rz (Rz*Ry*Rx). Composição A.compose(B)
aplica B primeiro e A depois. Os pontos brutos não mudam. Exportação transformada
é um arquivo novo; não altera assets nem afirma calibração metrológica.

Alternativas: JSON para pontos (volume e memória), banco (complexidade prematura),
PLY universal (superfície de parser desnecessária). Metadados legíveis e nuvens
binárias mantêm precisão f64; não há migração de schemas futuros nesta etapa.

Validação: round-trip exato de f64 finitos, IDs/unidades/cores/referencial,
composição, parser hostil, schema desconhecido, corrupção, falha sem alteração
do manifesto, integridade dos brutos e execução real da CLI.
