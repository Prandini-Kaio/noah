# E4 — primeiro incremento de distâncias/escala
Data: 2026-11-10. Continuação autorizada após a prévia E3/E5.

## Objetivo e implementação
Selecionar dois vértices pelo app, medir no estado montado, definir escala física
por distância conhecida e persistir anotações com procedência e incerteza desconhecida.

Arquivos: workspace/measurement.rs; sessão schema 3 (lê 1/2); campos de Scene;
controles, snap e desenho no desktop; exportação JSON por explorador integrado;
testes e ADR 0006. Sem alterar schemas E1/E2, geometria bruta ou transformações.

A seleção captura ID da peça e índice do vértice original. Unidade desconhecida
permanece unidade da cena até declarar referência. Unidade m/mm da calibração não
é atribuída silenciosamente aos assets. Distância usa composição montada, sem
explosão; linhas podem acompanhar a vista explodida com valor identificado como
montado. Exportação inclui âncoras, método, referência, hashes/caminhos de origem
e uncertainty=null; metrology_validated=false. Export não sobrescreve arquivo.

## Evidência executada
Linux x86_64, Rust/Cargo 1.99.1, display Xorg :99 e OpenGL Mesa software.

| Comando/fluxo | Resultado |
|---|---|
| cargo fmt --check | passou |
| cargo clippy --workspace --all-targets -- -D warnings | passou |
| cargo test --workspace --locked | 35 testes, 0 falhas |
| cargo build --release -p noah-desktop --locked | passou |
| Seleção de dois vértices pela janela | referência RGB, índices originais 1/2 (UI 2/3) |
| Distância antes de escala | 1.0000 unidade da cena |
| Referência 100 mm aplicada pela janela | distância exibida 100.0000 mm |
| Registrar e exportar relatório pela interface | JSON com valor 100, unidade mm e incerteza null |
| Salvar e reabrir workspace pela interface | sessão 3 com medida/âncoras/calibração preservadas |
| Integridade da fonte | SHA-256 conferido e inalterado |

Trinta testes anteriores e cinco novos: distância conhecida/unidade desconhecida,
calibração e conversão, explosão/rigidez e escala desatualizada, round-trip e
relatório, referências inválidas/âncoras corrompidas e snap de tela. Um teste pode
conter múltiplos cenários. Abertura de schema 1/2 continua coberta pelos testes
anteriores; E4 acrescenta schema 3 sem perder o estado anterior.

A referência numérica de teste usa dados sintéticos; não é medida de um objeto
real com instrumento calibrado. Capturas reais em docs/desktop/medicoes-escala.png
e e4-evidence/sessao-reaberta.png. Relatório exportado real em e4-evidence/.

## Demonstração
Abrir NOAH, importar uma malha/nuvem, clicar Medir e Escolher pontos A e B. Clique
nos vértices desejados, registre o nome da distância. Declare distância conhecida,
unidade e origem da referência e aplique. Salvar revisão preserva essas informações;
Abrir workspace restaura as anotações. Exportar relatório gera JSON local novo.
Para reproduzir o teste numérico, usar tests/datasets/reference-rgb.ply: vértices
2 e 3 mostrados na UI distam uma unidade da cena; declarar 100 mm produz 100 mm.

## Limites e pendências
E4 está parcial: não há ajuste de primitivas, dimensões curvas, interpolação de
ponto sobre face ou certificação metrológica. A incerteza é desconhecida. Seleção
faz snap de vértice até 12 pixels; oclusão continua aproximada. Conferir os
marcadores A/B antes de registrar. Limite de mil medidas por sessão; muitas
anotações/cenas grandes podem exigir mais tempo de cálculo e poluir a tela.
A escala é global ao workspace: arquivos em referenciais/unidades diferentes
exigem alinhamento/conversão explícitos antes de usar uma única referência.

Mudança de comprimento da referência além de tolerância relativa 1e-9 invalida
seu uso físico. Medidas permanecem ancoradas, mas o app pede recalibração e recusa
exportação física até corrigir/remover a referência. Estado desatualizado pode
ser salvo e reaberto para correção; não é descartado silenciosamente.

Caminhos absolutos, limites do renderer e restrições da prévia Linux anterior
permanecem. Dense CUDA de E2 ainda não validado. E5 permanece montagem manual;
E6/E7/E8 não iniciadas. Próximos incrementos sugeridos: instâncias repetidas para
montagens, projeto portátil e primitivas simples de medição, com contratos/testes.
