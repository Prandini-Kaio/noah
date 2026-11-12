# Continuação E4 — ângulos e dimensões XYZ

Entrega iniciada em 2026-11-10 e concluída/validada na retomada de 2026-11-12.
Autorização: continuidade das entregas pedida pelo usuário. Escopo e aceite em
../roadmap/E4-ANGULOS-DIMENSOES.md. Não declara E4 completa.

## Alterações e contratos
- Workspace/inspection.rs: ângulo ABC no vértice B (0–180°), f64/atan2; AABB XYZ
  mundial sobre todos os vértices/filhos, mesmo ocultos, no estado montado.
- lib.rs/session.rs: definições persistidas em sessão 5, leitura 1–4.
- measurement.rs: IDs/métodos validados entre tipos; relatório 2 acrescenta
  ângulos, caixas e transformações/hierarquia para reprodução.
- Desktop/metrology.rs: seleção 2/3 pontos ou índices, coordenadas, registro,
  remoção, análise, anotações e cache de caixas invalidado por geometria,
  transformações e hierarquia. main/shell integram o módulo; explosão não mede.
- Pacote: inclui inspection-box.obj CC0 e guia para testar sem terminal.
  Sem novas dependências ou mudanças em schemas E1/E2; ADR 0009.

## Verificação executada
Linux x86_64; Rust 1.99.1; Xorg dummy :99 e Mesa OpenGL software. Gates retomados
em 2026-11-12; validações visuais em 2026-11-10 e 2026-11-12.

| Comando ou fluxo | Resultado |
|---|---|
| `cargo fmt --all --check` | passou |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passou |
| `cargo test --workspace --locked` | 54 testes, 0 falhas |
| `cargo build --release -p noah-desktop --locked` | passou |
| `scripts/desktop/package-linux.sh <pasta-nova>` | produzido com exemplo/guia |
| Ângulo por índice 2–1–4 | 90° conferido na janela e no relatório |
| Snap na cena, 6–5–8 | 90°; janela retornou após C; âncoras [5,4,7] salvas |
| Dimensões sem escala | X=2, Y=3, Z=4 unidades de cena |
| Aresta 2–1 declarada 200 mm | dimensões 200×300×400 mm |
| Sessão 5 e relatório 2 pela interface | medidas/âncoras/unidades conferidas no JSON |
| Integridade da fonte | hash da fonte no arquivo salvo coincide com o OBJ |
| Reabertura da sessão | distâncias, ângulos, caixa e referência restaurados |
| Pacote final com exemplo incluído | janela aberta; X/Y/Z = 2/3/4 u. cena conferidos |
| Compatibilidade binária | maior símbolo GLIBC exigido: 2.39 |

Sete testes novos, além dos 47 anteriores: ângulos conhecidos/extremos/falhas;
invariância rígida/escala/explosão e calibração stale; caixas/filhos ocultos/escala;
rotação e alvos vazios; sessão5/legado/relatório2; corrupção/IDs/métodos/incerteza;
cache com troca de geometria, transformação, hierarquia e escala. Tolerâncias
numéricas de 1e-10 nos dados sintéticos; nenhuma certificação de instrumento.

Evidências: docs/desktop/e4-inspecao.png e docs/quality/e4-inspecao-evidence/.
Os JSONs de smoke são locais (fontes absolutas); o dataset reproduzível está em
tests/datasets/inspection-box.obj, também no pacote sob examples/.

## Demo reproduzível
Abra o exemplo por Importar → Geometria e selecione a peça. Em Medidas/Ângulo,
use 6–5–8: 90°. Em Distância, declare 200 mm para 2–1. Em Dimensões XYZ registre
200×300×400 mm. Salve uma revisão, exporte relatório e reabra a sessão.
Para desenvolvimento: `cargo run --locked -p noah-desktop -- tests/datasets/inspection-box.obj`.

## Aceite, limites e próximos passos
Conforme: ângulos/dimensões calculados sobre dados completos; seleção verificável,
registro/anotação/persistência/relatório; estado montado preservado; escala explícita;
gates e fluxos nativos conferidos. Nenhuma fonte foi reescrita.

AABB mundial não é caixa mínima orientada e pode mudar ao girar. Inclui filhos
ocultos; caixa gráfica só aparece no estado montado e com algum alvo visível.
Ângulos continuam calculáveis com escala linear inválida, mas o relatório completo
recusa a referência inválida. Definições são vivas; não são snapshots congelados.
O primeiro cálculo sobre modelos grandes pode ser mais lento; limites: 1000
ângulos/100 caixas e sessão até 4 MiB. Incerteza permanece desconhecida.

Apps anteriores não abrem sessão 5; consumidores de relatório 1 precisam reconhecer
versão 2. Fontes ainda são caminhos absolutos. Renderer tem oclusão aproximada;
precisão física/COLMAP dense/CUDA não foram validados nesta entrega.
Permanecem pendentes criação de primitivas E4 e alinhamento guiado/instâncias E5;
nenhum deles foi iniciado. E6/E7/E8 continuam fora do escopo atual.
