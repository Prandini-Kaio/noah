# Estilhaços e layout — relatório de entrega

Data: 2026-11-10. Incremento autorizado sobre Arctic/E3/E4/E5. Referência: pedido
direto do usuário e especificação visual anexada. Ferramentas futuras citadas no
texto não foram tratadas como requisitos de implementação neste incremento.

## Resultado e decisões
- `apps/desktop/src/blast.rs`: fragmentação visual de malha única, subdivisão de
  faces, impulso multidirecional, rotação rígida, pequena queda Z, seleção/cena,
  força e playback de 1,6s com pausa, tempo, reversão e restauração.
- `apps/desktop/src/shell.rs`: navegação de módulos, ações globais, fluxo real de
  reconstrução, árvore pesquisável, painéis redimensionáveis/ocultáveis, toolbar,
  maximização e inspetor contextual com Propriedades/Análise e seções recolhíveis.
- `main.rs`: integração da prévia, modos Sólido/Arame/Pontos, visibilidade de
  malhas/nuvens, estado vazio com ações e proteção das medições/edição na prévia.
- Guias, roadmap, AGENTS e fronteiras atualizados; ADR 0008 documenta a separação.
  Sem novas dependências, fontes, serviços, schema ou alterações no domínio.

## Verificação executada
Linux x86_64; Rust 1.99.1; Xorg dummy :99; OpenGL Mesa software.

| Comando/fluxo | Resultado |
|---|---|
| `cargo fmt --all --check` | passou |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passou |
| `cargo test --workspace --locked` | 47 testes, 0 falhas |
| `cargo build --release -p noah-desktop --locked` | passou |
| `scripts/desktop/package-linux.sh <pasta-nova>` | pacote Linux produzido |
| Octaedro único `reference-mesh.obj` | 8 faces → 32 estilhaços visuais |
| Pausa, tempo e reprodução inversa | conferidos; retorno ao octaedro intacto |
| Salvar sessão durante prévia pausada | schema 4, sem fragmentos; hash da fonte conferido |
| Abrir Medir durante a prévia | restaurou objeto; janela/Análise sobre o original |
| Busca sem resultado, maximização e redimensionamento | conferidos na janela |
| Pacote final aberto com sessão salva durante a prévia | objeto intacto; estilhaçamento conferido novamente |
| Estado vazio | ações de fotos, geometria e workspace visíveis |
| Gravação nativa | 120 frames, 1440×900, 20 fps, 6s; explosão e remontagem |

Seis testes novos: cobertura de superfície/retorno/integridade, determinismo e
rigidez, falhas de seleção vazia, nuvens/cores/hierarquia/visibilidade, orçamento
e enquadramento, e busca hierárquica. Quarenta e um testes anteriores preservados.
Cobertura/rigidez usam tolerância absoluta de 1e-10 nos dados sintéticos unitários.

Capturas: `docs/desktop/layout-workspace.png`, `estilhacos.png` e
`docs/quality/estilhacos-evidence/`. O vídeo nativo acompanha a distribuição da
prévia, separado do histórico de fonte. Não contém reconstruções de terceiros.

## Demo reproduzível
No pacote Linux: abra NOAH → Importar → Geometria e escolha uma malha triangular.
Para desenvolvimento: `cargo run --locked -p noah-desktop -- tests/datasets/reference-mesh.obj`.
Em Montagem: Estilhaçar → Explodir → Pausar → ajustar Tempo → Remontar/Restaurar.
Para uma montagem, use Workspace → Explorar montagem de exemplo; Somente seleção
restringe o efeito a uma peça ou grupo. Guia: `docs/desktop/ESTILHACOS-LAYOUT.md`.

## Aceite e limites
Conforme: malha única estilhaça temporariamente; componentes mantêm vista técnica;
fontes, medidas/exportação e sessão não recebem fragmentos; layout organizado,
com navegação real e controles contextualizados; gates passaram e demo nativa vista.

Não há fratura volumétrica, superfície interior ou colisão. Fragmentos são faces
subdivididas, e nuvens permanecem pontos. Até 20 mil fragmentos; modelos maiores
são amostrados e podem apresentar lacunas. Profundidade é aproximada. Alta força
pode levar fragmentos para fora do viewport; Enquadrar/F usa o instante atual.
O enquadramento da prévia considera os fragmentos-alvo, não objetos fora da seleção.
Efeito/painéis/modos de exibição são transitórios; câmera/tema/réguas seguem sessão.

Não foram executados novos ensaios COLMAP/CUDA ou de metrologia física. Mantêm-se
as limitações anteriores de escala declarada, fontes por caminho absoluto e Linux
glibc 2.39+. Fontes Inter/JetBrains Mono, ícones Lucide, seção, área, volume e ângulos
ficam para escopo futuro; não há controles fingindo implementá-los. E2 dense/CUDA
continua pendente. Nenhum próximo MVP foi iniciado.
